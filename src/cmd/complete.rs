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
    for branch in branches {
        println!("{branch}");
    }
    Ok(())
}
