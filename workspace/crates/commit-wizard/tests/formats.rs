//! Config files can be TOML, YAML or JSON; the extension picks the format.

mod common;

use common::{Repo, text};

const WIP_YAML: &str = "commit:\n  types:\n    wip:\n      description: Work in progress\n      bump: Patch\n      section: WIP\n";
const WIP_JSON: &str = r#"{"commit":{"types":{"wip":{"description":"Work in progress","bump":"Patch","section":"WIP"}}}}"#;
const WIP_TOML: &str =
    "[commit.types.wip]\ndescription = \"Work in progress\"\nbump = \"Patch\"\nsection = \"WIP\"\n";

/// A repo with no config and one staged file, ready for `cw commit`.
fn staged() -> Repo {
    let repo = Repo::bare();
    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);
    repo
}

fn commit_wip(repo: &Repo) -> std::process::Output {
    repo.cw(&["commit", "-t", "wip", "-m", "tracked work"])
}

#[test]
fn every_project_file_name_and_format_is_found() {
    for name in [
        ".cwizard.toml",
        ".cwizard.yaml",
        ".cwizard.yml",
        ".cwizard.json",
        "cwizard.toml",
        "cwizard.yaml",
        "cwizard.yml",
        "cwizard.json",
    ] {
        let repo = staged();
        let contents = match name.rsplit('.').next().unwrap() {
            "toml" => WIP_TOML,
            "json" => WIP_JSON,
            _ => WIP_YAML,
        };
        repo.write(name, contents);

        let out = commit_wip(&repo);
        assert!(out.status.success(), "{name}: {}", text(&out));
        assert_eq!(repo.subjects()[0], "wip: tracked work", "{name}");
    }
}

#[test]
fn without_the_config_the_custom_type_is_unknown() {
    // Control for the test above: the type really comes from the file.
    let out = commit_wip(&staged());
    assert!(!out.status.success(), "{}", text(&out));
}

#[test]
fn toml_wins_over_yaml_and_json_and_a_warning_names_the_ignored_files() {
    let repo = staged();
    repo.write(".cwizard.toml", WIP_TOML);
    // Different type, so it is clear which file was used.
    repo.write(".cwizard.yaml", &WIP_YAML.replace("wip", "other"));
    repo.write(".cwizard.json", &WIP_JSON.replace("wip", "third"));

    let out = commit_wip(&repo);
    assert!(out.status.success(), "{}", text(&out));
    let output = text(&out);
    assert!(output.contains("Several project config files"), "{output}");
    assert!(
        output.contains(".cwizard.yaml") && output.contains(".cwizard.json"),
        "{output}"
    );
}

