use anyhow::{Result, bail};

use crate::{cmd, repo::Repo};

/// Prints the worktree path and nothing else on stdout: the shell function cds to it.
pub fn run(branch: &str, create: bool) -> Result<()> {
    let repo = Repo::discover()?;

    if create {
        cmd::new::run(branch)?;
    }

    let resolves_to_main = [repo.default_branch()?, repo.main_branch()?]
        .iter()
        .any(|b| b.as_deref() == Some(branch));
    let path = if resolves_to_main {
        repo.main_worktree()?
    } else {
        repo.worktree_path_for(branch)?
    };

    if !path.is_dir() {
        bail!(
            "No worktree found for branch '{branch}' at: {}",
            path.display()
        );
    }

    println!("{}", path.display());
    Ok(())
}
