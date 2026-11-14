mod common;

use std::process::Command;

use common::{TestRepo, assert_ok, stdout};

#[test]
fn switch_offers_every_checked_out_branch() {
    let repo = TestRepo::new();
    repo.add_worktree("feat");

    let out = repo.wt(&repo.main, &["complete", "switch"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), "main\nfeat\n");
}

/// `toy()` plus a worktree on `feat`, a local-only `local-only`, and `both`, which is local and on
/// origin, neither checked out.
fn review_repo() -> TestRepo {
    let repo = TestRepo::toy();
    repo.add_worktree("feat");
    repo.git(&repo.main, &["branch", "local-only"]);
    repo.git(&repo.main, &["branch", "both"]);
    repo.git(&repo.main, &["push", "--quiet", "origin", "both"]);
    repo
}

#[test]
fn switch_also_offers_branches_without_a_worktree_marked_by_where_they_are() {
    let repo = review_repo();

    for subcommand in ["switch", "sw"] {
        let out = repo.wt(&repo.main, &["complete", subcommand]);

        assert_ok(&out);
        assert_eq!(
            stdout(&out),
            "main\nfeat\nboth\tlocal\nlocal-only\tlocal\nremote-feature\torigin\n",
            "complete {subcommand}"
        );
    }
}

#[test]
fn new_offers_only_the_branches_without_a_worktree() {
    let repo = review_repo();

    let out = repo.wt(&repo.main, &["complete", "new"]);

    assert_ok(&out);
    assert_eq!(
        stdout(&out),
        "both\tlocal\nlocal-only\tlocal\nremote-feature\torigin\n"
    );
}

#[test]
fn exec_offers_only_the_branches_with_a_worktree() {
    let repo = review_repo();

    let out = repo.wt(&repo.main, &["complete", "exec"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), "main\nfeat\n");
}

#[test]
fn remove_leaves_out_the_main_worktree() {
    let repo = TestRepo::new();
    repo.add_worktree("feat");

    for subcommand in ["remove", "rm"] {
        let out = repo.wt(&repo.main, &["complete", subcommand]);

        assert_ok(&out);
        assert_eq!(stdout(&out), "feat\n", "complete {subcommand}");
    }
}

/// `with_origin()` beside a worktree on `feat`, a plain `notes` directory and a local-only branch,
/// with the temp dir as the only project root.
fn projects() -> TestRepo {
    let repo = TestRepo::with_origin();
    repo.add_worktree("feat");
    repo.git(&repo.main, &["branch", "local-only"]);
    std::fs::create_dir(repo.root.join("notes")).unwrap();
    repo.projects_here();
    repo
}

#[test]
fn switch_outside_a_repo_offers_the_projects() {
    let repo = projects();

    let out = repo.wt(&repo.root, &["complete", "switch"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), "notes\nproj\n");
}

#[test]
fn switch_after_a_project_offers_its_branches_main_first() {
    let repo = projects();
    repo.git(&repo.main, &["branch", "aaa"]);

    let out = repo.wt(&repo.root, &["complete", "switch", "proj"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), "main\nfeat\naaa\tlocal\nlocal-only\tlocal\n");
}

#[test]
fn switch_offers_nothing_after_a_project_without_git_or_inside_a_repo_after_a_branch() {
    let repo = projects();

    for (dir, word) in [(&repo.root, "notes"), (&repo.main, "feat")] {
        let out = repo.wt(dir, &["complete", "switch", word]);

        assert_ok(&out);
        assert_eq!(stdout(&out), "", "complete switch {word}");
    }
}

#[test]
fn other_subcommands_still_need_a_repo() {
    let repo = projects();

    for subcommand in ["new", "exec", "remove"] {
        let out = repo.wt(&repo.root, &["complete", subcommand]);

        assert!(!out.status.success(), "complete {subcommand} succeeded");
        assert_eq!(stdout(&out), "", "complete {subcommand}");
    }
}

#[test]
fn a_reader_that_goes_away_is_a_clean_exit() {
    let repo = TestRepo::new();
    let (reader, writer) = std::io::pipe().unwrap();
    // fzf closing its end early, made deterministic by closing it before wt writes anything.
    drop(reader);

    let out = Command::new(env!("CARGO_BIN_EXE_wt"))
        .current_dir(&repo.main)
        .args(["complete", "switch"])
        .stdout(writer)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{stderr}");
    assert!(!stderr.contains("panicked"), "stderr:\n{stderr}");
}
