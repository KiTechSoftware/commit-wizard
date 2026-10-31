//! End-to-end tests for git-backed registries, served from local `file://`
//! repositories so no network is needed.

mod common;

use std::path::Path;
use std::process::Command;

use common::{Repo, text};
use tempfile::TempDir;

const CUSTOM_CONFIG: &str = r#"
[commit.types.wip]
description = "Work in progress"
bump = "Patch"
section = "WIP"

[commit.types.feat]
description = "A new feature"
bump = "Minor"
section = "Features"
"#;

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A registry repo with one section (`custom`) tagged `v0.1.0`.
/// `files` are (path, contents) pairs written into the repo.
fn registry(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "-q", "-b", "main"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    git(dir.path(), &["config", "user.name", "test"]);
    for (path, contents) in files {
        let full = dir.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, contents).unwrap();
    }
    git(dir.path(), &["add", "-A"]);
    git(dir.path(), &["commit", "-q", "-m", "registry"]);
    git(dir.path(), &["tag", "v0.1.0"]);
    dir
}

/// A repo whose only config is a pointer at `registry_url`, with a staged file.
fn repo_using(registry_url: &str) -> Repo {
    repo_with(registry_url, "v0.1.0", "")
}

/// Like `repo_using`, with a chosen ref and extra lines for the `[registry]` table.
fn repo_with(registry_url: &str, git_ref: &str, registry_options: &str) -> Repo {
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        &format!(
            r#"[registry]
use = "local"
section = "custom"
{registry_options}

[registries.local]
url = "{registry_url}"
ref = "{git_ref}"
sections = ["custom"]
"#
        ),
    );
    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);
    repo
}

/// Like `repo_using`, with extra TOML (local overrides) appended to the config.
fn repo_overriding(registry_url: &str, local_overrides: &str) -> Repo {
    let repo = repo_using(registry_url);
    let path = repo.path().join(".cwizard.toml");
    let mut config = std::fs::read_to_string(&path).unwrap();
    config.push_str(&format!("\n{local_overrides}\n"));
    std::fs::write(&path, config).unwrap();
    repo.git(&["add", "-A"]);
    repo
}

fn file_url(dir: &Path) -> String {
    format!("file://{}", dir.display())
}

#[test]
fn types_come_from_the_registry() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_using(&file_url(reg.path()));

    let out = repo.cw(&["commit", "-t", "wip", "-m", "try registry"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "wip: try registry");
}

#[test]
fn registry_types_are_absent_without_the_registry() {
    // Control: the same commit is rejected when no registry defines `wip`.
    let repo = Repo::new();
    repo.write("x.txt", "x");
    repo.git(&["add", "x.txt"]);

    let out = repo.cw(&["commit", "-t", "wip", "-m", "try registry"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(repo.subjects()[0] != "wip: try registry");
}

#[test]
fn cached_registry_works_after_the_remote_disappears() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_using(&file_url(reg.path()));

    let first = repo.cw(&["commit", "-t", "wip", "-m", "first"]);
    assert!(first.status.success(), "{}", text(&first));

    drop(reg); // the registry is gone; only the cache remains
    repo.write("y.txt", "y");
    repo.git(&["add", "y.txt"]);
    let second = repo.cw(&["commit", "-t", "wip", "-m", "second"]);
    assert!(second.status.success(), "{}", text(&second));
    assert_eq!(repo.subjects()[0], "wip: second");
}

// A registry that cannot be loaded is reported loudly but does not stop `cw`:
// it falls back to the built-in defaults. These tests pin that behaviour down.

#[test]
fn unreachable_registry_is_reported_and_defaults_apply() {
    let repo = repo_using("file:///definitely/not/a/registry");

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    let output = text(&out);
    assert!(output.contains("Registry load failed"), "{output}");
    assert!(output.contains("Registry sync failed"), "{output}");
    assert!(!output.contains("panicked"), "{output}");

    // Defaults apply: a built-in type works, a registry-only type does not.
    assert!(out.status.success(), "{output}");
    repo.write("y.txt", "y");
    repo.git(&["add", "y.txt"]);
    let wip = repo.cw(&["commit", "-t", "wip", "-m", "y"]);
    assert!(!wip.status.success(), "{}", text(&wip));
}

#[test]
fn registry_section_without_config_is_reported() {
    let reg = registry(&[("custom/readme.md", "no config here")]);
    let repo = repo_using(&file_url(reg.path()));

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    let output = text(&out);
    assert!(output.contains("Registry load failed"), "{output}");
    assert!(output.contains("config.toml"), "{output}");
    assert!(!output.contains("panicked"), "{output}");
}

#[test]
fn unreachable_registry_is_a_warning_and_doctor_counts_it() {
    let repo = repo_using("file:///definitely/not/a/registry");

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    assert!(text(&out).contains("[warn]"), "{}", text(&out));
    assert!(!text(&out).contains("[error]"), "{}", text(&out));

    let doctor = repo.cw(&["doctor"]);
    let report = text(&doctor);
    assert!(report.contains("warnings: 1"), "{report}");
    assert!(!report.contains("All checks passed"), "{report}");
}

#[test]
fn required_registry_that_is_unreachable_fails_hard() {
    let repo = repo_with(
        "file:///definitely/not/a/registry",
        "v0.1.0",
        "required = true",
    );
    let before = repo.subjects_or_empty();

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("Registry sync failed"),
        "{}",
        text(&out)
    );
    assert_eq!(repo.subjects_or_empty(), before, "no commit may be made");
}

