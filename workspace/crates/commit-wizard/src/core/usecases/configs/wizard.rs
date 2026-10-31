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
    let types: Vec<(String, bool)> = profile_types(&preset)
        .into_iter()
        .filter(|name| !MANDATORY_TYPES.contains(&name.as_str()))
        .map(|name| (name, true))
        .collect();
    let answers = ask_settings(ctx, &preset, types, false)?;

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

/// Put the option matching `current` first so it is what Enter picks, and
/// mark it when editing.
fn current_first(
    options: Vec<(&'static str, &'static str, &'static str)>,
    current: &str,
    mark_current: bool,
) -> Vec<SelectOption> {
    let mut options = options;
    if let Some(position) = options.iter().position(|(id, _, _)| *id == current) {
        let chosen = options.remove(position);
        options.insert(0, chosen);
    }
    options
        .into_iter()
        .map(|(id, label, description)| {
            let label = if mark_current && id == current {
                format!("{label} (current)")
            } else {
                label.to_string()
            };
            SelectOption::new(id, label).description(description.to_string())
        })
        .collect()
}

/// The questions about how the project works, with `current` supplying the
/// default of each. `types` are the optional commit types and whether each is
/// selected. Used by both `init` (the profile is `current`) and `config edit`
/// (the existing file is).
fn ask_settings(
    ctx: &Context,
    current: &BaseConfig,
    types: Vec<(String, bool)>,
    editing: bool,
) -> CoreResult<WizardAnswers> {
    let ui = ctx.ui();
    let mut answers = WizardAnswers::default();

    if !types.is_empty() {
        let chosen = ui
            .multiselect(&MultiSelectRequest::new(
                format!(
                    "Which commit types will you use? ({} are always included)",
                    MANDATORY_TYPES.join(" and ")
                ),
                types
                    .iter()
                    .map(|(name, selected)| {
                        MultiSelectOption::new(name.clone(), name.clone()).selected(*selected)
                    })
                    .collect(),
            ))
            .map_err(invalid("commit_types"))?;
        answers.keep_types = Some(chosen);
    }

    let current_scope = match current.commit_scopes_mode() {
        ScopeMode::Required => "required",
        ScopeMode::Disabled => "disabled",
        ScopeMode::Optional => "optional",
    };
    let scope_choice = ui
        .select(&SelectRequest::new(
            "How should commit scopes work?".to_string(),
            current_first(
                vec![
                    ("optional", "Optional", "feat(api): ... or feat: ..."),
                    ("required", "Required", "Every commit must name a scope"),
                    ("disabled", "Not used", "Scopes are not allowed"),
                ],
                current_scope,
                editing,
            ),
        ))
        .map_err(invalid("scopes"))?;
    answers.scope_mode = Some(match scope_choice.as_str() {
        "required" => ScopeMode::Required,
        "disabled" => ScopeMode::Disabled,
        _ => ScopeMode::Optional,
    });

    let default_branches = {
        let found = current.branch_protected_patterns();
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

    let current_enforce = match current.check_commits_enforce_on() {
        CommitEnforcementScope::AllBranches => "all_branches",
        CommitEnforcementScope::None => "none",
        CommitEnforcementScope::ProtectedBranches => "protected_branches",
    };
    let enforce = ui
        .select(&SelectRequest::new(
            "Where should commit messages be checked?".to_string(),
            current_first(
                vec![
                    (
                        "protected_branches",
                        "Protected branches only",
                        "Recommended: feature branches stay relaxed",
                    ),
                    ("all_branches", "Every branch", "Strict everywhere"),
                    ("none", "Nowhere", "Never block a push over message style"),
                ],
                current_enforce,
                editing,
            ),
        ))
        .map_err(invalid("enforce_on"))?;
    answers.enforce_on = Some(match enforce.as_str() {
        "all_branches" => CommitEnforcementScope::AllBranches,
        "none" => CommitEnforcementScope::None,
        _ => CommitEnforcementScope::ProtectedBranches,
    });

    answers.allow_push_to_protected = Some(ui.confirm(
        "Allow pushing straight to a protected branch?",
        current.push_allow_protected(),
    )?);

    if ui.confirm("Customize more options?", false)? {
        let length = ui.text(
            "Maximum length of a commit subject",
            Some(&current.commit_subject_max_length().to_string()),
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
            current.commit_use_emojis(),
        )?);
        answers.require_breaking_footer = Some(ui.confirm(
            "Require a description for breaking changes?",
            current.commit_breaking_require_footer(),
        )?);
        answers.require_ticket = Some(ui.confirm(
            "Require a ticket reference in commits?",
            current.commit_ticket_required(),
        )?);
        answers.tag_prefix = Some(ui.text(
            "Tag prefix",
            Some(&current.versioning_tag_prefix()),
            Some("v gives v1.2.3"),
        )?);
        answers.changelog_output =
            Some(ui.text("Changelog file", Some(&current.changelog_output()), None)?);
    }

    Ok(answers)
}

/// `cw config edit`: the same questions as `cw init`, asked about an existing
/// config, with its current values as the defaults. Shows what would change
/// and saves only if the person agrees.
pub fn edit_config(ctx: &Context, global: bool) -> CoreResult<()> {
    use std::time::Instant;

    use scriba::Table;

    use crate::engine::{
        capabilities::config::effective::{changes, to_document},
        config::{ProjectConfig, StandardConfig},
        constants::resolve_global_config_path,
    };

    let start = Instant::now();
    let ui = ctx.ui();

    if !(ctx.is_interactive() && !ctx.auto_yes()) {
        return Err(ErrorCode::UserInteractionRequired
            .error()
            .with_context("command", "config edit")
            .with_context(
                "hint",
                "config edit is a guided editor for a terminal; use `cw config set <key> <value>` in scripts",
            ));
    }

    let path = if global {
        resolve_global_config_path()?
    } else {
        ctx.project_config_path().ok_or_else(|| {
            ErrorCode::ConfigUnreadable.error().with_context(
                "reason",
                "no project config found; run `cw init` to create one",
            )
        })?
    };
    if !path.exists() {
        return Err(ErrorCode::ConfigUnreadable
            .error()
            .with_context("path", path.display().to_string())
            .with_context(
                "reason",
                "config file does not exist; run `cw init` to create one",
            ));
    }

    let format = ConfigFormat::from_path(&path);
    let raw = std::fs::read_to_string(&path)?;
    enum Loaded {
        Project(ProjectConfig),
        Global(StandardConfig),
    }
    let loaded = if global {
        Loaded::Global(StandardConfig::from_str_as(format, &raw)?)
    } else {
        Loaded::Project(ProjectConfig::from_str_as(format, &raw)?)
    };
    let current = match &loaded {
        Loaded::Project(project) => project.inner.config.clone(),
        Loaded::Global(standard) => standard.inner.clone(),
    };

    ui.logger().info(&format!("Editing {}", path.display()));

    // Offer the standard profile's types too, so one can be added; the file's
    // own types start selected. A file with no types table keeps using defaults.
    let own = profile_types(&current);
    let types: Vec<(String, bool)> = if own.is_empty() {
        Vec::new()
    } else {
        let mut names = own.clone();
        for name in profile_types(&BaseConfig::standard()) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
            .into_iter()
            .filter(|name| !MANDATORY_TYPES.contains(&name.as_str()))
            .map(|name| {
                let selected = own.contains(&name);
                (name, selected)
            })
            .collect()
    };

    let answers = ask_settings(ctx, &current, types, true)?;
    let updated = answers.apply(&current)?;

    let before = to_document(&current).ok_or_else(|| ErrorCode::SerializationFailure.error())?;
    let after = to_document(&updated).ok_or_else(|| ErrorCode::SerializationFailure.error())?;
    let differences = changes(&before, &after);

    if differences.is_empty() {
        ui.logger().info("Nothing changed");
        return Ok(());
    }
    for change in &differences {
        ui.logger().info(&format!("  {}", change.describe()));
    }

    let question = format!(
        "Save {} change{} to {}?",
        differences.len(),
        if differences.len() == 1 { "" } else { "s" },
        path.display()
    );
    if !ui.confirm(&question, true)? {
        ui.logger().warn("Nothing was saved");
        return Ok(());
    }

    if !ctx.dry_run() {
        let rendered = match loaded {
            Loaded::Project(mut project) => {
                project.inner.config = updated;
                format.render(&project)?
            }
            Loaded::Global(mut standard) => {
                standard.inner = updated;
                format.render(&standard)?
            }
        };
        std::fs::write(&path, rendered)?;
    }

    let table = Table::new(
        vec![
            "setting".to_string(),
            "before".to_string(),
            "after".to_string(),
        ],
        differences
            .iter()
            .map(|c| {
                vec![
                    c.key.clone(),
                    c.before.clone().unwrap_or_default(),
                    c.after.clone().unwrap_or_default(),
                ]
            })
            .collect(),
    );
    let meta = ui
        .new_output_meta()
        .with_command("config.edit".to_string())
        .with_scope(if global { "global" } else { "project" }.to_string())
        .with_dry_run(ctx.dry_run())
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string());
    let content = ui
        .new_output_content()
        .title("Config updated")
        .data("path", path.display().to_string())
        .data("changes", differences.len().to_string())
        .data("saved", (!ctx.dry_run()).to_string())
        .table(Some("Changes".to_string()), table);

    ui.print_with_meta(&content, Some(&meta), true)
}
