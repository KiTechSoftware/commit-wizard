//! Flat views of configuration: the settings in one file, or the resolved
//! configuration with the layer each value came from.

use std::collections::BTreeMap;

use std::path::Path;

use serde::Serialize;
use toml::Value;

use super::show::ConfigTarget;
use crate::engine::{
    config::{ConfigFormat, ProjectConfig, StandardConfig},
    error::{ErrorCode, Result},
};

/// One setting in the resolved configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveRow {
    pub key: String,
    pub value: String,
    /// The layer that supplied the value, or `default` for built-in defaults.
    pub source: String,
}

/// A configuration document as a neutral tree, whatever format it was written in.
pub fn to_document<T: Serialize>(config: &T) -> Option<Value> {
    Value::try_from(config).ok()
}

/// Read a config file (in whatever format it uses) as a neutral document.
pub fn file_document(target: ConfigTarget, path: &Path) -> Result<Value> {
    if !path.exists() {
        return Err(ErrorCode::ConfigUnreadable
            .error()
            .with_context("path", path.display().to_string())
            .with_context("reason", "config file does not exist"));
    }
    let raw = std::fs::read_to_string(path)?;
    let format = ConfigFormat::from_path(path);
    let document = match target {
        ConfigTarget::Project => to_document(&ProjectConfig::from_str_as(format, &raw)?),
        ConfigTarget::Global => to_document(&StandardConfig::from_str_as(format, &raw)?),
    };
    document.ok_or_else(|| ErrorCode::SerializationFailure.error())
}

/// Every leaf setting of a document, by dotted key. Scalars and arrays are
/// leaves (arrays print inline); tables are walked.
pub fn flatten(document: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    walk(document, String::new(), &mut out);
    out
}

fn walk(value: &Value, path: String, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Table(table) => {
            for (key, child) in table {
                let next = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                walk(child, next, out);
            }
        }
        leaf => {
            if !path.is_empty() {
                out.insert(path, leaf.to_string());
            }
        }
    }
}

/// Settings of the resolved configuration, each with the layer that set it.
///
/// `layers` are `(label, document)` pairs, highest priority first. A value is
/// attributed to the first layer that defines its key; a key no layer defines
/// can only have come from the built-in defaults.
pub fn effective_rows(
    resolved: &Value,
    layers: &[(String, Value)],
    prefix: Option<&str>,
) -> Vec<EffectiveRow> {
    let flattened: Vec<(&str, BTreeMap<String, String>)> = layers
        .iter()
        .map(|(label, document)| (label.as_str(), flatten(document)))
        .collect();

    filter_prefix(flatten(resolved), prefix)
        .into_iter()
        .map(|(key, value)| {
            let source = flattened
                .iter()
                .find(|(_, keys)| keys.contains_key(&key))
                .map(|(label, _)| (*label).to_string())
                .unwrap_or_else(|| "default".to_string());
            EffectiveRow { key, value, source }
        })
        .collect()
}

/// One line of a before/after comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// A setting, or a whole table (`commit.types.chore`) that appeared or vanished.
    pub key: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

impl Change {
    pub fn describe(&self) -> String {
        match (&self.before, &self.after) {
            (Some(before), Some(after)) => format!("{}: {before} -> {after}", self.key),
            (None, Some(after)) if after.is_empty() => format!("+ {}", self.key),
            (None, Some(after)) => format!("+ {} = {after}", self.key),
            (Some(_), None) => format!("- {}", self.key),
            (None, None) => self.key.clone(),
        }
    }
}

/// What differs between two documents. A table that was added or removed as a
/// whole (a commit type, say) is one line, not one per setting inside it.
pub fn changes(before: &Value, after: &Value) -> Vec<Change> {
    let old = flatten(before);
    let new = flatten(after);
    let mut out: Vec<Change> = Vec::new();

    for (key, value) in &old {
        match new.get(key) {
            Some(now) if now != value => out.push(Change {
                key: key.clone(),
                before: Some(value.clone()),
                after: Some(now.clone()),
            }),
            None => out.push(Change {
                key: key.clone(),
                before: Some(value.clone()),
                after: None,
            }),
            _ => {}
        }
    }
    for (key, value) in &new {
        if !old.contains_key(key) {
            out.push(Change {
                key: key.clone(),
                before: None,
                after: Some(value.clone()),
            });
        }
    }

    collapse_tables(&mut out, &old, &new);
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
}

/// Replace a group of removals (or additions) that empties (or creates) a whole
/// table with a single line naming the table.
fn collapse_tables(
    changes: &mut Vec<Change>,
    old: &BTreeMap<String, String>,
    new: &BTreeMap<String, String>,
) {
    for removing in [true, false] {
        let (source, other) = if removing { (old, new) } else { (new, old) };
        let parent_of = |key: &str| key.rsplit_once('.').map(|(parent, _)| parent.to_string());

        let candidates: Vec<String> = changes
            .iter()
            .filter(|c| {
                if removing {
                    c.after.is_none()
                } else {
                    c.before.is_none()
                }
            })
            .filter_map(|c| parent_of(&c.key))
            .collect();

        let mut candidates = candidates;
        candidates.sort();
        candidates.dedup();

        for parent in candidates {
            let prefix = format!("{parent}.");
            if !changes.iter().any(|c| c.key.starts_with(&prefix)) {
                continue; // already collapsed into a larger table
            }
            let leaves = |map: &BTreeMap<String, String>| {
                map.keys().filter(|k| k.starts_with(&prefix)).count()
            };
            // Every leaf of that table is on one side only, and it has several.
            if leaves(source) > 1 && leaves(other) == 0 {
                changes.retain(|c| !c.key.starts_with(&prefix));
                changes.push(Change {
                    key: parent,
                    before: removing.then(String::new),
                    after: (!removing).then(String::new),
                });
            }
        }
    }
}

