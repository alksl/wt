//! Read-only view of the repository through gix. Nothing here writes; mutations live in `git`.

use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};

/// One entry of `git worktree list`.
pub struct Worktree {
    pub path: PathBuf,
    /// Short branch name; `None` when detached.
    pub branch: Option<String>,
    /// Abbreviated HEAD commit, as `git worktree list` shows it; `None` before the first commit.
    pub head_short: Option<String>,
    pub is_main: bool,
    /// `git worktree lock`ed; never set on the main worktree.
    pub locked: bool,
    /// What `git worktree prune` would remove: its checkout is gone and it isn't locked.
    pub prunable: bool,
}

/// Each fact is read at most once, on first use, and kept for the rest of the run: shell
/// completion runs `wt` on every keystroke. A command reads what it needs before it changes
/// anything, so nothing cached here goes stale under it.
pub struct Repo {
    repo: gix::Repository,
    /// Only opened when running from a linked worktree; otherwise `repo` is the main one.
    main: OnceCell<gix::Repository>,
    main_worktree: OnceCell<PathBuf>,
    main_branch: OnceCell<Option<String>>,
    default_branch: OnceCell<Option<String>>,
    worktrees: OnceCell<Vec<Worktree>>,
}

impl Repo {
    /// Discover the repository from the current directory, from the main or any linked worktree.
    pub fn discover() -> Result<Self> {
        let repo = gix::discover_with_environment_overrides(".")
            .map_err(|_| anyhow!("Not inside a git repository"))?;
        Ok(Self {
            repo,
            main: OnceCell::new(),
            main_worktree: OnceCell::new(),
            main_branch: OnceCell::new(),
            default_branch: OnceCell::new(),
            worktrees: OnceCell::new(),
        })
    }

    /// All worktrees, main first, in the order `git worktree list` prints them.
    pub fn worktrees(&self) -> Result<&[Worktree]> {
        let all = try_init(&self.worktrees, || {
            let main = describe(self.main_repo()?, self.main_worktree()?.to_path_buf(), true)?;

            let mut linked = Vec::new();
            for proxy in self.repo.worktrees().context("Could not list worktrees")? {
                let path = proxy
                    .base()
                    .context("Could not read a worktree's location")?;
                let path = path.canonicalize().unwrap_or(path);
                let (locked, prunable) = (proxy.is_locked(), proxy.is_prunable());
                let repo = proxy
                    .into_repo_with_possibly_inaccessible_worktree()
                    .with_context(|| format!("Could not open worktree {}", path.display()))?;
                linked.push(Worktree {
                    locked,
                    prunable,
                    ..describe(&repo, path, false)?
                });
            }
            // git sorts linked worktrees bytewise by path, not by their id under .git/worktrees.
            linked.sort_by(|a, b| {
                a.path
                    .as_os_str()
                    .as_encoded_bytes()
                    .cmp(b.path.as_os_str().as_encoded_bytes())
            });

            Ok(std::iter::once(main).chain(linked).collect())
        })?;
        Ok(all)
    }

    /// The main worktree's path, even when running from a linked one.
    pub fn main_worktree(&self) -> Result<&Path> {
        let path = try_init(&self.main_worktree, || {
            // Like git: the real path of the common dir, minus a trailing `.git`.
            let mut path = self
                .repo
                .common_dir()
                .canonicalize()
                .context("Could not resolve the repository's git directory")?;
            if !path.ends_with(".git") {
                bail!("Bare repositories are not supported: {}", path.display());
            }
            path.pop();
            Ok(path)
        })?;
        Ok(path)
    }

    /// Where `wt new` puts a branch: `<parent of main>/<main basename>-<branch>`.
    pub fn worktree_path_for(&self, branch: &str) -> Result<PathBuf> {
        let main = self.main_worktree()?;
        let name = main
            .file_name()
            .context("The main worktree has no directory name")?;
        let parent = main.parent().expect("a path with a file name has a parent");
        Ok(parent.join(format!("{}-{branch}", name.to_string_lossy())))
    }

    pub fn local_branch_exists(&self, branch: &str) -> Result<bool> {
        self.ref_exists(&format!("refs/heads/{branch}"))
    }

    pub fn remote_branch_exists(&self, remote: &str, branch: &str) -> Result<bool> {
        self.ref_exists(&format!("refs/remotes/{remote}/{branch}"))
    }

    fn ref_exists(&self, name: &str) -> Result<bool> {
        let found = self
            .repo
            .try_find_reference(name)
            .with_context(|| format!("Could not read {name}"))?;
        Ok(found.is_some())
    }

