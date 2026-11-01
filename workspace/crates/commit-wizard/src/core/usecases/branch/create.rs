//! `cw branch new`: create a branch named from a type, a ticket and a description,
//! laid out as `branch.naming.pattern` says (`{type}/{ticket}-{slug}` by default).

use std::time::Instant;

use scriba::{SelectOption, SelectRequest};

use super::meta;
use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode, PromptTrait,
        capabilities::{
            branch::{BRANCH_KINDS, NameTemplate, NameToken},
            commit::ticket,
        },
        models::policy::enforcement::TicketSource,
    },
};

/// What was asked of `cw branch new`, from flags.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// The description as a shorthand, or a full branch name when it contains a `/`.
    pub name: Option<String>,
    /// What the branch is for (`--description`); turned into words joined by hyphens.
    pub description: Option<String>,
    pub kind: Option<String>,
    pub ticket: Option<String>,
    pub from: Option<String>,
    pub no_switch: bool,
}

pub fn run(ctx: &Context, options: Options) -> CoreResult<()> {
    let start = Instant::now();
    let git = ctx.git();
    let ui = ctx.ui();
    let policy = &ctx.policy().commit.ticket;
    let guided = ctx.is_interactive() && !ctx.auto_yes();

    let literal = options
        .name
        .as_deref()
        .is_some_and(|name| name.contains('/'));
    if literal && (options.kind.is_some() || options.ticket.is_some()) {
        return Err(ErrorCode::InvalidInput.error().with_context(
            "reason",
            "NAME already contains a /, so it is used as the whole branch name; drop --type and --ticket or leave the / out",
        ));
    }

    let name = if literal {
        options.name.clone().unwrap_or_default().trim().to_string()
    } else {
        let template = NameTemplate::parse(&ctx.policy().branch.naming_pattern)?;
        let pattern_in_use = &ctx.policy().branch.naming_pattern;

        // A flag for a part the template does not have would be silently dropped.
        for (given, token, flag) in [
            (options.kind.is_some(), NameToken::Type, "--type"),
            (options.ticket.is_some(), NameToken::Ticket, "--ticket"),
        ] {
            if given && !template.has(token) {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("flag", flag)
                    .with_context(
                        "reason",
                        format!(
                            "branch.naming.pattern is {pattern_in_use}, which has no place for it"
                        ),
                    ));
            }
        }

        let branch = &ctx.policy().branch;

        // A part the config requires has to have a place in the template.
        for (required, token, key) in [
            (
                branch.require_type,
                NameToken::Type,
                "branch.naming.require_type",
            ),
            (
                branch.require_ticket,
                NameToken::Ticket,
                "branch.naming.require_ticket",
            ),
        ] {
            if required && !template.has(token) {
                return Err(ErrorCode::ConfigInvalid
                    .error()
                    .with_context("key", key)
                    .with_context(
                        "reason",
                        format!(
                            "branch.naming.pattern is {pattern_in_use}, which has no place for it"
                        ),
                    ));
            }
        }

        // type: optional unless the config requires it
        let kind = match options.kind.clone() {
            Some(kind) => Some(kind),
            None if guided && template.has(NameToken::Type) => {
                let mut choices: Vec<SelectOption> = BRANCH_KINDS
                    .iter()
                    .map(|kind| SelectOption::new(kind.to_string(), kind.to_string()))
                    .collect();
                if !branch.require_type {
                    choices.push(SelectOption::new(
                        "none".to_string(),
                        "no prefix".to_string(),
                    ));
                }
                let picked = ui.select(
                    &SelectRequest::new("What kind of branch is it?".to_string(), choices)
                        .with_page_size(10),
                )?;
                (picked != "none").then_some(picked)
            }
            None if branch.require_type => {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "type")
                    .with_context(
                        "reason",
                        "a type is required (branch.naming.require_type); pass --type",
                    ));
            }
            None => None,
        };

        // ticket: optional unless the config requires it
        let pattern = policy.regex.as_deref();
        let ask_for_ticket = template.has(NameToken::Ticket)
            && (branch.require_ticket
                || policy.required
                || policy.source != TicketSource::Disabled);
        let ticket = match options.ticket.clone() {
            Some(value) => {
                let value = value.trim().to_string();
                if !ticket::is_valid(pattern, &value)? {
                    return Err(ErrorCode::InvalidInput
                        .error()
                        .with_context("field", "ticket")
                        .with_context("value", value)
                        .with_context("reason", "does not match commit.ticket.pattern"));
                }
                Some(value)
            }
            None if ask_for_ticket && guided => loop {
                let help = if branch.require_ticket {
                    "Required"
                } else {
                    "Leave empty for none"
                };
                let value = ui.text("Ticket", None, Some(help))?;
                let value = value.trim().to_string();
                if value.is_empty() {
                    if branch.require_ticket {
                        ui.logger().warn("A ticket is required");
                        continue;
                    }
                    break None;
                }
                if ticket::is_valid(pattern, &value)? {
                    break Some(value);
                }
                ui.logger()
                    .warn("That does not match commit.ticket.pattern");
            },
            None if branch.require_ticket => {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "ticket")
                    .with_context(
                        "reason",
                        "a ticket is required (branch.naming.require_ticket); pass --ticket",
                    ));
            }
            None => None,
        };

        // description: always required
        let description = match options.description.clone().or(options.name.clone()) {
            Some(text) if !text.trim().is_empty() => text,
            Some(_) => {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "description")
                    .with_context("reason", "the description cannot be empty"));
            }
            None if guided => loop {
                let text = ui.text(
                    "What are you working on?",
                    None,
                    Some("A few words, such as 'add login page'"),
                )?;
                if !text.trim().is_empty() {
                    break text;
                }
                ui.logger().warn("A description is required");
            },
            None => {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "description")
                    .with_context("reason", "a branch needs a description; pass --description"));
            }
        };

        template.render(kind.as_deref(), ticket.as_deref(), &description)?
    };

    if name.is_empty() || !git.is_valid_branch_name(&name) {
        return Err(ErrorCode::InvalidInput
            .error()
            .with_context("field", "name")
            .with_context("value", name)
            .with_context("reason", "not a valid Git branch name"));
    }
    if git.branch_exists(&name) {
        return Err(ErrorCode::InvalidInput
            .error()
            .with_context("field", "name")
            .with_context("value", name)
            .with_context(
                "reason",
                "a branch with that name already exists; use cw branch switch",
            ));
    }

    let from = options.from.as_deref();
    if guided && !ui.confirm(&format!("Create branch {name}?"), true)? {
        ui.logger().warn("Branch not created");
        return Ok(());
    }

    if ctx.dry_run() {
        let content = ui
            .new_output_content()
            .title("Branch Preview")
            .subtitle(format!("Dry run: would create {name}"))
            .data("branch", name)
            .data("from", from.unwrap_or("HEAD"))
            .data("switch", (!options.no_switch).to_string());
        return ui.print_with_meta(&content, Some(&meta(ctx, start, "branch.new")), true);
    }

    if options.no_switch {
        git.create_branch(&name, from)?;
    } else {
        git.create_and_switch_branch(&name, from)?;
    }

    let content = ui
        .new_output_content()
        .title("Branch Created")
        .subtitle(if options.no_switch {
            format!("Created {name}")
        } else {
            format!("Created and switched to {name}")
        })
        .data("branch", name)
        .data("from", from.unwrap_or("HEAD"))
        .data("switched", (!options.no_switch).to_string());
    ui.print_with_meta(&content, Some(&meta(ctx, start, "branch.new")), true)
}
