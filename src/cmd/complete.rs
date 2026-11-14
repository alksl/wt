use std::io::{self, Write};

use anyhow::{Result, bail};

use crate::{config::Config, projects, repo::Repo};

/// One branch per line, read by the fzf completion in zsh. A branch with no worktree yet carries a
/// tab and where it was found, `local` or `origin`, which the completion strips before inserting.
///
/// `args` are the words already typed after `wt switch`; the output completes the next one.
pub fn run(subcommand: &str, args: &[String]) -> Result<()> {
    if matches!(subcommand, "switch" | "sw") {
        return print_lines(&switch(args)?);
    }
    let repo = Repo::discover()?;
    let lines = match subcommand {
        "new" => without_worktree(&repo)?,
        "exec" => checked_out(&repo, true)?,
        "remove" | "rm" => checked_out(&repo, false)?,
        other => bail!("no completions for '{other}'"),
    };
    print_lines(&lines)
}

/// Branches inside a repository; outside one, projects, then a git project's branches.
fn switch(args: &[String]) -> Result<Vec<String>> {
    if let Some(repo) = Repo::try_discover()? {
        return if args.is_empty() {
            switch_lines(&repo)
        } else {
            Ok(Vec::new())
        };
    }
    let config = Config::load()?;
    match args {
        [] => Ok(projects::list(&config)?
            .into_iter()
            .map(|(name, _)| name)
            .collect()),
        [project] => match projects::find(&config, project) {
            Some(path) if projects::is_git(&path) => {
                std::env::set_current_dir(&path)?;
                switch_lines(&Repo::discover()?)
            }
            _ => Ok(Vec::new()),
        },
        _ => Ok(Vec::new()),
    }
}

/// The worktrees' branches, main first, then the branches without one.
fn switch_lines(repo: &Repo) -> Result<Vec<String>> {
    let mut lines = checked_out(repo, true)?;
    lines.extend(without_worktree(repo)?);
    Ok(lines)
}

fn checked_out(repo: &Repo, include_main: bool) -> Result<Vec<String>> {
    Ok(repo
        .worktrees()?
        .iter()
        .filter(|wt| include_main || !wt.is_main)
        .filter_map(|wt| wt.branch.clone())
        .collect())
}

/// Branches `wt new` and `wt sw -c` would give a worktree: local ones first, then those only origin has.
fn without_worktree(repo: &Repo) -> Result<Vec<String>> {
    let local = repo.local_branches()?;
    let mut lines = Vec::new();
    for branch in local {
        if repo.worktree_for_branch(branch)?.is_none() {
            lines.push(format!("{branch}\tlocal"));
        }
    }
    for branch in repo.origin_branches()? {
        if !local.contains(branch) && repo.worktree_for_branch(branch)?.is_none() {
            lines.push(format!("{branch}\torigin"));
        }
    }
    Ok(lines)
}

/// fzf closes the pipe as soon as a candidate is picked, which is not an error.
fn print_lines(lines: &[String]) -> Result<()> {
    let mut out = io::stdout().lock();
    for line in lines {
        match writeln!(out, "{line}") {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
            result => result?,
        }
    }
    Ok(())
}
