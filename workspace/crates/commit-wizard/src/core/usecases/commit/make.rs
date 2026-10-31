//! `cw commit`: builds a Conventional Commit message from flags, or by asking for the missing parts on a terminal, and creates the commit.

use crate::engine::capabilities::commit::fix::build_header;
use std::time::Instant;

use scriba::{SelectOption, SelectRequest};

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode, PromptTrait,
        capabilities::commit::ticket,
        constants::emoji::{ERROR, PREVIEW, SUCCESS},
        models::policy::{
            commit::{ScopeRequirement, TicketPolicy},
            enforcement::TicketSource,
        },
    },
};

/// What was asked of `cw commit`, from flags.
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub allow_empty: bool,
    pub commit_type: Option<String>,
    pub scope: Option<String>,
    pub message: Option<String>,
    pub breaking: bool,
    pub breaking_message: Option<String>,
    pub body: Option<String>,
    pub footer: Vec<String>,
    pub ticket: Option<String>,
    /// Stage every change, untracked files included (`-A`).
    pub stage_all: bool,
    /// Stage changes to tracked files only (`-a`).
    pub stage_tracked: bool,
}

pub fn run(ctx: &Context, options: Options) -> CoreResult<()> {
    let Options {
        allow_empty,
        commit_type,
        scope,
        message,
        breaking,
        breaking_message,
        body,
        footer,
        ticket,
        stage_all,
        stage_tracked,
    } = options;
    let ui = ctx.ui();
    let start = Instant::now();
    let policy = &ctx.policy().commit;

    let non_interactive = !ctx.is_interactive() || (commit_type.is_some() && message.is_some());

    // A breaking description is only mandatory when the policy requires the
    // footer; otherwise it is optional, and written when provided.
    if breaking
        && policy.breaking_footer_required
        && breaking_message
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(ErrorCode::InvalidInput.error().with_context(
            "reason",
            "breaking change flag was provided but no breaking message was supplied",
        ));
    }

    // commit type
    let commit_type = match commit_type {
        Some(value) => {
            if !policy.allows_type(&value) {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "type")
                    .with_context("value", value));
            }
            value
        }
        None if non_interactive => {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "type")
                .with_context("reason", "type is required in non-interactive mode"));
        }
        None => {
            let request = SelectRequest::new(
                "What type of change are you committing?".to_string(),
                policy
                    .types
                    .iter()
                    .map(|t| {
                        let label = if policy.use_emojis && t.emoji.is_some() {
                            format!("{} {}", t.emoji.as_deref().unwrap(), t.key)
                        } else {
                            t.key.clone()
                        };
                        SelectOption::new(t.key.clone(), label)
                            .description(t.description.as_deref().unwrap_or_default().to_string())
                    })
                    .collect(),
            )
            .with_page_size(10);

            ui.select(&request).map_err(|_| {
                ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "type")
            })?
        }
    };

    // scope
    let scope: Option<String> = match scope {
        Some(value) if policy.scope_requirement == ScopeRequirement::Disabled => {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "scope")
                .with_context("value", value)
                .with_context(
                    "reason",
                    "scopes are disabled (commit.scopes.mode = disabled)",
                ));
        }
        None if policy.scope_requirement == ScopeRequirement::Disabled => None,
        Some(value) => {
            if policy.restrict_scopes_to_defined && !policy.allows_scope(&value) {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "scope")
                    .with_context("value", value));
            }
            Some(value)
        }
        None if policy.header_format.require_scope && non_interactive => {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "scope")
                .with_context(
                    "reason",
                    "scope is required by policy in non-interactive mode",
                ));
        }
        None if policy.header_format.require_scope => {
            let value = ui.text("Scope", None, Some("Required by current commit policy"))?;
            let trimmed: String = value.trim().to_string();

            if trimmed.is_empty() {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "scope")
                    .with_context("reason", "scope is required by policy"));
            }

            if policy.restrict_scopes_to_defined && !policy.allows_scope(&trimmed) {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "scope")
                    .with_context("value", trimmed));
            }

            Some(trimmed)
        }
        None if non_interactive => None,
        None => {
            if ui.confirm("Add a scope?", false)? {
                let value = ui.text("Scope", None, Some("Leave empty to omit"))?;
                let trimmed: String = value.trim().to_string();

                if trimmed.is_empty() {
                    None
                } else {
                    if policy.restrict_scopes_to_defined && !policy.allows_scope(&trimmed) {
                        return Err(ErrorCode::InvalidInput
                            .error()
                            .with_context("field", "scope")
                            .with_context("value", trimmed));
                    }
                    Some(trimmed)
                }
            } else {
                None
            }
        }
    };

    // ticket
    let ticket = if policy.ticket.required
        || policy.ticket.source != TicketSource::Disabled
        || ticket.is_some()
    {
        resolve_ticket(ctx, &policy.ticket, ticket, non_interactive)?
    } else {
        None
    };

    // summary
    let summary = match message {
        Some(value) if value.trim().is_empty() => {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "message")
                .with_context("reason", "commit summary cannot be empty"));
        }
        Some(value) => value.trim().to_string(),
        None if non_interactive => {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "message")
                .with_context("reason", "message is required in non-interactive mode"));
        }
        None => loop {
            let value = ui.text(
                "Write a short summary",
                None,
                Some("Imperative tone, e.g. 'add parser'"),
            )?;
            let trimmed = value.trim().to_string();
            if !trimmed.is_empty() {
                break trimmed;
            }
            ui.logger().warn("Summary cannot be empty");
        },
    };

    // The ticket opens the subject, so it counts towards the length limit.
    let summary = match &ticket {
        Some(ticket) => ticket::with_ticket(&policy.ticket.header_format, ticket, &summary)?,
        None => summary,
    };

    let subject_max_length = usize::try_from(policy.subject_max_length).unwrap();

    if summary.len() > subject_max_length {
        return Err(ErrorCode::InvalidInput
            .error()
            .with_context("field", "message")
            .with_context("reason", "commit summary exceeds configured max length")
            .with_context("length", summary.len().to_string())
            .with_context("max", policy.subject_max_length.to_string()));
    }

    // breaking details
    let (breaking, breaking_message) = if breaking {
        (true, breaking_message.map(|s| s.trim().to_string()))
    } else if non_interactive {
        (false, None)
    } else if ui.confirm("Does this commit include breaking changes?", false)? {
        let msg = ui.text(
            "Describe the breaking change",
            None,
            Some(if policy.breaking_footer_required {
                "Required by current commit policy"
            } else {
                "Optional; leave empty to omit the BREAKING CHANGE footer"
            }),
        )?;
        let trimmed = msg.trim().to_string();
        if trimmed.is_empty() && policy.breaking_footer_required {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "breaking_message")
                .with_context("reason", "breaking change description cannot be empty"));
        }
        (true, (!trimmed.is_empty()).then_some(trimmed))
    } else {
        (false, None)
    };

    let body = match body {
        Some(value) if value.trim().is_empty() => None,
        Some(value) => Some(value.trim().to_string()),
        None if non_interactive => None,
        None => {
            if ui.confirm("Add a longer description?", false)? {
                let value = ui.text("Body", None, Some("Optional detailed description"))?;
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            } else {
                None
            }
        }
    };

    let footers = if !footer.is_empty() || non_interactive {
        footer
            .into_iter()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>()
    } else {
        let mut lines = Vec::new();
        if ui.confirm("Add footer lines?", false)? {
            loop {
                let value = ui.text(
                    "Footer",
                    None,
                    Some("Examples: Closes #123, Co-authored-by: Name <email>"),
                )?;
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    break;
                }
                lines.push(trimmed);
            }
        }
        lines
    };

    let type_emoji = policy
        .find_type(&commit_type)
        .and_then(|t| t.emoji.as_deref());

    let header = build_header(
        &commit_type,
        scope.as_deref(),
        &summary,
        breaking,
        type_emoji,
        policy.use_emojis,
    );
    let commit_message = build_commit_message(
        &header,
        body.as_deref(),
        if breaking {
            breaking_message.as_deref()
        } else {
            None
        },
        &footers,
        &policy.breaking_footer_key,
    );

    ui.logger().heading(&format!("{} Commit Preview", PREVIEW));
    eprintln!("-------------------------");
    for line in commit_message.lines() {
        eprintln!("{}", line);
    }
    eprintln!("-------------------------");
    eprintln!();

    let duration_ms = start.elapsed().as_millis() as u64;

    if ctx.dry_run() {
        let meta = ui
            .new_output_meta()
            .with_duration_ms(duration_ms)
            .with_timestamp(chrono::Utc::now().to_string())
            .with_command("commit".to_string())
            .with_dry_run(true);

        let content = ui
            .new_output_content()
            .title(format!("{} Commit Preview", PREVIEW))
            .subtitle("Dry run: no commit was created")
            .data(
                "staging",
                if stage_all {
                    "all"
                } else if stage_tracked {
                    "tracked"
                } else {
                    "none"
                },
            )
            .data("type", commit_type)
            .data("scope", scope.unwrap_or_default())
            .data("breaking", breaking.to_string())
            .data("message", commit_message);

        return ui.print_with_meta(&content, Some(&meta), true);
    }

    if !non_interactive && !ui.confirm("Create this commit?", true)? {
        ui.logger().warn(&format!("{} Commit cancelled", ERROR));
        return Ok(());
    }

    if stage_all {
        ctx.git().stage_all()?;
    } else if stage_tracked {
        ctx.git().stage_tracked()?;
    }

    ctx.git().commit(&commit_message, allow_empty)?;

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("commit".to_string())
        .with_dry_run(false);

    let content = ui
        .new_output_content()
        .title(format!("{} Commit Created", SUCCESS))
        .subtitle("Git commit completed successfully")
        .data("type", commit_type)
        .data("scope", scope.unwrap_or_default())
        .data("breaking", breaking.to_string())
        .data("header", header);

    ui.print_with_meta(&content, Some(&meta), true)
}

