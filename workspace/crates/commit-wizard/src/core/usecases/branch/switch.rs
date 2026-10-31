//! `cw branch switch`: check out a branch, picking from a list on a terminal.

use std::time::Instant;

use scriba::{SelectOption, SelectRequest};

use super::{meta, short};
use crate::{
    core::{Context, CoreResult},
    engine::{ErrorCode, PromptTrait, capabilities::branch::state_label},
};

pub fn run(ctx: &Context, name: Option<String>) -> CoreResult<()> {
    let start = Instant::now();
    let git = ctx.git();
    let ui = ctx.ui();

    let name = match name {
        Some(name) => name,
        None if !ctx.is_interactive() => {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("field", "name")
                .with_context(
                    "reason",
                    "no branch given and no terminal to choose one; pass cw branch switch NAME",
                ));
        }
        None => {
            let local = git.branch_infos()?;
            let mut options: Vec<SelectOption> = local
                .iter()
                .map(|branch| {
                    let label = if branch.current {
                        format!("{} (current)", branch.name)
                    } else {
                        branch.name.clone()
                    };
                    let mut detail = short(&branch.subject, 50);
                    if branch.upstream.is_some() {
                        detail = format!("{} · {}", state_label(&branch.state), detail);
                    }
                    SelectOption::new(branch.name.clone(), label).description(detail)
                })
                .collect();

            // Branches that exist only on a remote can be switched to as well: Git
            // creates a local branch that follows them.
            for remote in git.remote_branch_infos()? {
                let short_name = remote.name.split_once('/').map(|(_, rest)| rest);
                if let Some(short_name) = short_name
                    && !local.iter().any(|branch| branch.name == short_name)
                {
                    options.push(
                        SelectOption::new(
                            short_name.to_string(),
                            format!(
                                "{short_name} (on {})",
                                remote.name.split('/').next().unwrap_or("remote")
                            ),
                        )
                        .description(short(&remote.subject, 50)),
                    );
                }
            }

            if options.is_empty() {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("reason", "this repository has no branches yet"));
            }
            ui.select(
                &SelectRequest::new("Switch to which branch?".to_string(), options)
                    .with_page_size(12),
            )?
        }
    };

    if ctx.dry_run() {
        let content = ui
            .new_output_content()
            .title("Switch Preview")
            .subtitle(format!("Dry run: would switch to {name}"))
            .data("branch", name);
        return ui.print_with_meta(&content, Some(&meta(ctx, start, "branch.switch")), true);
    }

    git.switch_branch(&name)?;

    let content = ui
        .new_output_content()
        .title("Switched Branch")
        .subtitle(format!("Now on {name}"))
        .data("branch", name);
    ui.print_with_meta(&content, Some(&meta(ctx, start, "branch.switch")), true)
}
