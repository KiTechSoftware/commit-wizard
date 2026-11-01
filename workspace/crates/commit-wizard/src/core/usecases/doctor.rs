//! `cw doctor`: checks the environment and configuration and reports problems; `fix` repairs the ones it can.

use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::config::init::{self, ConfigContext, ConfigOption, ConfigSpec},
        config::resolver::{load_project_config, load_rules_config, load_standard_config},
        config::{ConfigFormat, lint::unknown_top_level_keys},
        constants::{
            competing_project_configs,
            emoji::{ERROR, INFO, SUCCESS, WARN},
            find_config_file,
            paths::resolve_new_project_config_path,
            resolve_global_config_path, resolve_global_rules_path, resolve_project_config_path,
        },
    },
};

enum CheckStatus {
    Ok,
    Warn,
    Info,
    Error,
}

struct CheckResult {
    label: String,
    status: CheckStatus,
    detail: String,
}

impl CheckResult {
    fn ok(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            status: CheckStatus::Ok,
            detail: detail.into(),
        }
    }

    fn warn(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            status: CheckStatus::Warn,
            detail: detail.into(),
        }
    }

    fn info(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            status: CheckStatus::Info,
            detail: detail.into(),
        }
    }

    fn error(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            status: CheckStatus::Error,
            detail: detail.into(),
        }
    }

    fn icon(&self) -> &str {
        match self.status {
            CheckStatus::Ok => SUCCESS,
            CheckStatus::Warn => WARN,
            CheckStatus::Info => INFO,
            CheckStatus::Error => ERROR,
        }
    }

    fn is_error(&self) -> bool {
        matches!(self.status, CheckStatus::Error)
    }
}

