//! `cw config list` and `cw config list --effective`.

mod common;

use common::{Repo, text};

fn row(output: &str, key: &str) -> String {
    output
        .lines()
        .find(|line| line.contains(&format!("│ {key} ")) || line.contains(&format!("| {key} ")))
        .unwrap_or_else(|| panic!("no row for {key} in:\n{output}"))
        .to_string()
}

#[test]
fn effective_attributes_each_value_to_its_layer() {
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        "[commit]\nsubject_max_length = 40\n[branch]\nremote = \"upstream\"\n",
    );

    let out = repo.cw(&["config", "list", "--effective"]);
    assert!(out.status.success(), "{}", text(&out));
    let output = text(&out);

    assert!(
        row(&output, "commit.subject_max_length").contains("40"),
        "{output}"
    );
    assert!(
        row(&output, "commit.subject_max_length").contains("project"),
        "{output}"
    );
    assert!(
        row(&output, "branch.remote").contains("\"upstream\""),
        "{output}"
    );
}

#[test]
fn env_beats_the_project_and_is_named_as_the_source() {
    let repo = Repo::bare();
    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 40\n");

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_cw"))
        .args(["config", "list", "--effective", "commit.subject"])
        .current_dir(repo.path())
        .env("HOME", repo.home_path())
        .env("XDG_CONFIG_HOME", repo.home_path())
        .env("CW_COMMIT_SUBJECT_MAX_LENGTH", "9")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let line = row(&text(&out), "commit.subject_max_length");

    assert!(line.contains(" 9 ") && line.contains("env"), "{line}");
}

#[test]
fn a_global_config_that_is_not_applied_is_reported_as_such() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");
    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 40\n");

    let output = text(&repo.cw(&["config", "list", "--effective"]));

    assert!(output.contains("not applied"), "{output}");
    assert!(
        row(&output, "commit.subject_max_length").contains("40"),
        "{output}"
    );
}

#[test]
fn the_global_config_is_the_only_layer_when_nothing_else_exists() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");

    let output = text(&repo.cw(&["config", "list", "--effective"]));

    let line = row(&output, "commit.subject_max_length");
    assert!(line.contains("11") && line.contains("global"), "{line}");
    assert!(!output.contains("not applied"), "{output}");
}

#[test]
fn registry_values_are_attributed_to_the_registry() {
    let reg = tempfile::TempDir::new().unwrap();
    let git = |args: &[&str]| {
        assert!(
            std::process::Command::new("git")
                .args(args)
                .current_dir(reg.path())
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "t"]);
    std::fs::create_dir_all(reg.path().join("custom")).unwrap();
    std::fs::write(
        reg.path().join("custom/config.toml"),
        "[commit]\nsubject_max_length = 100\n[commit.types.wip]\nbump = \"Patch\"\n",
    )
    .unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "registry"]);
    git(&["tag", "v0.1.0"]);

    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        &format!(
            "[commit]\nsubject_max_length = 50\n[registry]\nuse = \"local\"\nsection = \"custom\"\n[registries.local]\nurl = \"file://{}\"\nref = \"v0.1.0\"\nsections = [\"custom\"]\n",
            reg.path().display()
        ),
    );

    let output = text(&repo.cw(&["config", "list", "--effective", "commit"]));

    assert!(
        row(&output, "commit.subject_max_length").contains("project"),
        "{output}"
    );
    assert!(
        row(&output, "commit.types.wip.bump").contains("registry:local"),
        "{output}"
    );
}

#[test]
fn a_prefix_narrows_the_rows() {
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        "[commit]\nsubject_max_length = 40\n[branch]\nremote = \"x\"\n",
    );

    let output = text(&repo.cw(&["config", "list", "--effective", "branch"]));

    assert!(output.contains("branch.remote"), "{output}");
    assert!(!output.contains("commit.subject_max_length"), "{output}");
}

#[test]
fn list_shows_the_settings_in_a_yaml_file() {
    let repo = Repo::bare();
    repo.write(".cwizard.yaml", "commit:\n  subject_max_length: 40\n");

    let output = text(&repo.cw(&["config", "list"]));

    assert!(
        row(&output, "commit.subject_max_length").contains("40"),
        "{output}"
    );
}

#[test]
fn list_global_reads_the_global_file_not_the_project_one() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");
    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 40\n");

    let output = text(&repo.cw(&["config", "-g", "list"]));

    assert!(
        row(&output, "commit.subject_max_length").contains("11"),
        "{output}"
    );
}

#[test]
fn effective_cannot_be_combined_with_global() {
    let repo = Repo::bare();
    let out = repo.cw(&["config", "-g", "list", "--effective"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("cannot be combined"), "{}", text(&out));
}

#[test]
fn list_without_a_project_config_says_so() {
    let repo = Repo::bare();
    let out = repo.cw(&["config", "list"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("no project config"), "{}", text(&out));
}

#[test]
fn json_output_carries_the_table_with_sources() {
    let repo = Repo::bare();
    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 40\n");

    let out = repo.cw(&["--json", "config", "list", "--effective", "commit.subject"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));

    assert!(json.to_string().contains("project"), "{json}");
    assert!(
        json.to_string().contains("commit.subject_max_length"),
        "{json}"
    );
}
