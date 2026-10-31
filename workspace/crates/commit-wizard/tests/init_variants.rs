//! `cw init config` and `cw init rules`, with and without `--global`.

mod common;

use common::{Repo, text};

fn global_dir(repo: &Repo) -> std::path::PathBuf {
    repo.home_path().join("cwizard")
}

fn files_in(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

// --- init config (project) -----------------------------------------------------

#[test]
fn init_config_creates_the_same_project_config_as_bare_init() {
    let via_config = Repo::bare();
    let out = via_config.cw(&["init", "config"]);
    assert!(out.status.success(), "{}", text(&out));

    let bare = Repo::bare();
    assert!(bare.cw(&["init"]).status.success());

    let a = std::fs::read_to_string(via_config.path().join(".cwizard.toml")).unwrap();
    let b = std::fs::read_to_string(bare.path().join(".cwizard.toml")).unwrap();
    assert_eq!(a, b);
}

#[test]
fn init_config_respects_the_file_format() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "config", "--file-format", "yaml"]);

    assert!(out.status.success(), "{}", text(&out));
    assert!(repo.path().join(".cwizard.yaml").exists());
    assert!(!repo.path().join(".cwizard.toml").exists());
}

#[test]
fn init_config_names_the_file_when_it_already_exists_and_force_replaces_it() {
    let repo = Repo::bare();
    assert!(repo.cw(&["init", "config"]).status.success());

    let again = repo.cw(&["init", "config"]);
    assert!(!again.status.success());
    let output = text(&again);
    assert!(output.contains("already exists"), "{output}");
    assert!(
        output.contains(".cwizard.toml"),
        "the message must name the file, not the folder: {output}"
    );

    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 11\n");
    assert!(repo.cw(&["--force", "init", "config"]).status.success());
    let replaced = std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap();
    assert!(!replaced.contains("= 11"), "{replaced}");
}

#[test]
fn init_config_dry_run_shows_the_config_and_writes_nothing() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "config", "--dry-run"]);

    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("[commit]"), "{}", text(&out));
    assert!(!repo.path().join(".cwizard.toml").exists());
}

// --- init config --global ------------------------------------------------------

#[test]
fn the_global_config_is_reported_by_its_file_not_its_folder() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "config", "--global"]);

    assert!(out.status.success(), "{}", text(&out));
    let output = text(&out);
    assert!(
        output.contains(&format!("cwizard{}config.toml", std::path::MAIN_SEPARATOR)),
        "{output}"
    );
    assert!(global_dir(&repo).join("config.toml").exists());
}

// --- init rules --global -------------------------------------------------------

#[test]
fn global_rules_are_written_alone_without_creating_a_config() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "rules", "--global"]);

    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains(&format!("cwizard{}rules.toml", std::path::MAIN_SEPARATOR)),
        "{}",
        text(&out)
    );
    assert_eq!(files_in(&global_dir(&repo)), ["rules.toml"]);
}

#[test]
fn global_rules_can_be_added_when_a_global_config_already_exists() {
    let repo = Repo::bare();
    assert!(repo.cw(&["init", "config", "--global"]).status.success());

    let out = repo.cw(&["init", "rules", "--global"]);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(files_in(&global_dir(&repo)), ["config.toml", "rules.toml"]);
}

#[test]
fn global_rules_are_not_overwritten_without_force() {
    let repo = Repo::bare();
    assert!(repo.cw(&["init", "rules", "--global"]).status.success());
    std::fs::write(global_dir(&repo).join("rules.toml"), "keep = \"me\"\n").unwrap();

    let again = repo.cw(&["init", "rules", "--global"]);
    assert!(!again.status.success());
    assert!(text(&again).contains("rules.toml"), "{}", text(&again));
    assert_eq!(
        std::fs::read_to_string(global_dir(&repo).join("rules.toml")).unwrap(),
        "keep = \"me\"\n"
    );

    assert!(
        repo.cw(&["--force", "init", "rules", "--global"])
            .status
            .success()
    );
}

