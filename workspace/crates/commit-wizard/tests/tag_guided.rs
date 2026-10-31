//! `cw tag` on a terminal asks about whatever the flags did not decide.
#![cfg(unix)]

mod common;

use common::{Pty, Repo};

/// v0.1.0 is tagged and one feature has landed since, so the proposal is v0.2.0.
fn ready() -> Repo {
    let repo = Repo::new();
    repo.commit("feat: first");
    repo.git(&["tag", "v0.1.0"]);
    repo.commit("feat: second");
    repo
}

fn tags(repo: &Repo) -> Vec<String> {
    let mut tags: Vec<String> = repo
        .git(&["tag", "--list"])
        .lines()
        .map(str::to_string)
        .collect();
    tags.sort();
    tags
}

fn tag_message(repo: &Repo, tag: &str) -> String {
    repo.git(&["tag", "-l", "--format=%(contents:subject)", tag])
}

/// Accept the proposed version, prefix and message; say no to everything optional.
fn accept_the_proposal(cw: &mut Pty) {
    cw.expect("Use the next version, v0.2.0?");
    cw.line("y");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("");
    sign_push_message_and_confirm(cw, "n", "n", "", "y");
}

fn sign_push_message_and_confirm(cw: &mut Pty, sign: &str, push: &str, message: &str, go: &str) {
    cw.expect("Sign the tag with GPG?");
    cw.line(sign);
    cw.expect("Push the tag after creating it?");
    cw.line(push);
    cw.expect("Tag message");
    cw.line(message);
    cw.expect("Create tag");
    cw.line(go);
}

#[test]
fn accepting_every_default_creates_the_proposed_tag() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);
    accept_the_proposal(&mut cw);

    assert!(cw.finish());
    assert_eq!(tags(&repo), ["v0.1.0", "v0.2.0"]);
    assert_eq!(tag_message(&repo, "v0.2.0"), "Release 0.2.0");
}

#[test]
fn a_different_version_can_be_entered() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("Use the next version, v0.2.0?");
    cw.line("n");
    cw.expect("Version to tag");
    cw.line("3.1.4");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("");
    sign_push_message_and_confirm(&mut cw, "n", "n", "", "y");

    assert!(cw.finish());
    assert!(
        tags(&repo).contains(&"v3.1.4".to_string()),
        "{:?}",
        tags(&repo)
    );
}

#[test]
fn a_prerelease_label_makes_a_numbered_release_candidate() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("Use the next version, v0.2.0?");
    cw.line("y");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("rc");
    sign_push_message_and_confirm(&mut cw, "n", "n", "", "y");

    assert!(cw.finish());
    assert!(
        tags(&repo).contains(&"v0.2.0-rc.1".to_string()),
        "{:?}",
        tags(&repo)
    );
}

#[test]
fn the_prefix_can_be_changed() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("Use the next version, v0.2.0?");
    cw.line("y");
    cw.expect("Tag prefix");
    // Replace the default (the prompt is pre-filled with "v").
    cw.send("\u{7f}");
    cw.line("rel-");
    cw.expect("Prerelease label");
    cw.line("");
    sign_push_message_and_confirm(&mut cw, "n", "n", "", "y");

    assert!(cw.finish());
    assert!(
        tags(&repo).contains(&"rel-0.2.0".to_string()),
        "{:?}",
        tags(&repo)
    );
}

#[test]
fn a_custom_message_is_used() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("Use the next version, v0.2.0?");
    cw.line("y");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("");
    cw.expect("Sign the tag with GPG?");
    cw.line("n");
    cw.expect("Push the tag after creating it?");
    cw.line("n");
    cw.expect("Tag message");
    // The default, "Release 0.2.0", is pre-filled: delete all 13 characters.
    cw.send(&"\u{7f}".repeat("Release 0.2.0".len()));
    cw.line("Spring release");
    cw.expect("Create tag");
    cw.line("y");

    assert!(cw.finish());
    assert_eq!(tag_message(&repo, "v0.2.0"), "Spring release");
}

