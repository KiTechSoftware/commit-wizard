//! Surgical edits to TOML config files.
//!
//! Re-rendering a whole file after a one-key change drops the user's comments,
//! blank lines and key order. This module instead applies the difference
//! between two documents to the original text: only the keys that changed are
//! touched, and everything else is byte-for-byte what the user wrote.

use toml::Value;
use toml_edit::{DocumentMut, Item, TableLike, Value as EditValue};

use crate::engine::error::{ErrorCode, Result};

/// `raw` with the changes from `before` to `after` applied.
///
/// `before` and `after` describe the same document (parsed through the typed
/// config) so keys the typed config does not model are in neither, and are left
/// alone. Comments and formatting around untouched keys are preserved, and a
/// replaced value keeps the comment that trails it.
pub fn patch_toml(raw: &str, before: &Value, after: &Value) -> Result<String> {
    let mut document: DocumentMut = raw.parse().map_err(|err: toml_edit::TomlError| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("format", "toml")
            .with_context("error", err.to_string())
    })?;

    if let (Some(old), Some(new)) = (before.as_table(), after.as_table()) {
        patch_table(document.as_table_mut(), old, new)?;
    }
    Ok(document.to_string())
}

fn patch_table(
    target: &mut dyn TableLike,
    before: &toml::map::Map<String, Value>,
    after: &toml::map::Map<String, Value>,
) -> Result<()> {
    for (key, new) in after {
        match before.get(key) {
            Some(old) if old == new => {}
            Some(Value::Table(old_table)) if new.is_table() => {
                let new_table = new.as_table().expect("checked above");
                match target.get_mut(key).and_then(Item::as_table_like_mut) {
                    Some(child) => patch_table(child, old_table, new_table)?,
                    None => {
                        target.insert(key, to_item(new)?);
                    }
                }
            }
            _ => replace_value(target, key, new)?,
        }
    }

    for key in before.keys() {
        if !after.contains_key(key) {
            target.remove(key);
        }
    }
    Ok(())
}

/// Set `key` to `new`, keeping the comment and spacing around an existing value.
fn replace_value(target: &mut dyn TableLike, key: &str, new: &Value) -> Result<()> {
    if !new.is_table()
        && let Some(Item::Value(existing)) = target.get_mut(key)
    {
        let mut value = to_edit_value(new)?;
        *value.decor_mut() = existing.decor().clone();
        *existing = value;
        return Ok(());
    }
    target.insert(key, to_item(new)?);
    Ok(())
}

fn to_edit_value(value: &Value) -> Result<EditValue> {
    value.to_string().parse::<EditValue>().map_err(|err| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("error", err.to_string())
    })
}

