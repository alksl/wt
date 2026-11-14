use anyhow::{Context, Result, bail};

use crate::{cmd, config::Config, projects, repo::Repo};

/// Prints the worktree path and nothing else on stdout: the shell function cds to it.
///
/// Inside a repository `target` is a branch. Outside one it is a project, and `branch`, if given,
/// is switched to inside it.
pub fn run(target: &str, branch: Option<&str>, create: bool) -> Result<()> {
    if let Some(repo) = Repo::try_discover()? {
        if branch.is_some() {
            bail!("wt switch takes one branch inside a repository");
        }
        return switch(&repo, target, create);
    }

    let config = Config::load()?;
    let Some(project) = projects::find(&config, target) else {
        let roots: Vec<_> = config
            .projects
            .iter()
            .map(|root| root.display().to_string())
            .collect();
        bail!("No project '{target}' in {}", roots.join(", "));
    };
    let Some(branch) = branch else {
        println!("{}", project.display());
        return Ok(());
    };
    if !projects::is_git(&project) {
        bail!(
            "Project '{target}' is not a git repository: {}",
            project.display()
        );
    }
    std::env::set_current_dir(&project)
        .with_context(|| format!("Could not enter {}", project.display()))?;
    switch(&Repo::discover()?, branch, create)
}

fn switch(repo: &Repo, branch: &str, create: bool) -> Result<()> {
    let resolves_to_main = [repo.default_branch()?, repo.main_branch()?].contains(&Some(branch));

    // A branch that exists but isn't checked out anywhere, like a colleague's on origin, gets its
    // worktree on first switch; only a branch that exists nowhere needs `--create`.
    if create || (!resolves_to_main && needs_worktree(repo, branch)?) {
        cmd::new::create(repo, branch)?;
    }

    let path = if resolves_to_main {
        repo.main_worktree()?.to_path_buf()
    } else {
        repo.worktree_path_for(branch)?
    };

    if !path.is_dir() {
        bail!(
            "No worktree found for branch '{branch}' at: {}",
            path.display()
        );
    }

    println!("{}", path.display());
    Ok(())
}

fn needs_worktree(repo: &Repo, branch: &str) -> Result<bool> {
    Ok(repo.worktree_for_branch(branch)?.is_none()
        && (repo.local_branch_exists(branch)? || repo.remote_branch_exists("origin", branch)?))
}
