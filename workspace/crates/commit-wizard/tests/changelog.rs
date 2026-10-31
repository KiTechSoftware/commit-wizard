//! End-to-end tests for `cw changelog` against throwaway git repos.

mod common;

use common::{Repo, text};

/// Two tagged releases plus unreleased work:
///
/// v0.1.0  feat: first feature, fix(core): crash on empty input
/// v0.2.0  feat(cli): add --verbose, docs: explain flags, feat!: drop legacy api
/// (HEAD)  fix(core): off by one, "not conventional", feat(cli): colored output
fn history() -> Repo {
    let repo = Repo::new();
    repo.commit("feat: first feature");
    repo.commit("fix(core): crash on empty input");
    repo.git(&["tag", "v0.1.0"]);

    repo.commit("feat(cli): add --verbose flag");
    repo.commit("docs: explain flags");
    repo.commit_with_footer(
        "feat!: drop legacy api",
        "BREAKING CHANGE: the v1 api is gone",
    );
    repo.git(&["tag", "v0.2.0"]);

    repo.commit("fix(core): off by one");
    repo.commit("not conventional");
    repo.commit("feat(cli): colored output");
    repo
}

fn changelog(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path().join("CHANGELOG.md")).expect("CHANGELOG.md was written")
}

fn headings(markdown: &str) -> Vec<&str> {
    markdown.lines().filter(|l| l.starts_with("## ")).collect()
}

#[test]
fn default_writes_only_the_changes_since_the_last_tag() {
    let repo = history();
    let out = repo.cw(&["changelog"]);
    assert!(out.status.success(), "{}", text(&out));

    let file = changelog(&repo);
    assert!(file.starts_with("# Changelog\n"), "{file}");
    assert_eq!(headings(&file), ["## Unreleased"], "{file}");
    assert!(file.contains("**cli:** colored output"), "{file}");
    assert!(file.contains("**core:** off by one"), "{file}");
    // Released work and non-conventional commits stay out.
    assert!(!file.contains("first feature"), "{file}");
    assert!(!file.contains("not conventional"), "{file}");
}

#[test]
fn features_are_listed_before_fixes() {
    let repo = history();
    assert!(repo.cw(&["changelog"]).status.success());
    let file = changelog(&repo);
    assert!(file.find("### Features").unwrap() < file.find("### Bug Fixes").unwrap());
}

#[test]
fn all_builds_every_release_newest_first() {
    let repo = history();
    let out = repo.cw(&["changelog", "--all"]);
    assert!(out.status.success(), "{}", text(&out));

    let file = changelog(&repo);
    let found = headings(&file);
    assert_eq!(found.len(), 3, "{file}");
    assert_eq!(found[0], "## Unreleased");
    assert!(found[1].starts_with("## v0.2.0 - "), "{found:?}");
    assert!(found[2].starts_with("## v0.1.0 - "), "{found:?}");

    // Each release only holds its own commits.
    let v1 = &file[file.find("## v0.1.0").unwrap()..];
    assert!(
        v1.contains("first feature") && !v1.contains("add --verbose"),
        "{file}"
    );
    let v2 = &file[file.find("## v0.2.0").unwrap()..file.find("## v0.1.0").unwrap()];
    assert!(
        v2.contains("add --verbose") && !v2.contains("first feature"),
        "{file}"
    );
}

#[test]
fn breaking_changes_get_their_own_section_with_the_note() {
    let repo = history();
    assert!(repo.cw(&["changelog", "--all"]).status.success());
    let file = changelog(&repo);

    let v2 = &file[file.find("## v0.2.0").unwrap()..file.find("## v0.1.0").unwrap()];
    assert!(v2.contains("### Breaking Changes"), "{v2}");
    assert!(
        v2.contains("drop legacy api") && v2.contains("the v1 api is gone"),
        "{v2}"
    );
}

#[test]
fn running_twice_changes_nothing() {
    let repo = history();
    assert!(repo.cw(&["changelog", "--all"]).status.success());
    let first = changelog(&repo);

    assert!(repo.cw(&["changelog", "--all"]).status.success());
    assert_eq!(changelog(&repo), first);
}

#[test]
fn existing_header_and_older_entries_are_preserved() {
    let repo = history();
    std::fs::write(
        repo.path().join("CHANGELOG.md"),
        "# Our Changelog\n\nHand written intro.\n\n## v0.0.1 - 2020-01-01\n\n- ancient history\n",
    )
    .unwrap();

    assert!(repo.cw(&["changelog"]).status.success());
    let file = changelog(&repo);

    assert!(
        file.starts_with("# Our Changelog\n\nHand written intro.\n"),
        "{file}"
    );
    assert_eq!(
        headings(&file),
        ["## Unreleased", "## v0.0.1 - 2020-01-01"],
        "{file}"
    );
    assert!(file.contains("ancient history"), "{file}");
}

