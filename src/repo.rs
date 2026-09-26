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
        let main = self.repo.main_repo().context("Could not open the main worktree")?;
        let mut all = vec![describe(&main, self.main_worktree()?, true)?];

        let mut linked = Vec::new();
        for proxy in self.repo.worktrees().context("Could not list worktrees")? {
            let path = proxy.base().context("Could not read a worktree's location")?;
            let path = path.canonicalize().unwrap_or(path);
            let repo = proxy
                .into_repo_with_possibly_inaccessible_worktree()
                .with_context(|| format!("Could not open worktree {}", path.display()))?;
            linked.push(describe(&repo, path, false)?);
        }
        // git sorts linked worktrees bytewise by path, not by their id under .git/worktrees.
        linked.sort_by(|a, b| a.path.as_os_str().as_encoded_bytes().cmp(b.path.as_os_str().as_encoded_bytes()));
        all.extend(linked);
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
            Ok(common.parent().expect("a .git dir has a parent").to_path_buf())
        } else {
            bail!("Bare repositories are not supported: {}", common.display())
        }
    }
}

fn describe(repo: &gix::Repository, path: PathBuf, is_main: bool) -> Result<Worktree> {
    let head = repo
        .head()
        .with_context(|| format!("Could not read HEAD of {}", path.display()))?;
    let branch = head.referent_name().map(|name| name.shorten().to_string());
    let head_short = head.id().map(|id| id.shorten_or_id().to_string());
    Ok(Worktree { path, branch, head_short, is_main })
}
