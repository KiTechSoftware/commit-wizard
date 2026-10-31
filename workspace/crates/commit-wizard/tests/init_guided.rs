//! `cw init` on a terminal asks how the project works and writes a config to match.
#![cfg(unix)]

mod common;

use common::{Pty, Repo, text};

/// Backspaces enough to empty a pre-filled prompt of `len` characters.
fn clear(len: usize) -> String {
    "\u{7f}".repeat(len)
}

fn read(repo: &Repo, name: &str) -> toml::Value {
    let raw = std::fs::read_to_string(repo.path().join(name))
        .unwrap_or_else(|_| panic!("{name} was not written"));
    toml::from_str(&raw).unwrap_or_else(|e| panic!("{name} is not valid TOML ({e}):\n{raw}"))
}

fn at<'a>(doc: &'a toml::Value, path: &str) -> &'a toml::Value {
    path.split('.')
        .try_fold(doc, |value, key| value.get(key))
        .unwrap_or_else(|| panic!("no `{path}` in:\n{doc}"))
}

const PROFILE: &str = "Which profile fits your project?";
const FORMAT: &str = "Which file format do you prefer?";
const TYPES: &str = "Which commit types will you use?";
const SCOPES: &str = "How should commit scopes work?";
const BRANCHES: &str = "Which branches are protected?";
const CHECKED: &str = "Where should commit messages be checked?";
const PUSH: &str = "Allow pushing straight to a protected branch?";
const MORE: &str = "Customize more options?";

/// Answer every question with its default (the recommended choice).
fn defaults_through_the_basics(cw: &mut Pty) {
    cw.expect(PROFILE);
    cw.line("");
    cw.expect(FORMAT);
    cw.line("");
    cw.expect(TYPES);
    cw.line("");
    cw.expect(SCOPES);
    cw.line("");
    cw.expect(BRANCHES);
    cw.line("");
    cw.expect(CHECKED);
    cw.line("");
    cw.expect(PUSH);
    cw.line("n");
}

fn finish_without_extras(cw: &mut Pty) {
    cw.expect(MORE);
    cw.line("n");
    cw.expect("Create ");
    cw.line("y");
}

#[test]
fn accepting_every_default_writes_a_working_standard_config() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init"]);
    defaults_through_the_basics(&mut cw);
    finish_without_extras(&mut cw);
    assert!(cw.finish());

    let config = read(&repo, ".cwizard.toml");
    assert_eq!(at(&config, "commit.scopes.mode").as_str(), Some("optional"));
    assert_eq!(
        at(&config, "check.commits.enforce_on").as_str(),
        Some("protected_branches")
    );
    assert_eq!(at(&config, "push.allow.protected").as_bool(), Some(false));
    assert!(at(&config, "commit.types").get("feat").is_some());
    assert!(at(&config, "commit.types").get("chore").is_some());

    // The file is valid for cw itself.
    let doctor = repo.cw(&["doctor"]);
    assert!(text(&doctor).contains("errors: 0"), "{}", text(&doctor));
}

#[test]
fn the_scope_answer_changes_how_commits_are_checked() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init"]);
    cw.expect(PROFILE);
    cw.line("");
    cw.expect(FORMAT);
    cw.line("");
    cw.expect(TYPES);
    cw.line("");
    cw.expect(SCOPES);
    cw.line("Required");
    cw.expect(BRANCHES);
    cw.line("");
    cw.expect(CHECKED);
    cw.line("");
    cw.expect(PUSH);
    cw.line("n");
    finish_without_extras(&mut cw);
    assert!(cw.finish());

    assert_eq!(
        at(&read(&repo, ".cwizard.toml"), "commit.scopes.mode").as_str(),
        Some("required")
    );

    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);
    let without_scope = repo.cw(&["commit", "-t", "feat", "-m", "no scope"]);
    assert!(!without_scope.status.success(), "{}", text(&without_scope));
    let with_scope = repo.cw(&["commit", "-t", "feat", "-s", "core", "-m", "with scope"]);
    assert!(with_scope.status.success(), "{}", text(&with_scope));
}

#[test]
fn types_can_be_deselected_but_feat_and_fix_stay() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init"]);
    cw.expect(PROFILE);
    cw.line("");
    cw.expect(FORMAT);
    cw.line("");
    cw.expect(TYPES);
    cw.send(" "); // deselect the first entry, chore
    cw.line("");
    cw.expect(SCOPES);
    cw.line("");
    cw.expect(BRANCHES);
    cw.line("");
    cw.expect(CHECKED);
    cw.line("");
    cw.expect(PUSH);
    cw.line("n");
    finish_without_extras(&mut cw);
    assert!(cw.finish());

    let types = at(&read(&repo, ".cwizard.toml"), "commit.types").clone();
    assert!(types.get("chore").is_none(), "{types}");
    assert!(
        types.get("feat").is_some() && types.get("fix").is_some() && types.get("docs").is_some()
    );
}

#[test]
fn protected_branches_and_enforcement_follow_the_answers() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init"]);
    cw.expect(PROFILE);
    cw.line("");
    cw.expect(FORMAT);
    cw.line("");
    cw.expect(TYPES);
    cw.line("");
    cw.expect(SCOPES);
    cw.line("");
    cw.expect(BRANCHES);
    cw.send(&clear(40));
    cw.line("trunk, release/*");
    cw.expect(CHECKED);
    cw.line("Every branch");
    cw.expect(PUSH);
    cw.line("y");
    finish_without_extras(&mut cw);
    assert!(cw.finish());

    let config = read(&repo, ".cwizard.toml");
    let protected: Vec<&str> = at(&config, "branch.protected")
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(protected, ["trunk", "release/*"]);
    assert_eq!(
        at(&config, "check.commits.enforce_on").as_str(),
        Some("all_branches")
    );
    assert_eq!(at(&config, "push.allow.protected").as_bool(), Some(true));
}

