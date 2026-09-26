mod common;

use common::{TestRepo, assert_ok, stdout};

#[test]
fn shows_each_worktree_with_its_branch() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["list"]);

    assert_ok(&out);
    let stdout = stdout(&out);
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(repo.main.to_str().unwrap()) && lines[0].ends_with("[main]"));
    assert!(lines[1].starts_with(feat.to_str().unwrap()) && lines[1].ends_with("[feat]"));
}

#[test]
fn matches_git_worktree_list() {
    let repo = TestRepo::new();
    repo.add_worktree("a-longer-branch-name");
    repo.add_worktree("b");

    assert_matches_git(&repo);
}

#[test]
fn marks_a_locked_worktree_like_git() {
    let repo = TestRepo::new();
    let locked = repo.add_worktree("locked");
    let reasoned = repo.add_worktree("reasoned");
    repo.add_worktree("free");
    repo.git(&repo.main, &["worktree", "lock", locked.to_str().unwrap()]);
    repo.git(
        &repo.main,
        &[
            "worktree",
            "lock",
            "--reason",
            "on a USB stick",
            reasoned.to_str().unwrap(),
        ],
    );

    assert_matches_git(&repo);
}

#[test]
fn marks_a_worktree_whose_directory_is_gone_as_prunable_like_git() {
    let repo = TestRepo::new();
    let gone = repo.add_worktree("gone");
    let gone_but_locked = repo.add_worktree("gone-but-locked");
    repo.add_worktree("present");
    repo.git(
        &repo.main,
        &["worktree", "lock", gone_but_locked.to_str().unwrap()],
    );
    std::fs::remove_dir_all(&gone).unwrap();
    std::fs::remove_dir_all(&gone_but_locked).unwrap();

    assert_matches_git(&repo);
}

#[test]
fn quotes_unusual_paths_like_git() {
    let repo = TestRepo::new();
    repo.add_worktree("日本語");
    repo.add_worktree("say-\"hi\"");
    repo.add_worktree("plain");

    assert_matches_git(&repo);
}

#[test]
fn pads_unquoted_paths_by_display_width_like_git() {
    let repo = TestRepo::new();
    repo.git(&repo.main, &["config", "core.quotePath", "false"]);
    repo.add_worktree("日本語");
    repo.add_worktree("cafe\u{301}");
    repo.add_worktree("plain");

    assert_matches_git(&repo);
}

#[track_caller]
fn assert_matches_git(repo: &TestRepo) {
    let out = repo.wt(&repo.main, &["list"]);

    assert_ok(&out);
    assert_eq!(
        stdout(&out).trim_end(),
        repo.git(&repo.main, &["worktree", "list"])
    );
}
