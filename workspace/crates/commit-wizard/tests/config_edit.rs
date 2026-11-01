//! `cw config edit`: the init questions, asked about an existing config.

mod common;

use common::{Repo, text};

const TYPES: &str = "Which commit types will you use?";
const SCOPES: &str = "How should commit scopes work?";
const BRANCHES: &str = "Which branches are protected?";
const CHECKED: &str = "Where should commit messages be checked?";
const PUSH: &str = "Allow pushing straight to a protected branch?";
const MORE: &str = "Customize more options?";

/// A repo with a standard `.cwizard.toml` made the scripted way.
fn configured() -> Repo {
    let repo = Repo::bare();
    let out = repo.cw(&["init"]);
    assert!(out.status.success(), "{}", text(&out));
    repo
}

fn config_file(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap()
}

fn parsed(repo: &Repo) -> toml::Value {
    toml::from_str(&config_file(repo)).expect("valid TOML")
}

#[test]
fn script_use_is_pointed_at_config_set() {
    let repo = configured();
    let out = repo.cw(&["config", "edit"]);

    assert!(!out.status.success());
    let output = text(&out);
    assert!(output.contains("Interactive input required"), "{output}");
    assert!(output.contains("config set"), "{output}");
}

#[cfg(unix)]
mod on_a_terminal {
    use super::*;
    use common::Pty;

    /// Press Enter through every question, keeping each current value.
    fn keep_everything(cw: &mut Pty) {
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
    }

    #[test]
    fn without_a_config_it_says_how_to_make_one() {
        let repo = Repo::bare();
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect("run `cw init`");
        assert!(!cw.finish());
    }

    #[test]
    fn keeping_every_answer_changes_nothing() {
        let repo = configured();
        let before = config_file(&repo);
        let mut cw = repo.cw_pty(&["config", "edit"]);
        keep_everything(&mut cw);
        cw.expect("Nothing changed");

        assert!(cw.finish());
        assert_eq!(config_file(&repo), before, "the file must not be rewritten");
    }

    #[test]
    fn the_current_value_is_marked_and_comes_first() {
        let repo = configured();
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.expect("Optional (current)");
        cw.line("");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.expect("Protected branches only (current)");
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("Nothing changed");
        assert!(cw.finish());
    }

    #[test]
    fn a_changed_answer_is_previewed_then_saved() {
        let repo = configured();
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("Required");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("commit.scopes.mode");
        cw.expect("Save 1 change to");
        cw.line("y");

        assert!(cw.finish());
        let config = parsed(&repo);
        assert_eq!(
            config["commit"]["scopes"]["mode"].as_str(),
            Some("required")
        );
        // Everything else is as it was.
        assert_eq!(config["versioning"]["tag_prefix"].as_str(), Some("v"));
        assert!(config["commit"]["types"].get("chore").is_some());
    }

    #[test]
    fn declining_to_save_leaves_the_file_alone() {
        let repo = configured();
        let before = config_file(&repo);
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("Not used");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("Save 1 change to");
        cw.line("n");

        assert!(cw.finish());
        assert_eq!(config_file(&repo), before);
    }

    #[test]
    fn a_type_can_be_removed_and_shows_as_one_line() {
        let repo = configured();
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.send(" "); // deselect the first type, chore
        cw.line("");
        cw.expect(SCOPES);
        cw.line("");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("- commit.types.chore");
        cw.expect("Save 1 change to");
        cw.line("y");

        assert!(cw.finish());
        assert!(parsed(&repo)["commit"]["types"].get("chore").is_none());
    }

    #[test]
    fn a_type_the_file_lacks_can_be_added_with_its_definition() {
        let repo = Repo::bare();
        repo.write(
            ".cwizard.toml",
            "[commit.types.feat]\nbump = \"Minor\"\n[commit.types.fix]\nbump = \"Patch\"\n[commit.types.docs]\nbump = \"Patch\"\n",
        );
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        // Offered: docs (selected), then the standard types the file lacks.
        cw.send("\u{1b}[B"); // down to chore
        cw.send(" "); // select it
        cw.line("");
        cw.expect(SCOPES);
        cw.line("");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("+ commit.types.chore");
        cw.line("y");

        assert!(cw.finish());
        let chore = &parsed(&repo)["commit"]["types"]["chore"];
        assert!(chore.get("description").is_some(), "{chore}");
    }

