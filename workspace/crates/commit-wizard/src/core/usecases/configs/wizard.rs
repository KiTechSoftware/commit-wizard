//! The guided `cw init`: a short set of questions about how the project works,
//! then a few more for people who want to go further.

use std::path::PathBuf;

use scriba::{MultiSelectOption, MultiSelectRequest, SelectOption, SelectRequest};

use crate::{
    core::{Context, CoreResult},
    engine::{
        PromptTrait,
        capabilities::config::wizard::{MANDATORY_TYPES, WizardAnswers, profile_types, split_list},
        config::{BaseConfig, ConfigFormat},
        constants::{CONFIG_STEM, app_config_dir, paths::resolve_new_project_config_path},
        error::ErrorCode,
        models::policy::enforcement::{CommitEnforcementScope, ScopeMode},
    },
};

/// Which config file is being created.
#[derive(Debug, Clone, Copy)]
pub enum InitScope {
    /// The project's `.cwizard.*` (or `cwizard.*` when not hidden).
    Project { hidden: bool },
    /// The per-user `config.*`.
    Global,
}

/// What the wizard decided.
#[derive(Debug, Clone)]
pub struct InitPlan {
    pub profile: String,
    pub file_format: ConfigFormat,
    pub answers: WizardAnswers,
    /// The file already existed and the person agreed to replace it.
    pub overwrite: bool,
}

/// The built-in profile a name refers to.
pub fn profile_config(profile: &str) -> BaseConfig {
    match profile {
        "minimal" => BaseConfig::minimal(),
        "full" => BaseConfig::full(),
        _ => BaseConfig::standard(),
    }
}

fn target_path(ctx: &Context, scope: InitScope, format: ConfigFormat) -> CoreResult<PathBuf> {
    match scope {
        InitScope::Global => {
            Ok(app_config_dir()?.join(format!("{CONFIG_STEM}.{}", format.extension())))
        }
        InitScope::Project { hidden } => {
            let explicit = ctx.explicit_config_path().map(|p| p.as_path());
            let mut path = resolve_new_project_config_path(
                ctx.cwd(),
                Some(ctx.repo_root().as_path()),
                ctx.in_git_repo(),
                explicit,
                hidden,
            );
            if explicit.is_none() {
                path.set_extension(format.extension());
            }
            Ok(path)
        }
    }
}

fn invalid(field: &'static str) -> impl Fn(crate::engine::Error) -> crate::engine::Error {
    move |err| err.with_context("field", field)
}

