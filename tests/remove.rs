mod common;

use common::{TestRepo, assert_ok, assert_stderr_contains};

#[test]
fn removes_the_worktree_for_a_branch() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["remove", "feat"]);

    assert_ok(&out);
    assert!(!feat.exists());
    assert_stderr_contains(&out, &format!("Removed worktree: {}", feat.display()));
}

#[test]
fn fails_without_a_worktree_for_the_branch() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["remove", "nope"]);

    assert!(!out.status.success());
    assert_stderr_contains(&out, "No worktree found for branch 'nope'");
}

#[test]
fn refuses_a_worktree_with_untracked_files() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");
    std::fs::write(feat.join("scratch"), "").unwrap();

    let out = repo.wt(&repo.main, &["remove", "feat"]);

    assert!(!out.status.success());
    assert_stderr_contains(&out, "untracked files");
    assert!(feat.join("scratch").exists());
}

#[test]
fn rm_is_an_alias() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["rm", "feat"]);

    assert_ok(&out);
    assert!(!feat.exists());
}