#[test]
fn global_rules_follow_the_file_format() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "rules", "--global", "--file-format", "yaml"]);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(files_in(&global_dir(&repo)), ["rules.yaml"]);
}

#[test]
fn global_rules_dry_run_writes_nothing() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "rules", "--global", "--dry-run"]);

    assert!(out.status.success(), "{}", text(&out));
    assert!(files_in(&global_dir(&repo)).is_empty());
}

// --- init rules (project) --------------------------------------------------------

#[test]
fn project_rules_are_explained_not_attempted() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "rules"]);

    assert!(!out.status.success());
    let output = text(&out);
    assert!(output.contains("[rules] table"), "{output}");
    assert!(output.contains("--global"), "{output}");
    assert!(!repo.path().join(".cwizard.toml").exists());
}

// --- the wizard also runs for `init config` ----------------------------------------

#[cfg(unix)]
#[test]
fn init_config_on_a_terminal_runs_the_wizard() {
    let repo = Repo::bare();
    let mut cw = repo.cw_pty(&["init", "config"]);

    cw.expect("Which profile fits your project?");
    cw.line("");
    cw.expect("Which file format do you prefer?");
    cw.line("");
    cw.expect("Which commit types will you use?");
    cw.line("");
    cw.expect("How should commit scopes work?");
    cw.line("Required");
    cw.expect("Which branches are protected?");
    cw.line("");
    cw.expect("Where should commit messages be checked?");
    cw.line("");
    cw.expect("Allow pushing straight to a protected branch?");
    cw.line("n");
    cw.expect("Customize more options?");
    cw.line("n");
    cw.expect("Create ");
    cw.line("y");

    assert!(cw.finish());
    let config = std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap();
    assert!(config.contains("mode = \"required\""), "{config}");
}

#[test]
fn set_overrides_are_written_into_the_new_config() {
    let repo = Repo::bare();
    let out = repo.cw(&[
        "init",
        "--set",
        "commit.scopes.mode=required",
        "--set",
        "versioning.tag_prefix=rel-",
    ]);
    assert!(out.status.success(), "{}", text(&out));

    let raw = std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap();
    let doc: toml::Value = toml::from_str(&raw).unwrap();
    assert_eq!(doc["commit"]["scopes"]["mode"].as_str(), Some("required"));
    assert_eq!(doc["versioning"]["tag_prefix"].as_str(), Some("rel-"));
}

#[test]
fn a_malformed_set_writes_nothing_and_unknown_keys_or_values_are_rejected() {
    for bad in ["no-equals", "commit.nope=1", "commit.scopes.mode=bogus"] {
        let repo = Repo::bare();
        let out = repo.cw(&["init", "--set", bad]);
        let written = repo.path().join(".cwizard.toml").exists();
        assert!(!out.status.success(), "{bad}: {}", text(&out));
        // Unknown keys and values are rejected after the file exists, so the
        // file may be present, but a malformed pair must stop before writing.
        if bad == "no-equals" {
            assert!(!written, "{bad} wrote a config");
        }
    }
}

#[test]
fn registry_sections_and_rules_follow_the_flags() {
    let repo = Repo::bare();
    let out = repo.cw(&[
        "init",
        "registry",
        "reg",
        "--section",
        "frontend",
        "--section",
        "backend",
        "--rules",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    for path in [
        "reg/config.toml",
        "reg/rules.toml",
        "reg/frontend/config.toml",
        "reg/frontend/rules.toml",
        "reg/backend/config.toml",
    ] {
        assert!(repo.path().join(path).exists(), "{path} missing");
    }
    assert!(!repo.path().join("reg/standard").exists());

    let plain = repo.cw(&["init", "registry", "plain"]);
    assert!(plain.status.success(), "{}", text(&plain));
    assert!(repo.path().join("plain/config.toml").exists());
    assert!(!repo.path().join("plain/rules.toml").exists());
}
