use anyhow::{Context, Result, bail};

use crate::{git, log, repo::Repo};

/// A branch never advanced past its starting commit is trivially an ancestor of the default branch,
/// so the reflog check is what separates merged work from an untouched workspace.
pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    let default_branch = repo
        .default_branch()?
        .context("Could not determine default branch")?;
    log::info(&format!("Using default branch: {default_branch}"));

    let mut pruned = 0;
    let mut failed = 0;

    for wt in repo.worktrees()?.iter().filter(|w| !w.is_main) {
        if repo.is_default(wt)? {
            continue;
        }
        let Some(branch) = &wt.branch else { continue };
        if !repo.is_ancestor(branch, default_branch)? {
            continue;
        }
        let path = wt.path.display();
        if !repo.branch_has_own_commits(branch)? {
            log::warn(&format!(
                "Keeping worktree with no commits of its own: {path} (branch: {branch})"
            ));
        } else if git::worktree_remove(&wt.path).is_ok() {
            log::info(&format!(
                "Removed merged worktree: {path} (branch: {branch})"
            ));
            pruned += 1;
        } else {
            log::error(&format!(
                "Could not remove worktree: {path} (branch: {branch})"
            ));
            failed += 1;
        }
    }

    if pruned == 0 && failed == 0 {
        log::info("No merged worktrees found");
    }
    // A refused removal is prune's exit status, so a caller chaining onto it stops here.
    if failed > 0 {
        bail!("{failed} worktree(s) could not be removed");
    }
    Ok(())
}