/// Works out the ticket for a commit: the `--ticket` flag, else the branch name or a
/// question, as `commit.ticket.source` says. `None` means the commit has no ticket.
fn resolve_ticket(
    ctx: &Context,
    policy: &TicketPolicy,
    given: Option<String>,
    non_interactive: bool,
) -> CoreResult<Option<String>> {
    let pattern = policy.regex.as_deref();
    let ui = ctx.ui();

    let missing = || {
        ErrorCode::InvalidInput
            .error()
            .with_context("field", "ticket")
            .with_context(
                "reason",
                "a ticket is required (commit.ticket.required); pass --ticket",
            )
    };

    if let Some(value) = given {
        let value = value.trim().to_string();
        if !ticket::is_valid(pattern, &value)? {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "ticket")
                .with_context("value", value)
                .with_context("reason", "does not match commit.ticket.pattern"));
        }
        return Ok(Some(value));
    }

    // A required ticket needs a way to be given even when no source is configured.
    let source = if policy.source == TicketSource::Disabled && policy.required {
        TicketSource::Prompt
    } else {
        policy.source
    };

    let from_branch = match source {
        TicketSource::Branch | TicketSource::BranchOrPrompt => ctx
            .git()
            .current_branch()
            .ok()
            .map(|branch| ticket::from_branch(pattern, &branch))
            .transpose()?
            .flatten(),
        _ => None,
    };

    match source {
        TicketSource::Disabled => Ok(None),
        TicketSource::Branch => match from_branch {
            Some(found) => Ok(Some(found)),
            None if policy.required => Err(missing()),
            None => Ok(None),
        },
        TicketSource::Prompt | TicketSource::BranchOrPrompt => {
            if let Some(found) = from_branch {
                return Ok(Some(found));
            }
            if non_interactive {
                return if policy.required {
                    Err(missing())
                } else {
                    Ok(None)
                };
            }
            loop {
                let help = match (policy.required, pattern) {
                    (true, Some(p)) => format!("Required; must match {p}"),
                    (true, None) => "Required".to_string(),
                    (false, Some(p)) => format!("Leave empty for none; must match {p}"),
                    (false, None) => "Leave empty for none".to_string(),
                };
                let value = ui.text("Ticket", None, Some(&help))?;
                let value = value.trim().to_string();
                if value.is_empty() {
                    if policy.required {
                        ui.logger().warn("A ticket is required");
                        continue;
                    }
                    return Ok(None);
                }
                if ticket::is_valid(pattern, &value)? {
                    return Ok(Some(value));
                }
                ui.logger()
                    .warn("That does not match commit.ticket.pattern");
            }
        }
    }
}

fn build_commit_message(
    header: &str,
    body: Option<&str>,
    breaking_message: Option<&str>,
    footers: &[String],
    breaking_footer_key: &str,
) -> String {
    let mut out = String::from(header);

    if let Some(body) = body {
        let trimmed = body.trim();
        if !trimmed.is_empty() {
            out.push_str("\n\n");
            out.push_str(trimmed);
        }
    }

    if let Some(message) = breaking_message {
        let trimmed = message.trim();
        if !trimmed.is_empty() {
            out.push_str("\n\n");
            out.push_str(breaking_footer_key);
            out.push_str(": ");
            out.push_str(trimmed);
        }
    }

    for footer in footers {
        let trimmed = footer.trim();
        if !trimmed.is_empty() {
            out.push('\n');
            out.push_str(trimmed);
        }
    }

    out
}
