use anyhow::Result;

use crate::repo::Repo;

/// Same shape as `git worktree list`: path, short HEAD, `[branch]` or `(detached HEAD)`, then
/// ` locked` and ` prunable` where they apply.
pub fn run() -> Result<()> {
    let repo = Repo::discover()?;
    let worktrees = repo.worktrees()?;
    let heads: Vec<_> = worktrees
        .iter()
        .map(|w| w.head_short.as_deref().unwrap_or("0000000"))
        .collect();
    let path_width = worktrees
        .iter()
        .map(|w| w.path.display().to_string().len())
        .max()
        .unwrap_or(0);
    let head_width = heads.iter().map(|h| h.len()).max().unwrap_or(0);

    for (wt, head) in worktrees.iter().zip(heads) {
        let branch = match &wt.branch {
            Some(b) => format!("[{b}]"),
            None => "(detached HEAD)".to_string(),
        };
        let locked = if wt.locked { " locked" } else { "" };
        let prunable = if wt.prunable { " prunable" } else { "" };
        println!(
            "{:<path_width$} {head:<head_width$} {branch}{locked}{prunable}",
            wt.path.display()
        );
    }
    Ok(())
}
