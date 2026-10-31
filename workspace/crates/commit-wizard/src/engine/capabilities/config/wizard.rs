//! The answers collected by the `cw init` wizard, and how they shape a config.
//!
//! The wizard starts from one of the built-in profiles and then adjusts it.
//! Keeping that adjustment here, away from the prompts, makes it testable and
//! guarantees the result is a valid config: it is re-parsed through the typed
//! structures before it is returned.

use toml::Value;

use crate::engine::{
    config::BaseConfig,
    error::{ErrorCode, Result},
    models::policy::enforcement::{CommitEnforcementScope, ScopeMode},
};

/// Commit types that every project keeps: versioning and changelogs rely on them.
pub const MANDATORY_TYPES: &[&str] = &["feat", "fix"];

/// What the wizard learned. A field left `None` keeps the profile's value.
#[derive(Debug, Clone, Default)]
pub struct WizardAnswers {
    /// Commit types to keep; the others are removed from the profile.
    pub keep_types: Option<Vec<String>>,
    pub scope_mode: Option<ScopeMode>,
    pub protected_branches: Option<Vec<String>>,
    pub enforce_on: Option<CommitEnforcementScope>,
    pub allow_push_to_protected: Option<bool>,
    pub subject_max_length: Option<u32>,
    pub use_emojis: Option<bool>,
    pub require_breaking_footer: Option<bool>,
    pub require_ticket: Option<bool>,
    pub tag_prefix: Option<String>,
    pub changelog_output: Option<String>,
}

impl WizardAnswers {
    /// The profile with these answers applied.
    pub fn apply(&self, profile: &BaseConfig) -> Result<BaseConfig> {
        let mut document = Value::try_from(profile).map_err(serialization_error)?;

        if let Some(keep) = &self.keep_types
            && let Some(types) = document
                .get_mut("commit")
                .and_then(|commit| commit.get_mut("types"))
                .and_then(Value::as_table_mut)
        {
            types.retain(|name, _| {
                let name: &str = name;
                keep.iter().any(|kept| kept == name) || MANDATORY_TYPES.contains(&name)
            });
        }

        if let Some(mode) = self.scope_mode {
            set(
                &mut document,
                &["commit", "scopes", "mode"],
                to_value(mode)?,
            );
        }
        if let Some(branches) = &self.protected_branches {
            let list = branches.iter().cloned().map(Value::String).collect();
            set(&mut document, &["branch", "protected"], Value::Array(list));
        }
        if let Some(scope) = self.enforce_on {
            set(
                &mut document,
                &["check", "commits", "enforce_on"],
                to_value(scope)?,
            );
        }
        if let Some(allow) = self.allow_push_to_protected {
            set(
                &mut document,
                &["push", "allow", "protected"],
                Value::Boolean(allow),
            );
        }
        if let Some(length) = self.subject_max_length {
            set(
                &mut document,
                &["commit", "subject_max_length"],
                Value::Integer(i64::from(length)),
            );
        }
        if let Some(emojis) = self.use_emojis {
            set(
                &mut document,
                &["commit", "use_emojis"],
                Value::Boolean(emojis),
            );
        }
        if let Some(required) = self.require_breaking_footer {
            set(
                &mut document,
                &["commit", "breaking", "require_footer"],
                Value::Boolean(required),
            );
        }
        if let Some(required) = self.require_ticket {
            set(
                &mut document,
                &["commit", "ticket", "required"],
                Value::Boolean(required),
            );
        }
        if let Some(prefix) = &self.tag_prefix {
            set(
                &mut document,
                &["versioning", "tag_prefix"],
                Value::String(prefix.clone()),
            );
        }
        if let Some(output) = &self.changelog_output {
            set(
                &mut document,
                &["changelog", "output"],
                Value::String(output.clone()),
            );
        }

        // Re-reading through the typed config rejects anything invalid.
        document.try_into().map_err(|err: toml::de::Error| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("reason", "the answers do not form a valid config")
                .with_context("error", err.to_string())
        })
    }
}

/// Names of the commit types a profile defines, in order.
pub fn profile_types(profile: &BaseConfig) -> Vec<String> {
    profile
        .commit
        .as_ref()
        .and_then(|commit| commit.types.as_ref())
        .map(|types| types.keys().cloned().collect())
        .unwrap_or_default()
}

