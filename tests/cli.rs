mod common;

use common::{TestRepo, assert_ok, assert_stderr_contains, stderr, stdout};

#[test]
fn help_lists_the_commands_with_their_aliases() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["--help"]);

    assert_ok(&out);
    let help = stdout(&out);
    for line in [
        "new",
        "switch",
        "[alias: sw]",
        "remove",
        "[alias: rm]",
        "list",
        "prune",
    ] {
        assert!(help.contains(line), "missing {line:?} in:\n{help}");
    }
    assert!(
        !help.contains("complete"),
        "complete should be hidden:\n{help}"
    );
}

#[test]
fn switch_takes_create_as_short_and_long_flag() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["switch", "--help"]);

    assert_ok(&out);
    assert!(stdout(&out).contains("-c, --create"));
}

#[test]
fn logs_without_colour_when_stderr_is_not_a_terminal() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["sw", "nope"]);

    assert_stderr_contains(&out, "No worktree found");
    assert!(!stderr(&out).contains('\x1b'), "{:?}", stderr(&out));
}

#[test]
fn logs_in_colour_when_forced() {
    let repo = TestRepo::new();

    let out = repo.wt_env(&repo.main, &["sw", "nope"], &[("CLICOLOR_FORCE", "1")]);

    assert!(stderr(&out).contains("\x1b["), "{:?}", stderr(&out));
}
