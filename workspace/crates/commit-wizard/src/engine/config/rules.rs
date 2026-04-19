use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use toml::Value;

use crate::engine::error::{ErrorCode, Result};

// external representation of rules.toml for global and registry rules
// also used for the rules section in project config, but with different resolution semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RulesConfig(pub Value);

impl Default for RulesConfig {
    fn default() -> Self {
        Self(Value::Table(Default::default()))
    }
}

impl RulesConfig {
    pub fn get(&self, path: &str) -> Option<&Value> {
        let mut current = &self.0;

        for segment in path.split('.') {
            current = current.get(segment)?;
        }

        Some(current)
    }
    pub fn from_toml_str(input: &str) -> Result<Self> {
        toml::from_str(input).map_err(|err| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("error", err.to_string())
        })
    }
    pub fn resolve_ref(&self, reference: &str) -> Option<&Value> {
        let path = reference.strip_prefix("@rules.")?;
        self.get(path)
    }

    pub fn resolve<T>(&self, reference: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = self.resolve_ref(reference).cloned().ok_or_else(|| {
            ErrorCode::ConfigReferenceInvalid
                .error()
                .with_context("reference", reference)
        })?;

        value.try_into().map_err(|err| {
            ErrorCode::ConfigReferenceInvalid
                .error()
                .with_context("reference", reference)
                .with_context("error", err.to_string())
        })
    }

    pub fn is_reference(value: &str) -> bool {
        value.starts_with("@rules.")
    }

    /// Resolve a string that may contain a rule reference.
    /// If the string is a @rules.* reference, resolves it and returns the string value.
    /// Otherwise, returns the string as-is.
    pub fn resolve_string(&self, value: &str) -> Result<String> {
        if Self::is_reference(value) {
            let resolved: String = self.resolve(value)?;
            Ok(resolved)
        } else {
            Ok(value.to_string())
        }
    }

    pub fn resolve_value_refs(&self, value: &mut Value) -> Result<()> {
        let mut visiting = BTreeSet::new();
        self.resolve_value_refs_inner(value, "$", &mut visiting)
    }

    fn resolve_value_refs_inner(
        &self,
        value: &mut Value,
        path: &str,
        visiting: &mut BTreeSet<String>,
    ) -> Result<()> {
        match value {
            Value::String(s) if Self::is_reference(s) => {
                let reference = s.clone();
                let resolved = self.resolve_ref(&reference).cloned().ok_or_else(|| {
                    ErrorCode::ConfigReferenceInvalid
                        .error()
                        .with_context("reference", reference.clone())
                        .with_context("path", path)
                })?;

                if !visiting.insert(reference.clone()) {
                    return Err(ErrorCode::ConfigReferenceInvalid
                        .error()
                        .with_context("reference", reference)
                        .with_context("path", path)
                        .with_context("reason", "cyclic reference detected"));
                }

                let mut resolved = resolved;
                self.resolve_value_refs_inner(&mut resolved, path, visiting)?;
                visiting.remove(&reference);

                *value = resolved;
                Ok(())
            }
            Value::Array(items) => {
                for (index, item) in items.iter_mut().enumerate() {
                    let child_path = format!("{path}[{index}]");
                    self.resolve_value_refs_inner(item, &child_path, visiting)?;
                }
                Ok(())
            }
            Value::Table(table) => {
                for (key, item) in table.iter_mut() {
                    let child_path = format!("{path}.{key}");
                    self.resolve_value_refs_inner(item, &child_path, visiting)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_nested_rule_references() {
        let rules: RulesConfig = toml::from_str(
            r#"
            a = "@rules.b"
            b = "@rules.c"
            c = "final"
            "#,
        )
        .unwrap();

        let mut value = Value::String("@rules.a".into());
        rules.resolve_value_refs(&mut value).unwrap();

        assert_eq!(value, Value::String("final".into()));
    }

    #[test]
    fn rejects_cyclic_rule_references() {
        let rules: RulesConfig = toml::from_str(
            r#"
            a = "@rules.b"
            b = "@rules.a"
            "#,
        )
        .unwrap();

        let mut value = Value::String("@rules.a".into());
        let err = rules.resolve_value_refs(&mut value).unwrap_err();

        assert_eq!(err.code, ErrorCode::ConfigReferenceInvalid);
    }
}
