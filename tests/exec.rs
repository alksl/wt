mod common;

use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;

use common::{TestRepo, assert_ok, assert_stderr_contains, stderr, stdout};

#[test]
fn runs_in_the_named_branchs_worktree() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["exec", "feat", "--", "pwd"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", feat.display()));
    assert_eq!(stderr(&out), "");
}

#[test]
fn a_relative_command_is_relative_to_the_worktree() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");
    repo.write(&feat, "script/test", "#!/bin/sh\necho \"test in $(pwd)\"\n");
    let script = feat.join("script/test");
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

    let out = repo.wt(&repo.main, &["exec", "feat", "--", "./script/test"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("test in {}\n", feat.display()));
}

#[test]
fn pwd_in_the_environment_names_the_worktree() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["exec", "feat", "--", "printenv", "PWD"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", feat.display()));
}

#[test]
fn without_a_branch_runs_at_the_root_of_the_current_worktree() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    for root in [&repo.main, &feat] {
        let sub = root.join("deep/sub");
        std::fs::create_dir_all(&sub).unwrap();

        let out = repo.wt(&sub, &["exec", "--", "pwd"]);

        assert_ok(&out);
        assert_eq!(stdout(&out), format!("{}\n", root.display()));
    }
}

#[test]
fn main_runs_in_the_main_checkout() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&feat, &["exec", "main", "--", "pwd"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", repo.main.display()));
}

#[test]
fn exits_with_the_commands_status() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["exec", "--", "sh", "-c", "exit 3"]);

    assert_eq!(out.status.code(), Some(3));
    assert_eq!(stderr(&out), "");
}

#[test]
fn dies_by_the_commands_signal() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["exec", "--", "sh", "-c", "kill -TERM $$"]);

    assert_eq!(out.status.signal(), Some(15), "{:?}", out.status);
}

#[test]
fn arguments_after_the_separator_reach_the_command_untouched() {
    let repo = TestRepo::new();

    let out = repo.wt(
        &repo.main,
        &[
            "exec",
            "--",
            "sh",
            "-c",
            r#"printf '%s\n' "$@""#,
            "sh",
            "--help",
            "-c",
            "--",
        ],
    );

    assert_ok(&out);
    assert_eq!(stdout(&out), "--help\n-c\n--\n");
}

#[test]
fn a_command_is_required() {
    let repo = TestRepo::new();
    repo.add_worktree("feat");

    for args in [&["exec", "feat"][..], &["exec", "feat", "--"], &["exec"]] {
        let out = repo.wt(&repo.main, args);

        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
        assert_eq!(stdout(&out), "", "{args:?}");
    }
}

#[test]
fn a_branch_without_a_worktree_fails_without_running_anything() {
    let repo = TestRepo::new();
    repo.git(&repo.main, &["branch", "idle"]);

    for branch in ["nope", "idle"] {
        let out = repo.wt(&repo.main, &["exec", branch, "--", "touch", "ran"]);

        assert!(!out.status.success(), "wt exec {branch} succeeded");
        assert_eq!(stdout(&out), "");
        assert_stderr_contains(&out, &format!("No worktree found for branch '{branch}'"));
        assert!(!repo.main.join("ran").exists(), "{branch}: the command ran");
        assert!(
            !repo.sibling(branch).exists(),
            "{branch}: a worktree was made"
        );
    }
}

#[test]
fn a_missing_command_fails_like_a_shell() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["exec", "--", "no-such-command", "arg"]);

    assert_eq!(out.status.code(), Some(127), "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
    assert_stderr_contains(&out, "no-such-command");
}

#[test]
fn a_command_that_is_not_executable_fails() {
    let repo = TestRepo::new();
    repo.write(&repo.main, "script/test", "#!/bin/sh\n");

    let out = repo.wt(&repo.main, &["exec", "--", "./script/test"]);

    assert_eq!(out.status.code(), Some(126), "{}", stderr(&out));
    assert_stderr_contains(&out, "./script/test");
}
