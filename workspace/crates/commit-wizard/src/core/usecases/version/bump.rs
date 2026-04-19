use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::versioning,
        constants::emoji::{ARROW, SUCCESS},
        error::ErrorCode,
        models::policy::{
            commit::CommitModel,
            enforcement::BumpLevel,
            versioning::{Version, VersioningModel},
        },
    },
};
use std::time::Instant;

pub fn run(ctx: &Context, from: Option<String>, to: String, tail: Option<u32>) -> CoreResult<()> {
    let ui = ctx.ui();
    let start = Instant::now();

    // Get config and build policy
    let resolved_config = ctx.config().ok_or_else(|| {
        ErrorCode::ConfigUnreadable
            .error()
            .with_context("context", "Config not resolved")
    })?;

    let versioning_model = VersioningModel::from_config(resolved_config);
    let commit_model = CommitModel::from_config(resolved_config);

    // Get git adapter
    let git = ctx.git();

    // Fetch last tag (this is our current version)
    let last_tag = git.latest_tag().map_err(|_| {
        ErrorCode::GitCommandFailed
            .error()
            .with_context("operation", "get latest tag")
    })?;

    // Parse current version from tag
    let current_version = last_tag
        .as_ref()
        .and_then(|tag| Version::parse(tag, &versioning_model.tag_prefix));

    // Determine commit range
    let from_ref = from.clone().or_else(|| last_tag.clone());

    // Fetch commits
    let commits = git.list_commits(from_ref.as_deref(), &to).map_err(|_| {
        ErrorCode::GitCommandFailed
            .error()
            .with_context("operation", "list commits")
    })?;

    // Apply tail filter if specified
    let commits = if let Some(n) = tail {
        if n == 0 {
            commits
        } else {
            let n = n as usize;
            if commits.len() > n {
                commits[commits.len() - n..].to_vec()
            } else {
                commits
            }
        }
    } else {
        commits
    };

    // Classify commits by bump level
    let classified = versioning::classify_commits(&commits, &commit_model);

    // Calculate next version
    let next_version = versioning::calculate_next_version(current_version.clone(), &classified);

    // Log summary with colored output
    let current_str = current_version
        .as_ref()
        .map(|v| v.to_semver())
        .unwrap_or_else(|| "initial".to_string());
    let next_str = next_version.to_semver();
    ui.logger().ok(&format!(
        "{} Version bump: {} {} {}",
        SUCCESS, current_str, ARROW, next_str
    ));

    // Build output metadata
    let duration_ms = start.elapsed().as_millis() as u64;
    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("bump".to_string())
        .with_dry_run(ctx.dry_run());

    // Build output content
    let mut content = ui
        .new_output_content()
        .title("Version Bump Analysis")
        .subtitle("Semantic version calculation from commits")
        .data("commits_analyzed", commits.len().to_string())
        .data("next_version", next_version.to_semver())
        .data(
            "next_tag",
            next_version.format(&versioning_model.tag_prefix),
        );

    if let Some(tag) = &last_tag {
        content = content.data(
            "current_tag",
            current_version
                .as_ref()
                .map(|v| v.format(&versioning_model.tag_prefix))
                .unwrap_or_else(|| tag.clone()),
        );
        if let Some(version) = current_version.as_ref() {
            content = content.data("current_version", version.to_semver());
        }
    } else {
        content = content.data("current_version", "initial");
    }

    if let Some(from) = &from {
        content = content.data("from", from.clone());
    }
    content = content.data("to", to.clone());
    if let Some(tail) = tail {
        content = content.data("tail", tail.to_string());
    }

    // Build commit classifications by bump level
    let mut major_commits: Vec<String> = Vec::new();
    let mut minor_commits: Vec<String> = Vec::new();
    let mut patch_commits: Vec<String> = Vec::new();
    let mut none_commits: Vec<String> = Vec::new();

    for (commit, bump) in &classified {
        let line = format!(
            "{} {}",
            &commit.hash[..8.min(commit.hash.len())],
            &commit.summary
        );
        match bump {
            BumpLevel::Major => major_commits.push(line),
            BumpLevel::Minor => minor_commits.push(line),
            BumpLevel::Patch => patch_commits.push(line),
            BumpLevel::None => none_commits.push(line),
        }
    }

    if !major_commits.is_empty() {
        content = content.section(
            "Major Changes (Breaking)",
            major_commits.join("\n"),
            "sh".to_string(),
        );
    }
    if !minor_commits.is_empty() {
        content = content.section(
            "Minor Changes (Features)",
            minor_commits.join("\n"),
            "sh".to_string(),
        );
    }
    if !patch_commits.is_empty() {
        content = content.section(
            "Patch Changes (Fixes)",
            patch_commits.join("\n"),
            "sh".to_string(),
        );
    }
    if !none_commits.is_empty() {
        content = content.section(
            "Unclassified Commits",
            none_commits.join("\n"),
            "sh".to_string(),
        );
    }

    // Plain output: next version for machine consumption
    let plain = next_version.to_semver();
    content = content.plain(plain);

    let success = !classified.is_empty() || current_version.is_some();
    ui.print_with_meta(&content, Some(&meta), success)
}
