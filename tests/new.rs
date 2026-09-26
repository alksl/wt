mod common;

use common::{TestRepo, assert_ok, stderr, stdout};

#[test]
fn creates_the_branch_from_head_in_a_sibling_directory() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["new", "feat"]);

    assert_ok(&out);
    let path = repo.sibling("feat");
    assert_eq!(repo.git(&path, &["branch", "--show-current"]), "feat");
    assert_eq!(
        repo.git(&path, &["rev-parse", "HEAD"]),
        repo.git(&repo.main, &["rev-parse", "HEAD"])
    );
}

#[test]
fn keeps_stdout_empty() {
    let repo = TestRepo::new();

    let out = repo.wt(&repo.main, &["new", "feat"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), "");
}

#[test]
fn names_the_directory_after_the_main_worktree_from_a_linked_one() {
    let repo = TestRepo::new();
    let first = repo.add_worktree("first");

    let out = repo.wt(&first, &["new", "second"]);

    assert_ok(&out);
    assert!(repo.sibling("second").is_dir());
}

#[test]
fn uses_an_existing_local_branch() {
    let repo = TestRepo::new();
    repo.git(&repo.main, &["branch", "feat"]);
    repo.commit(&repo.main, "after feat");
    let feat_tip = repo.git(&repo.main, &["rev-parse", "feat"]);

    let out = repo.wt(&repo.main, &["new", "feat"]);

    assert_ok(&out);
    assert_eq!(
        repo.git(&repo.sibling("feat"), &["rev-parse", "HEAD"]),
        feat_tip
    );
}

#[test]
fn tracks_the_remote_branch_when_there_is_no_local_one() {
    let repo = TestRepo::with_origin();
    let other = repo.add_worktree("remote-only");
    repo.commit(&other, "on the remote");
    repo.git(&other, &["push", "--quiet", "origin", "remote-only"]);
    repo.git(&repo.main, &["worktree", "remove", other.to_str().unwrap()]);
    repo.git(&repo.main, &["branch", "-D", "--quiet", "remote-only"]);

    let out = repo.wt(&repo.main, &["new", "remote-only"]);

    assert_ok(&out);
    let path = repo.sibling("remote-only");
    assert_eq!(
        repo.git(&path, &["rev-parse", "--abbrev-ref", "@{upstream}"]),
        "origin/remote-only"
    );
}

#[test]
fn refuses_a_branch_that_already_has_a_worktree() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["new", "feat"]);

    assert!(!out.status.success());
    assert!(stderr(&out).contains(feat.to_str().unwrap()));
}

#[test]
fn refuses_when_the_target_directory_exists() {
    let repo = TestRepo::new();
    std::fs::create_dir(repo.sibling("feat")).unwrap();

    let out = repo.wt(&repo.main, &["new", "feat"]);

    assert!(!out.status.success());
    assert!(stderr(&out).contains("Target directory already exists"));
    assert_eq!(repo.git(&repo.main, &["branch", "--list", "feat"]), "");
}
