//! `cw branch`: list, switch, create and prune local branches.

use std::time::Instant;

use crate::core::Context;

pub mod create;
pub mod list;
pub mod prune;
pub mod switch;

/// The output metadata every branch subcommand reports.
fn meta(ctx: &Context, start: Instant, command: &str) -> scriba::Meta {
    ctx.ui()
        .new_output_meta()
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command(command.to_string())
        .with_dry_run(ctx.dry_run())
}

/// `2026-10-31` for a Unix timestamp.
fn date(seconds: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, 0)
        .map(|moment| moment.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// The first `max` characters of a commit subject, with an ellipsis if cut.
fn short(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{cut}…")
}
