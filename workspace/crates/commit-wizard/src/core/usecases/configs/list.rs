use std::time::Instant;

use scriba::Table;

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::config::{
            effective::{document_rows, effective_rows, file_document, to_document},
            show::ConfigTarget,
        },
        constants::resolve_global_config_path,
        error::ErrorCode,
    },
};

/// `cw config list`: the settings in one config file, or with `effective` the
/// resolved configuration and the layer each value came from.
pub fn config_list(
    ctx: &Context,
    global: bool,
    effective: bool,
    prefix: Option<String>,
) -> CoreResult<()> {
    if effective && global {
        return Err(ErrorCode::InvalidInput
            .error()
            .with_context("reason", "--effective cannot be combined with --global"));
    }
    if effective {
        list_effective(ctx, prefix.as_deref())
    } else {
        list_file(ctx, global, prefix.as_deref())
    }
}

fn list_file(ctx: &Context, global: bool, prefix: Option<&str>) -> CoreResult<()> {
    let ui = ctx.ui();
    let start = Instant::now();
    let (target, scope, path) = if global {
        (
            ConfigTarget::Global,
            "global",
            resolve_global_config_path()?,
        )
    } else {
        let path = ctx.project_config_path().ok_or_else(|| {
            ErrorCode::ConfigUnreadable
                .error()
                .with_context("reason", "no project config file found")
        })?;
        (ConfigTarget::Project, "project", path)
    };

    let rows = document_rows(&file_document(target, &path)?, prefix);
    let table = Table::new(
        vec!["key".to_string(), "value".to_string()],
        rows.iter()
            .map(|(key, value)| vec![key.clone(), value.clone()])
            .collect(),
    );

    let meta = ui
        .new_output_meta()
        .with_command("config.list".to_string())
        .with_scope(scope.to_string())
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string());
    let content = ui
        .new_output_content()
        .title(format!("Config settings ({scope} scope)"))
        .data("path", path.display().to_string())
        .data("settings", rows.len().to_string())
        .table(Some("Settings".to_string()), table);

    ui.print_with_meta(&content, Some(&meta), true)
}

fn list_effective(ctx: &Context, prefix: Option<&str>) -> CoreResult<()> {
    let ui = ctx.ui();
    let start = Instant::now();
    let resolved = ctx.config().ok_or_else(|| {
        ErrorCode::ConfigUnreadable
            .error()
            .with_context("context", "Config not resolved")
    })?;
    let resolved_doc =
        to_document(&resolved.base).ok_or_else(|| ErrorCode::SerializationFailure.error())?;

    // Highest priority first, as the values are attributed.
    let mut applied = ctx.runtime().config_layers();
    applied.reverse();
    let layers: Vec<_> = applied
        .iter()
        .filter_map(|layer| Some((layer.label.clone(), to_document(&layer.base)?)))
        .collect();
    let rows = effective_rows(&resolved_doc, &layers, prefix);

    let settings = Table::new(
        vec!["key".to_string(), "value".to_string(), "source".to_string()],
        rows.iter()
            .map(|row| vec![row.key.clone(), row.value.clone(), row.source.clone()])
            .collect(),
    );

    let mut layer_rows: Vec<Vec<String>> = applied
        .iter()
        .map(|layer| {
            vec![
                layer.label.clone(),
                layer.detail.clone(),
                "applied".to_string(),
            ]
        })
        .collect();
    if layer_rows.is_empty() {
        layer_rows.push(vec![
            "default".to_string(),
            "built-in defaults".to_string(),
            "applied".to_string(),
        ]);
    }
    for (layer, reason) in ctx.runtime().skipped_layers() {
        layer_rows.push(vec![layer.label, layer.detail, reason]);
    }

    let meta = ui
        .new_output_meta()
        .with_command("config.list".to_string())
        .with_scope("effective".to_string())
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string());
    let content = ui
        .new_output_content()
        .title("Effective configuration")
        .subtitle("Resolved settings and the layer each came from; unlisted settings use built-in defaults")
        .data("settings", rows.len().to_string())
        .table(Some("Layers (highest priority first)".to_string()), Table::new(
            vec!["layer".to_string(), "from".to_string(), "status".to_string()],
            layer_rows,
        ))
        .table(Some("Settings".to_string()), settings);

    ui.print_with_meta(&content, Some(&meta), true)
}