/// Walk through the questions. `profile` and `file_format` are `Some` when a
/// flag already decided them. Returns `None` if the person backs out.
pub fn guide(
    ctx: &Context,
    scope: InitScope,
    profile: Option<String>,
    file_format: Option<ConfigFormat>,
) -> CoreResult<Option<InitPlan>> {
    let ui = ctx.ui();

    // --- the basics ---------------------------------------------------------
    let profile = match profile {
        Some(profile) => profile,
        None => ui
            .select(&SelectRequest::new(
                "Which profile fits your project?".to_string(),
                vec![
                    SelectOption::new("standard", "Standard").description(
                        "Common commit types, optional scopes, protected main branch (recommended)"
                            .to_string(),
                    ),
                    SelectOption::new("minimal", "Minimal").description(
                        "Almost nothing written down; built-in defaults apply".to_string(),
                    ),
                    SelectOption::new("full", "Full")
                        .description("Every setting spelled out, to edit by hand".to_string()),
                ],
            ))
            .map_err(invalid("profile"))?,
    };

    let file_format = match file_format {
        Some(format) => format,
        None => {
            let choice = ui
                .select(&SelectRequest::new(
                    "Which file format do you prefer?".to_string(),
                    vec![
                        SelectOption::new("toml", "TOML").description(".cwizard.toml".to_string()),
                        SelectOption::new("yaml", "YAML").description(".cwizard.yaml".to_string()),
                        SelectOption::new("json", "JSON").description(".cwizard.json".to_string()),
                    ],
                ))
                .map_err(invalid("file_format"))?;
            ConfigFormat::from_extension(&choice).unwrap_or_default()
        }
    };

    let path = target_path(ctx, scope, file_format)?;
    let mut overwrite = false;
    if path.exists() && !ctx.force() {
        if !ui.confirm(
            &format!("{} already exists. Replace it?", path.display()),
            false,
        )? {
            ui.logger().warn("Nothing was written");
            return Ok(None);
        }
        overwrite = true;
    }

    // --- how the project works ----------------------------------------------
    let preset = profile_config(&profile);
    let mut answers = WizardAnswers::default();

    let available: Vec<String> = profile_types(&preset)
        .into_iter()
        .filter(|name| !MANDATORY_TYPES.contains(&name.as_str()))
        .collect();
    if !available.is_empty() {
        let chosen = ui
            .multiselect(&MultiSelectRequest::new(
                format!(
                    "Which commit types will you use? ({} are always included)",
                    MANDATORY_TYPES.join(" and ")
                ),
                available
                    .iter()
                    .map(|name| MultiSelectOption::new(name.clone(), name.clone()).selected(true))
                    .collect(),
            ))
            .map_err(invalid("commit_types"))?;
        answers.keep_types = Some(chosen);
    }

    let scope_choice = ui
        .select(&SelectRequest::new(
            "How should commit scopes work?".to_string(),
            vec![
                SelectOption::new("optional", "Optional")
                    .description("feat(api): ... or feat: ...".to_string()),
                SelectOption::new("required", "Required")
                    .description("Every commit must name a scope".to_string()),
                SelectOption::new("disabled", "Not used")
                    .description("Scopes are not allowed".to_string()),
            ],
        ))
        .map_err(invalid("scopes"))?;
    answers.scope_mode = Some(match scope_choice.as_str() {
        "required" => ScopeMode::Required,
        "disabled" => ScopeMode::Disabled,
        _ => ScopeMode::Optional,
    });

    let default_branches = {
        let found = preset.branch_protected_patterns();
        if found.is_empty() {
            vec!["main".to_string(), "master".to_string()]
        } else {
            found
        }
    };
    let branches = ui.text(
        "Which branches are protected?",
        Some(&default_branches.join(", ")),
        Some("Comma-separated; patterns such as release/* work"),
    )?;
    let branches = split_list(&branches);
    if !branches.is_empty() {
        answers.protected_branches = Some(branches);
    }

    let enforce = ui
        .select(&SelectRequest::new(
            "Where should commit messages be checked?".to_string(),
            vec![
                SelectOption::new("protected_branches", "Protected branches only")
                    .description("Recommended: feature branches stay relaxed".to_string()),
                SelectOption::new("all_branches", "Every branch")
                    .description("Strict everywhere".to_string()),
                SelectOption::new("none", "Nowhere")
                    .description("Never block a push over message style".to_string()),
            ],
        ))
        .map_err(invalid("enforce_on"))?;
    answers.enforce_on = Some(match enforce.as_str() {
        "all_branches" => CommitEnforcementScope::AllBranches,
        "none" => CommitEnforcementScope::None,
        _ => CommitEnforcementScope::ProtectedBranches,
    });

    answers.allow_push_to_protected = Some(ui.confirm(
        "Allow pushing straight to a protected branch?",
        preset.push_allow_protected(),
    )?);

    // --- more, for those who want it -----------------------------------------
    if ui.confirm("Customize more options?", false)? {
        let length = ui.text(
            "Maximum length of a commit subject",
            Some(&preset.commit_subject_max_length().to_string()),
            None,
        )?;
        answers.subject_max_length = Some(length.trim().parse::<u32>().map_err(|_| {
            ErrorCode::InvalidInput
                .error()
                .with_context("field", "subject_max_length")
                .with_context("value", length.trim())
                .with_context("reason", "expected a whole number")
        })?);
        answers.use_emojis = Some(ui.confirm(
            "Put an emoji before each commit type?",
            preset.commit_use_emojis(),
        )?);
        answers.require_breaking_footer = Some(ui.confirm(
            "Require a description for breaking changes?",
            preset.commit_breaking_require_footer(),
        )?);
        answers.require_ticket = Some(ui.confirm(
            "Require a ticket reference in commits?",
            preset.commit_ticket_required(),
        )?);
        answers.tag_prefix = Some(ui.text(
            "Tag prefix",
            Some(&preset.versioning_tag_prefix()),
            Some("v gives v1.2.3"),
        )?);
        answers.changelog_output =
            Some(ui.text("Changelog file", Some(&preset.changelog_output()), None)?);
    }

    // --- last word -------------------------------------------------------------
    if !ui.confirm(&format!("Create {}?", path.display()), true)? {
        ui.logger().warn("Nothing was written");
        return Ok(None);
    }

    Ok(Some(InitPlan {
        profile,
        file_format,
        answers,
        overwrite,
    }))
}
