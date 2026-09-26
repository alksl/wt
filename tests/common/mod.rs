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
        let repo = Self {
            _tmp: tmp,
            root,
            main,
        };
        repo.git(&repo.main, &["init", "--initial-branch=main"]);
        repo.commit(&repo.main, "initial");
        repo
    }

    /// With a bare `origin` holding `main`, and `origin/HEAD` pointing at it.
    pub fn with_origin() -> Self {
        let repo = Self::new();
        let origin = repo.root.join("origin.git");
        repo.git(&repo.root, &["init", "--bare", "origin.git"]);
        repo.git(
            &repo.main,
            &["remote", "add", "origin", origin.to_str().unwrap()],
        );
        repo.git(&repo.main, &["push", "--quiet", "-u", "origin", "main"]);
        repo.git(&repo.main, &["remote", "set-head", "origin", "main"]);
        repo
    }

    /// A toy project with real files, pushed to its origin, plus a `remote-feature` branch that
    /// only origin has.
    pub fn toy() -> Self {
        let repo = Self::with_origin();
        repo.write(&repo.main, "README.md", "# toy\n");
        repo.write(
            &repo.main,
            "src/lib.rs",
            "pub fn answer() -> u32 {\n    42\n}\n",
        );
        repo.git(&repo.main, &["add", "."]);
        repo.git(
            &repo.main,
            &["commit", "--quiet", "-m", "Add the toy project"],
        );
        repo.git(&repo.main, &["push", "--quiet", "origin", "main"]);

        repo.git(&repo.main, &["switch", "--quiet", "-c", "remote-feature"]);
        repo.write(&repo.main, "src/feature.rs", "pub fn feature() {}\n");
        repo.git(&repo.main, &["add", "."]);
        repo.git(&repo.main, &["commit", "--quiet", "-m", "Add a feature"]);
        repo.git(&repo.main, &["push", "--quiet", "origin", "remote-feature"]);
        repo.git(&repo.main, &["switch", "--quiet", "main"]);
        repo.git(&repo.main, &["branch", "--quiet", "-D", "remote-feature"]);
        repo
    }

    /// Writes `contents` to `rel` under `dir`, creating parent directories.
    pub fn write(&self, dir: &Path, rel: &str, contents: &str) {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// Where `wt new <branch>` puts the worktree.
    pub fn sibling(&self, branch: &str) -> PathBuf {
        self.root.join(format!("proj-{branch}"))
    }

    /// Creates `branch` from HEAD with a worktree at its sibling path, without going through `wt`.
    pub fn add_worktree(&self, branch: &str) -> PathBuf {
        let path = self.sibling(branch);
        self.git(
            &self.main,
            &[
                "worktree",
                "add",
                "--quiet",
                "-b",
                branch,
                path.to_str().unwrap(),
            ],
        );
        path
    }

    pub fn commit(&self, dir: &Path, message: &str) {
        self.git(dir, &["commit", "--quiet", "--allow-empty", "-m", message]);
    }

    /// Runs git in `dir`, panicking on failure; returns trimmed stdout.
    pub fn git(&self, dir: &Path, args: &[&str]) -> String {
        let out = self
            .isolated(&mut Command::new("git"))
            .current_dir(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed:\n{}",
            stderr(&out)
        );
        stdout(&out).trim().to_string()
    }

    /// Runs `wt` in `dir` without asserting on the outcome.
    pub fn wt(&self, dir: &Path, args: &[&str]) -> Output {
        self.wt_env(dir, args, &[])
    }

    /// Runs `wt` in `dir` with extra environment variables.
    pub fn wt_env(&self, dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_wt"));
        self.isolated(&mut cmd)
            .current_dir(dir)
            .args(args)
            .envs(env.iter().copied());
        cmd.output().unwrap()
    }

    fn isolated<'a>(&self, cmd: &'a mut Command) -> &'a mut Command {
        cmd.env_remove("CLICOLOR_FORCE")
            .env_remove("NO_COLOR")
            .env("HOME", &self.root)
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

/// Asserts `wt` wrote `needle` to stderr, showing all of stderr when it didn't.
#[track_caller]
pub fn assert_stderr_contains(out: &Output, needle: &str) {
    let stderr = stderr(out);
    assert!(
        stderr.contains(needle),
        "missing {needle:?} in stderr:\n{stderr}"
    );
}
