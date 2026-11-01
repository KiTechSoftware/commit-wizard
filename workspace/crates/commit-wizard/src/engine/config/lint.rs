//! Checks on config documents that typed parsing does not catch.
//!
//! The top level of a config file is flattened into several tables, which
//! defeats `deny_unknown_fields`: a misspelled table such as `[comit]` parses
//! fine and is silently ignored. These checks find such keys so they can be
//! reported, with a suggestion when one is close.

use super::ConfigFormat;

/// Tables and keys that may appear at the top level of a config file.
pub const KNOWN_TOP_LEVEL_KEYS: &[&str] = &[
    "version",
    "commit",
    "branch",
    "pr",
    "check",
    "push",
    "versioning",
    "changelog",
    "release",
    "hooks",
    "ai",
    "registry",
    "registries",
    "global",
    "rules",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKey {
    pub key: String,
    /// The known key it is probably meant to be.
    pub suggestion: Option<String>,
}

impl UnknownKey {
    pub fn describe(&self) -> String {
        match &self.suggestion {
            Some(suggestion) => format!(
                "unknown top-level key `{}`; did you mean `{suggestion}`?",
                self.key
            ),
            None => format!("unknown top-level key `{}`", self.key),
        }
    }
}

/// Top-level keys of `input` that cw does not know, in document order.
/// A document that does not parse yields nothing; that error is reported elsewhere.
pub fn unknown_top_level_keys(format: ConfigFormat, input: &str) -> Vec<UnknownKey> {
    let Ok(document) = format.parse::<serde_json::Map<String, serde_json::Value>>(input) else {
        return Vec::new();
    };
    document
        .keys()
        .filter(|key| !KNOWN_TOP_LEVEL_KEYS.contains(&key.as_str()))
        .map(|key| UnknownKey {
            key: key.clone(),
            suggestion: closest(key),
        })
        .collect()
}

/// The known key closest to `key`, if it is close enough to be a typo.
fn closest(key: &str) -> Option<String> {
    let key = key.to_ascii_lowercase();
    KNOWN_TOP_LEVEL_KEYS
        .iter()
        .map(|known| (edit_distance(&key, known), *known))
        .filter(|(distance, known)| *distance <= (known.len() / 3).max(1))
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, known)| known.to_string())
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            current.push(substitution.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(format: ConfigFormat, input: &str) -> Vec<(String, Option<String>)> {
        unknown_top_level_keys(format, input)
            .into_iter()
            .map(|k| (k.key, k.suggestion))
            .collect()
    }

    #[test]
    fn known_tables_are_fine_in_every_format() {
        assert!(
            keys(
                ConfigFormat::Toml,
                "[commit]\na = 1\n[global]\ninherit = true\n"
            )
            .is_empty()
        );
        assert!(keys(ConfigFormat::Yaml, "commit:\n  a: 1\nregistry:\n  use: x\n").is_empty());
        assert!(keys(ConfigFormat::Json, r#"{"commit":{},"version":1}"#).is_empty());
    }

    #[test]
    fn typos_get_a_suggestion() {
        assert_eq!(
            keys(ConfigFormat::Toml, "[comit]\na = 1\n"),
            [("comit".to_string(), Some("commit".to_string()))]
        );
        assert_eq!(
            keys(ConfigFormat::Yaml, "chnagelog:\n  output: x\n"),
            [("chnagelog".to_string(), Some("changelog".to_string()))]
        );
        assert_eq!(
            keys(ConfigFormat::Json, r#"{"registies":{}}"#),
            [("registies".to_string(), Some("registries".to_string()))]
        );
    }

    #[test]
    fn unrelated_keys_are_reported_without_a_suggestion() {
        assert_eq!(
            keys(ConfigFormat::Toml, "[telemetry]\nx = 1\n"),
            [("telemetry".to_string(), None)]
        );
    }

    #[test]
    fn capitalisation_slips_are_caught() {
        assert_eq!(
            keys(ConfigFormat::Yaml, "Commit:\n  a: 1\n"),
            [("Commit".to_string(), Some("commit".to_string()))]
        );
    }

    #[test]
    fn an_unparseable_document_yields_nothing() {
        assert!(keys(ConfigFormat::Json, "{ not json").is_empty());
    }

    #[test]
    fn the_message_names_the_key_and_the_fix() {
        let key = &unknown_top_level_keys(ConfigFormat::Toml, "[comit]\n")[0];
        assert_eq!(
            key.describe(),
            "unknown top-level key `comit`; did you mean `commit`?"
        );
    }
}
