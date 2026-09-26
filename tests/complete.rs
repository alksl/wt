mod common;

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