/// A comma- or space-separated list typed by a person: trimmed, without empty
/// entries or repeats, keeping the order given.
pub fn split_list(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in text.split([',', ' ', '\t']) {
        let item = item.trim();
        if !item.is_empty() && !out.iter().any(|seen| seen == item) {
            out.push(item.to_string());
        }
    }
    out
}

fn to_value<T: serde::Serialize>(value: T) -> Result<Value> {
    Value::try_from(value).map_err(serialization_error)
}

fn serialization_error(err: toml::ser::Error) -> crate::engine::Error {
    ErrorCode::SerializationFailure
        .error()
        .with_context("error", err.to_string())
}

/// Set `path` in a document, creating the tables on the way.
fn set(document: &mut Value, path: &[&str], value: Value) {
    let mut current = document;
    for (index, key) in path.iter().enumerate() {
        let Some(table) = current.as_table_mut() else {
            return;
        };
        if index + 1 == path.len() {
            table.insert((*key).to_string(), value);
            return;
        }
        current = table
            .entry((*key).to_string())
            .or_insert_with(|| Value::Table(Default::default()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard() -> BaseConfig {
        BaseConfig::standard()
    }

    #[test]
    fn no_answers_leave_the_profile_untouched() {
        let profile = standard();
        let applied = WizardAnswers::default().apply(&profile).unwrap();
        assert_eq!(
            Value::try_from(&applied).unwrap(),
            Value::try_from(&profile).unwrap()
        );
    }

    #[test]
    fn unselected_types_are_removed_but_feat_and_fix_always_stay() {
        let answers = WizardAnswers {
            keep_types: Some(vec!["docs".to_string()]),
            ..Default::default()
        };
        let types = profile_types(&answers.apply(&standard()).unwrap());

        assert_eq!(types, ["docs", "feat", "fix"]);
    }

    #[test]
    fn scalar_answers_land_on_the_right_keys() {
        let answers = WizardAnswers {
            scope_mode: Some(ScopeMode::Required),
            protected_branches: Some(vec!["main".to_string(), "release/*".to_string()]),
            enforce_on: Some(CommitEnforcementScope::AllBranches),
            allow_push_to_protected: Some(true),
            subject_max_length: Some(50),
            use_emojis: Some(true),
            require_breaking_footer: Some(true),
            require_ticket: Some(true),
            tag_prefix: Some("rel-".to_string()),
            changelog_output: Some("docs/CHANGES.md".to_string()),
            ..Default::default()
        };
        let config = answers.apply(&standard()).unwrap();

        assert!(matches!(config.commit_scopes_mode(), ScopeMode::Required));
        assert_eq!(config.commit_subject_max_length(), 50);
        assert!(config.commit_use_emojis());
        assert!(config.commit_breaking_require_footer());
        assert_eq!(config.versioning_tag_prefix(), "rel-");
        assert_eq!(config.changelog_output(), "docs/CHANGES.md");
        assert!(config.push_allow_protected());
        assert_eq!(
            config.branch.as_ref().unwrap().protected.as_deref(),
            Some(&["main".to_string(), "release/*".to_string()][..])
        );
        assert!(
            config
                .commit
                .as_ref()
                .unwrap()
                .ticket
                .as_ref()
                .unwrap()
                .required
                .unwrap()
        );
    }

    #[test]
    fn answers_work_on_a_profile_with_no_tables_at_all() {
        let answers = WizardAnswers {
            scope_mode: Some(ScopeMode::Disabled),
            tag_prefix: Some(String::new()),
            ..Default::default()
        };
        let config = answers.apply(&BaseConfig::minimal()).unwrap();

        assert!(matches!(config.commit_scopes_mode(), ScopeMode::Disabled));
        assert_eq!(config.versioning_tag_prefix(), "");
    }

    #[test]
    fn lists_are_split_trimmed_and_deduplicated() {
        assert_eq!(
            split_list("main, master  release/*,,main"),
            ["main", "master", "release/*"]
        );
        assert!(split_list("  ,  ").is_empty());
    }
}
