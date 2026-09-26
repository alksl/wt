//! Mutations go through the git binary so hooks, config and git's own safety checks apply.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

fn git() -> Command {
    Command::new("git")
}

/// Runs git with its stdout sent to our stderr: stdout belongs to `switch`'s path.
fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd
        .stdout(std::io::stderr())
        .status()
        .context("Could not run git")?;
    if !status.success() {
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
        bail!("git {} failed", args.join(" "));
    }
    Ok(())
}

/// With a remote start point, git's default `branch.autoSetupMerge` sets it as upstream.
pub fn branch_create(branch: &str, start_point: Option<&str>) -> Result<()> {
    run(git().arg("branch").arg(branch).args(start_point))
}

pub fn worktree_add(path: &Path, branch: &str) -> Result<()> {
    run(git().args(["worktree", "add"]).arg(path).arg(branch))
}

/// Without `--force`, so git refuses to remove a dirty or locked worktree.
pub fn worktree_remove(path: &Path) -> Result<()> {
    run(git().args(["worktree", "remove"]).arg(path))
}
