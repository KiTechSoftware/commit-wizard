//! Config edits touch only what changed, and never leave a half-written file.

mod common;

use common::{Repo, text};

const ANNOTATED: &str = "\
# Team conventions -- please do not reorder
[commit]
# keep subjects short
subject_max_length = 72 # characters
use_emojis = false

# where releases come from
[branch]
remote = \"origin\" # the upstream
protected = [\"main\", \"master\"] # never push here
";

fn annotated() -> Repo {
    let repo = Repo::bare();
    repo.write(".cwizard.toml", ANNOTATED);
    repo
}

fn config(repo: &Repo) -> String {
    std::fs::read_to_string(repo.path().join(".cwizard.toml")).unwrap()
}

fn differing_lines(before: &str, after: &str) -> usize {
    before
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .count()
        + before.lines().count().abs_diff(after.lines().count())
}

#[test]
fn set_changes_one_line_and_keeps_every_comment() {
    let repo = annotated();
    let out = repo.cw(&["config", "set", "commit.subject_max_length", "50"]);
    assert!(out.status.success(), "{}", text(&out));

    let after = config(&repo);
    assert!(
        after.contains("subject_max_length = 50 # characters"),
        "{after}"
    );
    assert!(
        after.starts_with("# Team conventions -- please do not reorder\n"),
        "{after}"
    );
    assert!(
        after.contains("# keep subjects short") && after.contains("# where releases come from")
    );
    assert!(after.contains("# the upstream") && after.contains("# never push here"));
    assert_eq!(differing_lines(ANNOTATED, &after), 1, "{after}");
}

#[test]
fn unset_removes_only_that_key() {
    let repo = annotated();
    let out = repo.cw(&["config", "unset", "branch.remote"]);
    assert!(out.status.success(), "{}", text(&out));

    let after = config(&repo);
    assert!(!after.contains("remote ="), "{after}");
    assert!(
        after.contains("subject_max_length = 72 # characters"),
        "{after}"
    );
    assert!(after.contains("# where releases come from"), "{after}");
    assert!(
        after.contains("protected = [\"main\", \"master\"] # never push here"),
        "{after}"
    );
    // Exactly one line is gone and nothing else moved.
    assert_eq!(
        after,
        ANNOTATED.replace("remote = \"origin\" # the upstream\n", "")
    );
}

#[test]
fn a_key_in_a_new_table_is_appended_without_moving_anything() {
    let repo = annotated();
    let out = repo.cw(&["config", "set", "versioning.tag_prefix", "rel-"]);
    assert!(out.status.success(), "{}", text(&out));

    let after = config(&repo);
    assert!(
        after.starts_with(ANNOTATED),
        "existing text moved:\n{after}"
    );
    assert!(
        after.contains("[versioning]") && after.contains("tag_prefix = \"rel-\""),
        "{after}"
    );
}

#[test]
fn a_misspelled_table_is_left_exactly_as_written() {
    let repo = Repo::bare();
    repo.write(
        ".cwizard.toml",
        "[comit] # typo, ignored\nsubject_max_length = 5\n\n[commit]\nsubject_max_length = 72\n",
    );

    let out = repo.cw(&["config", "set", "commit.subject_max_length", "60"]);
    assert!(out.status.success(), "{}", text(&out));

    let after = config(&repo);
    assert!(
        after.starts_with("[comit] # typo, ignored\nsubject_max_length = 5\n"),
        "{after}"
    );
    assert!(after.contains("subject_max_length = 60"), "{after}");
}

#[test]
fn an_invalid_value_leaves_the_file_byte_for_byte_unchanged() {
    let repo = annotated();
    let out = repo.cw(&["config", "set", "commit.subject_max_length", "plenty"]);

    assert!(!out.status.success());
    assert_eq!(config(&repo), ANNOTATED);
}

#[test]
fn no_temporary_files_are_left_next_to_the_config() {
    let repo = annotated();
    for value in ["60", "61", "62"] {
        assert!(
            repo.cw(&["config", "set", "commit.subject_max_length", value])
                .status
                .success()
        );
    }

    let mut names: Vec<String> = std::fs::read_dir(repo.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with(".git") && name != "x.txt")
        .collect();
    names.sort();
    assert_eq!(names, [".cwizard.toml"], "unexpected files: {names:?}");
}

#[test]
fn json_and_yaml_are_still_rewritten_correctly() {
    // Surgical edits are TOML-only; the other formats are re-rendered, and must stay valid.
    let repo = Repo::bare();
    repo.write("cwizard.json", r#"{"commit":{"subject_max_length":40}}"#);
    assert!(
        repo.cw(&["config", "set", "commit.subject_max_length", "55"])
            .status
            .success()
    );
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(repo.path().join("cwizard.json")).unwrap())
            .unwrap();
    assert_eq!(json["commit"]["subject_max_length"], 55);

    let repo = Repo::bare();
    repo.write(".cwizard.yaml", "commit:\n  subject_max_length: 40\n");
    assert!(
        repo.cw(&["config", "set", "commit.subject_max_length", "55"])
            .status
            .success()
    );
    let yaml = std::fs::read_to_string(repo.path().join(".cwizard.yaml")).unwrap();
    assert!(yaml.contains("subject_max_length: 55"), "{yaml}");
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn a_symlinked_config_is_edited_through_the_link() {
        let repo = Repo::bare();
        let real = repo.path().join("dotfiles-cwizard.toml");
        std::fs::write(&real, ANNOTATED).unwrap();
        std::os::unix::fs::symlink(&real, repo.path().join(".cwizard.toml")).unwrap();

        let out = repo.cw(&["config", "set", "commit.subject_max_length", "50"]);
        assert!(out.status.success(), "{}", text(&out));

        let link = repo.path().join(".cwizard.toml");
        assert!(
            std::fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        let after = std::fs::read_to_string(&real).unwrap();
        assert!(
            after.contains("subject_max_length = 50 # characters"),
            "{after}"
        );
    }

    #[test]
    fn the_file_keeps_its_permissions() {
        let repo = annotated();
        let path = repo.path().join(".cwizard.toml");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();

        assert!(
            repo.cw(&["config", "set", "commit.subject_max_length", "50"])
                .status
                .success()
        );

        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }

    #[test]
    fn a_directory_that_cannot_be_written_leaves_the_original_intact() {
        let repo = annotated();
        let dir = repo.path();
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        let out = repo.cw(&["config", "set", "commit.subject_max_length", "50"]);

        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        // (Running as root would ignore the permission, so only check what must always hold.)
        if !out.status.success() {
            assert_eq!(config(&repo), ANNOTATED);
        }
    }
}

#[cfg(unix)]
mod edit {
    use super::*;
    use common::Pty;

    const TYPES: &str = "Which commit types will you use?";
    const SCOPES: &str = "How should commit scopes work?";
    const BRANCHES: &str = "Which branches are protected?";
    const CHECKED: &str = "Where should commit messages be checked?";
    const PUSH: &str = "Allow pushing straight to a protected branch?";
    const MORE: &str = "Customize more options?";

    /// A hand-annotated file with types, so the types question appears.
    const WITH_TYPES: &str = "\
# Team conventions
[commit]
subject_max_length = 72 # characters

# feature work
[commit.types.feat]
bump = \"Minor\"

[commit.types.fix]
bump = \"Patch\"

# docs only
[commit.types.docs]
bump = \"Patch\" # no release
";

    fn answer_through(cw: &mut Pty, scope: &str, first_type_toggle: bool) {
        cw.expect(TYPES);
        if first_type_toggle {
            cw.send(" ");
        }
        cw.line("");
        cw.expect(SCOPES);
        cw.line(scope);
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
    fn the_guided_editor_keeps_comments_and_order_too() {
        let repo = Repo::bare();
        repo.write(".cwizard.toml", WITH_TYPES);

        let mut cw = repo.cw_pty(&["config", "edit"]);
        answer_through(&mut cw, "Required", false);
        cw.expect("Save ");
        cw.line("y");
        assert!(cw.finish());

        let after = config(&repo);
        assert!(after.contains("# Team conventions"), "{after}");
        assert!(
            after.contains("subject_max_length = 72 # characters"),
            "{after}"
        );
        assert!(
            after.contains("# feature work") && after.contains("# docs only"),
            "{after}"
        );
        assert!(after.contains("bump = \"Patch\" # no release"), "{after}");
        assert!(after.contains("mode = \"required\""), "{after}");
        // The original text is still there, in order, with additions after it.
        assert!(
            after.starts_with("# Team conventions\n[commit]\nsubject_max_length = 72 # characters")
        );
    }

    #[test]
    fn removing_a_type_removes_only_that_table() {
        let repo = Repo::bare();
        repo.write(".cwizard.toml", WITH_TYPES);

        let mut cw = repo.cw_pty(&["config", "edit"]);
        // docs is the only optional type in the file, and the first option.
        answer_through(&mut cw, "Optional", true);
        cw.expect("- commit.types.docs");
        cw.line("y");
        assert!(cw.finish());

        let after = config(&repo);
        assert!(!after.contains("[commit.types.docs]"), "{after}");
        assert!(
            after.contains("[commit.types.feat]") && after.contains("# feature work"),
            "{after}"
        );
        assert!(
            after.contains("subject_max_length = 72 # characters"),
            "{after}"
        );
    }
}
