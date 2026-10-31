//! `cw branch new`: create a branch named from a type, a ticket and a description.

use std::time::Instant;

use scriba::{SelectOption, SelectRequest};

use super::meta;
use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode, PromptTrait,
        capabilities::{
            branch::{BRANCH_KINDS, branch_name},
            commit::ticket,
        },
        models::policy::enforcement::TicketSource,
    },
};

/// What was asked of `cw branch new`, from flags.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// A short description (turned into words joined by hyphens), or a full branch
    /// name when it contains a `/`.
    pub name: Option<String>,
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
        // type
        let kind = match options.kind.clone() {
            Some(kind) => Some(kind),
            None if guided => {
                let mut choices: Vec<SelectOption> = BRANCH_KINDS
                    .iter()
                    .map(|kind| SelectOption::new(kind.to_string(), kind.to_string()))
                    .collect();
                choices.push(SelectOption::new(
                    "none".to_string(),
                    "no prefix".to_string(),
                ));
                let picked = ui.select(
                    &SelectRequest::new("What kind of branch is it?".to_string(), choices)
                        .with_page_size(10),
                )?;
                (picked != "none").then_some(picked)
            }
            None => None,
        };

        // ticket
        let ticket_matters =
            policy.required || policy.source != TicketSource::Disabled || options.ticket.is_some();
        let pattern = policy.regex.as_deref();
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
            None if ticket_matters && guided => loop {
                let help = if policy.required {
                    "Required"
                } else {
                    "Leave empty for none"
                };
                let value = ui.text("Ticket", None, Some(help))?;
                let value = value.trim().to_string();
                if value.is_empty() {
                    if policy.required {
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
            None if policy.required => {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "ticket")
                    .with_context(
                        "reason",
                        "a ticket is required (commit.ticket.required); pass --ticket",
                    ));
            }
            None => None,
        };

        // description
        let description = match options.name.clone() {
            Some(name) => name,
            None if guided => {
                let help = if ticket.is_some() {
                    "Leave empty to name the branch after the ticket"
                } else {
                    "A few words, such as 'add login page'"
                };
                ui.text("What are you working on?", None, Some(help))?
            }
            None => String::new(),
        };

        branch_name(kind.as_deref(), ticket.as_deref(), &description)?
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
