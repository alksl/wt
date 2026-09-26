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
    assert_eq!(
        stdout(&out),
        format!("{}\n", repo.sibling("feat").display())
    );
}

#[test]
fn create_has_a_long_form() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["switch", "--create", "feat"]);

    assert_ok(&out);
    assert_eq!(
        stdout(&out),
        format!("{}\n", repo.sibling("feat").display())
    );
}

#[test]
fn fails_without_a_worktree() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["sw", "nope"]);

    assert!(!out.status.success());
    assert_eq!(stdout(&out), "");
    assert!(stderr(&out).contains("No worktree found for branch 'nope'"));
}

#[test]
fn the_default_branch_resolves_to_the_main_worktree_whatever_its_name() {
    let repo = TestRepo::with_origin();
    repo.git(&repo.main, &["branch", "--quiet", "trunk"]);
    repo.git(&repo.main, &["push", "--quiet", "origin", "trunk"]);
    repo.git(&repo.main, &["remote", "set-head", "origin", "trunk"]);
    repo.git(&repo.main, &["checkout", "--quiet", "-b", "hotfix"]);

    let out = repo.wt(&repo.main, &["sw", "trunk"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", repo.main.display()));
}

#[test]
fn master_is_not_special() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["sw", "master"]);

    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("No worktree found for branch 'master'"),
        "{}",
        stderr(&out)
    );
}
