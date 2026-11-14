//! Projects: the directories directly under the roots in `~/.wt.toml`, for `wt sw <project>`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::Config;

/// Every project by name, sorted. A name in more than one root resolves to the first root's.
pub fn list(config: &Config) -> Result<Vec<(String, PathBuf)>> {
    let mut projects: Vec<(String, PathBuf)> = Vec::new();
    for root in config.projects.iter().filter(|root| root.is_dir()) {
        let entries = std::fs::read_dir(root)
            .with_context(|| format!("Could not read {}", root.display()))?;
        for entry in entries {
            let entry = entry.with_context(|| format!("Could not read {}", root.display()))?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if is_project(&path, &name) && !projects.iter().any(|(n, _)| *n == name) {
                projects.push((name, path));
            }
        }
    }
    projects.sort();
    Ok(projects)
}

/// The project called `name` in the first root that has one. A name is one path component.
pub fn find(config: &Config, name: &str) -> Option<PathBuf> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') {
        return None;
    }
    config
        .projects
        .iter()
        .map(|root| root.join(name))
        .find(|path| is_project(path, name))
}

/// Whether `path` holds a main checkout, as opposed to a plain directory.
pub fn is_git(path: &Path) -> bool {
    path.join(".git").is_dir()
}

/// A directory, but not a linked worktree (whose `.git` is a file) nor a bare `*.git` repository.
fn is_project(path: &Path, name: &str) -> bool {
    path.is_dir() && !name.ends_with(".git") && !path.join(".git").is_file()
}