#[test]
fn the_hidden_name_beats_the_visible_one() {
    let repo = staged();
    repo.write("cwizard.yaml", &WIP_YAML.replace("wip", "visible"));
    repo.write(".cwizard.yaml", WIP_YAML);

    let out = commit_wip(&repo);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn doctor_warns_about_competing_config_files() {
    let repo = Repo::new();
    repo.write(".cwizard.yaml", WIP_YAML);

    let out = repo.cw(&["doctor"]);
    let report = text(&out);
    assert!(report.contains("warnings: 1"), "{report}");
    assert!(!report.contains("All checks passed"), "{report}");
}

#[test]
fn doctor_reports_a_malformed_yaml_or_json_config() {
    for (name, broken) in [
        (".cwizard.yaml", "commit: [unclosed"),
        (".cwizard.json", "{ not json"),
        // Valid syntax, but not a key cw knows inside a known table.
        (".cwizard.yaml", "commit:\n  no_such_key: 1\n"),
        (".cwizard.json", r#"{"commit":{"no_such_key":1}}"#),
    ] {
        let repo = Repo::bare();
        repo.write(name, broken);

        let out = repo.cw(&["doctor"]);
        let report = text(&out);
        assert!(report.contains("errors: 1"), "{name} {broken:?}: {report}");
        // The load error says what could not be parsed, and in which format.
        let format = if name.ends_with("json") {
            "json"
        } else {
            "yaml"
        };
        assert!(report.contains("Parse error"), "{name}: {report}");
        assert!(
            report.contains(&format!("format: {format}")),
            "{name}: {report}"
        );
    }
}

#[test]
fn global_config_may_be_yaml_or_json() {
    for (name, contents) in [
        ("config.yaml", "commit:\n  subject_max_length: 11\n"),
        ("config.yml", "commit:\n  subject_max_length: 11\n"),
        ("config.json", r#"{"commit":{"subject_max_length":11}}"#),
    ] {
        let repo = staged();
        repo.write_global_file(name, contents);

        let out = repo.cw(&["commit", "-t", "feat", "-m", "this subject is far too long"]);
        assert!(!out.status.success(), "{name}: {}", text(&out));
        assert!(text(&out).contains("max length"), "{name}: {}", text(&out));
    }
}

#[test]
fn config_path_reports_the_yaml_global_file() {
    let repo = Repo::bare();
    repo.write_global_file("config.yaml", "commit:\n  subject_max_length: 50\n");

    let out = repo.cw(&["config", "-g", "path"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("config.yaml"), "{}", text(&out));
}

#[test]
fn config_get_set_and_unset_keep_a_yaml_file_yaml() {
    let repo = Repo::bare();
    repo.write(".cwizard.yaml", "commit:\n  subject_max_length: 40\n");

    let got = repo.cw(&["config", "get", "commit.subject_max_length"]);
    assert!(text(&got).contains("value: 40"), "{}", text(&got));

    let set = repo.cw(&["config", "set", "commit.subject_max_length", "60"]);
    assert!(set.status.success(), "{}", text(&set));
    let raw = std::fs::read_to_string(repo.path().join(".cwizard.yaml")).unwrap();
    assert!(raw.contains("subject_max_length: 60"), "{raw}");
    assert!(raw.trim_start().starts_with("commit:"), "{raw}");
    assert!(!repo.path().join(".cwizard.toml").exists());

    let unset = repo.cw(&["config", "unset", "commit.subject_max_length"]);
    assert!(unset.status.success(), "{}", text(&unset));
    let raw = std::fs::read_to_string(repo.path().join(".cwizard.yaml")).unwrap();
    assert!(!raw.contains("subject_max_length"), "{raw}");
}

#[test]
fn config_set_keeps_a_json_file_json() {
    let repo = Repo::bare();
    repo.write("cwizard.json", r#"{"commit":{"subject_max_length":40}}"#);

    let set = repo.cw(&["config", "set", "commit.subject_max_length", "55"]);
    assert!(set.status.success(), "{}", text(&set));

    let raw = std::fs::read_to_string(repo.path().join("cwizard.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).expect("still valid JSON");
    assert_eq!(json["commit"]["subject_max_length"], 55);
}

#[test]
fn config_show_prints_a_yaml_file_as_written() {
    let repo = Repo::bare();
    repo.write(".cwizard.yaml", "commit:\n  subject_max_length: 40\n");

    let out = repo.cw(&["config", "show"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("subject_max_length: 40"),
        "{}",
        text(&out)
    );
}

#[test]
fn init_can_create_yaml_and_json_configs_that_load() {
    for (format, file) in [("yaml", ".cwizard.yaml"), ("json", ".cwizard.json")] {
        let repo = Repo::bare();
        let out = repo.cw(&["init", "--file-format", format]);
        assert!(out.status.success(), "{format}: {}", text(&out));
        assert!(repo.path().join(file).exists(), "{format}");
        assert!(!repo.path().join(".cwizard.toml").exists(), "{format}");

        // The generated file is valid for cw itself: doctor finds no errors and
        // a standard type from the profile works.
        let doctor = repo.cw(&["doctor"]);
        assert!(
            text(&doctor).contains("errors: 0"),
            "{format}: {}",
            text(&doctor)
        );
        repo.write("x.txt", format);
        repo.git(&["add", "-A"]);
        let commit = repo.cw(&["commit", "-t", "feat", "-m", "uses generated config"]);
        assert!(commit.status.success(), "{format}: {}", text(&commit));
    }
}

#[test]
fn init_writes_the_global_config_in_the_requested_format() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "config", "--global", "--file-format", "yaml"]);
    assert!(out.status.success(), "{}", text(&out));

    let path = repo.cw(&["config", "-g", "path"]);
    assert!(text(&path).contains("config.yaml"), "{}", text(&path));
}

#[test]
fn init_with_an_explicit_config_path_uses_that_files_format() {
    let repo = Repo::bare();
    let out = repo.cw(&["init", "--config", "team.yaml"]);
    assert!(out.status.success(), "{}", text(&out));

    let raw = std::fs::read_to_string(repo.path().join("team.yaml")).unwrap();
    assert!(
        raw.starts_with("commit:") || raw.contains("\ncommit:"),
        "{raw}"
    );
    assert!(
        !raw.contains("[commit"),
        "TOML written into a .yaml file: {raw}"
    );
}

#[test]
fn init_refuses_to_overwrite_without_force() {
    let repo = Repo::bare();
    assert!(repo.cw(&["init", "--file-format", "yaml"]).status.success());
    let again = repo.cw(&["init", "--file-format", "yaml"]);
    assert!(!again.status.success(), "{}", text(&again));
    assert!(text(&again).contains("already exists"), "{}", text(&again));
}

// --- registries -----------------------------------------------------------

#[test]
fn a_registry_may_use_yaml_config_files() {
    use std::process::Command;
    let reg = tempfile::TempDir::new().unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(reg.path())
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "test"]);
    std::fs::create_dir_all(reg.path().join("custom")).unwrap();
    std::fs::write(reg.path().join("custom/config.yaml"), WIP_YAML).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "registry"]);
    git(&["tag", "v0.1.0"]);

    let repo = Repo::bare();
    repo.write(
        ".cwizard.yaml",
        &format!(
            "registry:\n  use: local\n  section: custom\nregistries:\n  local:\n    url: {}\n    ref: v0.1.0\n    sections: [custom]\n",
            common::file_url(reg.path())
        ),
    );
    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);

    let out = commit_wip(&repo);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "wip: tracked work");
}

