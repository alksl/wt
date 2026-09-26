mod common;

use common::{TestRepo, assert_ok, stdout};

#[test]
fn help_lists_the_commands_with_their_aliases() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["--help"]);

    assert_ok(&out);
    let help = stdout(&out);
    for line in ["new", "switch", "[alias: sw]", "remove", "[alias: rm]", "list", "prune"] {
        assert!(help.contains(line), "missing {line:?} in:\n{help}");
    }
    assert!(!help.contains("complete"), "complete should be hidden:\n{help}");
}

#[test]
fn switch_takes_create_as_short_and_long_flag() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["switch", "--help"]);

    assert_ok(&out);
    assert!(stdout(&out).contains("-c, --create"));
}
