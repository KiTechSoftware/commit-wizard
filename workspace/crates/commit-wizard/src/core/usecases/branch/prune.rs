//! `cw branch prune`: delete local branches whose remote branch no longer exists.
//!
//! The usual case is a branch that was merged and deleted on the remote. Only
//! branches already merged into the checked-out history are deleted; one with
//! commits found nowhere else (a squash-merged branch looks like this) needs
//! `--force`. The current branch and protected branches are never touched.

use std::time::Instant;

use scriba::Table;

use super::meta;
use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode, PromptTrait,
        capabilities::branch::{PruneVerdict, plan_prune},
    },
};

pub fn run(ctx: &Context, force: bool, no_fetch: bool) -> CoreResult<()> {
    let start = Instant::now();
    let git = ctx.git();
    let ui = ctx.ui();

    // A branch is "gone" only as far as the last fetch knows.
    if !no_fetch && let Err(err) = git.fetch_prune() {
        ui.logger().warn(&format!(
            "Could not update from the remote ({err}); using what is already known"
        ));
    }

    let branches = git.branch_infos()?;
    let plan = plan_prune(&branches, &ctx.policy().branch.protected_patterns);

    let doomed: Vec<_> = plan
        .candidates
        .iter()
        .filter(|(_, verdict)| force || *verdict == PruneVerdict::Delete)
        .map(|(branch, _)| branch.clone())
        .collect();
    let needs_force = plan
        .candidates
        .iter()
        .filter(|(_, verdict)| !force && *verdict == PruneVerdict::Unmerged)
        .count();

    let mut preview = Vec::new();
    for (branch, verdict) in &plan.candidates {
        let outcome = match (verdict, force) {
            (PruneVerdict::Delete, _) => "merged: will be deleted".to_string(),
            (PruneVerdict::Unmerged, true) => "not merged: deleted because of --force".to_string(),
            (PruneVerdict::Unmerged, false) => "not merged: kept, needs --force".to_string(),
        };
        preview.push(vec![branch.name.clone(), branch.hash.clone(), outcome]);
    }
    for (branch, why) in &plan.kept {
        preview.push(vec![
            branch.name.clone(),
            branch.hash.clone(),
            format!("kept: {why}"),
        ]);
    }
    let headers = ["branch", "tip", "outcome"].map(str::to_string).to_vec();

    if preview.is_empty() {
        let content = ui
            .new_output_content()
            .title("Nothing to Prune")
            .subtitle("No local branch follows a remote branch that is gone")
            .data("deleted", "0");
        return ui.print_with_meta(&content, Some(&meta(ctx, start, "branch.prune")), true);
    }

    if ctx.dry_run() || doomed.is_empty() {
        let content = ui
            .new_output_content()
            .title("Prune Preview")
            .subtitle(if ctx.dry_run() {
                "Dry run: nothing was deleted"
            } else {
                "Nothing can be deleted without --force"
            })
            .data("would_delete", doomed.len().to_string())
            .data("needs_force", needs_force.to_string())
            .table(None, Table::new(headers, preview));
        return ui.print_with_meta(&content, Some(&meta(ctx, start, "branch.prune")), true);
    }

    if !ctx.auto_yes() {
        if !ctx.is_interactive() {
            return Err(ErrorCode::InvalidInput.error().with_context(
                "reason",
                "pruning deletes branches; pass --yes to go ahead without asking, or --dry-run to preview",
            ));
        }
        let shown = ui
            .new_output_content()
            .title("Branches to Prune")
            .subtitle("Their remote branch is gone")
            .table(None, Table::new(headers.clone(), preview.clone()));
        ui.print_with_meta(&shown, Some(&meta(ctx, start, "branch.prune")), true)?;
        let question = format!(
            "Delete {} branch{}?",
            doomed.len(),
            if doomed.len() == 1 { "" } else { "es" }
        );
        if !ui.confirm(&question, false)? {
            ui.logger().warn("Nothing deleted");
            return Ok(());
        }
    }

    let mut results = Vec::new();
    let mut deleted = 0;
    for branch in &doomed {
        match git.delete_branch(&branch.name, force) {
            Ok(()) => {
                deleted += 1;
                results.push(vec![
                    branch.name.clone(),
                    branch.hash.clone(),
                    format!(
                        "deleted (restore: git branch {} {})",
                        branch.name, branch.hash
                    ),
                ]);
            }
            Err(err) => results.push(vec![
                branch.name.clone(),
                branch.hash.clone(),
                format!("not deleted: {err}"),
            ]),
        }
    }

    let content = ui
        .new_output_content()
        .title("Branches Pruned")
        .subtitle(format!("Deleted {deleted} of {}", doomed.len()))
        .data("deleted", deleted.to_string())
        .data("needs_force", needs_force.to_string())
        .table(None, Table::new(headers, results));
    ui.print_with_meta(&content, Some(&meta(ctx, start, "branch.prune")), true)
}
