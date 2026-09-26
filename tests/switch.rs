mod common;

use common::{TestRepo, assert_ok, stderr, stdout};

#[test]
fn prints_only_the_worktree_path() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["sw", "feat"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", feat.display()));
}

#[test]
fn main_branch_resolves_to_the_main_worktree() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&feat, &["sw", "main"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", repo.main.display()));
}

#[test]
fn create_keeps_stdout_to_the_path() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["sw", "-c", "feat"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", repo.sibling("feat").display()));
}

#[test]
fn create_has_a_long_form() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["switch", "--create", "feat"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", repo.sibling("feat").display()));
}

#[test]
fn fails_without_a_worktree() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["sw", "nope"]);

    assert!(!out.status.success());
    assert_eq!(stdout(&out), "");
    assert!(stderr(&out).contains("No worktree found for branch 'nope'"));
}
