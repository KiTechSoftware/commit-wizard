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
}