#[test]
fn next_tag_replaces_the_unreleased_block() {
    let repo = history();
    assert!(repo.cw(&["changelog"]).status.success());
    assert_eq!(headings(&changelog(&repo)), ["## Unreleased"]);

    // `0.3.0` and `v0.3.0` name the same release.
    assert!(
        repo.cw(&["changelog", "--next-tag", "0.3.0"])
            .status
            .success()
    );
    let file = changelog(&repo);
    let found = headings(&file);
    assert_eq!(found.len(), 1, "{file}");
    assert!(found[0].starts_with("## v0.3.0 - "), "{found:?}");
    assert!(!file.contains("Unreleased"), "{file}");
}

#[test]
fn force_recreates_the_file_from_scratch() {
    let repo = history();
    std::fs::write(
        repo.path().join("CHANGELOG.md"),
        "# Old\n\n## v0.0.1 - 2020-01-01\n\n- ancient history\n",
    )
    .unwrap();

    assert!(repo.cw(&["changelog", "--force"]).status.success());
    let file = changelog(&repo);

    assert!(!file.contains("ancient history"), "{file}");
    assert!(file.starts_with("# Changelog\n"), "{file}");
}

#[test]
fn dry_run_previews_without_writing() {
    let repo = history();
    let out = repo.cw(&["changelog", "--dry-run"]);

    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("colored output"), "{}", text(&out));
    assert!(!repo.path().join("CHANGELOG.md").exists());
}

#[test]
fn stdout_mode_prints_only_the_changelog() {
    let repo = history();
    let out = repo.cw(&["changelog", "--output", "-"]);
    assert!(out.status.success(), "{}", text(&out));

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("# Changelog\n"), "{stdout}");
    assert!(
        !stdout.contains("written:"),
        "status report leaked into stdout: {stdout}"
    );
    assert!(!repo.path().join("CHANGELOG.md").exists());
}

