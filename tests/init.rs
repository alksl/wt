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

#[test]
fn switch_help_is_printed_not_cd_into() {
    let repo = TestRepo::new();

    for flag in ["-h", "--help"] {
        let out = zsh(
            &repo,
            &repo.main,
            &format!(r#"eval "$(wt init zsh)"; wt sw {flag} && pwd"#),
        );

        assert_ok(&out);
        let stdout = stdout(&out);
        assert!(stdout.starts_with("Print the worktree path"), "{stdout}");
        assert!(
            stdout.ends_with(&format!("\n{}\n", repo.main.display())),
            "{stdout}"
        );
    }
}

#[test]
fn completion_inserts_only_the_branch_name() {
    let repo = TestRepo::new();

    let out = zsh(
        &repo,
        &repo.main,
        r#"eval "$(wt init zsh)"; printf 'feat\nfix-typo\torigin\nlocal-only\tlocal\n' | _fzf_complete_wt_post"#,
    );

    assert_ok(&out);
    assert_eq!(stdout(&out), "feat\nfix-typo\nlocal-only\n");
}

#[test]
fn creating_triggers_complete_from_the_branches_without_a_worktree() {
    let repo = TestRepo::new();
    repo.add_worktree("feat");
    repo.git(&repo.main, &["branch", "local-only"]);
    // Stands in for fzf's helper, which reads the candidates on stdin.
    let complete = |lbuf: &str| {
        let out = zsh(
            &repo,
            &repo.main,
            &format!(
                r#"eval "$(wt init zsh)"; _fzf_complete() {{ cat }}; _fzf_complete_wt "{lbuf}""#
            ),
        );
        assert_ok(&out);
        stdout(&out)
    };

    for lbuf in ["wt new ", "wt sw -c ", "wt switch --create "] {
        assert_eq!(complete(lbuf), "local-only\tlocal\n", "{lbuf:?}");
    }
    assert_eq!(complete("wt sw "), "main\nfeat\nlocal-only\tlocal\n");
}

#[test]
fn exec_completes_branches_before_the_separator_and_defers_after_it() {
    let repo = TestRepo::new();
    repo.add_worktree("feat");
    repo.git(&repo.main, &["branch", "local-only"]);
    // Stand in for fzf's helper, which reads the candidates on stdin, and for zle's fallback.
    let complete = |lbuf: &str| {
        let out = zsh(
            &repo,
            &repo.main,
            &format!(
                r#"eval "$(wt init zsh)"; _fzf_complete() {{ cat }}; zle() {{ echo "zle $*" }}; _fzf_complete_wt "{lbuf}""#
            ),
        );
        assert_ok(&out);
        stdout(&out)
    };

    assert_eq!(complete("wt exec "), "main\nfeat\n");
    assert_eq!(
        complete("wt exec feat -- ./scr"),
        "zle expand-or-complete\n"
    );
}
