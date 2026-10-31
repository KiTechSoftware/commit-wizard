use std::path::{Path, PathBuf};

use toml::Value;

use crate::engine::{
    config::{ConfigFormat, ProjectConfig, StandardConfig},
    error::{ErrorCode, Result},
};

use super::{
    patch::patch_toml,
    show::{ConfigTarget, resolve_config_path},
};

#[derive(Debug, Clone)]
pub struct ConfigGetInput<'a> {
    pub cwd: &'a Path,
    pub target: ConfigTarget,
    pub explicit_path: Option<&'a Path>,
    pub key: &'a str,
}

pub struct ConfigGetOutput {
    pub path: PathBuf,
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct ConfigSetInput<'a> {
    pub cwd: &'a Path,
    pub target: ConfigTarget,
    pub key: &'a str,
    pub value: &'a str,
    pub dry_run: bool,
    pub explicit_path: Option<&'a Path>,
}

pub struct ConfigSetOutput {
    pub path: PathBuf,
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct ConfigUnsetInput<'a> {
    pub cwd: &'a Path,
    pub target: ConfigTarget,
    pub key: &'a str,
    pub dry_run: bool,
    pub explicit_path: Option<&'a Path>,
}

pub struct ConfigUnsetOutput {
    pub path: PathBuf,
    pub key: String,
    pub removed: bool,
}

pub fn config_get(input: &ConfigGetInput<'_>) -> Result<ConfigGetOutput> {
    ensure_supported_key(input.key)?;

    let path = if let Some(explicit_path) = input.explicit_path {
        explicit_path.to_path_buf()
    } else {
        resolve_config_path(input.target, input.cwd)?
    };
    if !path.exists() {
        return Err(ErrorCode::ConfigUnreadable
            .error()
            .with_context("path", path.display().to_string())
            .with_context("reason", "config file does not exist"));
    }

    let raw = std::fs::read_to_string(&path)?;
    let doc = parse_doc(input.target, ConfigFormat::from_path(&path), &raw)?;

    let value = get_value(&doc, input.key).cloned().ok_or_else(|| {
        ErrorCode::ConfigInvalid
            .error()
            .with_context("key", input.key)
            .with_context("reason", "unknown key")
    })?;

    Ok(ConfigGetOutput {
        path,
        key: input.key.to_string(),
        value,
    })
}

pub fn config_set(input: &ConfigSetInput<'_>) -> Result<ConfigSetOutput> {
    ensure_supported_key(input.key)?;

    let path = if let Some(explicit_path) = input.explicit_path {
        explicit_path.to_path_buf()
    } else {
        resolve_config_path(input.target, input.cwd)?
    };
    let format = ConfigFormat::from_path(&path);
    let raw = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        default_doc(input.target, format)?
    };

    let mut doc = parse_doc(input.target, format, &raw)?;
    let before = doc.clone();
    let value = parse_value_for_key(input.key, input.value)?;

    set_value(&mut doc, input.key, value.clone())?;
    let validated = validate_doc(input.target, doc)?;
    let rendered = render_for_save(input.target, format, &raw, &before, &validated)?;

    if !input.dry_run {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        crate::engine::fs::write_text(&path, &rendered)?;
    }

    Ok(ConfigSetOutput {
        path,
        key: input.key.to_string(),
        value,
    })
}

pub fn config_unset(input: &ConfigUnsetInput<'_>) -> Result<ConfigUnsetOutput> {
    ensure_supported_key(input.key)?;

    let path = if let Some(explicit_path) = input.explicit_path {
        explicit_path.to_path_buf()
    } else {
        resolve_config_path(input.target, input.cwd)?
    };
    if !path.exists() {
        return Err(ErrorCode::ConfigUnreadable
            .error()
            .with_context("path", path.display().to_string())
            .with_context("reason", "config file does not exist"));
    }

    let format = ConfigFormat::from_path(&path);
    let raw = std::fs::read_to_string(&path)?;
    let mut doc = parse_doc(input.target, format, &raw)?;
    let before = doc.clone();
    let removed = unset_value(&mut doc, input.key)?;

    let validated = validate_doc(input.target, doc)?;
    let rendered = render_for_save(input.target, format, &raw, &before, &validated)?;

    if !input.dry_run {
        crate::engine::fs::write_text(&path, &rendered)?;
    }

    Ok(ConfigUnsetOutput {
        path,
        key: input.key.to_string(),
        removed,
    })
}

/// The text to save for `after`, given the file as it was.
///
/// A TOML file is patched in place, so comments, spacing and key order survive
/// and only the changed keys differ. The patch is checked by reading it back:
/// if it does not parse to exactly `after` (it always should), the whole file
/// is re-rendered instead, which is always correct but drops comments. JSON
/// and YAML files are always re-rendered.
fn render_for_save(
    target: ConfigTarget,
    format: ConfigFormat,
    raw: &str,
    before: &Value,
    after: &Value,
) -> Result<String> {
    if format == ConfigFormat::Toml
        && let Ok(patched) = patch_toml(raw, before, after)
        && parse_doc(target, format, &patched).is_ok_and(|reparsed| reparsed == *after)
    {
        return Ok(patched);
    }
    format.render(after)
}

