//! Builds the files `cw init` writes for a project, user, rules or registry setup.

use std::path::{Path, PathBuf};

use crate::engine::{
    config::{ConfigFormat, ProjectConfig, RulesConfig, StandardConfig},
    constants::{CONFIG_FILE_NAME, CONFIG_STEM, RULES_FILE_NAME, RULES_STEM, app_config_dir},
    error::{ErrorCode, Result},
};

pub enum ConfigOption {
    Project,
    Global,
    /// Only the user-level rules file, leaving the config alone.
    Rules,
    Registry,
}

pub enum ConfigSpec {
    Minimal,
    Standard,
    Full,
}

pub struct RegistryOptions {
    pub name: String,
    pub git_init: bool,
    pub sections: bool,
}

impl Default for RegistryOptions {
    fn default() -> Self {
        Self {
            name: "registry".to_string(),
            git_init: false,
            sections: false,
        }
    }
}

pub struct ConfigContext {
    pub option: ConfigOption,
    pub spec: ConfigSpec,
    pub output_path: PathBuf,
    pub force: bool,
    pub auto_yes: bool,
    pub dry_run: bool,
    pub hidden: bool,
    pub with_rules: bool,
    pub registry_options: RegistryOptions,
    /// Format for files init chooses the name of (the global config and rules).
    /// Project files are rendered in the format their path's extension implies.
    pub file_format: ConfigFormat,
    /// Answers from the guided wizard, applied on top of the profile.
    pub answers: Option<super::wizard::WizardAnswers>,
}

pub struct ConfigPayload {
    pub config_type: ConfigOption,
    pub with_rules: bool,
}

/// Output returned from init_config, carrying back info the usecase needs (e.g. dry-run content).
pub struct InitOutput {
    /// The primary file path that was written (or would be written on dry-run).
    pub path: PathBuf,
    /// Content that would be written, in the target file's format; only populated when dry_run=true.
    pub dry_run_content: Option<String>,
}

pub fn init_config(context: &ConfigContext) -> Result<InitOutput> {
    let rules = RulesConfig::default();
    match context.option {
        ConfigOption::Project => {
            let mut input = match context.spec {
                ConfigSpec::Minimal => ProjectConfig::minimal(),
                ConfigSpec::Standard => ProjectConfig::standard(),
                ConfigSpec::Full => ProjectConfig::full(),
            };
            if let Some(answers) = &context.answers {
                input.inner.config = answers.apply(&input.inner.config)?;
            }
            if context.with_rules {
                input.inner.rules = Some(rules);
            }
            // output_path is already the target file path (.cwizard.toml or cwizard.toml)
            let config_path = &context.output_path;

            if context.dry_run {
                let content = ConfigFormat::from_path(config_path).render(&input)?;
                return Ok(InitOutput {
                    path: config_path.clone(),
                    dry_run_content: Some(content),
                });
            }

            save_project(config_path, &input, context.force, context.dry_run)?;
            Ok(InitOutput {
                path: config_path.clone(),
                dry_run_content: None,
            })
        }
        ConfigOption::Global => {
            let input = match context.spec {
                ConfigSpec::Minimal => StandardConfig::minimal(),
                ConfigSpec::Standard => StandardConfig::standard(),
                ConfigSpec::Full => StandardConfig::full(),
            };

            let input = match &context.answers {
                Some(answers) => StandardConfig {
                    inner: answers.apply(&input.inner)?,
                    ..input
                },
                None => input,
            };

            let root = app_config_dir()?;
            let extension = context.file_format.extension();
            let config_path = root.join(format!("{CONFIG_STEM}.{extension}"));

            let dry_run_content = if context.dry_run {
                Some(context.file_format.render(&input)?)
            } else {
                save_standard(&config_path, &input, context.force, context.dry_run)?;
                None
            };

            if context.with_rules {
                let rules_path = root.join(format!("{RULES_STEM}.{extension}"));
                if !context.dry_run {
                    save_rules(&rules_path, &rules, context.force, context.dry_run)?;
                }
            }

            Ok(InitOutput {
                path: config_path,
                dry_run_content,
            })
        }
        ConfigOption::Rules => {
            let path =
                app_config_dir()?.join(format!("{RULES_STEM}.{}", context.file_format.extension()));
            if context.dry_run {
                return Ok(InitOutput {
                    dry_run_content: Some(context.file_format.render(&rules)?),
                    path,
                });
            }
            save_rules(&path, &rules, context.force, context.dry_run)?;
            Ok(InitOutput {
                path,
                dry_run_content: None,
            })
        }
        ConfigOption::Registry => init_registry(context),
    }
}

pub fn save_project(path: &Path, input: &ProjectConfig, force: bool, dry_run: bool) -> Result<()> {
    save_doc(path, input, force, dry_run)
}

pub fn save_standard(
    path: &Path,
    input: &StandardConfig,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    save_doc(path, input, force, dry_run)
}

pub fn save_rules(path: &Path, input: &RulesConfig, force: bool, dry_run: bool) -> Result<()> {
    save_doc(path, input, force, dry_run)
}

