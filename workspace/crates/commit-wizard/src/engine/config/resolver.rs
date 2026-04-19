use crate::engine::{
    LoggerTrait,
    config::{BaseConfig, ProjectConfig, RulesConfig, StandardConfig},
    constants::{resolve_global_config_path, resolve_global_rules_path},
    error::{ErrorCode, Result},
};

pub fn resolve_global_configs() -> (Option<BaseConfig>, Option<RulesConfig>) {
    // if global config path exists, load it
    let global_config = if let Ok(path) = resolve_global_config_path() {
        resolve_standard_configs(&path)
    } else {
        None
    };

    // if global rules path exists, load it
    let global_rules = if let Ok(path) = resolve_global_rules_path() {
        load_rules_config(&path)
    } else {
        None
    };
    (global_config, global_rules)
}

pub fn resolve_project_configs(
    path: &std::path::Path,
    logger: Option<&dyn LoggerTrait>,
) -> (Option<BaseConfig>, Option<RulesConfig>) {
    if let Some(project_config) = load_project_config(path, logger) {
        let (base_config, rules_config) = extract_configs_from_project_config(&project_config);
        (Some(base_config), rules_config)
    } else {
        (None, None)
    }
}

pub fn resolve_standard_configs(path: &std::path::Path) -> Option<BaseConfig> {
    if let Some(standard_config) = load_standard_config(path) {
        let base_config = extract_config_from_standard_config(&standard_config);
        Some(base_config)
    } else {
        None
    }
}

pub fn load_project_config(
    path: &std::path::Path,
    logger: Option<&dyn LoggerTrait>,
) -> Option<ProjectConfig> {
    if !path.exists() {
        if let Some(logger) = logger {
            let msg = format!("load_project_config: path does not exist: {:?}", path);
            logger.debug(&msg);
        }
        return None;
    }

    match std::fs::read_to_string(path) {
        Ok(content) => {
            if let Some(logger) = logger {
                let msg = format!(
                    "load_project_config: Read {} bytes from {:?}",
                    content.len(),
                    path
                );
                logger.debug(&msg);
            }
            match ProjectConfig::from_toml_str(&content) {
                Ok(config) => {
                    if let Some(logger) = logger {
                        logger.debug("load_project_config: Successfully parsed config");
                    }
                    Some(config)
                }
                Err(e) => {
                    if let Some(logger) = logger {
                        // Include both the main error message and any context details
                        let msg = format!("load_project_config: Parse error: {}", e);
                        logger.error(&msg);
                        // Also log context information if available
                        for (key, value) in &e.context {
                            let ctx_msg = format!("    {}: {}", key, value);
                            logger.error(&ctx_msg);
                        }
                    }
                    None
                }
            }
        }
        Err(e) => {
            if let Some(logger) = logger {
                let msg = format!("load_project_config: File read error: {}", e);
                logger.error(&msg);
            }
            None
        }
    }
}

pub fn load_standard_config(path: &std::path::Path) -> Option<StandardConfig> {
    if !path.exists() {
        return None;
    }

    match std::fs::read_to_string(path) {
        Ok(content) => StandardConfig::from_toml_str(&content).ok(),
        Err(_) => None,
    }
}

pub fn load_rules_config(path: &std::path::Path) -> Option<RulesConfig> {
    if !path.exists() {
        return None;
    }

    match std::fs::read_to_string(path) {
        Ok(content) => RulesConfig::from_toml_str(&content).ok(),
        Err(_) => None,
    }
}

pub fn extract_configs_from_project_config(
    project_config: &ProjectConfig,
) -> (BaseConfig, Option<RulesConfig>) {
    (
        project_config.inner.config.clone(),
        project_config.inner.rules.clone(),
    )
}

pub fn extract_config_from_standard_config(standard_config: &StandardConfig) -> BaseConfig {
    standard_config.inner.clone()
}

/// Merge rules into base config by resolving all `@rules.*` references.
///
/// Strategy: serialize `BaseConfig` to a `toml::Value` tree, recursively resolve
/// every string that is a `@rules.*` reference using the existing `resolve_value_refs`
/// walker, then deserialize back to `BaseConfig`.
///
/// Per SRS §4.3: resolution is evaluated after rules merging, before validation.
/// Failure to resolve MUST error.
pub fn merge_rules_into_base(base: BaseConfig, rules: &RulesConfig) -> Result<BaseConfig> {
    // Serialize to an intermediate toml::Value so we can walk the whole tree
    // generically without enumerating every field individually.
    let mut value = toml::Value::try_from(&base).map_err(|e| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "merge_rules_into_base: serialize")
            .with_context("error", e.to_string())
    })?;

    // Walk the entire tree and resolve any @rules.* strings in-place.
    // This covers every Option<String>, Vec<String>, and nested struct field.
    rules.resolve_value_refs(&mut value)?;

    // Deserialize back into BaseConfig — if a resolved value is the wrong type,
    // this will produce a descriptive error before validation runs.
    toml::Value::try_into(value).map_err(|e| {
        ErrorCode::SerializationFailure
            .error()
            .with_context("operation", "merge_rules_into_base: deserialize")
            .with_context("error", e.to_string())
    })
}
