use std::ffi::OsString;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{self, Command};

use anyhow::{Result, bail};

use crate::{log, repo::Repo};

/// Replaces wt with `command` in the worktree, so its exit status and signals reach the caller
/// as they are. Returns only when there is no worktree to run in.
pub fn run(branch: Option<&str>, command: &[OsString]) -> Result<()> {
    let dir = worktree(branch)?;
    let (program, args) = command.split_first().expect("clap requires a command");

    let err = Command::new(program)
        .args(args)
        .current_dir(&dir)
        .env("PWD", &dir)
        .exec();

    log::error(&format!("Could not run {}: {err}", program.display()));
    // What a shell exits with for a command it can't find, and for one it can't execute.
    process::exit(match err.kind() {
        io::ErrorKind::NotFound => 127,
        _ => 126,
    })
}

/// Resolves like `wt switch`, but only to a worktree that already exists.
fn worktree(branch: Option<&str>) -> Result<PathBuf> {
    let repo = Repo::discover()?;
    let Some(branch) = branch else {
        return repo.current_worktree();
    };

    let path = if [repo.default_branch()?, repo.main_branch()?].contains(&Some(branch)) {
        Some(repo.main_worktree()?)
    } else {
        repo.worktree_for_branch(branch)?
    };
    match path {
        Some(path) if path.is_dir() => Ok(path.to_path_buf()),
        _ => bail!("No worktree found for branch '{branch}'"),
    }
}
