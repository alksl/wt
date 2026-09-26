mod common;

use common::{TestRepo, assert_ok, stderr};

#[test]
fn removes_a_worktree_whose_commits_are_merged() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");
    repo.commit(&feat, "work");
    repo.git(&repo.main, &["merge", "--quiet", "--ff-only", "feat"]);

    let out = repo.wt(&repo.main, &["prune"]);

    assert_ok(&out);
    assert!(!feat.exists());
    let expected = format!("Removed merged worktree: {} (branch: feat)", feat.display());
    assert!(stderr(&out).contains(&expected), "{}", stderr(&out));
}

#[test]
fn keeps_a_worktree_with_unmerged_commits() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");
    repo.commit(&feat, "work");

    let out = repo.wt(&repo.main, &["prune"]);

    assert_ok(&out);
    assert!(feat.exists());
    assert!(
        stderr(&out).contains("No merged worktrees found"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn keeps_a_fresh_branch_that_is_trivially_an_ancestor() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["prune"]);

    assert_ok(&out);
    assert!(feat.exists());
    let expected = format!(
        "Keeping worktree with no commits of its own: {} (branch: feat)",
        feat.display()
    );
    assert!(stderr(&out).contains(&expected), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("No merged worktrees found"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn keeps_a_merged_branch_whose_reflog_is_missing() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");
    repo.commit(&feat, "work");
    repo.git(&repo.main, &["merge", "--quiet", "--ff-only", "feat"]);
    std::fs::remove_file(repo.main.join(".git/logs/refs/heads/feat")).unwrap();

    let out = repo.wt(&repo.main, &["prune"]);

    assert_ok(&out);
    assert!(feat.exists());
    assert!(
        stderr(&out).contains("Keeping worktree with no commits of its own"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_refused_removal_fails_after_removing_the_rest() {
    let repo = TestRepo::new();
    let dirty = repo.add_worktree("dirty");
    repo.commit(&dirty, "dirty work");
    repo.git(&repo.main, &["merge", "--quiet", "--ff-only", "dirty"]);
    let clean = repo.add_worktree("zclean");
    repo.commit(&clean, "clean work");
    repo.git(&repo.main, &["merge", "--quiet", "--ff-only", "zclean"]);
    std::fs::write(dirty.join("scratch"), "").unwrap();

    let out = repo.wt(&repo.main, &["prune"]);

    assert!(!out.status.success());
    assert!(dirty.join("scratch").exists());
    assert!(!clean.exists());
    let expected = format!(
        "Could not remove worktree: {} (branch: dirty)",
        dirty.display()
    );
    assert!(stderr(&out).contains(&expected), "{}", stderr(&out));
    assert!(
        !stderr(&out).contains("No merged worktrees found"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn uses_origin_head_as_the_default_branch() {
    let repo = TestRepo::with_origin();
    repo.git(&repo.main, &["branch", "-m", "main", "trunk"]);
    repo.git(&repo.main, &["push", "--quiet", "origin", "trunk"]);
    repo.git(&repo.main, &["remote", "set-head", "origin", "trunk"]);
    let feat = repo.add_worktree("feat");
    repo.commit(&feat, "work");
    repo.git(&repo.main, &["merge", "--quiet", "--ff-only", "feat"]);

    let out = repo.wt(&repo.main, &["prune"]);

    assert_ok(&out);
    assert!(!feat.exists());
    assert!(
        stderr(&out).contains("Using default branch: trunk"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn falls_back_to_the_main_worktrees_branch_whatever_its_name() {
    let repo = TestRepo::new();
    repo.git(&repo.main, &["branch", "-m", "main", "trunk"]);
    let feat = repo.add_worktree("feat");
    repo.commit(&feat, "work");
    repo.git(&repo.main, &["merge", "--quiet", "--ff-only", "feat"]);

    let out = repo.wt(&repo.main, &["prune"]);

    assert_ok(&out);
    assert!(!feat.exists());
    assert!(
        stderr(&out).contains("Using default branch: trunk"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn fails_without_origin_head_and_with_a_detached_main_worktree() {
    let repo = TestRepo::new();
    repo.git(&repo.main, &["checkout", "--quiet", "--detach"]);

    let out = repo.wt(&repo.main, &["prune"]);

    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("Could not determine default branch"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn never_removes_the_worktree_holding_the_default_branch() {
    let repo = TestRepo::with_origin();
    repo.git(&repo.main, &["checkout", "--quiet", "-b", "hotfix"]);
    let default = repo.sibling("main");
    repo.git(
        &repo.main,
        &[
            "worktree",
            "add",
            "--quiet",
            default.to_str().unwrap(),
            "main",
        ],
    );
    repo.commit(&default, "work on the default branch");

    let out = repo.wt(&repo.main, &["prune"]);

    assert_ok(&out);
    assert!(default.exists());
}
