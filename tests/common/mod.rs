#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// A real repository at `<tmp>/proj`, so `wt` puts linked worktrees beside it at `<tmp>/proj-<b>`.
/// Git runs with no system or global config, so the host's settings and hooks can't leak in.
pub struct TestRepo {
    _tmp: TempDir,
    pub root: PathBuf,
    pub main: PathBuf,
}

impl TestRepo {
    /// On branch `main`, with one commit.
    pub fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        // Git records canonical paths, so compare against the canonical form.
        let root = tmp.path().canonicalize().unwrap();
        let main = root.join("proj");
        std::fs::create_dir(&main).unwrap();
        let repo = Self { _tmp: tmp, root, main };
        repo.git(&repo.main, &["init", "--initial-branch=main"]);
        repo.commit(&repo.main, "initial");
        repo
    }

    /// With a bare `origin` holding `main`, and `origin/HEAD` pointing at it.
    pub fn with_origin() -> Self {
        let repo = Self::new();
        let origin = repo.root.join("origin.git");
        repo.git(&repo.root, &["init", "--bare", "origin.git"]);
        repo.git(&repo.main, &["remote", "add", "origin", origin.to_str().unwrap()]);
        repo.git(&repo.main, &["push", "--quiet", "-u", "origin", "main"]);
        repo.git(&repo.main, &["remote", "set-head", "origin", "main"]);
        repo
    }

    /// Where `wt new <branch>` puts the worktree.
    pub fn sibling(&self, branch: &str) -> PathBuf {
        self.root.join(format!("proj-{branch}"))
    }

    /// Creates `branch` from HEAD with a worktree at its sibling path, without going through `wt`.
    pub fn add_worktree(&self, branch: &str) -> PathBuf {
        let path = self.sibling(branch);
        self.git(&self.main, &["worktree", "add", "--quiet", "-b", branch, path.to_str().unwrap()]);
        path
    }

    pub fn commit(&self, dir: &Path, message: &str) {
        self.git(dir, &["commit", "--quiet", "--allow-empty", "-m", message]);
    }

    /// Runs git in `dir`, panicking on failure; returns trimmed stdout.
    pub fn git(&self, dir: &Path, args: &[&str]) -> String {
        let out = self.isolated(&mut Command::new("git")).current_dir(dir).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?} failed:\n{}", stderr(&out));
        stdout(&out).trim().to_string()
    }

    /// Runs `wt` in `dir` without asserting on the outcome.
    pub fn wt(&self, dir: &Path, args: &[&str]) -> Output {
        self.isolated(&mut Command::new(env!("CARGO_BIN_EXE_wt"))).current_dir(dir).args(args).output().unwrap()
    }

    fn isolated<'a>(&self, cmd: &'a mut Command) -> &'a mut Command {
        cmd.env("HOME", &self.root)
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
    }
}

pub fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Asserts `wt` succeeded, showing its stderr when it didn't.
#[track_caller]
pub fn assert_ok(out: &Output) {
    assert!(out.status.success(), "wt failed:\n{}", stderr(out));
}
