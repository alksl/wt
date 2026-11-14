//! `~/.wt.toml`. Only `wt sw` outside a repository reads it, so nothing else pays for it.

use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

pub struct Config {
    /// Directories whose subdirectories are projects, in the order they are searched.
    pub projects: Vec<PathBuf>,
}

#[derive(Deserialize)]
struct File {
    projects: Option<Vec<String>>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let home = PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?);
        let path = home.join(".wt.toml");
        let roots = match std::fs::read_to_string(&path) {
            Ok(text) => {
                toml::from_str::<File>(&text)
                    .with_context(|| format!("Could not read {}", path.display()))?
                    .projects
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => None,
            Err(e) => return Err(e).with_context(|| format!("Could not read {}", path.display())),
        };
        let roots = roots.unwrap_or_else(|| vec!["~/code".to_owned()]);
        Ok(Self {
            projects: roots.iter().map(|root| expand(&home, root)).collect(),
        })
    }
}

fn expand(home: &Path, root: &str) -> PathBuf {
    match root.strip_prefix("~/") {
        Some(rest) => home.join(rest),
        None => PathBuf::from(root),
    }
}
