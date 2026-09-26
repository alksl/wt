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
