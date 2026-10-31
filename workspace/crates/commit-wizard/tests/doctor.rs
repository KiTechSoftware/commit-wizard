//! End-to-end tests for `cw doctor` and `cw doctor fix`.

mod common;

use common::{Repo, text};

#[test]
fn doctor_fix_creates_a_missing_project_config() {
    let repo = Repo::bare();
    assert!(!repo.path().join(".cwizard.toml").exists());

    let out = repo.cw(&["doctor", "fix"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("Created project config"),
        "{}",
        text(&out)
    );
    assert!(repo.path().join(".cwizard.toml").exists());

    // The repaired config is usable.
    let check = repo.cw(&["config", "path"]);
    assert!(check.status.success(), "{}", text(&check));
}

#[test]
fn doctor_fix_leaves_an_existing_config_alone() {
    let repo = Repo::new();
    let before = std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap();

    let out = repo.cw(&["doctor", "fix"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("Project config present"),
        "{}",
        text(&out)
    );
    assert_eq!(
        std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap(),
        before
    );
}

#[test]
fn doctor_fix_dry_run_creates_nothing() {
    let repo = Repo::bare();
    let out = repo.cw(&["doctor", "fix", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("Would create project config"),
        "{}",
        text(&out)
    );
    assert!(!repo.path().join(".cwizard.toml").exists());
}

#[test]
fn doctor_fix_outside_a_repo_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cw"))
        .args(["doctor", "fix"])
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path())
        .env("APPDATA", home.path())
        .env("LOCALAPPDATA", home.path())
        .env("USERPROFILE", home.path())
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("Not inside a git repository"),
        "{}",
        text(&out)
    );
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn doctor_reports_whether_a_project_config_exists() {
    let with = Repo::new().cw(&["doctor"]);
    assert!(with.status.success(), "{}", text(&with));
    assert!(
        text(&with).contains("has_project_config: true"),
        "{}",
        text(&with)
    );

    let without = Repo::bare().cw(&["doctor"]);
    assert!(without.status.success(), "{}", text(&without));
    assert!(
        text(&without).contains("has_project_config: false"),
        "{}",
        text(&without)
    );
}
