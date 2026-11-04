//! `cw release`: opens a release branch from the source branch, and finishes it by merging into the target branch, tagging, pushing and merging back, as the `[release]` settings describe.

use std::time::Instant;

use crate::{
    core::{
        Context, CoreResult,
        usecases::version::tag::{next_version, parse_version},
    },
    engine::{
        PromptTrait,
        constants::emoji::SUCCESS,
        error::ErrorCode,
        models::policy::{
            commit::CommitModel,
            release::ReleaseModel,
            versioning::{Version, VersioningModel},
        },
    },
};

struct Setup {
    release: ReleaseModel,
    versioning: VersioningModel,
    commit: CommitModel,
}

fn setup(ctx: &Context) -> CoreResult<Setup> {
    let config = ctx
        .config()
        .ok_or_else(|| ErrorCode::ConfigUnreadable.error())?;
    let release = ReleaseModel::from_config(config);
    if !release.enabled {
        return Err(ErrorCode::ReleaseInvalid.error().with_context(
            "reason",
            "releases are off; set `enabled = true` under [release]",
        ));
    }
    Ok(Setup {
        release,
        versioning: VersioningModel::from_config(config),
        commit: CommitModel::from_config(config),
    })
}

fn require_clean(ctx: &Context, setup: &Setup) -> CoreResult<()> {
    if setup.release.validation.require_clean_worktree && !ctx.git().list_changes()?.is_empty() {
        return Err(ErrorCode::ReleaseInvalid
            .error()
            .with_context("reason", "the working tree has uncommitted changes"));
    }
    Ok(())
}

fn branch_name(setup: &Setup, version: &Version) -> String {
    setup
        .release
        .branch_format
        .replace("{version}", &version.to_semver())
}

/// The version a branch name encodes, per `branch_format`.
fn version_from_branch(setup: &Setup, branch: &str) -> Option<Version> {
    let (head, tail) = setup.release.branch_format.split_once("{version}")?;
    let text = branch.strip_prefix(head)?.strip_suffix(tail)?;
    parse_version(text, &setup.versioning.tag_prefix).ok()
}

fn report(
    ctx: &Context,
    command: &str,
    start: Instant,
    data: Vec<(&str, String)>,
) -> CoreResult<()> {
    let ui = ctx.ui();
    let meta = ui
        .new_output_meta()
        .with_command(command.to_string())
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_dry_run(ctx.dry_run());
    let mut content = ui.new_output_content().title("Release");
    for (key, value) in data {
        content = content.data(key, value);
    }
    ui.print_with_meta(&content, Some(&meta), true)
}

pub fn start(ctx: &Context, set_version: Option<String>) -> CoreResult<()> {
    let start = Instant::now();
    let setup = setup(ctx)?;
    let git = ctx.git();
    require_clean(ctx, &setup)?;

    let source = setup.release.source_branch.clone();
    let branches = git.list_branches()?;
    if !branches.contains(&source) {
        return Err(ErrorCode::ReleaseInvalid.error().with_context(
            "reason",
            format!("the source branch `{source}` does not exist"),
        ));
    }

    let version = match set_version.as_deref() {
        Some(text) => parse_version(text, &setup.versioning.tag_prefix)?,
        None => next_version(&git, &setup.versioning, &setup.commit, &source)?,
    };
    let tag = version.format(&setup.versioning.tag_prefix);
    let branch = branch_name(&setup, &version);

    if setup.release.validation.fail_if_tag_exists && git.list_tags_sorted()?.contains(&tag) {
        return Err(ErrorCode::ReleaseTagExists.error().with_context("tag", tag));
    }
    if setup.release.validation.fail_if_release_branch_exists && branches.contains(&branch) {
        return Err(ErrorCode::ReleaseBranchExists
            .error()
            .with_context("branch", branch));
    }

    if ctx.dry_run() {
        ctx.ui()
            .logger()
            .ok(&format!("{SUCCESS} Would create {branch} from {source}"));
    } else {
        git.create_branch(&branch, Some(&source))?;
        git.checkout_branch(&branch)?;
        ctx.ui().logger().ok(&format!(
            "{SUCCESS} Release branch {branch} created from {source}"
        ));
    }
    report(
        ctx,
        "release start",
        start,
        vec![
            ("version", version.to_semver()),
            ("branch", branch),
            ("source", source),
        ],
    )
}

