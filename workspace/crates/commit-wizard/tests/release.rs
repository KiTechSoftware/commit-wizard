//! End-to-end tests for `cw release`.

mod common;

use common::{Repo, text};

fn release_repo(push: bool) -> Repo {
    let repo = Repo::new();
    let config = repo.path().join(".cwizard.toml");
    let body = std::fs::read_to_string(&config)
        .unwrap()
        .replace("[release]\nenabled = false", "[release]\nenabled = true")
        .replace(
            "tag = true\npush = true",
            &format!("tag = true\npush = {push}"),
        );
    std::fs::write(&config, body).unwrap();
    repo.git(&["add", "-A"]);
    repo.commit("chore: enable releases");
    repo
}

#[test]
fn release_is_off_unless_enabled() {
    let repo = Repo::new();
    let out = repo.cw(&["release", "start", "--set-version", "1.0.0"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("enabled"), "{}", text(&out));
}

#[test]
fn start_creates_and_switches_to_the_release_branch() {
    let repo = release_repo(false);
    let out = repo.cw(&["release", "start", "--set-version", "1.2.3"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]),
        "release/1.2.3"
    );

    // A second start for the same version is refused.
    repo.git(&["checkout", "-q", "main"]);
    let again = repo.cw(&["release", "start", "--set-version", "1.2.3"]);
    assert!(!again.status.success(), "{}", text(&again));
}

#[test]
fn start_dry_run_changes_nothing() {
    let repo = release_repo(false);
    let out = repo.cw(&["release", "start", "--set-version", "1.0.0", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]), "main");
    assert!(
        !repo
            .git(&["branch", "--list", "release/1.0.0"])
            .contains("release")
    );
}

#[test]
fn start_refuses_a_dirty_worktree() {
    let repo = release_repo(false);
    std::fs::write(repo.path().join("dirty.txt"), "x").unwrap();
    let out = repo.cw(&["release", "start", "--set-version", "1.0.0"]);
    assert!(!out.status.success(), "{}", text(&out));
}

#[test]
fn finish_merges_tags_and_pushes() {
    let repo = release_repo(true);
    assert!(
        repo.cw(&["release", "start", "--set-version", "1.0.0"])
            .status
            .success()
    );
    repo.commit("fix: last minute fix");

    let out = repo.cw(&["release", "finish", "--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]), "main");
    assert!(
        repo.git(&["log", "--format=%s", "-1"])
            .contains("last minute fix")
    );
    assert!(
        repo.tags().contains(&"v1.0.0".to_string()),
        "{:?}",
        repo.tags()
    );
    assert!(repo.remote_tags().contains(&"v1.0.0".to_string()));
    assert!(repo.remote_branches().contains(&"main".to_string()));
}

#[test]
fn finish_outside_a_release_branch_needs_a_version() {
    let repo = release_repo(false);
    let out = repo.cw(&["release", "finish", "--yes"]);
    assert!(!out.status.success(), "{}", text(&out));
}