pub fn run(ctx: &Context) -> CoreResult<()> {
    let ui = ctx.ui();
    let start = Instant::now();
    let git = ctx.git();

    let mut checks: Vec<CheckResult> = Vec::new();

    // --- Git ---
    let git_installed = git.is_installed();
    checks.push(if git_installed {
        CheckResult::ok("Git", "git is available in PATH")
    } else {
        CheckResult::error("Git", "git is not available in PATH")
    });

    let in_git_repo = ctx.in_git_repo();
    checks.push(if in_git_repo {
        CheckResult::ok(
            "Git repository",
            format!("repo root: {}", ctx.repo_root().display()),
        )
    } else {
        CheckResult::warn("Git repository", "not inside a git repository")
    });

    // --- Project config / rules ---
    let repo_config_path = resolve_project_config_path(
        ctx.cwd(),
        if in_git_repo {
            Some(ctx.repo_root())
        } else {
            None
        },
        in_git_repo,
        ctx.runtime().explicit_config_path().map(|p| p.as_path()),
    );

    checks.push(match &repo_config_path {
        Some(path) => {
            if load_project_config(path, Some(&ui)).is_none() {
                CheckResult::error(
                    "Project config",
                    format!("malformed config at {}", path.display()),
                )
            } else {
                CheckResult::ok("Project config", path.display().to_string())
            }
        }
        None => CheckResult::info("Project config", "not found — default policy will be used"),
    });

    for (label, path) in [
        ("Project config", repo_config_path.clone()),
        ("Global config", resolve_global_config_path().ok()),
    ] {
        if let Some(path) = path
            && let Ok(raw) = std::fs::read_to_string(&path)
        {
            for unknown in unknown_top_level_keys(ConfigFormat::from_path(&path), &raw) {
                let detail = format!("{}: {}", path.display(), unknown.describe());
                // The project config already logs this when it loads; say it for the global one.
                if label == "Global config" {
                    ui.logger().warn(&detail);
                }
                checks.push(CheckResult::warn(label, detail));
            }
        }
    }

    if let Some(chosen) = &repo_config_path {
        let ignored = competing_project_configs(chosen);
        if !ignored.is_empty() {
            checks.push(CheckResult::warn(
                "Project config files",
                format!(
                    "using {}; ignoring {}",
                    chosen.display(),
                    ignored
                        .iter()
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }
    }

    let repo_rules_path = repo_config_path
        .as_ref()
        .and_then(|p| find_config_file(p.parent().unwrap_or(p), ".cwizard.rules"));
    if let Some(ref path) = repo_rules_path {
        checks.push(if load_rules_config(path).is_none() {
            CheckResult::error(
                "Project rules",
                format!("malformed rules at {}", path.display()),
            )
        } else {
            CheckResult::ok("Project rules", path.display().to_string())
        });
    }

    // --- Global config / rules ---
    let global_config_path = resolve_global_config_path().ok();
    checks.push(match &global_config_path {
        Some(path) if path.exists() => {
            if load_standard_config(path).is_none() {
                CheckResult::error(
                    "Global config",
                    format!("malformed config at {}", path.display()),
                )
            } else {
                CheckResult::ok("Global config", path.display().to_string())
            }
        }
        _ => CheckResult::info("Global config", "not found — default policy will be used"),
    });

    let global_rules_path = resolve_global_rules_path().ok();
    checks.push(match &global_rules_path {
        Some(path) if path.exists() => {
            if load_rules_config(path).is_none() {
                CheckResult::error(
                    "Global rules",
                    format!("malformed rules at {}", path.display()),
                )
            } else {
                CheckResult::ok("Global rules", path.display().to_string())
            }
        }
        _ => CheckResult::info("Global rules", "not found — default policy will be used"),
    });

    // --- Registries ---
    let sources = &ctx.sources();
    let registry_count = sources.registries.len();
    checks.push(if registry_count > 0 {
        let ids: Vec<&str> = sources.registries.iter().map(|r| r.id.as_str()).collect();
        CheckResult::ok(
            "Registries",
            format!("{} loaded: {}", registry_count, ids.join(", ")),
        )
    } else {
        CheckResult::info("Registries", "none configured")
    });
    for failure in &sources.registry_failures {
        checks.push(CheckResult::warn("Registry", failure.clone()));
    }

    // --- Print results ---
    for check in &checks {
        ui.logger()
            .kv(&format!("{}  {}", check.icon(), check.label), &check.detail);
    }

    let error_count = checks.iter().filter(|c| c.is_error()).count();
    let warn_count = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Warn))
        .count();
    let duration_ms = start.elapsed().as_millis() as u64;

    ui.logger().detail("");
    if error_count == 0 && warn_count == 0 {
        ui.logger().ok(&format!("{SUCCESS}  All checks passed"));
    } else if error_count > 0 {
        ui.logger()
            .warn(&format!("{WARN}  {error_count} error(s) found"));
    } else {
        ui.logger()
            .warn(&format!("{WARN}  {warn_count} warning(s) found"));
    }

    let content = ui
        .new_output_content()
        .title("Doctor")
        .subtitle("Environment diagnostics and configuration status")
        .heading(2, "Paths")
        .key_value("cwd", ctx.cwd().display().to_string())
        .key_value("repo_root", ctx.repo_root().display().to_string())
        .key_value(
            "global_config",
            ctx.runtime().global_config_path().display().to_string(),
        )
        .key_value(
            "global_state",
            ctx.runtime().global_state_path().display().to_string(),
        )
        .key_value(
            "global_cache",
            ctx.runtime().global_cache_path().display().to_string(),
        )
        .heading(2, "Environment")
        .key_value("in_git_repo", in_git_repo.to_string())
        .key_value("git_installed", git_installed.to_string())
        .key_value(
            "has_project_config",
            sources
                .repo_config
                .as_ref()
                .map(|c| c.base.is_some())
                .unwrap_or(false)
                .to_string(),
        )
        .key_value(
            "has_project_rules",
            sources
                .repo_config
                .as_ref()
                .map(|c| c.rules.is_some())
                .unwrap_or(false)
                .to_string(),
        )
        .key_value(
            "has_global_config",
            sources
                .global_config
                .as_ref()
                .map(|c| c.base.is_some())
                .unwrap_or(false)
                .to_string(),
        )
        .key_value(
            "has_global_rules",
            sources
                .global_config
                .as_ref()
                .map(|c| c.rules.is_some())
                .unwrap_or(false)
                .to_string(),
        )
        .key_value(
            "has_env_config",
            sources
                .env_config
                .as_ref()
                .map(|c| c.base.is_some())
                .unwrap_or(false)
                .to_string(),
        )
        .key_value(
            "has_cli_config",
            sources
                .cli_config
                .as_ref()
                .map(|c| c.base.is_some())
                .unwrap_or(false)
                .to_string(),
        )
        .heading(2, "Registry Pool")
        .key_value("total_registries", registry_count.to_string());

    // Add details for each registry
    let mut content = content;
    for (idx, registry) in sources.registries.iter().enumerate() {
        let status = if registry.is_active {
            "[ACTIVE]"
        } else {
            "[available]"
        };
        let registry_info = format!(
            "{}: {} (tag: {}) {}",
            idx + 1,
            registry.url,
            registry.tag,
            status
        );
        content = content.key_value(format!("registry_{}", idx + 1), registry_info);
    }

    // Load and display state information
    let state_file_path = ctx.runtime().state_file_path();
    use crate::engine::models::state::AppState;
    let state = AppState::load(&state_file_path).unwrap_or_default();

    let content = if let Some(registry_state) = &state.registry {
        content
            .heading(2, "Registry State")
            .key_value("state_file", state_file_path.display().to_string())
            .key_value(
                "registry_name",
                registry_state.name.as_deref().unwrap_or("(unnamed)"),
            )
            .key_value("registry_url", registry_state.url.clone())
            .key_value("registry_ref", registry_state.r#ref.clone())
            .key_value(
                "registry_section",
                registry_state.section.as_deref().unwrap_or("(root)"),
            )
            .key_value("resolved_commit", registry_state.resolved_commit.clone())
            .key_value("cache_path", registry_state.cache_path.clone())
    } else {
        content
            .heading(2, "Registry State")
            .key_value("state_file", state_file_path.display().to_string())
            .plain("No registry state saved")
    };

    let content = content
        .heading(2, "AI")
        .heading(2, "Summary")
        .key_value("errors", error_count.to_string())
        .key_value("warnings", warn_count.to_string())
        .plain(format!(
            "{} check(s) passed, {} warning(s) found, {} error(s) found",
            checks.len() - error_count - warn_count,
            warn_count,
            error_count
        ));

    let meta = ui
        .new_output_meta()
        .with_command("doctor".to_string())
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_dry_run(false);

    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn fix(ctx: &Context) -> CoreResult<()> {
    let ui = ctx.ui();
    let start = Instant::now();

    ui.logger().info("Checking for fixable issues...");

    let sources = ctx.sources();
    let mut repaired = 0usize;
    let mut already_ok = 0usize;
    let mut unfixable = 0usize;

    if !ctx.in_git_repo() {
        ui.logger().warn(&format!(
            "{WARN}  Not inside a git repository — no automatic fix available"
        ));
        unfixable += 1;
    } else {
        ui.logger()
            .ok(&format!("{SUCCESS}  Git repository detected"));
        already_ok += 1;

        if sources.repo_config.is_some() {
            ui.logger()
                .ok(&format!("{SUCCESS}  Project config present"));
            already_ok += 1;
        } else {
            let output_path = resolve_new_project_config_path(
                ctx.cwd(),
                Some(ctx.repo_root().as_path()),
                true,
                ctx.explicit_config_path().map(|p| p.as_path()),
                true,
            );
            init::init_config(&ConfigContext {
                option: ConfigOption::Project,
                output_path: output_path.clone(),
                spec: ConfigSpec::Standard,
                force: false,
                auto_yes: ctx.auto_yes(),
                dry_run: ctx.dry_run(),
                hidden: true,
                with_rules: false,
                registry_options: Default::default(),
                file_format: ConfigFormat::from_path(&output_path),
                answers: None,
            })?;
            let verb = if ctx.dry_run() {
                "Would create"
            } else {
                "Created"
            };
            ui.logger().ok(&format!(
                "{SUCCESS}  {verb} project config at {}",
                output_path.display()
            ));
            repaired += 1;
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    ui.logger().detail("");
    if repaired > 0 || already_ok > 0 {
        ui.logger()
            .info("Run 'cw doctor' to verify the environment.");
    }

    let content = ui
        .new_output_content()
        .title("Doctor Fix")
        .subtitle("Automatic repair attempt")
        .key_value("repaired", repaired.to_string())
        .key_value("already_ok", already_ok.to_string())
        .key_value("unfixable", unfixable.to_string())
        .heading(1, "Repairs attempted")
        .paragraph(format!(
            "Repaired {repaired} item(s); {already_ok} already fine; {unfixable} could not be fixed automatically."
        ));

    let meta = ui
        .new_output_meta()
        .with_command("doctor fix".to_string())
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_dry_run(ctx.dry_run());

    ui.print_with_meta(&content, Some(&meta), true)
}