#[test]
fn required_registry_without_config_fails_hard() {
    let reg = registry(&[("custom/readme.md", "no config here")]);
    let repo = repo_with(&file_url(reg.path()), "v0.1.0", "required = true");

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("config.toml"), "{}", text(&out));
}

#[test]
fn required_registry_that_is_not_configured_fails_hard() {
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        "[registry]\nuse = \"missing\"\nrequired = true\n",
    );
    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("required but not configured"),
        "{}",
        text(&out)
    );
}

#[test]
fn required_registry_that_loads_works_normally() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_with(&file_url(reg.path()), "v0.1.0", "required = true");

    let out = repo.cw(&["commit", "-t", "wip", "-m", "ok"]);
    assert!(out.status.success(), "{}", text(&out));
}

#[test]
fn disabling_the_cache_refetches_on_every_run() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_with(
        &file_url(reg.path()),
        "v0.1.0",
        "required = true\ncache = false",
    );

    let first = repo.cw(&["commit", "-t", "wip", "-m", "first"]);
    assert!(first.status.success(), "{}", text(&first));

    drop(reg); // nothing cached, so the next run cannot load it
    repo.write("y.txt", "y");
    repo.git(&["add", "y.txt"]);
    let second = repo.cw(&["commit", "-t", "wip", "-m", "second"]);
    assert!(!second.status.success(), "{}", text(&second));
}

#[test]
fn stale_cache_is_used_when_a_branch_registry_cannot_sync() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let url = file_url(reg.path());
    let repo = repo_with(&url, "main", "required = true");

    let first = repo.cw(&["commit", "-t", "wip", "-m", "first"]);
    assert!(first.status.success(), "{}", text(&first));

    drop(reg); // the remote is gone; the cached copy remains
    repo.write("y.txt", "y");
    repo.git(&["add", "y.txt"]);
    let second = repo.cw(&["commit", "-t", "wip", "-m", "second"]);
    assert!(second.status.success(), "{}", text(&second));
    assert_eq!(repo.subjects()[0], "wip: second");
}

// --- layering: global config vs repo config --------------------------------

#[test]
fn global_required_applies_when_the_repo_registry_table_omits_it() {
    let repo = repo_using("file:///definitely/not/a/registry");
    repo.write_global_config("[registry]\nrequired = true\n");

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("Registry sync failed"),
        "{}",
        text(&out)
    );
}

#[test]
fn repo_can_override_a_global_required() {
    let repo = repo_with(
        "file:///definitely/not/a/registry",
        "v0.1.0",
        "required = false",
    );
    repo.write_global_config("[registry]\nrequired = true\n");

    let out = repo.cw(&["commit", "-t", "feat", "-m", "x"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("Registry load failed"),
        "{}",
        text(&out)
    );
}

#[test]
fn global_cache_setting_applies_to_the_repo_registry() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_with(&file_url(reg.path()), "v0.1.0", "required = true");
    repo.write_global_config("[registry]\ncache = false\n");

    let first = repo.cw(&["commit", "-t", "wip", "-m", "first"]);
    assert!(first.status.success(), "{}", text(&first));

    drop(reg); // global cache = false: nothing was kept
    repo.write("y.txt", "y");
    repo.git(&["add", "y.txt"]);
    let second = repo.cw(&["commit", "-t", "wip", "-m", "second"]);
    assert!(!second.status.success(), "{}", text(&second));
}

// --- no usable config: built-in defaults ------------------------------------

