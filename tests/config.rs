//! `~/.wt.toml`, as `wt sw <project>` sees it from outside any repository.

mod common;

use common::{TestRepo, assert_ok, assert_stderr_contains, stdout};

#[test]
fn without_a_file_projects_live_in_code() {
    let repo = TestRepo::new();
    let project = repo.root.join("code/x");
    std::fs::create_dir_all(&project).unwrap();

    let out = repo.wt(&repo.root, &["sw", "x"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", project.display()));
}

#[test]
fn a_leading_tilde_is_home() {
    let repo = TestRepo::new();
    let project = repo.root.join("work/y");
    std::fs::create_dir_all(&project).unwrap();
    repo.config("projects = [\"~/work\"]\n");

    let out = repo.wt(&repo.root, &["sw", "y"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", project.display()));
}

#[test]
fn a_root_that_does_not_exist_is_skipped() {
    let repo = TestRepo::new();
    let project = repo.root.join("work/y");
    std::fs::create_dir_all(&project).unwrap();
    repo.config("projects = [\"~/missing\", \"~/work\"]\n");

    let out = repo.wt(&repo.root, &["sw", "y"]);

    assert_ok(&out);
    assert_eq!(stdout(&out), format!("{}\n", project.display()));
}

#[test]
fn a_malformed_file_fails_naming_it() {
    let repo = TestRepo::new();
    repo.config("projects = [\n");

    let out = repo.wt(&repo.root, &["sw", "proj"]);

    assert!(!out.status.success(), "a malformed config was accepted");
    assert_eq!(stdout(&out), "");
    assert_stderr_contains(&out, &repo.root.join(".wt.toml").display().to_string());
}