pub fn finish(ctx: &Context, set_version: Option<String>, remote: String) -> CoreResult<()> {
    let start = Instant::now();
    let setup = setup(ctx)?;
    let git = ctx.git();
    require_clean(ctx, &setup)?;

    let version = match set_version.as_deref() {
        Some(text) => parse_version(text, &setup.versioning.tag_prefix)?,
        None => {
            let current = git.current_branch()?;
            version_from_branch(&setup, &current).ok_or_else(|| {
                ErrorCode::ReleaseInvalid.error().with_context(
                    "reason",
                    format!(
                        "`{current}` is not a release branch ({}); switch to it or pass --set-version",
                        setup.release.branch_format
                    ),
                )
            })?
        }
    };
    let branch = branch_name(&setup, &version);
    let tag = version.format(&setup.versioning.tag_prefix);
    let target = setup.release.target_branch.clone();
    let backmerge = setup.release.finish.backmerge_branch.clone();
    let do_backmerge = backmerge != target;

    let branches = git.list_branches()?;
    for needed in [&branch, &target] {
        if !branches.contains(needed) {
            return Err(ErrorCode::ReleaseInvalid
                .error()
                .with_context("reason", format!("the branch `{needed}` does not exist")));
        }
    }
    if do_backmerge && !branches.contains(&backmerge) {
        return Err(ErrorCode::ReleaseInvalid
            .error()
            .with_context("reason", format!("the branch `{backmerge}` does not exist")));
    }
    if setup.release.finish.tag
        && setup.release.validation.fail_if_tag_exists
        && git.list_tags_sorted()?.contains(&tag)
    {
        return Err(ErrorCode::ReleaseTagExists.error().with_context("tag", tag));
    }

    let mut plan = vec![format!("merge {branch} into {target}")];
    if setup.release.finish.tag {
        plan.push(format!("tag {tag}"));
    }
    if do_backmerge {
        plan.push(format!("merge {branch} into {backmerge}"));
    }
    if setup.release.finish.push {
        plan.push(format!("push to {remote}"));
    }

    if ctx.dry_run() {
        ctx.ui()
            .logger()
            .ok(&format!("{SUCCESS} Would: {}", plan.join(", ")));
    } else {
        if ctx.is_interactive()
            && !ctx.auto_yes()
            && !ctx.ui().confirm(
                &format!(
                    "Finish release {}: {}?",
                    version.to_semver(),
                    plan.join(", ")
                ),
                true,
            )?
        {
            ctx.ui().logger().warn("Release cancelled");
            return Ok(());
        }

        git.checkout_branch(&target)?;
        git.merge(&branch)?;
        if setup.release.finish.tag {
            git.create_tag_at(&tag, &format!("Release {}", version.to_semver()), "HEAD")?;
        }
        if do_backmerge {
            git.checkout_branch(&backmerge)?;
            git.merge(&branch)?;
        }
        if setup.release.finish.push {
            git.push_branch(&remote, &target)?;
            if do_backmerge {
                git.push_branch(&remote, &backmerge)?;
            }
            if setup.release.finish.tag {
                git.push_tag(&remote, &tag)?;
            }
        }
        ctx.ui().logger().ok(&format!(
            "{SUCCESS} Release {} finished",
            version.to_semver()
        ));
    }

    report(
        ctx,
        "release finish",
        start,
        vec![
            ("version", version.to_semver()),
            ("branch", branch),
            ("target", target),
            (
                "tag",
                if setup.release.finish.tag {
                    tag
                } else {
                    String::new()
                },
            ),
            ("pushed", setup.release.finish.push.to_string()),
        ],
    )
}