fn ensure_supported_key(key: &str) -> Result<()> {
    if is_supported_key(key) {
        Ok(())
    } else {
        Err(ErrorCode::ConfigInvalid
            .error()
            .with_context("key", key)
            .with_context("reason", "unsupported or unknown key"))
    }
}

fn is_supported_key(key: &str) -> bool {
    matches!(
        key,
        "commit.subject_max_length"
            | "commit.ticket.required"
            | "commit.ticket.pattern"
            | "commit.ticket.header_format"
            | "branch.remote"
            | "release.enabled"
            | "release.source_branch"
            | "release.target_branch"
            | "ai.enabled"
            | "ai.provider"
            | "global.inherit"
            | "changelog.output"
            | "versioning.tag_prefix"
    )
}

fn parse_doc(target: ConfigTarget, format: ConfigFormat, raw: &str) -> Result<Value> {
    match target {
        ConfigTarget::Project => {
            let parsed = ProjectConfig::from_str_as(format, raw)?;
            toml::Value::try_from(parsed).map_err(|err| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("error", err.to_string())
            })
        }
        ConfigTarget::Global => {
            let parsed = StandardConfig::from_str_as(format, raw)?;
            toml::Value::try_from(parsed).map_err(|err| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("error", err.to_string())
            })
        }
    }
}

fn validate_doc(target: ConfigTarget, value: Value) -> Result<Value> {
    match target {
        ConfigTarget::Project => {
            let parsed: ProjectConfig = value.clone().try_into().map_err(|err| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("error", err.to_string())
            })?;
            toml::Value::try_from(parsed).map_err(|err| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("error", err.to_string())
            })
        }
        ConfigTarget::Global => {
            let parsed: StandardConfig = value.clone().try_into().map_err(|err| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("error", err.to_string())
            })?;
            toml::Value::try_from(parsed).map_err(|err| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("error", err.to_string())
            })
        }
    }
}

fn default_doc(target: ConfigTarget, format: ConfigFormat) -> Result<String> {
    match target {
        ConfigTarget::Project => format.render(&ProjectConfig::minimal()),
        ConfigTarget::Global => format.render(&StandardConfig::minimal()),
    }
}

fn get_value<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    let mut current = root;
    for segment in key.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn set_value(root: &mut Value, key: &str, value: Value) -> Result<()> {
    let mut current = root;
    let mut segments = key.split('.').peekable();

    while let Some(segment) = segments.next() {
        let is_last = segments.peek().is_none();

        if is_last {
            let table = current.as_table_mut().ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("key", key)
                    .with_context("reason", "target parent is not a table")
            })?;
            table.insert(segment.to_string(), value);
            return Ok(());
        }

        let table = current.as_table_mut().ok_or_else(|| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("key", key)
                .with_context("reason", "intermediate path is not a table")
        })?;

        current = table
            .entry(segment.to_string())
            .or_insert_with(|| Value::Table(Default::default()));
    }

    Err(ErrorCode::ConfigInvalid.error().with_context("key", key))
}

fn unset_value(root: &mut Value, key: &str) -> Result<bool> {
    let mut current = root;
    let mut segments = key.split('.').peekable();

    while let Some(segment) = segments.next() {
        let is_last = segments.peek().is_none();

        if is_last {
            let table = current.as_table_mut().ok_or_else(|| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("key", key)
                    .with_context("reason", "target parent is not a table")
            })?;
            return Ok(table.remove(segment).is_some());
        }

        current = match current.get_mut(segment) {
            Some(next) => next,
            None => return Ok(false),
        };
    }

    Ok(false)
}

fn parse_value_for_key(key: &str, raw: &str) -> Result<Value> {
    match key {
        "commit.subject_max_length" => raw.parse::<i64>().map(Value::Integer).map_err(|_| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("key", key)
                .with_context("value", raw)
                .with_context("reason", "expected integer")
        }),
        "commit.ticket.required" | "release.enabled" | "ai.enabled" | "global.inherit" => {
            raw.parse::<bool>().map(Value::Boolean).map_err(|_| {
                ErrorCode::ConfigInvalid
                    .error()
                    .with_context("key", key)
                    .with_context("value", raw)
                    .with_context("reason", "expected boolean")
            })
        }
        "branch.remote"
        | "commit.ticket.pattern"
        | "commit.ticket.header_format"
        | "release.source_branch"
        | "release.target_branch"
        | "ai.provider"
        | "changelog.output"
        | "versioning.tag_prefix" => Ok(Value::String(raw.to_string())),
        _ => Err(ErrorCode::ConfigInvalid
            .error()
            .with_context("key", key)
            .with_context("reason", "unsupported or unknown key")),
    }
}
