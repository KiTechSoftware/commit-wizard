use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::commit::{check::validate_commits, push::evaluate_push},
        error::ErrorCode,
        models::policy::commit::CommitModel,
    },
};

pub fn run(
    ctx: &Context,
    from: Option<String>,
    to: String,
    remote: String,
    branch: Option<String>,
    allow_empty: bool,
) -> CoreResult<()> {
    let ui = ctx.ui();
    let start = Instant::now();

    let resolved_config = ctx.config().ok_or_else(|| {
        ErrorCode::ConfigUnreadable
            .error()
            .with_context("context", "Config not resolved")
    })?;
    let policy = CommitModel::from_config(resolved_config);
    let push_policy = ctx.policy();
    let git = ctx.git();

    let current_branch = git.current_branch()?;
    let push_branch = branch.unwrap_or_else(|| current_branch.clone());

    let raw_commits = git.list_commits(from.as_deref(), &to)?;
    let checked = validate_commits(raw_commits, &policy);
    let report = evaluate_push(push_policy, &push_branch, checked.commits);

    let duration_ms = start.elapsed().as_millis() as u64;

    let mut content = ctx
        .ui()
        .new_output_content()
        .title("Push Validation")
        .subtitle("Validate push against active rules before pushing")
        .data("current_branch", current_branch.clone())
        .data("push_branch", push_branch.clone())
        .data("remote", remote.clone())
        .data("range_to", to.clone())
        .data("protected_branch", report.protected_branch.to_string())
        .data("total_checked", report.total_checked.to_string())
        .data("invalid_count", report.invalid_count.to_string())
        .data("blocked", report.blocked.to_string())
        .data("allow_empty", allow_empty.to_string());

    if let Some(from_ref) = from.clone() {
        content = content.data("range_from", from_ref);
    }

    if !report.block_reasons.is_empty() {
        content = content.section(
            "Block Reasons",
            report.block_reasons.join("\n"),
            "sh".to_string(),
        );
    }

    let invalid_lines = report
        .commits
        .iter()
        .filter(|c| !c.valid)
        .map(|c| format!("{} {}", c.hash, c.summary))
        .collect::<Vec<_>>();

    if !invalid_lines.is_empty() {
        content = content.section(
            "Invalid Commits",
            invalid_lines.join("\n"),
            "sh".to_string(),
        );
    }

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("push".to_string())
        .with_dry_run(ctx.dry_run());

    if report.blocked {
        return ui.print_with_meta(&content, Some(&meta), true);
    }

    if ctx.dry_run() {
        content = content
            .title("Push Preview")
            .subtitle("Dry run: no push was performed");
        return ui.print_with_meta(&content, Some(&meta), true);
    }

    git.push_branch(&remote, &push_branch)?;

    content = content
        .title("Push Completed")
        .subtitle("Push completed successfully");

    ui.print_with_meta(&content, Some(&meta), true)
}
