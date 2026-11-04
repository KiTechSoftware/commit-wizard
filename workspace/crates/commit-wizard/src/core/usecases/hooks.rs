//! `cw hooks`: installs, removes and reports the Git hooks that run `cw`. A hook that `cw` wrote carries a marker line, so `cw` only ever replaces or removes its own.

use std::{fs, path::Path, time::Instant};

use crate::{
    cli::cmd::hooks::HookName,
    core::{Context, CoreResult},
    engine::{constants::emoji::SUCCESS, error::ErrorCode, models::policy::hooks::HooksModel},
};

const MARKER: &str = "# managed by commit-wizard (cw hooks)";

const COMMIT_MSG: &str = r#"#!/bin/sh
# managed by commit-wizard (cw hooks)
# Validates the commit message against the project's rules.
if ! command -v cw >/dev/null 2>&1; then
  echo "cw: not found on PATH; skipping the commit-msg check" >&2
  exit 0
fi
exec cw check --message-file "$1"
"#;

const PRE_PUSH: &str = r#"#!/bin/sh
# managed by commit-wizard (cw hooks)
# Validates the commits that are about to be pushed.
if ! command -v cw >/dev/null 2>&1; then
  echo "cw: not found on PATH; skipping the pre-push check" >&2
  exit 0
fi
is_zero() { case "$1" in *[!0]*) return 1 ;; *) return 0 ;; esac; }
while read -r local_ref local_sha remote_ref remote_sha; do
  is_zero "$local_sha" && continue
  if is_zero "$remote_sha"; then
    cw check --to "$local_sha" --tail 50 || exit 1
  else
    cw check --from "$remote_sha" --to "$local_sha" || exit 1
  fi
done
exit 0
"#;

const ALL: [HookName; 2] = [HookName::CommitMsg, HookName::PrePush];

fn script(hook: HookName) -> &'static str {
    match hook {
        HookName::CommitMsg => COMMIT_MSG,
        HookName::PrePush => PRE_PUSH,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    Absent,
    Managed,
    Foreign,
}

fn state_of(path: &Path) -> State {
    match fs::read_to_string(path) {
        Ok(text) if text.contains(MARKER) => State::Managed,
        Ok(_) => State::Foreign,
        // An unreadable (e.g. binary) hook is not ours either.
        Err(_) if path.exists() => State::Foreign,
        Err(_) => State::Absent,
    }
}

fn io_error(op: &str, path: &Path, err: std::io::Error) -> crate::engine::Error {
    ErrorCode::HookInstallFailed
        .error()
        .with_context("operation", op.to_string())
        .with_context("path", path.display().to_string())
        .with_context("error", err.to_string())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

fn require_repo(ctx: &Context) -> CoreResult<()> {
    if ctx.in_git_repo() {
        Ok(())
    } else {
        Err(ErrorCode::StateInvalid
            .error()
            .with_context("reason", "not inside a Git repository"))
    }
}

fn report(
    ctx: &Context,
    title: &str,
    command: &str,
    start: Instant,
    lines: Vec<(String, String)>,
) -> CoreResult<()> {
    let ui = ctx.ui();
    let meta = ui
        .new_output_meta()
        .with_command(command.to_string())
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_dry_run(ctx.dry_run());
    let mut content = ui.new_output_content().title(title);
    for (hook, result) in lines {
        content = content.data(&hook, result);
    }
    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn install(ctx: &Context, requested: Vec<HookName>) -> CoreResult<()> {
    require_repo(ctx)?;
    let start = Instant::now();
    let git = ctx.git();
    let dir = git.hooks_dir()?;

    let selected: Vec<HookName> = if !requested.is_empty() {
        requested
    } else {
        let model = ctx
            .config()
            .map(HooksModel::from_config)
            .unwrap_or_default();
        let configured: Vec<HookName> = ALL
            .into_iter()
            .filter(|hook| match hook {
                HookName::CommitMsg => model.commit_msg,
                HookName::PrePush => model.pre_push,
            })
            .collect();
        if configured.is_empty() {
            ALL.to_vec()
        } else {
            configured
        }
    };

    // Check everything first so a refusal leaves no hook half-installed.
    for hook in &selected {
        let path = dir.join(hook.file_name());
        if state_of(&path) == State::Foreign && !ctx.force() {
            return Err(ErrorCode::HookInstallFailed
                .error()
                .with_context("hook", hook.file_name().to_string())
                .with_context("path", path.display().to_string())
                .with_context(
                    "reason",
                    "a hook that cw did not write already exists; use --force to replace it",
                ));
        }
    }

    let mut lines = Vec::new();
    for hook in selected {
        let path = dir.join(hook.file_name());
        let verb = if ctx.dry_run() {
            "would install"
        } else {
            "installed"
        };
        if !ctx.dry_run() {
            fs::create_dir_all(&dir).map_err(|e| io_error("create hooks directory", &dir, e))?;
            fs::write(&path, script(hook)).map_err(|e| io_error("write hook", &path, e))?;
            make_executable(&path).map_err(|e| io_error("make hook executable", &path, e))?;
        }
        ctx.ui().logger().ok(&format!(
            "{SUCCESS} {verb} {} ({})",
            hook.file_name(),
            path.display()
        ));
        lines.push((hook.file_name().to_string(), verb.to_string()));
    }
    report(ctx, "Git Hooks", "hooks", start, lines)
}

pub fn uninstall(ctx: &Context, requested: Vec<HookName>) -> CoreResult<()> {
    require_repo(ctx)?;
    let start = Instant::now();
    let dir = ctx.git().hooks_dir()?;
    let selected = if requested.is_empty() {
        ALL.to_vec()
    } else {
        requested
    };

    let mut lines = Vec::new();
    for hook in selected {
        let path = dir.join(hook.file_name());
        let result = match state_of(&path) {
            State::Absent => "not installed",
            State::Foreign => "left alone (not written by cw)",
            State::Managed => {
                if ctx.dry_run() {
                    "would remove"
                } else {
                    fs::remove_file(&path).map_err(|e| io_error("remove hook", &path, e))?;
                    "removed"
                }
            }
        };
        ctx.ui()
            .logger()
            .info(&format!("{}: {result}", hook.file_name()));
        lines.push((hook.file_name().to_string(), result.to_string()));
    }
    report(ctx, "Git Hooks", "hooks", start, lines)
}

pub fn status(ctx: &Context) -> CoreResult<()> {
    require_repo(ctx)?;
    let start = Instant::now();
    let dir = ctx.git().hooks_dir()?;

    let mut lines = Vec::new();
    for hook in ALL {
        let result = match state_of(&dir.join(hook.file_name())) {
            State::Absent => "not installed",
            State::Managed => "installed (cw)",
            State::Foreign => "present (not managed by cw)",
        };
        ctx.ui()
            .logger()
            .info(&format!("{}: {result}", hook.file_name()));
        lines.push((hook.file_name().to_string(), result.to_string()));
    }
    report(ctx, "Git Hooks", "hooks", start, lines)
}
