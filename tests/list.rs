mod common;

use common::{TestRepo, assert_ok, stdout};

#[test]
fn shows_each_worktree_with_its_branch() {
    let repo = TestRepo::new();
    let feat = repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["list"]);

    assert_ok(&out);
    let lines: Vec<_> = stdout(&out).lines().map(str::to_owned).collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(repo.main.to_str().unwrap()) && lines[0].ends_with("[main]"));
    assert!(lines[1].starts_with(feat.to_str().unwrap()) && lines[1].ends_with("[feat]"));
}

#[test]
fn matches_git_worktree_list() {
    let repo = TestRepo::new();
    repo.add_worktree("a-longer-branch-name");
    repo.add_worktree("b");

    let out = repo.wt(&repo.main, &["list"]);

    assert_ok(&out);
    assert_eq!(
        stdout(&out).trim_end(),
        repo.git(&repo.main, &["worktree", "list"])
    );
}
