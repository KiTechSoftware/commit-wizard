//! End-to-end tests for `cw bump` and `cw tag` against throwaway git repos.

mod common;

use common::{Repo, released, text};

#[test]
fn bump_levels() {
    let repo = Repo::new();
    released(&repo, "1.2.3");

    repo.commit("fix: patch change");
    assert_eq!(repo.next_tag(), "v1.2.4");

    repo.commit("feat: minor change");
    assert_eq!(repo.next_tag(), "v1.3.0");

    repo.commit("feat!: major change");
    assert_eq!(repo.next_tag(), "v2.0.0");
}

#[test]
fn breaking_change_footer_bumps_major() {
    let repo = Repo::new();
    released(&repo, "1.2.3");
    repo.commit_with_footer("feat: drop x", "BREAKING CHANGE: x is gone");
    assert_eq!(repo.next_tag(), "v2.0.0");
}

#[test]
fn tag_creates_the_computed_version() {
    let repo = Repo::new();
    released(&repo, "0.1.0");
    repo.commit("feat: add thing");

    let out = repo.tag(&[]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(repo.tags().contains(&"v0.2.0".to_string()));
}

#[test]
fn dry_run_tag_creates_nothing() {
    let repo = Repo::new();
    released(&repo, "0.1.0");
    repo.commit("fix: small");

    let out = repo.tag(&["--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("Would create tag: v0.1.1"));
    assert_eq!(repo.tags(), vec!["v0.1.0".to_string()]);
}

#[test]
fn bump_uses_highest_tag_when_tags_share_a_commit() {
    let repo = Repo::new();
    // Created back to back, so they share a timestamp and a commit.
    released(&repo, "0.1.0");
    released(&repo, "1.0.0");
    released(&repo, "1.1.0");
    repo.commit("fix: after tags");
    assert_eq!(repo.next_tag(), "v1.1.1");
}

#[test]
fn bump_ignores_tags_from_other_branches() {
    let repo = Repo::new();
    released(&repo, "1.0.0");

    repo.git(&["checkout", "-q", "-b", "other"]);
    repo.commit("feat: elsewhere");
    released(&repo, "9.0.0");

    repo.git(&["checkout", "-q", "main"]);
    repo.commit("fix: on main");
    assert_eq!(repo.next_tag(), "v1.0.1");
}

#[test]
fn suffix_creates_numbered_release_candidates() {
    let repo = Repo::new();
    released(&repo, "1.0.0");
    repo.commit("feat: next");

    for expected in ["v1.1.0-rc.1", "v1.1.0-rc.2"] {
        let out = repo.tag(&["--suffix", "rc"]);
        assert!(out.status.success(), "{}", text(&out));
        assert!(
            repo.tags().contains(&expected.to_string()),
            "{:?}",
            repo.tags()
        );
        repo.commit("fix: rc feedback");
    }
}

#[test]
fn set_version_accepts_prereleases_and_rejects_malformed_ones() {
    let repo = Repo::new();
    assert!(repo.tag(&["--set-version", "2.0.0-rc.1"]).status.success());
    assert!(repo.tags().contains(&"v2.0.0-rc.1".to_string()));

    let bad = repo.tag(&["--set-version", "2.0.0rc.1"]);
    assert!(!bad.status.success());
    assert!(!repo.tags().contains(&"v2.0.0rc.1".to_string()));
}

#[test]
fn prerelease_is_promoted_by_bump_and_tag() {
    let repo = Repo::new();
    released(&repo, "1.0.0");
    repo.commit("feat: next");
    assert!(repo.tag(&["--suffix", "rc"]).status.success());

    // No new commits: the rc is promoted to its release.
    assert_eq!(repo.next_tag(), "v1.1.0");
    let out = repo.tag(&[]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(repo.tags().contains(&"v1.1.0".to_string()));

    // The next cycle starts from the final release.
    repo.commit("fix: later");
    assert_eq!(repo.next_tag(), "v1.1.1");
}
