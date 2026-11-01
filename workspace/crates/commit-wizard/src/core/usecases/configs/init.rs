//! `cw init`: creates the project config, the user config, a rules file or a registry skeleton.

use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode,
        capabilities::config::{
            edit,
            init::{self, ConfigContext, ConfigOption, ConfigSpec},
            show::ConfigTarget,
            wizard::WizardAnswers,
        },
        config::ConfigFormat,
        constants::{app_config_dir, paths::resolve_new_project_config_path},
    },
};

pub fn init_project_config(
    ctx: &Context,
    hidden: bool,
    profile: String,
    file_format: ConfigFormat,
    answers: Option<WizardAnswers>,
    overwrite: bool,
) -> CoreResult<()> {
    let ui = ctx.ui();

    let scope = if hidden { "hidden" } else { "visible" };
    let spec = match profile.as_str() {
        "minimal" => ConfigSpec::Minimal,
        "full" => ConfigSpec::Full,
        _ => ConfigSpec::Standard,
    };

    let explicit_path = ctx.explicit_config_path().map(|p| p.as_path());
    let mut config_path = resolve_new_project_config_path(
        ctx.cwd(),
        Some(ctx.repo_root().as_path()),
        ctx.in_git_repo(),
        explicit_path,
        hidden,
    );
    // An explicit path keeps its own extension (and so its format); otherwise the
    // chosen file format decides it.
    if explicit_path.is_none() {
        config_path.set_extension(file_format.extension());
    }
    let config_filename = config_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    let config_context = ConfigContext {
        option: ConfigOption::Project,
        output_path: config_path.clone(),
        spec,
        force: ctx.force() || overwrite,
        auto_yes: ctx.auto_yes(),
        dry_run: ctx.dry_run(),
        hidden,
        with_rules: false,
        registry_options: Default::default(),
        file_format,
        answers,
    };

    let start = Instant::now();
    let init_output = init::init_config(&config_context)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("init".to_string())
        .with_scope(scope.to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Initialized Project Configuration")
        .subtitle(format!(
            "Initialized {} config at {}",
            config_filename,
            config_path.display(),
        ))
        .data("path", config_path.display().to_string())
        .data("profile", profile.clone());

    if ctx.dry_run()
        && let Some(rendered) = init_output.dry_run_content.clone()
    {
        content = content.section(
            "Rendered Config",
            rendered,
            ConfigFormat::from_path(&config_path).name().to_string(),
        );
    }

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn init_config(
    ctx: &Context,
    global: bool,
    profile: String,
    file_format: ConfigFormat,
    answers: Option<WizardAnswers>,
    overwrite: bool,
) -> CoreResult<()> {
    // Without --global this is the project config, the same file bare `cw init` writes.
    if !global {
        return init_project_config(ctx, true, profile, file_format, answers, overwrite);
    }

    let ui = ctx.ui();

    let spec = match profile.as_str() {
        "minimal" => init::ConfigSpec::Minimal,
        "full" => init::ConfigSpec::Full,
        _ => init::ConfigSpec::Standard,
    };

    let target = app_config_dir()?;

    let config_context = init::ConfigContext {
        option: init::ConfigOption::Global,
        output_path: target.clone(),
        spec,
        force: ctx.force() || overwrite,
        auto_yes: ctx.auto_yes(),
        dry_run: ctx.dry_run(),
        hidden: false,
        with_rules: false,
        registry_options: Default::default(),
        file_format,
        answers,
    };

    let start = Instant::now();
    let init_output = init::init_config(&config_context)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("init.config".to_string())
        .with_scope("global".to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Initialized Configuration")
        .subtitle(format!("Path: {}", init_output.path.display()))
        .data("path", init_output.path.display().to_string())
        .data("profile", profile.clone());

    if ctx.dry_run()
        && let Some(rendered) = init_output.dry_run_content.clone()
    {
        content = content.section("Rendered Config", rendered, file_format.name().to_string());
    }

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn init_rules(
    ctx: &Context,
    global: bool,
    _profile: String,
    file_format: ConfigFormat,
) -> CoreResult<()> {
    // A project has no rules file of its own: its rules are the [rules] table in
    // the project config, which is where cw reads them from.
    if !global {
        return Err(crate::engine::error::ErrorCode::InvalidInput
            .error()
            .with_context("command", "init rules")
            .with_context(
                "reason",
                "project rules live in the [rules] table of your project config",
            )
            .with_context(
                "hint",
                "add a [rules] table to .cwizard.toml, or run `cw init rules --global` for a rules file shared by all your projects",
            ));
    }

    let ui = ctx.ui();
    let config_context = init::ConfigContext {
        option: init::ConfigOption::Rules,
        output_path: app_config_dir()?,
        spec: init::ConfigSpec::Standard,
        force: ctx.force(),
        auto_yes: ctx.auto_yes(),
        dry_run: ctx.dry_run(),
        hidden: false,
        with_rules: true,
        registry_options: Default::default(),
        file_format,
        answers: None,
    };

    let start = Instant::now();
    let init_output = init::init_config(&config_context)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("init.rules".to_string())
        .with_scope("global".to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Initialized Rules")
        .subtitle(format!("Path: {}", init_output.path.display()))
        .data("path", init_output.path.display().to_string());

    if ctx.dry_run()
        && let Some(rendered) = init_output.dry_run_content.clone()
    {
        content = content.section("Rendered Rules", rendered, file_format.name().to_string());
    }

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn init_registry(
    ctx: &Context,
    path: std::path::PathBuf,
    with_rules: bool,
    sections: Vec<String>,
    profile: String,
) -> CoreResult<()> {
    let ui = ctx.ui();

    let spec = match profile.as_str() {
        "minimal" => init::ConfigSpec::Minimal,
        "full" => init::ConfigSpec::Full,
        _ => init::ConfigSpec::Standard,
    };

    let registry_options = init::RegistryOptions {
        git_init: false, // User can git init if needed
        sections,
        ..Default::default()
    };

    let config_context = init::ConfigContext {
        option: init::ConfigOption::Registry,
        output_path: path.clone(),
        spec,
        force: ctx.force(),
        auto_yes: ctx.auto_yes(),
        dry_run: ctx.dry_run(),
        hidden: false,
        with_rules,
        registry_options,
        file_format: ConfigFormat::Toml,
        answers: None,
    };

    let start = Instant::now();
    let init_output = init::init_config(&config_context)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("init.registry".to_string())
        .with_scope("registry".to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Initialized Registry")
        .subtitle(format!("Path: {}", path.display()))
        .data("path", path.display().to_string())
        .data("profile", profile);

    if ctx.dry_run()
        && let Some(rendered) = init_output.dry_run_content.clone()
    {
        content = content.section("Rendered Config", rendered, "toml".to_string());
    }

    ui.print_with_meta(&content, Some(&meta), true)
}

/// Splits `--set KEY=VALUE` arguments, so a malformed one stops `cw init` before it writes anything.
pub fn parse_sets(sets: &[String]) -> CoreResult<Vec<(String, String)>> {
    sets.iter()
        .map(|pair| {
            pair.split_once('=')
                .filter(|(key, _)| !key.trim().is_empty())
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .ok_or_else(|| {
                    ErrorCode::InvalidInput
                        .error()
                        .with_context("flag", "--set")
                        .with_context("value", pair.clone())
                        .with_context("reason", "expected KEY=VALUE")
                })
        })
        .collect()
}

/// Applies `--set` overrides to the config `cw init` just wrote, one validated edit at a time.
pub fn apply_sets(ctx: &Context, sets: &[(String, String)], global: bool) -> CoreResult<()> {
    if ctx.dry_run() {
        return Ok(());
    }
    for (key, value) in sets {
        edit::config_set(&edit::ConfigSetInput {
            cwd: ctx.cwd(),
            target: if global {
                ConfigTarget::Global
            } else {
                ConfigTarget::Project
            },
            key,
            value,
            dry_run: false,
            explicit_path: if global {
                None
            } else {
                ctx.explicit_config_path().map(|p| p.as_path())
            },
        })?;
    }
    Ok(())
}