    #[test]
    fn the_extra_options_can_be_changed_too() {
        let repo = configured();
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("y");
        cw.expect("Maximum length of a commit subject");
        cw.send(&"\u{7f}".repeat(4));
        cw.line("55");
        cw.expect("Put an emoji before each commit type?");
        cw.line("");
        cw.expect("Require a description for breaking changes?");
        cw.line("");
        cw.expect("Require a ticket reference in commits?");
        cw.line("");
        cw.expect("Tag prefix");
        cw.line("");
        cw.expect("Changelog file");
        cw.line("");
        cw.expect("commit.subject_max_length: 72 -> 55");
        cw.expect("Save 1 change to");
        cw.line("y");

        assert!(cw.finish());
        assert_eq!(
            parsed(&repo)["commit"]["subject_max_length"].as_integer(),
            Some(55)
        );
    }

    #[test]
    fn settings_the_editor_does_not_ask_about_are_kept() {
        let repo = configured();
        let path = repo.path().join(".cwizard.toml");
        let edited = std::fs::read_to_string(&path)
            .unwrap()
            .replace("[ai]\nenabled = false", "[ai]\nenabled = true");
        std::fs::write(&path, edited).unwrap();

        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("Required");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("Save 1 change to");
        cw.line("y");

        assert!(cw.finish());
        assert_eq!(parsed(&repo)["ai"]["enabled"].as_bool(), Some(true));
    }

    #[test]
    fn a_yaml_file_stays_yaml() {
        let repo = Repo::bare();
        assert!(repo.cw(&["init", "--file-format", "yaml"]).status.success());
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("Required");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("Save 1 change to");
        cw.line("y");

        assert!(cw.finish());
        let raw = std::fs::read_to_string(repo.path().join(".cwizard.yaml")).unwrap();
        assert!(raw.contains("mode: required"), "{raw}");
        assert!(!repo.path().join(".cwizard.toml").exists());
    }

    #[test]
    fn dry_run_previews_without_saving() {
        let repo = configured();
        let before = config_file(&repo);
        let mut cw = repo.cw_pty(&["config", "edit", "--dry-run"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("Required");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("Save 1 change to");
        cw.line("y");

        assert!(cw.finish());
        assert_eq!(config_file(&repo), before);
    }

    #[test]
    fn an_invalid_number_is_rejected_and_nothing_is_saved() {
        let repo = configured();
        let before = config_file(&repo);
        let mut cw = repo.cw_pty(&["config", "edit"]);
        cw.expect(TYPES);
        cw.line("");
        cw.expect(SCOPES);
        cw.line("");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("y");
        cw.expect("Maximum length of a commit subject");
        cw.send(&"\u{7f}".repeat(4));
        cw.line("abc");

        assert!(!cw.finish());
        assert_eq!(config_file(&repo), before);
    }

    #[test]
    fn the_global_config_is_edited_instead_of_the_project_one() {
        let repo = configured();
        repo.write_global_file("config.toml", "[commit.scopes]\nmode = \"optional\"\n");
        let project_before = config_file(&repo);

        let mut cw = repo.cw_pty(&["config", "-g", "edit"]);
        cw.expect(SCOPES);
        cw.line("Required");
        cw.expect(BRANCHES);
        cw.line("");
        cw.expect(CHECKED);
        cw.line("");
        cw.expect(PUSH);
        cw.line("");
        cw.expect(MORE);
        cw.line("n");
        cw.expect("Save ");
        cw.line("y");

        assert!(cw.finish());
        let global = std::fs::read_to_string(repo.home_path().join("cwizard/config.toml")).unwrap();
        assert!(global.contains("required"), "{global}");
        assert_eq!(
            config_file(&repo),
            project_before,
            "the project file must not change"
        );
    }
}
