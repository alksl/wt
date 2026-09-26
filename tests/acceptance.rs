//! Whole workflows on a toy project, the way they're used day to day, checking the files on disk
//! and git's own view after each step.

mod common;

use std::path::{Path, PathBuf};

use common::{TestRepo, assert_ok, stderr, stdout};

fn wt_ok(repo: &TestRepo, dir: &Path, args: &[&str]) -> String {
    let out = repo.wt(dir, args);
    assert_ok(&out);
    stdout(&out)
}

/// What `git worktree list` knows, as (path, branch) pairs.
fn git_worktrees(repo: &TestRepo) -> Vec<(PathBuf, String)> {
    let porcelain = repo.git(&repo.main, &["worktree", "list", "--porcelain"]);
    let mut worktrees = Vec::new();
    let mut path = PathBuf::new();
    for line in porcelain.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            path = PathBuf::from(p);
        } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
            worktrees.push((path.clone(), b.to_string()));
        }
    }
    worktrees
}

fn branch_exists(repo: &TestRepo, branch: &str) -> bool {
    !repo
        .git(&repo.main, &["branch", "--list", branch])
        .is_empty()
}

#[test]
fn a_feature_from_start_to_prune() {
    let repo = TestRepo::toy();

    let path = wt_ok(&repo, &repo.main, &["switch", "--create", "login"]);
    let feature = repo.sibling("login");
    assert_eq!(path, format!("{}\n", feature.display()));
    assert!(
        feature.join("src/lib.rs").is_file(),
        "the checkout has the project's files"
    );

    repo.write(&feature, "src/login.rs", "pub fn login() {}\n");
    repo.git(&feature, &["add", "."]);
    repo.git(&feature, &["commit", "--quiet", "-m", "Add login"]);

    let list = wt_ok(&repo, &feature, &["list"]);
    assert_eq!(list.trim_end(), repo.git(&repo.main, &["worktree", "list"]));
    assert_eq!(wt_ok(&repo, &feature, &["complete", "remove"]), "login\n");

    let out = repo.wt(&repo.main, &["prune"]);
    assert_ok(&out);
    assert!(feature.exists(), "unmerged work is kept");

    repo.git(&repo.main, &["merge", "--quiet", "--ff-only", "login"]);
    let out = repo.wt(&repo.main, &["prune"]);
    assert_ok(&out);
    assert!(!feature.exists(), "merged work is pruned");
    assert!(repo.main.join("src/login.rs").is_file());
    assert_eq!(
        git_worktrees(&repo),
        vec![(repo.main.clone(), "main".to_string())]
    );
    assert!(
        branch_exists(&repo, "login"),
        "prune leaves the branch itself behind"
    );
}

#[test]
fn picking_up_a_branch_that_only_origin_has() {
    let repo = TestRepo::toy();
    assert!(!branch_exists(&repo, "remote-feature"));

    let path = wt_ok(&repo, &repo.main, &["sw", "-c", "remote-feature"]);
    let feature = repo.sibling("remote-feature");
    assert_eq!(path, format!("{}\n", feature.display()));
    assert!(
        feature.join("src/feature.rs").is_file(),
        "the checkout has origin's commits"
    );
    assert_eq!(
        repo.git(&feature, &["rev-parse", "--abbrev-ref", "@{upstream}"]),
        "origin/remote-feature"
    );

    assert_eq!(
        wt_ok(&repo, &feature, &["sw", "main"]),
        format!("{}\n", repo.main.display())
    );

    wt_ok(&repo, &repo.main, &["remove", "remote-feature"]);
    assert!(!feature.exists());
    assert_eq!(
        git_worktrees(&repo),
        vec![(repo.main.clone(), "main".to_string())]
    );
}

#[test]
fn working_from_inside_a_linked_worktree() {
    let repo = TestRepo::toy();
    let first = PathBuf::from(wt_ok(&repo, &repo.main, &["sw", "-c", "first"]).trim_end());

    let second = PathBuf::from(wt_ok(&repo, &first, &["sw", "-c", "second"]).trim_end());
    assert_eq!(
        second,
        repo.sibling("second"),
        "named after the main worktree, not this one"
    );
    assert_eq!(
        wt_ok(&repo, &second, &["sw", "first"]),
        format!("{}\n", first.display())
    );
    assert_eq!(
        wt_ok(&repo, &second, &["complete", "switch"]),
        "main\nfirst\nsecond\n"
    );

    repo.write(&first, "notes.txt", "scratch\n");
    let out = repo.wt(&second, &["rm", "first"]);
    assert!(
        !out.status.success(),
        "a worktree with untracked files is refused"
    );
    assert!(first.join("notes.txt").is_file(), "{}", stderr(&out));

    std::fs::remove_file(first.join("notes.txt")).unwrap();
    wt_ok(&repo, &second, &["rm", "first"]);
    assert!(!first.exists());
    assert_eq!(
        git_worktrees(&repo),
        vec![
            (repo.main.clone(), "main".to_string()),
            (second, "second".to_string())
        ]
    );
}
