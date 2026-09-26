use anyhow::{Result, bail};

use crate::{git, log, repo::Repo};

pub fn run(branch: &str) -> Result<()> {
    let repo = Repo::discover()?;

    if let Some(existing) = repo.worktree_for_branch(branch)? {
        bail!("A worktree for branch '{branch}' already exists at: {}", existing.display());
    }

    let path = repo.worktree_path_for(branch)?;
    if path.exists() {
        bail!("Target directory already exists: {}", path.display());
    }

    if repo.local_branch_exists(branch)? {
        log::info(&format!("Using existing branch '{branch}'"));
    } else if repo.remote_branch_exists("origin", branch)? {
        log::info(&format!("Creating local branch '{branch}' tracking origin/{branch}"));
        git::branch_create(branch, Some(&format!("origin/{branch}")))?;
    } else {
        log::info(&format!("Creating branch '{branch}' from HEAD"));
        git::branch_create(branch, None)?;
    }

    log::info(&format!("Creating worktree at {}", path.display()));
    git::worktree_add(&path, branch)?;
    log::info(&format!("Done: {}", path.display()));
    Ok(())
}
