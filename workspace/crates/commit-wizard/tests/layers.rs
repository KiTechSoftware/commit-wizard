//! Which config layers apply, and `config -g` touching only the global file.

mod common;

use common::{Repo, text};

const LONG: &str = "this subject is longer than eleven characters";

fn rejects_long_subject(repo: &Repo, envs: &[(&str, &str)]) -> bool {
    repo.write("x.txt", LONG);
    repo.git(&["add", "-A"]);
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_cw"));
    cmd.args(["commit", "-t", "feat", "-m", LONG])
        .current_dir(repo.path())
        .env("HOME", repo.home_path())
        .env("XDG_CONFIG_HOME", repo.home_path())
        .stdin(std::process::Stdio::null());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    text(&cmd.output().unwrap()).contains("max length")
}

#[test]
fn env_overrides_the_project_config() {
    let repo = Repo::bare();
    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 100\n");
    assert!(!rejects_long_subject(&repo, &[]));
    assert!(rejects_long_subject(
        &repo,
        &[("CW_COMMIT_SUBJECT_MAX_LENGTH", "11")]
    ));
}

#[test]
fn env_applies_without_any_config_and_can_be_disabled() {
    let repo = Repo::bare();
    let env = [("CW_COMMIT_SUBJECT_MAX_LENGTH", "11")];
    assert!(rejects_long_subject(&repo, &env));
    let off = [env[0], ("CW_ALLOW_ENV_OVERRIDE", "false")];
    assert!(!rejects_long_subject(&repo, &off));
}

#[test]
fn global_config_commands_never_touch_the_project_file() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");
    repo.write(".cwizard.toml", "[commit]\nsubject_max_length = 40\n");

    let got = repo.cw(&["config", "-g", "get", "commit.subject_max_length"]);
    assert!(text(&got).contains("value: 11"), "{}", text(&got));

    let set = repo.cw(&["config", "-g", "set", "commit.subject_max_length", "22"]);
    assert!(set.status.success(), "{}", text(&set));
    let project = std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap();
    assert!(
        project.contains("= 40"),
        "project file was modified: {project}"
    );
}

#[test]
fn global_is_a_lower_layer_when_the_project_opts_in() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");
    repo.write(
        ".cwizard.toml",
        "[global]\ninherit = true\n[branch]\nremote = \"upstream\"\n",
    );

    // Nothing in the project sets the limit, so the global value applies.
    assert!(rejects_long_subject(&repo, &[]));
}

#[test]
fn a_project_value_still_beats_an_inherited_global_one() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");
    repo.write(
        ".cwizard.toml",
        "[global]\ninherit = true\n[commit]\nsubject_max_length = 100\n",
    );

    assert!(!rejects_long_subject(&repo, &[]));
}

#[test]
fn without_the_opt_in_the_global_config_is_ignored_next_to_a_project_config() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");
    repo.write(".cwizard.toml", "[branch]\nremote = \"upstream\"\n");

    assert!(!rejects_long_subject(&repo, &[]));
}

#[test]
fn the_opt_in_can_be_set_from_the_cli_and_shows_in_the_effective_view() {
    let repo = Repo::bare();
    repo.write_global_file("config.toml", "[commit]\nsubject_max_length = 11\n");
    repo.write(".cwizard.toml", "[branch]\nremote = \"upstream\"\n");

    let set = repo.cw(&["config", "set", "global.inherit", "true"]);
    assert!(set.status.success(), "{}", text(&set));

    let view = text(&repo.cw(&["config", "list", "--effective", "commit.subject"]));
    assert!(view.contains("11") && view.contains("global"), "{view}");
}

#[test]
fn global_flag_works_before_or_after_the_config_subcommand() {
    let repo = Repo::bare();
    let before = repo.cw(&[
        "config",
        "--global",
        "set",
        "commit.subject_max_length",
        "61",
    ]);
    assert!(before.status.success(), "{}", text(&before));
    let after = repo.cw(&["config", "get", "commit.subject_max_length", "--global"]);
    assert!(after.status.success(), "{}", text(&after));
    assert!(text(&after).contains("61"), "{}", text(&after));
}
