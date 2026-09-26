use anyhow::Result;
use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub enum Shell {
    Zsh,
}

/// Prints the shell integration for `shell`, meant to be `eval`ed from its rc file.
pub fn run(shell: Shell) -> Result<()> {
    let script = match shell {
        Shell::Zsh => include_str!("init.zsh"),
    };
    print!("{script}");
    Ok(())
}