// --- misspelled top-level tables ------------------------------------------

#[test]
fn a_misspelled_top_level_table_is_warned_about_with_a_suggestion() {
    for (name, contents) in [
        (".cwizard.toml", "[comit]\nsubject_max_length = 5\n"),
        (".cwizard.yaml", "comit:\n  subject_max_length: 5\n"),
        (".cwizard.json", r#"{"comit":{"subject_max_length":5}}"#),
    ] {
        let repo = staged();
        repo.write(name, contents);

        let out = repo.cw(&["commit", "-t", "feat", "-m", "still works"]);
        let output = text(&out);
        assert!(output.contains("[warn]"), "{name}: {output}");
        assert!(
            output.contains("unknown top-level key `comit`"),
            "{name}: {output}"
        );
        assert!(
            output.contains("did you mean `commit`?"),
            "{name}: {output}"
        );
        // A warning, not an error: the command still runs, on defaults.
        assert!(out.status.success(), "{name}: {output}");
    }
}

#[test]
fn doctor_counts_an_unknown_top_level_key_as_a_warning() {
    let repo = staged();
    repo.write(".cwizard.toml", "[telemetry]\nx = 1\n");

    let report = text(&repo.cw(&["doctor"]));

    assert!(report.contains("warnings: 1"), "{report}");
    assert!(report.contains("telemetry"), "{report}");
}

#[test]
fn doctor_also_checks_the_global_config() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[comit]\nsubject_max_length = 5\n");

    let report = text(&repo.cw(&["doctor"]));

    assert!(report.contains("warnings: 1"), "{report}");
    assert!(report.contains("did you mean `commit`?"), "{report}");
}

#[test]
fn valid_configs_produce_no_unknown_key_warning() {
    for (name, contents) in [
        (".cwizard.toml", WIP_TOML),
        (".cwizard.yaml", WIP_YAML),
        (".cwizard.json", WIP_JSON),
    ] {
        let repo = staged();
        repo.write(name, contents);
        let out = commit_wip(&repo);
        assert!(
            !text(&out).contains("unknown top-level key"),
            "{name}: {}",
            text(&out)
        );
    }
    // So does a freshly generated config.
    let generated = Repo::new();
    assert!(!text(&generated.cw(&["doctor"])).contains("unknown top-level key"));
}
