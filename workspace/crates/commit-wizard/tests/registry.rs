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
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        &format!(
            r#"[registry]
use = "local"
section = "custom"

[registries.local]
url = "{registry_url}"
ref = "v0.1.0"
sections = ["custom"]
"#
        ),
    );
    repo.write("x.txt", "x");
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
