use anyhow::{Context, Result};

use crate::{git, log, repo::Repo};

pub fn run(branch: &str) -> Result<()> {
    let repo = Repo::discover()?;
    let path = repo
        .worktree_for_branch(branch)?
        .with_context(|| format!("No worktree found for branch '{branch}'"))?;
    git::worktree_remove(path)?;
    log::info(&format!("Removed worktree: {}", path.display()));
    Ok(())
}
