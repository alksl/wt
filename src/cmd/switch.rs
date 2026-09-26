use anyhow::{Result, bail};

use crate::{cmd, repo::Repo};

/// Prints the worktree path and nothing else on stdout: the shell function cds to it.
pub fn run(branch: &str, create: bool) -> Result<()> {
    let repo = Repo::discover()?;
    let resolves_to_main = [repo.default_branch()?, repo.main_branch()?].contains(&Some(branch));

    // A branch that exists but isn't checked out anywhere, like a colleague's on origin, gets its
    // worktree on first switch; only a branch that exists nowhere needs `--create`.
    if create || (!resolves_to_main && needs_worktree(&repo, branch)?) {
        cmd::new::create(&repo, branch)?;
    }

    let path = if resolves_to_main {
        repo.main_worktree()?.to_path_buf()
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

fn needs_worktree(repo: &Repo, branch: &str) -> Result<bool> {
    Ok(repo.worktree_for_branch(branch)?.is_none()
        && (repo.local_branch_exists(branch)? || repo.remote_branch_exists("origin", branch)?))
}