/// Settings of a single document, in key order.
pub fn document_rows(document: &Value, prefix: Option<&str>) -> Vec<(String, String)> {
    filter_prefix(flatten(document), prefix)
        .into_iter()
        .collect()
}

fn filter_prefix(
    settings: BTreeMap<String, String>,
    prefix: Option<&str>,
) -> BTreeMap<String, String> {
    match prefix.map(str::trim).filter(|p| !p.is_empty()) {
        Some(prefix) => settings
            .into_iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .collect(),
        None => settings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Value {
        toml::from_str(text).expect("valid toml")
    }

    #[test]
    fn flattens_tables_to_dotted_keys_and_keeps_arrays_inline() {
        let flat = flatten(&doc(
            "[commit]\nsubject_max_length = 72\n[commit.types.feat]\ndescription = \"A feature\"\n[branch]\nprotected = [\"main\", \"dev\"]\nflag = true\n",
        ));

        assert_eq!(flat["commit.subject_max_length"], "72");
        assert_eq!(flat["commit.types.feat.description"], "\"A feature\"");
        assert_eq!(flat["branch.protected"], "[\"main\", \"dev\"]");
        assert_eq!(flat["branch.flag"], "true");
        assert!(!flat.contains_key("commit"), "tables are not rows");
    }

    #[test]
    fn each_value_is_attributed_to_the_highest_priority_layer_that_sets_it() {
        let resolved = doc("[commit]\nsubject_max_length = 50\nuse_emojis = true\n[push]\nx = 1\n");
        let layers = [
            (
                "cli".to_string(),
                doc("[commit]\nsubject_max_length = 50\n"),
            ),
            (
                "project".to_string(),
                doc("[commit]\nsubject_max_length = 72\n"),
            ),
            (
                "registry:team".to_string(),
                doc("[commit]\nuse_emojis = true\nsubject_max_length = 100\n"),
            ),
        ];

        let rows = effective_rows(&resolved, &layers, None);
        let source = |key: &str| rows.iter().find(|r| r.key == key).unwrap().source.clone();

        assert_eq!(source("commit.subject_max_length"), "cli");
        assert_eq!(source("commit.use_emojis"), "registry:team");
        assert_eq!(source("push.x"), "default", "no layer sets it");
    }

    #[test]
    fn prefix_narrows_the_rows() {
        let resolved = doc("[commit]\na = 1\n[commit.types.fix]\nb = 2\n[branch]\nc = 3\n");

        let all = effective_rows(&resolved, &[], None);
        let commit = effective_rows(&resolved, &[], Some("commit"));
        let types = effective_rows(&resolved, &[], Some("commit.types"));

        assert_eq!(all.len(), 3);
        assert_eq!(commit.len(), 2);
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].key, "commit.types.fix.b");
        // Blank prefixes do not filter.
        assert_eq!(effective_rows(&resolved, &[], Some("  ")).len(), 3);
    }

    #[test]
    fn rows_are_sorted_by_key() {
        let rows = effective_rows(&doc("[b]\nx = 1\n[a]\ny = 2\n"), &[], None);
        let keys: Vec<&str> = rows.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(keys, ["a.y", "b.x"]);
    }

    #[test]
    fn changed_added_and_removed_settings_are_reported() {
        let before = doc("[commit]\nsubject_max_length = 72\nuse_emojis = false\n");
        let after = doc("[commit]\nsubject_max_length = 50\n[branch]\nremote = \"up\"\n");

        let found = changes(&before, &after);
        let lines: Vec<String> = found.iter().map(Change::describe).collect();

        assert_eq!(
            lines,
            [
                "+ branch.remote = \"up\"",
                "commit.subject_max_length: 72 -> 50",
                "- commit.use_emojis",
            ]
        );
    }

    #[test]
    fn a_whole_table_added_or_removed_is_one_line() {
        let with = doc(
            "[commit.types.chore]\ndescription = \"x\"\nbump = \"Patch\"\nsection = \"S\"\n[commit.types.feat]\nbump = \"Minor\"\n",
        );
        let without = doc("[commit.types.feat]\nbump = \"Minor\"\n");

        let removed = changes(&with, &without);
        let added = changes(&without, &with);

        assert_eq!(
            removed.iter().map(Change::describe).collect::<Vec<_>>(),
            ["- commit.types.chore"]
        );
        assert_eq!(
            added.iter().map(Change::describe).collect::<Vec<_>>(),
            ["+ commit.types.chore"]
        );
    }

    #[test]
    fn identical_documents_have_no_changes() {
        let a = doc("[commit]\nsubject_max_length = 72\n");
        assert!(changes(&a, &a).is_empty());
    }
}
