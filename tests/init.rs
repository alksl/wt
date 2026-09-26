mod common;

use std::path::Path;
use std::process::{Command, Output};

use common::{TestRepo, assert_ok, stderr, stdout};

/// Runs `script` in a zsh with no rc files, in `dir`, with the built `wt` first on PATH and git
/// isolated from the host's config the way the fixture isolates it.
fn zsh(repo: &TestRepo, dir: &Path, script: &str) -> Output {
    let bin = Path::new(env!("CARGO_BIN_EXE_wt")).parent().unwrap();
    let path = std::env::join_paths(
        std::iter::once(bin.to_path_buf())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    Command::new("zsh")
        .args(["-f", "-c", script])
        .current_dir(dir)
        .env("PATH", path)
        .env_remove("CLICOLOR_FORCE")
        .env_remove("NO_COLOR")
        .env("HOME", &repo.root)
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .unwrap()
}

#[test]
fn zsh_script_parses() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["init", "zsh"]);
    assert_ok(&out);

    let check = Command::new("zsh")
        .args(["-f", "-n", "-c", &stdout(&out)])
        .output()
        .unwrap();
    assert!(check.status.success(), "zsh -n failed:\n{}", stderr(&check));
}

#[test]
fn switch_cds_into_the_worktree() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = zsh(
        &repo,
        &repo.main,
        r#"eval "$(wt init zsh)"; wt sw feat; pwd"#,
    );

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", feat.display()));
}

#[test]
fn switch_create_cds_into_the_new_worktree() {
    let repo = TestRepo::new();

    let out = zsh(
        &repo,
        &repo.main,
        r#"eval "$(wt init zsh)"; wt sw -c fresh; pwd"#,
    );

    assert_ok(&out);
    assert_eq!(
        stdout(&out),
        format!("{}\n", repo.sibling("fresh").display())
    );
}

#[test]
fn failed_switch_stays_put_and_fails() {
    let repo = TestRepo::new();

    let out = zsh(
        &repo,
        &repo.main,
        r#"eval "$(wt init zsh)"; wt sw nope; rc=$?; pwd; exit $rc"#,
    );

    assert!(!out.status.success(), "wt sw nope succeeded");
    assert_eq!(stdout(&out), format!("{}\n", repo.main.display()));
}

#[test]
fn other_commands_pass_through() {
    let repo = TestRepo::new();
    repo.add_worktree("feat");

    let out = zsh(
        &repo,
        &repo.main,
        r#"eval "$(wt init zsh)"; wt complete rm"#,
    );

    assert_ok(&out);
    assert_eq!(stdout(&out), "feat\n");
}

#[test]
fn defines_the_fzf_completion() {
    let repo = TestRepo::new();

    let out = zsh(
        &repo,
        &repo.main,
        r#"eval "$(wt init zsh)"; whence -w _fzf_complete_wt _fzf_complete_wt_post"#,
    );

    assert_ok(&out);
    assert_eq!(
        stdout(&out),
        "_fzf_complete_wt: function\n_fzf_complete_wt_post: function\n"
    );
}