#[test]
fn commands_run_on_defaults_when_there_is_no_config_at_all() {
    let repo = Repo::bare();
    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);

    let out = repo.cw(&["commit", "-t", "feat", "-m", "first"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.subjects()[0], "feat: first");

    let check = repo.cw(&["check"]);
    assert!(check.status.success(), "{}", text(&check));
}

#[test]
fn failed_cli_registry_falls_back_to_defaults_instead_of_failing() {
    let repo = Repo::bare();
    repo.write("x.txt", "x");
    repo.git(&["add", "-A"]);

    let out = repo.cw(&[
        "commit",
        "--registry",
        "file:///definitely/not/a/registry",
        "-t",
        "feat",
        "-m",
        "first",
    ]);
    let output = text(&out);
    assert!(output.contains("Registry load failed"), "{output}");
    assert!(!output.contains("Config not resolved"), "{output}");
    assert!(out.status.success(), "{output}");
}

// --- local config takes precedence over the registry ------------------------
//
// Users replace a subset (or all) of what a remote registry defines. Settings
// the local config does not mention keep coming from the registry.

#[test]
fn local_scalar_overrides_one_setting_and_keeps_registry_types() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_overriding(&file_url(reg.path()), "[commit]\nsubject_max_length = 20");

    // Registry-only type still works...
    let ok = repo.cw(&["commit", "-t", "wip", "-m", "short"]);
    assert!(ok.status.success(), "{}", text(&ok));

    // ...and the local limit applies.
    repo.write("y.txt", "y");
    repo.git(&["add", "y.txt"]);
    let long = repo.cw(&["commit", "-t", "wip", "-m", &"x".repeat(30)]);
    assert!(!long.status.success(), "{}", text(&long));
    assert!(text(&long).contains("max length"), "{}", text(&long));
}

#[test]
fn local_types_add_to_the_registry_types() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_overriding(
        &file_url(reg.path()),
        "[commit.types.fix]\ndescription = \"A bug fix\"\nbump = \"Patch\"\nsection = \"Fixes\"",
    );

    // The local type is new, and the registry's types are still there.
    for (ty, msg) in [("fix", "local type"), ("wip", "registry type")] {
        repo.write(&format!("{ty}.txt"), ty);
        repo.git(&["add", "-A"]);
        let out = repo.cw(&["commit", "-t", ty, "-m", msg]);
        assert!(out.status.success(), "{ty}: {}", text(&out));
    }
}

#[test]
fn local_type_can_be_disabled_with_enabled_false() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_overriding(&file_url(reg.path()), "[commit.types.wip]\nenabled = false");

    let dropped = repo.cw(&["commit", "-t", "wip", "-m", "gone"]);
    assert!(!dropped.status.success(), "{}", text(&dropped));

    // Every other registry type is untouched.
    let kept = repo.cw(&["commit", "-t", "feat", "-m", "still here"]);
    assert!(kept.status.success(), "{}", text(&kept));
}

#[test]
fn local_override_of_one_type_field_keeps_the_rest_of_that_type() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    // The registry makes `wip` a patch bump; the project makes it a minor one.
    let repo = repo_overriding(
        &file_url(reg.path()),
        "[commit.types.wip]\nbump = \"Minor\"",
    );
    repo.git(&["commit", "-q", "--allow-empty", "-m", "chore: base"]);
    let tagged = repo.tag(&["--set-version", "1.0.0"]);
    assert!(tagged.status.success(), "{}", text(&tagged));

    // `wip` is still a known type from the registry, but now bumps minor.
    repo.write("wip.txt", "wip");
    repo.git(&["add", "-A"]);
    let out = repo.cw(&["commit", "-t", "wip", "-m", "tweak"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.next_tag(), "v1.1.0");
}

#[test]
fn local_scopes_override_without_touching_registry_types() {
    let reg = registry(&[("custom/config.toml", CUSTOM_CONFIG)]);
    let repo = repo_overriding(
        &file_url(reg.path()),
        "[commit.scopes]\nmode = \"required\"",
    );

    // Scope policy is local...
    let missing = repo.cw(&["commit", "-t", "wip", "-m", "no scope"]);
    assert!(!missing.status.success(), "{}", text(&missing));

    // ...while the type still comes from the registry.
    let scoped = repo.cw(&["commit", "-t", "wip", "-s", "core", "-m", "with scope"]);
    assert!(scoped.status.success(), "{}", text(&scoped));
    assert_eq!(repo.subjects()[0], "wip(core): with scope");
}
