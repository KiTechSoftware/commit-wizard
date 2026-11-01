//! `cw branch list`: local branches with what they follow and where they stand.

use std::time::Instant;

use scriba::Table;

use super::{date, meta, short};
use crate::{
    core::{Context, CoreResult},
    engine::{capabilities::branch::state_label, models::git::BranchInfo},
};

pub fn run(ctx: &Context, remote: bool, all: bool) -> CoreResult<()> {
    let start = Instant::now();
    let git = ctx.git();

    let mut rows = Vec::new();
    let mut count = 0;

    if !remote || all {
        for branch in git.branch_infos()? {
            rows.push(row(&branch, if branch.current { "*" } else { "" }));
            count += 1;
        }
    }
    if remote || all {
        for branch in git.remote_branch_infos()? {
            rows.push(row(&branch, ""));
            count += 1;
        }
    }

    let table = Table::new(
        ["", "branch", "upstream", "status", "updated", "last commit"]
            .map(str::to_string)
            .to_vec(),
        rows,
    );
    let content = ctx
        .ui()
        .new_output_content()
        .title("Branches")
        .subtitle(if remote {
            "Remote-tracking branches, as of the last fetch"
        } else if all {
            "Local and remote-tracking branches"
        } else {
            "Local branches, most recently committed to first"
        })
        .data("branches", count.to_string())
        .table(None, table);

    ctx.ui()
        .print_with_meta(&content, Some(&meta(ctx, start, "branch.list")), true)
}

fn row(branch: &BranchInfo, marker: &str) -> Vec<String> {
    vec![
        marker.to_string(),
        branch.name.clone(),
        branch.upstream.clone().unwrap_or_default(),
        match branch.upstream {
            Some(_) => state_label(&branch.state),
            None => String::new(),
        },
        date(branch.updated),
        short(&branch.subject, 50),
    ]
}
