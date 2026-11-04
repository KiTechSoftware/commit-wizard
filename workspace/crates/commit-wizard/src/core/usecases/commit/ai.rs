//! `cw commit --ai`: asks the configured AI backend for a commit message for the pending changes.

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode, PromptTrait,
        capabilities::ai::{Suggestion, commit_request, parse_suggestion},
        models::policy::ai::AiModel,
    },
    infra::ai,
};

/// Returns the AI's suggestion, or `None` when a person at a terminal declined it.
pub fn suggest(
    ctx: &Context,
    stage_all: bool,
    stage_tracked: bool,
) -> CoreResult<Option<Suggestion>> {
    let config = ctx
        .config()
        .ok_or_else(|| ErrorCode::ConfigUnreadable.error())?;
    let model = AiModel::from_config(config);
    if !model.enabled || !model.commands.commit {
        return Err(ErrorCode::AiUnsupported.error().with_context(
            "reason",
            "AI commit messages are off; set `enabled = true` under [ai] and `commit = true` under [ai.commands]",
        ));
    }

    let diff = ctx.git().diff_for_commit(stage_all, stage_tracked)?;
    if diff.trim().is_empty() {
        return Err(ErrorCode::InvalidInput.error().with_context(
            "reason",
            "there are no changes to describe; stage some first",
        ));
    }

    let ui = ctx.ui();
    let policy = &ctx.policy().commit;
    ui.logger().info(&format!(
        "Asking {} for a commit message...",
        model.provider
    ));
    let text = ai::generate(
        model.provider,
        model.model.as_deref(),
        &commit_request(&diff, policy),
    )?;

    let suggestion = parse_suggestion(&text, policy).ok_or_else(|| {
        ErrorCode::AiProviderFailed
            .error()
            .with_context(
                "reason",
                "the AI did not return a commit message the rules allow",
            )
            .with_context("answer", text.lines().next().unwrap_or("").to_string())
    })?;

    if ctx.is_interactive() && !ctx.auto_yes() {
        eprintln!("-------------------------");
        eprintln!(
            "{}{}{}: {}",
            suggestion.commit_type,
            suggestion
                .scope
                .as_ref()
                .map(|s| format!("({s})"))
                .unwrap_or_default(),
            if suggestion.breaking { "!" } else { "" },
            suggestion.summary
        );
        if let Some(body) = &suggestion.body {
            eprintln!("\n{body}");
        }
        eprintln!("-------------------------");
        if !ui.confirm("Use this message?", true)? {
            return Ok(None);
        }
    }
    Ok(Some(suggestion))
}
