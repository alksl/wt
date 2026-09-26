use anyhow::{Result, bail};

use crate::repo::Repo;

/// One branch per line, read by the fzf completion in zsh.
pub fn run(subcommand: &str) -> Result<()> {
    let include_main = match subcommand {
        "switch" | "sw" => true,
        "remove" | "rm" => false,
        other => bail!("no completions for '{other}'"),
    };

    for wt in Repo::discover()?.worktrees()? {
        if let Some(branch) = wt.branch.filter(|_| include_main || !wt.is_main) {
            println!("{branch}");
        }
    }
    Ok(())
}