#[test]
fn the_extra_questions_are_behind_a_gate_and_are_written_when_answered() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init"]);
    defaults_through_the_basics(&mut cw);
    cw.expect(MORE);
    cw.line("y");
    cw.expect("Maximum length of a commit subject");
    cw.send(&clear(4));
    cw.line("50");
    cw.expect("Put an emoji before each commit type?");
    cw.line("y");
    cw.expect("Require a description for breaking changes?");
    cw.line("y");
    cw.expect("Require a ticket reference in commits?");
    cw.line("y");
    cw.expect("Tag prefix");
    cw.send(&clear(2));
    cw.line("rel-");
    cw.expect("Changelog file");
    cw.send(&clear(20));
    cw.line("docs/CHANGES.md");
    cw.expect("Create ");
    cw.line("y");
    assert!(cw.finish());

    let config = read(&repo, ".cwizard.toml");
    assert_eq!(
        at(&config, "commit.subject_max_length").as_integer(),
        Some(50)
    );
    assert_eq!(at(&config, "commit.use_emojis").as_bool(), Some(true));
    assert_eq!(
        at(&config, "commit.breaking.require_footer").as_bool(),
        Some(true)
    );
    assert_eq!(at(&config, "commit.ticket.required").as_bool(), Some(true));
    assert_eq!(at(&config, "versioning.tag_prefix").as_str(), Some("rel-"));
    assert_eq!(
        at(&config, "changelog.output").as_str(),
        Some("docs/CHANGES.md")
    );
}

#[test]
fn flags_answer_their_own_questions() {
    // The profile and format are given, so neither is asked, and minimal has no
    // commit types to choose from.
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init", "--minimal", "--file-format", "yaml"]);
    cw.expect(SCOPES);
    cw.line("Not used");
    cw.expect(BRANCHES);
    cw.line("");
    cw.expect(CHECKED);
    cw.line("");
    cw.expect(PUSH);
    cw.line("n");
    finish_without_extras(&mut cw);
    assert!(cw.finish());

    let raw = std::fs::read_to_string(repo.path().join(".cwizard.yaml")).unwrap();
    assert!(raw.contains("mode: disabled"), "{raw}");
    assert!(!repo.path().join(".cwizard.toml").exists());
}

#[test]
fn the_format_question_picks_the_file_type() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init"]);
    cw.expect(PROFILE);
    cw.line("");
    cw.expect(FORMAT);
    cw.line("YAML");
    cw.expect(TYPES);
    cw.line("");
    cw.expect(SCOPES);
    cw.line("");
    cw.expect(BRANCHES);
    cw.line("");
    cw.expect(CHECKED);
    cw.line("");
    cw.expect(PUSH);
    cw.line("n");
    finish_without_extras(&mut cw);
    assert!(cw.finish());

    assert!(repo.path().join(".cwizard.yaml").exists());
    assert!(!repo.path().join(".cwizard.toml").exists());
}

#[test]
fn declining_the_last_question_writes_nothing() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init"]);
    defaults_through_the_basics(&mut cw);
    cw.expect(MORE);
    cw.line("n");
    cw.expect("Create ");
    cw.line("n");

    assert!(cw.finish(), "backing out is not an error");
    assert!(!repo.path().join(".cwizard.toml").exists());
}

#[test]
fn an_existing_file_is_only_replaced_when_the_person_agrees() {
    let repo = Repo::bare();
    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 11\n");

    let mut cw = repo.cw_pty(&["init"]);
    cw.expect(PROFILE);
    cw.line("");
    cw.expect(FORMAT);
    cw.line("");
    cw.expect("already exists. Replace it?");
    cw.line("n");
    assert!(cw.finish());
    let kept = std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap();
    assert!(kept.contains("= 11"), "the file was changed: {kept}");

    let mut cw = repo.cw_pty(&["init"]);
    cw.expect(PROFILE);
    cw.line("");
    cw.expect(FORMAT);
    cw.line("");
    cw.expect("already exists. Replace it?");
    cw.line("y");
    cw.expect(TYPES);
    cw.line("");
    cw.expect(SCOPES);
    cw.line("");
    cw.expect(BRANCHES);
    cw.line("");
    cw.expect(CHECKED);
    cw.line("");
    cw.expect(PUSH);
    cw.line("n");
    finish_without_extras(&mut cw);
    assert!(cw.finish());
    assert!(
        !std::fs::read_to_string(repo.path().join(".cwizard.toml"))
            .unwrap()
            .contains("= 11")
    );
}

#[test]
fn the_global_config_can_be_created_the_same_way() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init", "config", "--global"]);
    defaults_through_the_basics(&mut cw);
    finish_without_extras(&mut cw);
    assert!(cw.finish());

    let global = repo.home_path().join("cwizard").join("config.toml");
    assert!(global.exists(), "{}", global.display());
}

#[test]
fn dry_run_asks_but_writes_nothing() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init", "--dry-run"]);
    defaults_through_the_basics(&mut cw);
    finish_without_extras(&mut cw);
    assert!(cw.finish());
    assert!(!repo.path().join(".cwizard.toml").exists());
}

#[test]
fn yes_and_non_interactive_skip_the_wizard_and_use_the_profile() {
    for flag in ["--yes", "--non-interactive"] {
        let repo = Repo::bare();
        let cw = repo.cw_pty(&["init", flag]);

        assert!(cw.finish(), "{flag}");
        let config = read(&repo, ".cwizard.toml");
        assert_eq!(
            at(&config, "commit.scopes.mode").as_str(),
            Some("optional"),
            "{flag}"
        );
    }
}