#[test]
fn json_output_is_valid_and_goes_to_its_own_file() {
    let repo = history();
    assert!(
        repo.cw(&["changelog", "--all", "--output-format", "json"])
            .status
            .success()
    );

    assert!(
        !repo.path().join("CHANGELOG.md").exists(),
        "markdown file must not be created"
    );
    let raw = std::fs::read_to_string(repo.path().join("CHANGELOG.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).expect("valid JSON");

    let labels: Vec<&str> = json["releases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, ["Unreleased", "v0.2.0", "v0.1.0"]);
}

#[test]
fn custom_range_makes_a_single_labelled_section() {
    let repo = history();
    let out = repo.cw(&[
        "changelog",
        "--output",
        "-",
        "--from",
        "v0.1.0",
        "--to",
        "v0.2.0",
    ]);
    assert!(out.status.success(), "{}", text(&out));

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(headings(&stdout).len(), 1, "{stdout}");
    assert!(stdout.contains("## v0.1.0..v0.2.0"), "{stdout}");
    assert!(
        stdout.contains("add --verbose") && !stdout.contains("first feature"),
        "{stdout}"
    );
}

#[test]
fn all_cannot_be_combined_with_from() {
    let repo = history();
    let out = repo.cw(&["changelog", "--all", "--from", "v0.1.0"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("cannot be combined"), "{}", text(&out));
}

#[test]
fn an_unknown_revision_is_reported_with_its_name() {
    let repo = history();
    let out = repo.cw(&["changelog", "--from", "no-such-ref"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("no-such-ref"), "{}", text(&out));
}

#[test]
fn non_conventional_commits_appear_only_when_asked() {
    let repo = history();

    let out = repo.cw(&["changelog", "--output", "-", "--include-non-cc"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("### Miscellaneous"), "{stdout}");
    assert!(stdout.contains("not conventional"), "{stdout}");
}

#[test]
fn group_by_scope_adds_scope_headings() {
    let repo = history();
    let out = repo.cw(&["changelog", "--output", "-", "--group-by-scope"]);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        stdout.contains("#### cli") && stdout.contains("#### core"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("\n\n\n"),
        "no double blank lines: {stdout:?}"
    );
}

#[test]
fn nothing_to_report_writes_no_file() {
    let repo = Repo::new();
    repo.commit("feat: shipped");
    repo.git(&["tag", "v1.0.0"]);

    let out = repo.cw(&["changelog"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("No unreleased changes"),
        "{}",
        text(&out)
    );
    assert!(!repo.path().join("CHANGELOG.md").exists());
}

#[test]
fn a_repo_without_tags_reports_its_whole_history_as_unreleased() {
    let repo = Repo::new();
    repo.commit("feat: one");
    repo.commit("fix: two");

    assert!(repo.cw(&["changelog"]).status.success());
    let file = changelog(&repo);
    assert!(file.contains("one") && file.contains("two"), "{file}");
}

#[test]
fn prereleases_roll_into_the_stable_release_unless_requested() {
    let repo = Repo::new();
    repo.commit("feat: base");
    repo.git(&["tag", "v1.0.0"]);
    repo.commit("feat: candidate work");
    repo.git(&["tag", "v1.1.0-rc.1"]);
    repo.commit("fix: after rc");
    repo.git(&["tag", "v1.1.0"]);

    let default = repo.cw(&["changelog", "--all", "--output", "-"]);
    let stdout = String::from_utf8_lossy(&default.stdout).to_string();
    assert_eq!(headings(&stdout).len(), 2, "{stdout}");
    let v110 = &stdout[stdout.find("## v1.1.0").unwrap()..stdout.find("## v1.0.0").unwrap()];
    assert!(
        v110.contains("candidate work") && v110.contains("after rc"),
        "{v110}"
    );

    let split = repo.cw(&["changelog", "--all", "--prereleases", "--output", "-"]);
    let stdout = String::from_utf8_lossy(&split.stdout).to_string();
    assert_eq!(headings(&stdout).len(), 3, "{stdout}");
    let rc = &stdout[stdout.find("## v1.1.0-rc.1").unwrap()..stdout.find("## v1.0.0").unwrap()];
    assert!(
        rc.contains("candidate work") && !rc.contains("after rc"),
        "{rc}"
    );
}

#[test]
fn tags_from_other_branches_are_ignored() {
    let repo = Repo::new();
    repo.commit("feat: base");
    repo.git(&["tag", "v1.0.0"]);

    repo.git(&["checkout", "-q", "-b", "other"]);
    repo.commit("feat: elsewhere");
    repo.git(&["tag", "v9.0.0"]);

    repo.git(&["checkout", "-q", "main"]);
    repo.commit("fix: on main");

    let out = repo.cw(&["changelog", "--all", "--output", "-"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("v9.0.0") && !stdout.contains("elsewhere"),
        "{stdout}"
    );
    assert!(stdout.contains("on main"), "{stdout}");
}

#[test]
fn config_controls_titles_order_and_the_header() {
    let repo = history();
    let path = repo.path().join(".cwizard.toml");
    let mut config = std::fs::read_to_string(&path).unwrap();
    config.push_str(
        "\n[changelog.header]\nuse = true\ntitle = \"Release Notes\"\ndescription = \"What changed.\"\n\
         \n[changelog.sections.fix]\ntitle = \"Squashed Bugs\"\norder = -1\n",
    );
    std::fs::write(&path, config).unwrap();

    let out = repo.cw(&["changelog", "--output", "-"]);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(
        stdout.starts_with("# Release Notes\n\nWhat changed.\n"),
        "{stdout}"
    );
    assert!(stdout.contains("### Squashed Bugs"), "{stdout}");
    // order = -1 moves the renamed fix section ahead of features.
    assert!(stdout.find("### Squashed Bugs").unwrap() < stdout.find("### Features").unwrap());
}

#[test]
fn section_titles_default_to_the_commit_types_section() {
    // With no [changelog.sections] entry, the title is the commit type's own section.
    let repo = history();
    let out = repo.cw(&["changelog", "--all", "--output", "-"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("### Documentation"), "{stdout}");
}

/// Run git with fixed author/committer dates.
fn git_at(repo: &Repo, date: &str, args: &[&str]) {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(repo.path())
        .env("GIT_COMMITTER_DATE", date)
        .env("GIT_AUTHOR_DATE", date)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

#[test]
fn an_annotated_tag_is_dated_by_when_it_was_tagged() {
    let repo = Repo::new();
    git_at(
        &repo,
        "2020-01-05T12:00:00Z",
        &["commit", "-q", "--allow-empty", "-m", "feat: first"],
    );
    git_at(
        &repo,
        "2021-06-15T12:00:00Z",
        &["tag", "-a", "v1.0.0", "-m", "release"],
    );

    let out = repo.cw(&["changelog", "--all", "--output", "-"]);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(stdout.contains("## v1.0.0 - 2021-06-15"), "{stdout}");
}

#[test]
fn a_lightweight_tag_is_dated_by_its_commit() {
    let repo = Repo::new();
    git_at(
        &repo,
        "2020-01-05T12:00:00Z",
        &["commit", "-q", "--allow-empty", "-m", "feat: first"],
    );
    git_at(&repo, "2021-06-15T12:00:00Z", &["tag", "v1.0.0"]);

    let out = repo.cw(&["changelog", "--all", "--output", "-"]);
    let stdout = String::from_utf8_lossy(&out.stdout);

    assert!(stdout.contains("## v1.0.0 - 2020-01-05"), "{stdout}");
}

#[test]
fn show_empty_sections_and_scopes_work_from_config() {
    let repo = history();
    let path = repo.path().join(".cwizard.toml");
    let config = std::fs::read_to_string(&path)
        .unwrap()
        .replace("show_empty_sections = false", "show_empty_sections = true")
        .replace("scope_order = []", "scope_order = [\"api\"]")
        .replace("show_empty_scopes = false", "show_empty_scopes = true");
    std::fs::write(&path, config).unwrap();

    let out = repo.cw(&["changelog", "--output", "-", "--group-by-scope"]);
    let stdout = String::from_utf8_lossy(&out.stdout);

    // `docs` and `test` have no unreleased commits but are listed anyway.
    assert!(
        stdout.contains("### Documentation\n\n_No changes._"),
        "{stdout}"
    );
    assert!(stdout.contains("#### api\n\n_No changes._"), "{stdout}");
    assert!(stdout.contains("#### cli"), "{stdout}");
}
