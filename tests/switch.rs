mod common;

use common::{TestRepo, assert_ok, assert_stderr_contains, stdout};

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
    assert_stderr_contains(&out, "No worktree found for branch 'nope'");
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
    assert_stderr_contains(&out, "No worktree found for branch 'master'");
}

#[test]
fn creates_the_worktree_for_a_branch_only_origin_has() {
    let repo = TestRepo::toy();

    let out = repo.wt(&repo.main, &["sw", "remote-feature"]);

    assert_ok(&out);
    let path = repo.sibling("remote-feature");
    assert_eq!(stdout(&out), format!("{}\n", path.display()));
    assert_eq!(
        repo.git(&path, &["rev-parse", "--abbrev-ref", "@{upstream}"]),
        "origin/remote-feature"
    );
    assert_stderr_contains(&out, "tracking origin/remote-feature");
}

#[test]
fn creates_the_worktree_for_a_local_branch_without_one() {
    let repo = TestRepo::new();
    repo.git(&repo.main, &["branch", "feat"]);
    repo.commit(&repo.main, "after feat");
    let feat_tip = repo.git(&repo.main, &["rev-parse", "feat"]);

    let out = repo.wt(&repo.main, &["sw", "feat"]);

    assert_ok(&out);
    let path = repo.sibling("feat");
    assert_eq!(stdout(&out), format!("{}\n", path.display()));
    assert_eq!(repo.git(&path, &["rev-parse", "HEAD"]), feat_tip);
    assert_stderr_contains(&out, "Using existing branch 'feat'");
}