#[test]
fn pushing_asks_for_the_remote_and_publishes_the_tag() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("Use the next version, v0.2.0?");
    cw.line("y");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("");
    cw.expect("Sign the tag with GPG?");
    cw.line("n");
    cw.expect("Push the tag after creating it?");
    cw.line("y");
    cw.expect("Remote to push to");
    cw.line("");
    cw.expect("Tag message");
    cw.line("");
    cw.expect("and push it to origin");
    cw.line("y");

    assert!(cw.finish());
    assert_eq!(repo.remote_tags(), ["v0.2.0"]);
}

#[test]
fn declining_the_final_question_creates_nothing() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("Use the next version, v0.2.0?");
    cw.line("y");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("");
    sign_push_message_and_confirm(&mut cw, "n", "n", "", "n");

    assert!(cw.finish(), "cancelling is not an error");
    assert_eq!(tags(&repo), ["v0.1.0"]);
}

#[test]
fn flags_skip_the_questions_they_answer() {
    let repo = ready();
    // Version, prefix and prerelease label are all given, so none is asked.
    let mut cw = repo.cw_pty(&[
        "tag",
        "--set-version",
        "5.0.0",
        "--prefix",
        "x",
        "--suffix",
        "beta",
    ]);

    sign_push_message_and_confirm(&mut cw, "n", "n", "", "y");

    assert!(cw.finish());
    assert_eq!(tags(&repo), ["v0.1.0", "x5.0.0-beta.1"]);
}

#[test]
fn a_taken_version_goes_straight_to_asking_for_another() {
    // Nothing new since v0.1.0, so the proposal would be the existing tag.
    let repo = Repo::new();
    repo.commit("feat: shipped");
    repo.git(&["tag", "v0.1.0"]);
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("already exists");
    cw.expect("Version to tag");
    cw.line("0.2.0");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("");
    sign_push_message_and_confirm(&mut cw, "n", "n", "", "y");

    assert!(cw.finish());
    assert!(
        tags(&repo).contains(&"v0.2.0".to_string()),
        "{:?}",
        tags(&repo)
    );
}

#[test]
fn an_invalid_version_is_rejected_before_anything_is_created() {
    let repo = ready();
    let mut cw = repo.cw_pty(&["tag"]);

    cw.expect("Use the next version, v0.2.0?");
    cw.line("n");
    cw.expect("Version to tag");
    cw.line("not-a-version");

    assert!(!cw.finish());
    assert_eq!(tags(&repo), ["v0.1.0"]);
}

#[test]
fn yes_and_non_interactive_keep_the_flag_driven_behaviour() {
    for flag in ["--yes", "--non-interactive"] {
        let repo = ready();
        // No questions: if one appeared, the process would wait and time out.
        let cw = repo.cw_pty(&["tag", flag]);

        assert!(cw.finish(), "{flag}");
        assert_eq!(tags(&repo), ["v0.1.0", "v0.2.0"], "{flag}");
    }
}

#[test]
fn answering_yes_to_signing_makes_a_verifiable_signed_tag() {
    let mut repo = Repo::new();
    if !repo.enable_signing() {
        eprintln!("skipping: gpg unavailable");
        return;
    }
    repo.commit("feat: first");
    repo.git(&["tag", "v0.1.0"]);
    repo.commit("feat: second");

    let mut cw = repo.cw_pty(&["tag"]);
    cw.expect("Use the next version, v0.2.0?");
    cw.line("y");
    cw.expect("Tag prefix");
    cw.line("");
    cw.expect("Prerelease label");
    cw.line("");
    sign_push_message_and_confirm(&mut cw, "y", "n", "", "y");

    assert!(cw.finish());
    assert!(repo.tag_is_signed("v0.2.0"));
}
