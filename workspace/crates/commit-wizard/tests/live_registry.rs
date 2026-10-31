//! Tests against the real public registry. They need network access, so they
//! are ignored by default:
//!
//!     cargo test --test live_registry -- --ignored

mod common;

use common::{Repo, text};

const URL: &str = "https://github.com/KiTechSoftware/commit-wizard-registry";
const TAG: &str = "v0.1.0";

/// `cw commit` against one registry section, selected with CLI flags.
fn commit(
    repo: &Repo,
    section: Option<&str>,
    git_ref: &str,
    ty: &str,
    msg: &str,
) -> std::process::Output {
    repo.write("f.txt", &format!("{ty}{msg}"));
    repo.git(&["add", "f.txt"]);
    let mut args = vec!["commit", "--registry", URL, "--registry-ref", git_ref];
    if let Some(section) = section {
        args.extend(["--registry-section", section]);
    }
    args.extend(["-t", ty, "-m", msg]);
    repo.cw(&args)
}

#[test]
#[ignore = "needs network"]
fn angular_section_provides_conventional_types() {
    let repo = Repo::bare();
    let ok = commit(&repo, Some("angular"), TAG, "ci", "add pipeline");
    assert!(ok.status.success(), "{}", text(&ok));
    assert_eq!(repo.subjects()[0], "ci: add pipeline");

    let rejected = commit(&repo, Some("angular"), TAG, "hotfix", "nope");
    assert!(!rejected.status.success(), "{}", text(&rejected));
}

#[test]
#[ignore = "needs network"]
fn gitmoji_section_adds_emojis_and_a_shorter_subject_limit() {
    let repo = Repo::bare();
    let ok = commit(&repo, Some("gitmoji"), TAG, "hotfix", "patch it");
    assert!(ok.status.success(), "{}", text(&ok));
    assert!(
        repo.subjects()[0].ends_with("hotfix: patch it"),
        "{:?}",
        repo.subjects()
    );
    assert!(
        repo.subjects()[0].len() > "hotfix: patch it".len(),
        "emoji prefix expected"
    );

    // 66 characters: over gitmoji's limit of 60, but fine for angular (100).
    let long = "x".repeat(66);
    let too_long = commit(&repo, Some("gitmoji"), TAG, "fix", &long);
    assert!(!too_long.status.success(), "{}", text(&too_long));
    let fine = commit(&repo, Some("angular"), TAG, "fix", &long);
    assert!(fine.status.success(), "{}", text(&fine));
}

#[test]
#[ignore = "needs network"]
fn minimal_and_root_sections_only_have_feat_and_fix() {
    let repo = Repo::bare();
    for section in [Some("minimal"), None] {
        let rejected = commit(&repo, section, TAG, "ci", "nope");
        assert!(
            !rejected.status.success(),
            "{section:?}: {}",
            text(&rejected)
        );
        let ok = commit(
            &repo,
            section,
            TAG,
            "fix",
            &format!("ok {}", section.unwrap_or("root")),
        );
        assert!(ok.status.success(), "{section:?}: {}", text(&ok));
    }
}

#[test]
#[ignore = "needs network"]
fn check_applies_the_section_rules_to_history() {
    let repo = Repo::bare();
    let ok = commit(&repo, Some("angular"), TAG, "ci", "add pipeline");
    assert!(ok.status.success(), "{}", text(&ok));

    let angular = repo.cw(&[
        "check",
        "--registry",
        URL,
        "--registry-ref",
        TAG,
        "--registry-section",
        "angular",
        "--tail",
        "1",
    ]);
    assert!(text(&angular).contains("valid: true"), "{}", text(&angular));

    let minimal = repo.cw(&[
        "check",
        "--registry",
        URL,
        "--registry-ref",
        TAG,
        "--registry-section",
        "minimal",
        "--tail",
        "1",
    ]);
    assert!(
        text(&minimal).contains("invalid_count: 1"),
        "{}",
        text(&minimal)
    );
}

#[test]
#[ignore = "needs network"]
fn project_config_can_select_the_registry_and_section() {
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        &format!(
            "[registry]\nuse = \"cw\"\nsection = \"gitmoji\"\n\n[registries.cw]\nurl = \"{URL}\"\nref = \"{TAG}\"\nsections = [\"angular\", \"gitmoji\", \"minimal\"]\n"
        ),
    );
    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);

    let out = repo.cw(&["commit", "-t", "hotfix", "-m", "from config"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
#[ignore = "needs network"]
fn doctor_records_the_resolved_commit() {
    let repo = Repo::bare();
    let ok = commit(&repo, Some("angular"), TAG, "ci", "x");
    assert!(ok.status.success(), "{}", text(&ok));

    let doctor = repo.cw(&[
        "doctor",
        "--registry",
        URL,
        "--registry-ref",
        TAG,
        "--registry-section",
        "angular",
    ]);
    let report = text(&doctor);
    assert!(report.contains("commit-wizard-registry"), "{report}");
    assert!(report.contains(&format!("registry_ref: {TAG}")), "{report}");
    let commit_line = report
        .lines()
        .find(|l| l.starts_with("resolved_commit: "))
        .unwrap_or("");
    assert_eq!(
        commit_line.len(),
        "resolved_commit: ".len() + 40,
        "{report}"
    );
}

#[test]
#[ignore = "needs network"]
fn branch_refs_load_and_sync() {
    let repo = Repo::bare();
    let first = commit(&repo, Some("angular"), "main", "ci", "branch ref");
    assert!(first.status.success(), "{}", text(&first));
    // A second run against the cached branch checks the remote for changes.
    let second = commit(&repo, Some("angular"), "main", "ci", "branch ref again");
    assert!(second.status.success(), "{}", text(&second));
}

#[test]
#[ignore = "needs network"]
fn a_missing_section_or_ref_warns_by_default_and_fails_when_required() {
    for (section, git_ref, expected) in [
        ("nope", TAG, "Registry section is missing"),
        ("angular", "v9.9.9", "Registry sync failed"),
    ] {
        let warn = Repo::bare();
        warn.write(
            ".cwizard.toml",
            &format!("[registry]\nuse = \"cw\"\nsection = \"{section}\"\n\n[registries.cw]\nurl = \"{URL}\"\nref = \"{git_ref}\"\n"),
        );
        warn.write("x.txt", "x");
        warn.git(&["add", "-A"]);
        let out = warn.cw(&["commit", "-t", "feat", "-m", "x"]);
        assert!(text(&out).contains(expected), "{}", text(&out));
        assert!(
            out.status.success(),
            "default must continue: {}",
            text(&out)
        );

        let hard = Repo::bare();
        hard.write(
            ".cwizard.toml",
            &format!("[registry]\nuse = \"cw\"\nsection = \"{section}\"\nrequired = true\n\n[registries.cw]\nurl = \"{URL}\"\nref = \"{git_ref}\"\n"),
        );
        hard.write("x.txt", "x");
        hard.git(&["add", "-A"]);
        let out = hard.cw(&["commit", "-t", "feat", "-m", "x"]);
        assert!(!out.status.success(), "required must fail: {}", text(&out));
        assert!(text(&out).contains(expected), "{}", text(&out));
    }
}
