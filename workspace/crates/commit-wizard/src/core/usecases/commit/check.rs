use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::commit::check::validate_commits,
        constants::emoji::{ERROR, SUCCESS},
        error::ErrorCode,
        models::{git::CommitSummary, policy::commit::CommitModel},
    },
};

pub fn run(
    ctx: &Context,
    tail: Option<u32>,
    from: Option<String>,
    to: Option<String>,
    full_commit_hash: bool,
) -> CoreResult<()> {
    let ui = ctx.ui();
    let start = Instant::now();

    let format_hash = |hash: &str| {
        if full_commit_hash {
            hash.to_string()
        } else {
            hash[..8.min(hash.len())].to_string()
        }
    };

    // Get the resolved config and build the commit policy
    let resolved_config = ctx.config().ok_or_else(|| {
        ErrorCode::ConfigUnreadable
            .error()
            .with_context("context", "Config not resolved")
    })?;
    let policy = CommitModel::from_config(resolved_config);

    // Fetch commits in the specified range
    let to_ref = to.as_deref().unwrap_or("HEAD").to_string();
    let mut raw_commits = ctx.git().list_commits(from.as_deref(), &to_ref)?;

    if let Some(n) = tail {
        let n = n as usize;
        if raw_commits.len() > n {
            raw_commits = raw_commits.into_iter().take(n).collect();
        }
    }

    let commits = raw_commits
        .into_iter()
        .map(|c| CommitSummary {
            full_message: c.full_message,
            hash: c.hash,
            summary: c.summary,
        })
        .collect::<Vec<_>>();

    // Validate commits against policy
    let report = validate_commits(commits, &policy);

    let all_valid = report.invalid_count == 0;

    // Log summary
    if all_valid {
        ui.logger().ok(&format!(
            "{} All {} commit(s) are valid",
            SUCCESS, report.total
        ));
    } else {
        ui.logger().warn(&format!(
            "{} {} of {} commit(s) are invalid",
            ERROR, report.invalid_count, report.total
        ));
    }

    // Build output metadata
    let duration_ms = start.elapsed().as_millis() as u64;
    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("check".to_string())
        .with_dry_run(ctx.dry_run());

    // Build output content
    let mut content = ui
        .new_output_content()
        .title(if all_valid {
            "Commit History Valid"
        } else {
            "Commit History Invalid"
        })
        .subtitle(if all_valid {
            "All commits conform to the active rules"
        } else {
            "Some commits do not conform to the active rules"
        })
        .data("total", report.total.to_string())
        .data("invalid_count", report.invalid_count.to_string())
        .data("valid", all_valid.to_string());

    if let Some(from) = from {
        content = content.data("from", from);
    }
    if let Some(to) = to {
        content = content.data("to", to);
    }
    if let Some(tail) = tail {
        content = content.data("tail", tail.to_string());
    }

    // Report invalid commits with violation details
    let invalid_lines: Vec<String> = report
        .commits
        .iter()
        .filter(|c| !c.valid)
        .map(|c| {
            let violations_str = c
                .violations
                .iter()
                .map(|v| v.message())
                .collect::<Vec<_>>()
                .join(" | ");
            format!(
                "{} {} - {}\n       {}",
                ERROR,
                format_hash(&c.hash),
                c.summary,
                violations_str
            )
        })
        .collect();

    if !invalid_lines.is_empty() {
        content = content.section(
            "Invalid Commits",
            invalid_lines.join("\n"),
            "sh".to_string(),
        );
    }

    // Report valid commits
    let valid_lines: Vec<String> = report
        .commits
        .iter()
        .filter(|c| c.valid)
        .map(|c| format!("{} {} {}", SUCCESS, format_hash(&c.hash), c.summary))
        .collect();

    if !valid_lines.is_empty() {
        content = content.section("Valid Commits", valid_lines.join("\n"), "sh".to_string());
    }

    // Plain output: invalid count for machine consumption
    let plain = report.invalid_count.to_string();
    content = content.plain(plain);

    ui.print_with_meta(&content, Some(&meta), all_valid)
}
