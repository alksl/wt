//! Read-only view of the repository through gix. Nothing here writes; mutations live in `git`.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};

pub struct Worktree {
    pub path: PathBuf,
    /// Short branch name; `None` when detached.
    pub branch: Option<String>,
    /// Abbreviated HEAD commit, as `git worktree list` shows it; `None` before the first commit.
    pub head_short: Option<String>,
    pub is_main: bool,
    /// Checked out on the default branch; see `Repo::default_branch`.
    pub is_default: bool,
}

pub struct Repo {
    repo: gix::Repository,
}

impl Repo {
    /// Discover the repository from the current directory, from the main or any linked worktree.
    pub fn discover() -> Result<Self> {
        let repo = gix::discover_with_environment_overrides(".")
            .map_err(|_| anyhow!("Not inside a git repository"))?;
        Ok(Self { repo })
    }

    /// All worktrees, main first, in the order `git worktree list` prints them.
    pub fn worktrees(&self) -> Result<Vec<Worktree>> {
        let main = self
            .repo
            .main_repo()
            .context("Could not open the main worktree")?;
        let mut all = vec![describe(&main, self.main_worktree()?, true)?];

        let mut linked = Vec::new();
        for proxy in self.repo.worktrees().context("Could not list worktrees")? {
            let path = proxy
                .base()
                .context("Could not read a worktree's location")?;
            let path = path.canonicalize().unwrap_or(path);
            let repo = proxy
                .into_repo_with_possibly_inaccessible_worktree()
                .with_context(|| format!("Could not open worktree {}", path.display()))?;
            linked.push(describe(&repo, path, false)?);
        }
        // git sorts linked worktrees bytewise by path, not by their id under .git/worktrees.
        linked.sort_by(|a, b| {
            a.path
                .as_os_str()
                .as_encoded_bytes()
                .cmp(b.path.as_os_str().as_encoded_bytes())
        });
        all.extend(linked);

        let default = self.default_branch()?;
        for wt in &mut all {
            wt.is_default = default.is_some() && wt.branch == default;
        }
        Ok(all)
    }

    /// The main worktree's path, even when running from a linked one.
    pub fn main_worktree(&self) -> Result<PathBuf> {
        // Like git: the real path of the common dir, minus a trailing `.git`.
        let common = self
            .repo
            .common_dir()
            .canonicalize()
            .context("Could not resolve the repository's git directory")?;
        if common.file_name().is_some_and(|name| name == ".git") {
            Ok(common
                .parent()
                .expect("a .git dir has a parent")
                .to_path_buf())
        } else {
            bail!("Bare repositories are not supported: {}", common.display())
        }
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

    /// The worktree that has `branch` checked out, if any.
    pub fn worktree_for_branch(&self, branch: &str) -> Result<Option<PathBuf>> {
        Ok(self
            .worktrees()?
            .into_iter()
            .find(|wt| wt.branch.as_deref() == Some(branch))
            .map(|wt| wt.path))
    }

    /// The branch checked out in the main worktree, if it isn't detached.
    pub fn main_branch(&self) -> Result<Option<String>> {
        let main = self
            .repo
            .main_repo()
            .context("Could not open the main worktree")?;
        let head = main
            .head()
            .context("Could not read the main worktree's HEAD")?;
        Ok(head.referent_name().map(|name| name.shorten().to_string()))
    }

    /// `origin/HEAD`'s target, else the branch checked out in the main worktree, which by convention
    /// stays on the default branch.
    pub fn default_branch(&self) -> Result<Option<String>> {
        let origin_head = self
            .repo
            .try_find_reference("refs/remotes/origin/HEAD")
            .context("Could not read refs/remotes/origin/HEAD")?;
        if let Some(gix::refs::TargetRef::Symbolic(target)) =
            origin_head.as_ref().map(|r| r.target())
        {
            let target = target.as_bstr().to_string();
            return Ok(Some(
                target
                    .strip_prefix("refs/remotes/origin/")
                    .unwrap_or(&target)
                    .to_string(),
            ));
        }
        self.main_branch()
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
        is_default: false,
    })
}
