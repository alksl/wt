mod common;

use common::{TestRepo, assert_ok, stderr};

#[test]
fn removes_the_worktree_for_a_branch() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["remove", "feat"]);

    assert_ok(&out);
    assert!(!feat.exists());
    assert!(
        stderr(&out).contains(&format!("Removed worktree: {}", feat.display())),
        "{}",
        stderr(&out)
    );
}

#[test]
fn fails_without_a_worktree_for_the_branch() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["remove", "nope"]);

    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("No worktree found for branch 'nope'"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn refuses_a_worktree_with_untracked_files() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");
    std::fs::write(feat.join("scratch"), "").unwrap();

    let out = repo.wt(&repo.main, &["remove", "feat"]);

    assert!(!out.status.success());
    assert!(stderr(&out).contains("untracked files"), "{}", stderr(&out));
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
