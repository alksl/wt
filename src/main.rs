mod cmd;
mod git;
mod log;
mod repo;

use std::ffi::OsString;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Manage sibling git worktrees named <repo>-<branch>")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a worktree for a branch, creating the branch if needed
    New { branch: String },
    /// Print the worktree path for a branch; the shell function cds to it
    ///
    /// A local or origin branch without a worktree gets one first.
    #[command(visible_alias = "sw")]
    Switch {
        /// Also create a branch that exists nowhere yet, from HEAD
        #[arg(short, long)]
        create: bool,
        branch: String,
    },
    /// Remove the worktree checked out on a branch
    #[command(visible_alias = "rm")]
    Remove { branch: String },
    /// Execute a command at a worktree's root without leaving the current directory
    ///
    /// Without a branch, runs at the root of the worktree you are in. Exits with the command's
    /// status.
    Exec {
        /// A branch with a worktree; the default branch runs in the main checkout
        branch: Option<String>,
        /// The command and its arguments, after `--`
        #[arg(last = true, required = true, value_name = "COMMAND")]
        command: Vec<OsString>,
    },
    /// List worktrees
    List,
    /// Remove worktrees whose branch is merged into the default branch
    Prune,
    /// Print shell integration to eval from your rc file: eval "$(wt init zsh)"
    Init { shell: cmd::init::Shell },
    /// Print candidates for shell completion
    #[command(hide = true)]
    Complete { subcommand: String },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::New { branch } => cmd::new::run(&branch),
        Command::Switch { create, branch } => cmd::switch::run(&branch, create),
        Command::Remove { branch } => cmd::remove::run(&branch),
        Command::Exec { branch, command } => cmd::exec::run(branch.as_deref(), &command),
        Command::List => cmd::list::run(),
        Command::Prune => cmd::prune::run(),
        Command::Init { shell } => cmd::init::run(shell),
        Command::Complete { subcommand } => cmd::complete::run(&subcommand),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error(&format!("{e:#}"));
            ExitCode::FAILURE
        }
    }
}
