//! End-to-end tests for `cw push`, `cw tag --push` and signed tags.

mod common;

use common::{Repo, released, text};

/// Push validation reads the generated config's defaults, so the repo is put
/// on a feature branch with an invalid commit unless a test says otherwise.
fn repo_with_invalid_commit_on(branch: &str) -> Repo {
    let repo = Repo::new();
    if branch != "main" {
        repo.git(&["checkout", "-q", "-b", branch]);
    }
    repo.commit("not a conventional commit");
    repo
}

#[test]
fn push_to_protected_branch_is_blocked_with_nonzero_exit() {
    let repo = repo_with_invalid_commit_on("main");
    let out = repo.cw(&["push"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("blocked"), "{}", text(&out));
    assert!(repo.remote_branches().is_empty());
}

#[test]
fn force_overrides_a_blocked_push() {
    let repo = repo_with_invalid_commit_on("main");
    let out = repo.cw(&["push", "--force"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.remote_branches(), vec!["main".to_string()]);
}

#[test]
fn feature_branch_push_succeeds_by_default() {
    let repo = repo_with_invalid_commit_on("feat/x");
    let out = repo.cw(&["push"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.remote_branches(), vec!["feat/x".to_string()]);
}

#[test]
fn dry_run_push_pushes_nothing() {
    let repo = repo_with_invalid_commit_on("feat/x");
    let out = repo.cw(&["push", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("no push was performed"),
        "{}",
        text(&out)
    );
    assert!(repo.remote_branches().is_empty());
}

#[test]
fn tag_push_publishes_only_that_tag() {
    let repo = Repo::new();
    released(&repo, "1.0.0");
    let out = repo.tag(&["--set-version", "1.1.0", "--push"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.remote_tags(), vec!["v1.1.0".to_string()]);
}

#[test]
fn tagging_an_existing_version_reports_tag_exists() {
    let repo = Repo::new();
    released(&repo, "1.0.0");
    let out = repo.tag(&["--set-version", "1.0.0"]);
    assert!(!out.status.success());
    assert!(
        text(&out).contains("Release tag already exists"),
        "{}",
        text(&out)
    );
}

#[test]
fn sign_creates_a_verifiable_signed_tag() {
    let mut repo = Repo::new();
    if !repo.enable_signing() {
        eprintln!("skipping: gpg unavailable");
        return;
    }
    repo.commit("feat: signed");

    let out = repo.tag(&["--sign", "--set-version", "1.0.0"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(repo.tag_is_signed("v1.0.0"));

    let plain = repo.tag(&["--set-version", "1.0.1"]);
    assert!(plain.status.success(), "{}", text(&plain));
    assert!(!repo.tag_is_signed("v1.0.1"));
}
