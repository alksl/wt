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
    for wt in repo.worktrees()? {
        if let Some(branch) = wt.branch.as_ref().filter(|_| include_main || !wt.is_main) {
            println!("{branch}");
        }
    }
    Ok(())
}
