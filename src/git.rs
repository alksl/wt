//! Mutations go through the git binary so hooks, config and git's own safety checks apply.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

fn git() -> Command {
    Command::new("git")
}

/// Runs git with its stdout sent to our stderr: stdout belongs to `switch`'s path.
fn run(cmd: &mut Command) -> Result<()> {
    let status = cmd.stdout(std::io::stderr()).status().context("Could not run git")?;
    if !status.success() {
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy()).collect();
        bail!("git {} failed", args.join(" "));
    }
    Ok(())
}

/// Without `--force`, so git refuses to remove a dirty or locked worktree.
pub fn worktree_remove(path: &Path) -> Result<()> {
    run(git().args(["worktree", "remove"]).arg(path))
}
