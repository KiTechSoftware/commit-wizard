use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::config::{
            edit,
            show::{ConfigShowFormat, ConfigShowInput, ConfigTarget, config_show as show_config},
        },
        constants::{PROJECT_CONFIG_FILE_NAME, app_config_dir, resolve_new_project_config_path},
    },
};

pub fn config_unset(ctx: &Context, key: &str, global: bool) -> CoreResult<()> {
    let ui = ctx.ui();
    let dry_run = ctx.dry_run();

    let scope = if global { "global" } else { "project" };
    let target = if global {
        ConfigTarget::Global
    } else {
        ConfigTarget::Project
    };

    let project_config_path = ctx.project_config_path();
    let input = edit::ConfigUnsetInput {
        cwd: ctx.cwd(),
        target,
        key,
        dry_run,
        explicit_path: project_config_path.as_deref(),
    };

    let start = Instant::now();
    let output = edit::config_unset(&input)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("config.unset".to_string())
        .with_scope(scope.to_string())
        .with_dry_run(dry_run);

    let content = ui
        .new_output_content()
        .title(format!("Unset config `{}` in {} scope", output.key, scope))
        .subtitle(format!(
            "Path: {}, Dry run: {}",
            output.path.display(),
            ctx.dry_run()
        ))
        .data("key", output.key)
        .data("removed", output.removed)
        .data("path", output.path.display().to_string());

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn config_set(ctx: &Context, key: &str, value: &str, global: bool) -> CoreResult<()> {
    let ui = ctx.ui();
    let dry_run = ctx.dry_run();
    let scope = if global { "global" } else { "project" };
    let target = if global {
        ConfigTarget::Global
    } else {
        ConfigTarget::Project
    };
    let project_config_path = ctx.project_config_path();
    let input = edit::ConfigSetInput {
        cwd: ctx.cwd(),
        target,
        key,
        value,
        dry_run,
        explicit_path: project_config_path.as_deref(),
    };

    let start = Instant::now();
    let output = edit::config_set(&input)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("config.set".to_string())
        .with_scope(scope.to_string())
        .with_dry_run(dry_run);

    let content = ui
        .new_output_content()
        .title(format!("Set config `{}` in {} scope", output.key, scope))
        .subtitle(format!(
            "Path: {}, Dry run: {}",
            output.path.display(),
            ctx.dry_run()
        ))
        .data("key", output.key)
        .data("value", output.value)
        .data("path", output.path.display().to_string());

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn config_path(ctx: &Context, global: bool) -> CoreResult<()> {
    let ui = ctx.ui();
    let scope = if global { "global" } else { "project" };

    let path = if global {
        app_config_dir()?.join(PROJECT_CONFIG_FILE_NAME)
    } else {
        let project_config_path = ctx.project_config_path();
        ctx.project_config_path().unwrap_or_else(|| {
            resolve_new_project_config_path(
                ctx.cwd(),
                Some(ctx.repo_root().as_path()),
                ctx.in_git_repo(),
                project_config_path.as_deref(),
                true,
            )
        })
    };

    let meta = ui
        .new_output_meta()
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("config.path".to_string())
        .with_scope(scope.to_string());

    let content = ui
        .new_output_content()
        .title(format!("Config path for {} scope", scope))
        .data("path", path.display().to_string());

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn config_show(ctx: &Context, global: bool) -> CoreResult<()> {
    let ui = ctx.ui();
    let scope = if global { "global" } else { "project" };
    let target = if global {
        ConfigTarget::Global
    } else {
        ConfigTarget::Project
    };
    let project_config_path = ctx.project_config_path();
    let input = ConfigShowInput {
        cwd: ctx.cwd(),
        target,
        explicit_path: project_config_path.as_deref(),
        format: ConfigShowFormat::Human,
    };

    let start = Instant::now();
    let output = show_config(&input)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("config.show".to_string())
        .with_scope(scope.to_string());

    let content = ui
        .new_output_content()
        .title(format!("Config for {} scope", scope))
        .subtitle(format!("Path: {}", output.path.display()))
        .data("path", output.path.display().to_string())
        .data("exists", output.exists)
        .data("content", output.content);

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn config_get(ctx: &Context, key: &str, global: bool) -> CoreResult<()> {
    let ui = ctx.ui();
    let scope = if global { "global" } else { "project" };
    let target = if global {
        ConfigTarget::Global
    } else {
        ConfigTarget::Project
    };
    let project_config_path = ctx.project_config_path();
    let input = edit::ConfigGetInput {
        cwd: ctx.cwd(),
        target,
        key,
        explicit_path: project_config_path.as_deref(),
    };

    let start = Instant::now();
    let output = edit::config_get(&input)?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("config.get".to_string())
        .with_scope(scope.to_string());

    let content = ui
        .new_output_content()
        .title(format!("Config `{}` in {} scope", output.key, scope))
        .data("key", output.key)
        .data("value", output.value.to_string())
        .data("path", output.path.display().to_string());

    ui.print_with_meta(&content, Some(&meta), true)
}