/// Write a document in the format its file extension implies.
fn save_doc<T: serde::Serialize>(path: &Path, input: &T, force: bool, dry_run: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(ErrorCode::ConfigInvalid
            .error()
            .with_context("path", path.display().to_string())
            .with_context("reason", "file already exists; use --force to overwrite"));
    }

    let content = ConfigFormat::from_path(path)
        .render(input)
        .map_err(|err| err.with_context("path", path.display().to_string()))?;

    if dry_run {
        return Ok(());
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    crate::engine::fs::write_text(path, &content)?;
    Ok(())
}

fn save_text(path: &Path, content: &str, force: bool, dry_run: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(ErrorCode::ConfigInvalid
            .error()
            .with_context("path", path.display().to_string())
            .with_context("reason", "file already exists; use --force to overwrite"));
    }

    if dry_run {
        return Ok(());
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    crate::engine::fs::write_text(path, content)?;
    Ok(())
}

fn init_registry(context: &ConfigContext) -> Result<InitOutput> {
    let root = &context.output_path;

    if !context.dry_run {
        std::fs::create_dir_all(root)?;
    }

    let config = StandardConfig::minimal();

    let config_path = root.join(CONFIG_FILE_NAME);
    save_standard(&config_path, &config, context.force, context.dry_run)?;

    let rules_path = root.join(RULES_FILE_NAME);
    save_rules(
        &rules_path,
        &RulesConfig::default(),
        context.force,
        context.dry_run,
    )?;

    let readme_path = root.join("README.md");
    save_text(
        &readme_path,
        &registry_readme(context),
        context.force,
        context.dry_run,
    )?;

    if context.registry_options.sections {
        for section in ["standard", "team1"] {
            let section_dir = root.join(section);

            if !context.dry_run {
                std::fs::create_dir_all(&section_dir)?;
            }

            save_standard(
                &section_dir.join(CONFIG_FILE_NAME),
                &config,
                context.force,
                context.dry_run,
            )?;

            save_rules(
                &section_dir.join(RULES_FILE_NAME),
                &RulesConfig::default(),
                context.force,
                context.dry_run,
            )?;
        }
    }

    Ok(InitOutput {
        path: root.join(CONFIG_FILE_NAME),
        dry_run_content: None,
    })
}
fn registry_readme(context: &ConfigContext) -> String {
    let mut s = String::new();

    s.push_str("# commit-wizard registry\n\n");
    s.push_str("This repository is a commit-wizard registry.\n\n");
    s.push_str("A registry is a Git repository that provides shared `config.toml` and `rules.toml` files for commit-wizard consumers.\n\n");

    s.push_str("## Supported layouts\n\n");
    s.push_str("### Single-source registry\n\n");
    s.push_str("```text\n");
    s.push_str("config.toml\n");
    s.push_str("rules.toml\n");
    s.push_str("```\n\n");

    s.push_str("### Sectioned registry\n\n");
    s.push_str("```text\n");
    s.push_str("standard/config.toml\n");
    s.push_str("standard/rules.toml\n\n");
    s.push_str("team1/config.toml\n");
    s.push_str("team1/rules.toml\n");
    s.push_str("```\n\n");

    s.push_str("## Consumer configuration\n\n");
    s.push_str("```toml\n");
    s.push_str("version = 1\n\n");
    s.push_str("[registry]\n");
    s.push_str("use = \"my-org\"\n\n");
    s.push_str("[registries.my-org]\n");
    s.push_str("url = \"https://github.com/org/registry.git\"\n");
    s.push_str("ref = \"main\"\n");
    s.push_str("section = \"standard\"\n");
    s.push_str("```\n\n");

    s.push_str("## Resolution rules\n\n");
    s.push_str("- Registry selection precedence: CLI, then ENV, then config.\n");
    s.push_str("- `ref` may be a branch, tag, or commit SHA.\n");
    s.push_str("- If `section` is set, commit-wizard loads `<section>/config.toml` and `<section>/rules.toml`.\n");
    s.push_str("- If `section` is not set, commit-wizard loads `config.toml` and `rules.toml` from the repository root.\n");
    s.push_str("- Missing files are fatal.\n");
    s.push_str("- Invalid TOML or schema violations are fatal.\n");
    s.push_str("- No silent fallback is allowed.\n\n");

    s.push_str("## Update workflow\n\n");
    s.push_str("1. Edit the shared configuration files in this repository.\n");
    s.push_str("2. Commit and push the changes.\n");
    s.push_str("3. Update consuming projects to the desired `ref` when needed.\n");
    s.push_str("4. Consumers will re-fetch and re-validate the registry on each run.\n\n");

    s.push_str("## Notes\n\n");
    s.push_str("- The local registry cache is disposable and may be deleted safely.\n");
    s.push_str("- State is advisory only and not authoritative.\n");
    s.push_str("- Registries are trusted input; commit-wizard reads files only and must not execute code from the registry.\n");

    if context.registry_options.sections {
        s.push_str("\nThis scaffold includes example sections: `standard/` and `team1/`.\n");
    }

    s
}
