use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::config::init::{self, ConfigContext, ConfigOption, ConfigSpec},
        constants::{
            PROJECT_CONFIG_FILE_NAME, PROJECT_CONFIG_FILE_NAME_HIDDEN, app_config_dir,
            paths::resolve_new_project_config_path,
        },
    },
};

pub fn init_project_config(ctx: &Context, hidden: bool, profile: String) -> CoreResult<()> {
    let ui = ctx.ui();

    let scope = if hidden { "hidden" } else { "visible" };
    let spec = match profile.as_str() {
        "minimal" => ConfigSpec::Minimal,
        "full" => ConfigSpec::Full,
        _ => ConfigSpec::Standard,
    };

    let config_filename = if hidden {
        PROJECT_CONFIG_FILE_NAME_HIDDEN
    } else {
        PROJECT_CONFIG_FILE_NAME
    };

    let config_path = resolve_new_project_config_path(
        ctx.cwd(),
        Some(ctx.repo_root().as_path()),
        ctx.in_git_repo(),
        ctx.explicit_config_path().map(|p| p.as_path()),
        hidden,
    );

    let config_context = ConfigContext {
        option: ConfigOption::Project,
        output_path: config_path.clone(),
        spec,
        force: ctx.force(),
        auto_yes: ctx.auto_yes(),
        dry_run: ctx.dry_run(),
        hidden,
        with_rules: false,
        registry_options: Default::default(),
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
        content = content.section("Rendered Config", rendered, "toml".to_string());
    }

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn init_config(ctx: &Context, global: bool, profile: String) -> CoreResult<()> {
    let ui = ctx.ui();

    let spec = match profile.as_str() {
        "minimal" => init::ConfigSpec::Minimal,
        "full" => init::ConfigSpec::Full,
        _ => init::ConfigSpec::Standard,
    };

    let target = if global {
        app_config_dir()?
    } else {
        ctx.cwd().clone()
    };

    let config_context = init::ConfigContext {
        option: if global {
            init::ConfigOption::Global
        } else {
            init::ConfigOption::Project
        },
        output_path: target.clone(),
        spec,
        force: ctx.force(),
        auto_yes: ctx.auto_yes(),
        dry_run: ctx.dry_run(),
        hidden: false,
        with_rules: false,
        registry_options: Default::default(),
    };

    let start = Instant::now();
    let init_output = init::init_config(&config_context)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let scope = if global { "global" } else { "project" };

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("init.config".to_string())
        .with_scope(scope.to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Initialized Configuration")
        .subtitle(format!("Path: {}", target.display()))
        .data("path", target.display().to_string())
        .data("profile", profile.clone());

    if ctx.dry_run()
        && let Some(rendered) = init_output.dry_run_content.clone()
    {
        content = content.section("Rendered Config", rendered, "toml".to_string());
    }

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn init_rules(ctx: &Context, global: bool, profile: String) -> CoreResult<()> {
    let ui = ctx.ui();

    let spec = match profile.as_str() {
        "minimal" => init::ConfigSpec::Minimal,
        "full" => init::ConfigSpec::Full,
        _ => init::ConfigSpec::Standard,
    };

    let target = if global {
        app_config_dir()?
    } else {
        ctx.cwd().clone()
    };

    let config_context = init::ConfigContext {
        option: if global {
            init::ConfigOption::Global
        } else {
            init::ConfigOption::Project
        },
        output_path: target.clone(),
        spec,
        force: ctx.force(),
        auto_yes: ctx.auto_yes(),
        dry_run: ctx.dry_run(),
        hidden: false,
        with_rules: true, // Include rules for this operation
        registry_options: Default::default(),
    };

    let start = Instant::now();
    let init_output = init::init_config(&config_context)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let scope = if global { "global" } else { "project" };

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("init.rules".to_string())
        .with_scope(scope.to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Initialized Rules")
        .subtitle(format!("Path: {}", target.display()))
        .data("path", target.display().to_string())
        .data("profile", profile.clone());

    if ctx.dry_run()
        && let Some(rendered) = init_output.dry_run_content.clone()
    {
        content = content.section("Rendered Config", rendered, "toml".to_string());
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
        sections: !sections.is_empty(),
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