    /// Whether `wt` is checked out on the default branch; see `default_branch`.
    pub fn is_default(&self, wt: &Worktree) -> Result<bool> {
        Ok(wt.branch.is_some() && wt.branch.as_deref() == self.default_branch()?)
    }

    /// The worktree that has `branch` checked out, if any.
    pub fn worktree_for_branch(&self, branch: &str) -> Result<Option<&Path>> {
        Ok(self
            .worktrees()?
            .iter()
            .find(|wt| wt.branch.as_deref() == Some(branch))
            .map(|wt| wt.path.as_path()))
    }

    /// The branch checked out in the main worktree, if it isn't detached.
    pub fn main_branch(&self) -> Result<Option<&str>> {
        let branch = try_init(&self.main_branch, || {
            let head = self
                .main_repo()?
                .head()
                .context("Could not read the main worktree's HEAD")?;
            Ok(head.referent_name().map(|name| name.shorten().to_string()))
        })?;
        Ok(branch.as_deref())
    }

    /// `origin/HEAD`'s target, else the branch checked out in the main worktree, which by
    /// convention stays on the default branch.
    pub fn default_branch(&self) -> Result<Option<&str>> {
        let branch = try_init(&self.default_branch, || {
            let origin_head = self
                .repo
                .try_find_reference("refs/remotes/origin/HEAD")
                .context("Could not read refs/remotes/origin/HEAD")?;
            if let Some(gix::refs::TargetRef::Symbolic(target)) =
                origin_head.as_ref().map(gix::Reference::target)
            {
                let target = target.as_bstr().to_string();
                let branch = target
                    .strip_prefix("refs/remotes/origin/")
                    .unwrap_or(&target);
                return Ok(Some(branch.to_owned()));
            }
            Ok(self.main_branch()?.map(str::to_owned))
        })?;
        Ok(branch.as_deref())
    }

    /// Whether local `branch` is reachable from `target`, like `git merge-base --is-ancestor`.
    pub fn is_ancestor(&self, branch: &str, target: &str) -> Result<bool> {
        let branch_id = self
            .repo
            .rev_parse_single(format!("refs/heads/{branch}").as_str())
            .with_context(|| format!("Could not resolve branch '{branch}'"))?
            .detach();
        let target_id = self
            .repo
            .rev_parse_single(target)
            .with_context(|| format!("Could not resolve '{target}'"))?
            .detach();
        let bases = self
            .repo
            .merge_bases_many(branch_id, &[target_id])
            .with_context(|| format!("Could not find a merge base of '{branch}' and '{target}'"))?;
        Ok(bases.iter().any(|base| *base == branch_id))
    }

    /// Whether `branch` moved after it was created: its reflog has more than one entry, as counted
    /// by `git reflog show refs/heads/<branch>`. A missing reflog counts as no.
    pub fn branch_has_own_commits(&self, branch: &str) -> Result<bool> {
        let Some(reference) = self.local_ref(branch)? else {
            return Ok(false);
        };
        let mut platform = reference.log_iter();
        let entries = platform
            .all()
            .with_context(|| format!("Could not read the reflog of '{branch}'"))?
            .map_or(0, |log| log.take(2).count());
        Ok(entries > 1)
    }

    fn local_ref(&self, branch: &str) -> Result<Option<gix::Reference<'_>>> {
        self.repo
            .try_find_reference(format!("refs/heads/{branch}").as_str())
            .with_context(|| format!("Could not read branch '{branch}'"))
    }

    fn main_repo(&self) -> Result<&gix::Repository> {
        if !matches!(self.repo.kind(), gix::repository::Kind::LinkedWorkTree) {
            return Ok(&self.repo);
        }
        try_init(&self.main, || {
            self.repo
                .main_repo()
                .context("Could not open the main worktree")
        })
    }
}

/// `OnceCell::get_or_init` for a fallible `init`, which leaves the cell empty when it fails.
fn try_init<T>(cell: &OnceCell<T>, init: impl FnOnce() -> Result<T>) -> Result<&T> {
    if let Some(value) = cell.get() {
        return Ok(value);
    }
    let value = init()?;
    Ok(cell.get_or_init(|| value))
}

fn describe(repo: &gix::Repository, path: PathBuf, is_main: bool) -> Result<Worktree> {
    let head = repo
        .head()
        .with_context(|| format!("Could not read HEAD of {}", path.display()))?;
    let branch = head.referent_name().map(|name| name.shorten().to_string());
    let head_short = head.id().map(|id| id.shorten_or_id().to_string());
    Ok(Worktree {
        path,
        branch,
        head_short,
        is_main,
        locked: false,
        prunable: false,
    })
}