fn to_item(value: &Value) -> Result<Item> {
    match value {
        Value::Table(table) => {
            let text = toml::to_string(table).map_err(|err| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("error", err.to_string())
            })?;
            let document: DocumentMut = text.parse().map_err(|err: toml_edit::TomlError| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("error", err.to_string())
            })?;
            Ok(Item::Table(document.as_table().clone()))
        }
        other => Ok(Item::Value(to_edit_value(other)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Value {
        toml::from_str(text).expect("valid toml")
    }

    /// Patch `raw` so it represents `after`, asserting the result parses to it.
    fn patch(raw: &str, after_text: &str) -> String {
        let patched = patch_toml(raw, &doc(raw), &doc(after_text)).unwrap();
        assert_eq!(doc(&patched), doc(after_text), "patched text:\n{patched}");
        patched
    }

    const ANNOTATED: &str = "\
# Project settings -- hand written
[commit]
# how long a subject may be
subject_max_length = 72 # keep it short
use_emojis = false

# branch rules
[branch]
remote = \"origin\"
protected = [\"main\", \"master\"] # never push here
";

    #[test]
    fn no_change_returns_the_text_untouched() {
        let patched = patch_toml(ANNOTATED, &doc(ANNOTATED), &doc(ANNOTATED)).unwrap();
        assert_eq!(patched, ANNOTATED);
    }

    #[test]
    fn changing_a_value_keeps_every_comment_and_the_order() {
        let after = ANNOTATED.replace("subject_max_length = 72", "subject_max_length = 50");
        let patched = patch(ANNOTATED, &after);

        assert!(
            patched.contains("subject_max_length = 50 # keep it short"),
            "{patched}"
        );
        assert!(patched.contains("# Project settings -- hand written"));
        assert!(patched.contains("# how long a subject may be"));
        assert!(patched.contains("# branch rules"));
        // Only that one line differs.
        let changed = ANNOTATED
            .lines()
            .zip(patched.lines())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(changed, 1, "{patched}");
    }

    #[test]
    fn replacing_an_array_keeps_the_comment_that_trails_it() {
        let after = ANNOTATED.replace("[\"main\", \"master\"]", "[\"trunk\"]");
        let patched = patch(ANNOTATED, &after);
        assert!(
            patched.contains("protected = [\"trunk\"] # never push here"),
            "{patched}"
        );
    }

    #[test]
    fn a_new_key_is_added_to_its_table_without_disturbing_the_rest() {
        let after = format!("{ANNOTATED}\n").replace(
            "remote = \"origin\"\n",
            "remote = \"origin\"\nnaming = \"x\"\n",
        );
        let patched = patch(ANNOTATED, &after);
        assert!(patched.contains("naming = \"x\""));
        assert!(patched.contains("# never push here"));
    }

    #[test]
    fn a_new_table_is_appended() {
        let after = format!("{ANNOTATED}\n[versioning]\ntag_prefix = \"v\"\n");
        let patched = patch(ANNOTATED, &after);
        assert!(
            patched.starts_with(ANNOTATED),
            "existing text moved:\n{patched}"
        );
        assert!(patched.contains("[versioning]"));
    }

    #[test]
    fn a_removed_key_leaves_its_neighbours_and_comments() {
        let after = ANNOTATED.replace("use_emojis = false\n", "");
        let patched = patch(ANNOTATED, &after);
        assert!(!patched.contains("use_emojis"));
        assert!(patched.contains("# how long a subject may be"));
        assert!(patched.contains("subject_max_length = 72 # keep it short"));
    }

    #[test]
    fn a_removed_table_goes_but_other_tables_stay() {
        let after = "[commit]\nsubject_max_length = 72\nuse_emojis = false\n";
        let patched = patch(ANNOTATED, after);
        assert!(!patched.contains("[branch]"));
        assert!(patched.contains("# Project settings -- hand written"));
    }

    #[test]
    fn nested_tables_are_patched_in_place() {
        let raw = "\
[commit.types.feat]
# minor bump
bump = \"Minor\" # features
description = \"A feature\"

[commit.types.fix]
bump = \"Patch\"
";
        let after = raw.replace("\"Minor\"", "\"Major\"");
        let patched = patch(raw, &after);
        assert!(patched.contains("bump = \"Major\" # features"), "{patched}");
        assert!(patched.contains("# minor bump"));
    }

    #[test]
    fn a_whole_new_nested_table_is_written_as_a_table() {
        let raw = "[commit.types.feat]\nbump = \"Minor\"\n";
        let after = "[commit.types.feat]\nbump = \"Minor\"\n[commit.types.docs]\ndescription = \"d\"\nbump = \"Patch\"\n";
        let patched = patch(raw, after);
        assert!(patched.starts_with(raw), "{patched}");
        assert!(patched.contains("[commit.types.docs]"), "{patched}");
    }

    #[test]
    fn keys_the_typed_config_does_not_model_are_never_touched() {
        let raw = "[comit]\nsubject_max_length = 5 # typo\n\n[commit]\nsubject_max_length = 72\n";
        // The typed view contains only `commit`.
        let before = doc("[commit]\nsubject_max_length = 72\n");
        let after = doc("[commit]\nsubject_max_length = 60\n");

        let patched = patch_toml(raw, &before, &after).unwrap();

        assert!(
            patched.contains("[comit]\nsubject_max_length = 5 # typo"),
            "{patched}"
        );
        assert!(patched.contains("subject_max_length = 60"));
    }

    #[test]
    fn inline_tables_and_dotted_keys_are_patched() {
        let raw = "commit = { subject_max_length = 72, use_emojis = false }\n";
        let after = "commit = { subject_max_length = 50, use_emojis = false }\n";
        assert!(patch(raw, after).contains("subject_max_length = 50"));

        let raw = "commit.subject_max_length = 72 # dotted\n";
        let after = "commit.subject_max_length = 50\n";
        assert!(patch(raw, after).contains("commit.subject_max_length = 50 # dotted"));
    }

    #[test]
    fn invalid_toml_is_an_error_not_a_panic() {
        assert!(patch_toml("not = [valid", &doc(""), &doc("a = 1")).is_err());
    }
}
