use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::versioning,
        constants::emoji::SUCCESS,
        error::ErrorCode,
        models::policy::{
            commit::CommitModel,
            versioning::{Version, VersioningModel},
        },
    },
};

#[allow(clippy::too_many_arguments)]
pub fn run(
    ctx: &Context,
    set_version: Option<String>,
    prefix: Option<String>,
    suffix: Option<String>,
    sign: bool,
    push: bool,
    branch: Option<String>,
    remote: Option<String>,
    message: Option<String>,
) -> CoreResult<()> {
    let ui = ctx.ui();
    let git = ctx.git();
    let start = Instant::now();

    // Get configuration
    let resolved_config = ctx
        .config()
        .ok_or_else(|| ErrorCode::ConfigUnreadable.error())?;

    let versioning_model = VersioningModel::from_config(resolved_config);
    let commit_model = CommitModel::from_config(resolved_config);

    // Determine the version to tag
    let version = if let Some(explicit_version) = set_version {
        // User provided explicit version (may or may not have prefix)
        Version::parse(&explicit_version, &versioning_model.tag_prefix)
            .or_else(|| Version::parse(&explicit_version, ""))
            .ok_or_else(|| {
                ErrorCode::InvalidInput
                    .error()
                    .with_context("invalid_version", explicit_version)
            })?
    } else {
        // Compute version from commits
        let last_tag = git
            .latest_tag()
            .map_err(|_| ErrorCode::GitCommandFailed.error())?;

        let current_version = last_tag
            .as_ref()
            .and_then(|tag| Version::parse(tag, &versioning_model.tag_prefix));

        // Get commits since last tag
        let commits = git
            .list_commits(last_tag.as_deref(), "HEAD")
            .map_err(|_| ErrorCode::GitCommandFailed.error())?;

        if commits.is_empty() {
            // No new commits - use current version
            current_version
                .or(Some(versioning_model.initial_version.clone()))
                .ok_or_else(|| {
                    ErrorCode::StateInvalid
                        .error()
                        .with_context("reason", "No commits and no version computed")
                })?
        } else {
            // Classify and compute next version
            let classified = versioning::classify_commits(&commits, &commit_model);
            versioning::calculate_next_version(current_version.clone(), &classified)
        }
    };

    // Apply prefix and suffix
    let tag_prefix = prefix
        .clone()
        .unwrap_or_else(|| versioning_model.tag_prefix.clone());
    let formatted_version = version.to_semver();
    let tag_name = format!(
        "{}{}{}",
        tag_prefix,
        formatted_version,
        suffix.clone().unwrap_or_default()
    );

    // In interactive mode, ask for confirmation
    if ctx.is_interactive() && !ctx.force() {
        ui.logger().info(&format!("Creating tag: {}", tag_name));
        // TODO: Add interactive confirmation when UI prompt trait is available
    }

    // Prepare tag message
    let tag_message = message.unwrap_or_else(|| format!("Release {}", formatted_version));

    // Create tag (dry-run support)
    if !ctx.dry_run() {
        if sign {
            git.create_signed_tag(&tag_name, &tag_message)
                .map_err(|e| e.with_context("operation", "create signed tag"))?;
        } else {
            git.create_tag(&tag_name, &tag_message)
                .map_err(|e| e.with_context("operation", "create tag"))?;
        }

        // Push tag if requested
        if push {
            let remote_name = remote.clone().unwrap_or_else(|| "origin".to_string());
            git.push_tag(&remote_name, &tag_name)
                .map_err(|e| e.with_context("operation", "push tag"))?;
        }
    }

    // Log success
    ui.logger()
        .ok(&format!("{} Tag created: {}", SUCCESS, tag_name));

    // Build output
    let duration_ms = start.elapsed().as_millis() as u64;
    let meta = ui
        .new_output_meta()
        .with_command("tag".to_string())
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Create Release Tag")
        .subtitle("Tagging current repository with release version")
        .data("tag", tag_name.clone())
        .data("version", formatted_version)
        .data("message", tag_message.clone());

    if let Some(ref p) = prefix {
        content = content.data("prefix", p.clone());
    }
    if let Some(ref s) = suffix {
        content = content.data("suffix", s.clone());
    }
    if sign {
        content = content.data("signed", "true");
    }
    if push {
        content = content.data("pushed", "true");
        let remote_to_display = remote.clone().unwrap_or_else(|| "origin".to_string());
        content = content.data("remote", remote_to_display);
    }
    if let Some(ref b) = branch {
        content = content.data("branch", b.clone());
    }

    ui.print_with_meta(&content, Some(&meta), true)
}
