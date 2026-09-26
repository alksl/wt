use std::io::{self, Write};

use anyhow::{Result, bail};

use crate::repo::Repo;

/// One branch per line, read by the fzf completion in zsh.
pub fn run(subcommand: &str) -> Result<()> {
    let include_main = match subcommand {
        "switch" | "sw" => true,
        "remove" | "rm" => false,
        other => bail!("no completions for '{other}'"),
    };

    let repo = Repo::discover()?;
    let branches = repo
        .worktrees()?
        .iter()
        .filter(|wt| include_main || !wt.is_main)
        .filter_map(|wt| wt.branch.as_deref());
    print_lines(branches)
}

/// fzf closes the pipe as soon as a candidate is picked, which is not an error.
fn print_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> Result<()> {
    let mut out = io::stdout().lock();
    for line in lines {
        match writeln!(out, "{line}") {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
            result => result?,
        }
    }
    Ok(())
}
