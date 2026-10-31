//! `cw tag`: works out the next version (release candidates included) and creates, signs and pushes the tag, asking about the open choices on a terminal.

use std::time::Instant;

use crate::{
    core::{Context, CoreResult},
    engine::{
        PromptTrait,
        capabilities::versioning,
        constants::emoji::SUCCESS,
        error::ErrorCode,
        models::policy::{
            commit::CommitModel,
            versioning::{Version, VersioningModel},
        },
    },
};

/// Everything a tag is made from. Flags fill it in; in guided mode the
/// questions fill in whatever the flags left open.
struct Choices {
    set_version: Option<String>,
    prefix: Option<String>,
    suffix: Option<String>,
    sign: bool,
    push: bool,
    remote: Option<String>,
    message: Option<String>,
}

fn parse_version(text: &str, prefix: &str) -> CoreResult<Version> {
    Version::parse(text.trim(), prefix)
        .or_else(|| Version::parse(text.trim(), ""))
        .ok_or_else(|| {
            ErrorCode::InvalidInput
                .error()
                .with_context("invalid_version", text.trim())
        })
}

/// The version the commits since the last tag call for.
fn next_version(
    git: &crate::infra::git::Git,
    versioning_model: &VersioningModel,
    commit_model: &CommitModel,
) -> CoreResult<Version> {
    let last_tag = git
        .latest_tag()
        .map_err(|_| ErrorCode::GitCommandFailed.error())?;

    let current_version = last_tag
        .as_ref()
        .and_then(|tag| Version::parse(tag, &versioning_model.tag_prefix));

    let commits = git
        .list_commits(last_tag.as_deref(), "HEAD")
        .map_err(|_| ErrorCode::GitCommandFailed.error())?;

    if commits.is_empty() {
        // No new commits - use current version (a prerelease is promoted)
        current_version
            .map(|v| v.release())
            .or(Some(versioning_model.initial_version.clone()))
            .ok_or_else(|| {
                ErrorCode::StateInvalid
                    .error()
                    .with_context("reason", "No commits and no version computed")
            })
    } else {
        let classified = versioning::classify_commits(&commits, commit_model);
        Ok(versioning::calculate_next_version(
            current_version,
            &classified,
        ))
    }
}

/// Ask for the parts of the tag the flags did not decide: the version, its
/// prefix, a prerelease label, signing, and pushing.
fn ask_about_the_tag(
    ctx: &Context,
    versioning_model: &VersioningModel,
    commit_model: &CommitModel,
    choices: &mut Choices,
) -> CoreResult<()> {
    let ui = ctx.ui();
    let git = ctx.git();
    let default_prefix = versioning_model.tag_prefix.clone();

    if choices.set_version.is_none() {
        let proposed = next_version(&git, versioning_model, commit_model)?;
        let proposed_tag = proposed.format(&default_prefix);
        let taken = git
            .list_tags_sorted()
            .map(|tags| tags.contains(&proposed_tag))
            .unwrap_or(false);

        let accept = if taken {
            ui.logger().warn(&format!(
                "{proposed_tag} already exists, and there are no new commits that call for another version"
            ));
            false
        } else {
            ui.confirm(&format!("Use the next version, {proposed_tag}?"), true)?
        };

        if accept {
            choices.set_version = Some(proposed.to_semver());
        } else {
            let entered = ui.text("Version to tag", None, Some("For example 1.2.3 or v1.2.3"))?;
            if entered.trim().is_empty() {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("field", "version")
                    .with_context("reason", "a version is required"));
            }
            parse_version(&entered, &default_prefix)?;
            choices.set_version = Some(entered.trim().to_string());
        }
    }

    if choices.prefix.is_none() {
        choices.prefix = Some(ui.text(
            "Tag prefix",
            Some(&default_prefix),
            Some("Press Enter to keep the project default"),
        )?);
    }

    // A version that is already a prerelease needs no label.
    let already_prerelease = choices
        .set_version
        .as_deref()
        .and_then(|text| parse_version(text, &default_prefix).ok())
        .is_some_and(|version| version.is_prerelease());
    if choices.suffix.is_none() && !already_prerelease {
        let label = ui.text(
            "Prerelease label",
            None,
            Some("For example rc or beta; leave empty for a normal release"),
        )?;
        if !label.trim().is_empty() {
            choices.suffix = Some(label.trim().to_string());
        }
    }

    if !choices.sign {
        choices.sign = ui.confirm("Sign the tag with GPG?", false)?;
    }

    if !choices.push {
        choices.push = ui.confirm("Push the tag after creating it?", false)?;
    }
    if choices.push && choices.remote.is_none() {
        choices.remote = Some(ui.text("Remote to push to", Some("origin"), None)?);
    }

    Ok(())
}

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

    let mut choices = Choices {
        set_version,
        prefix,
        suffix,
        sign,
        push,
        remote,
        message,
    };

    // Guided: a person at a terminal is walked through what the flags left open.
    // Scripts, `--yes`, `--ci` and `--non-interactive` keep the flag-driven path.
    let guided = ctx.is_interactive() && !ctx.auto_yes();
    if guided {
        ask_about_the_tag(ctx, &versioning_model, &commit_model, &mut choices)?;
    }

    // Determine the version to tag
    let mut version = match choices.set_version.as_deref() {
        Some(explicit_version) => parse_version(explicit_version, &versioning_model.tag_prefix)?,
        None => next_version(&git, &versioning_model, &commit_model)?,
    };

    // A suffix turns the version into a prerelease (`rc` -> `-rc.N`)
    if let Some(ref label) = choices.suffix {
        let existing: Vec<Version> = git
            .list_tags_sorted()
            .map_err(|e| e.with_context("operation", "list tags"))?
            .iter()
            .filter_map(|tag| Version::parse(tag, &versioning_model.tag_prefix))
            .collect();
        version = version.with_prerelease(label, &existing);
    }

    // Apply prefix
    let tag_prefix = choices
        .prefix
        .clone()
        .unwrap_or_else(|| versioning_model.tag_prefix.clone());
    let formatted_version = version.to_semver();
    let tag_name = format!("{}{}", tag_prefix, formatted_version);

    if guided && choices.message.is_none() {
        let default_message = format!("Release {formatted_version}");
        choices.message = Some(ui.text("Tag message", Some(&default_message), None)?);
    }

    if guided {
        let mut question = format!("Create tag {tag_name}");
        if choices.sign {
            question.push_str(", signed");
        }
        if choices.push {
            question.push_str(&format!(
                ", and push it to {}",
                choices.remote.as_deref().unwrap_or("origin")
            ));
        }
        if !ui.confirm(&format!("{question}?"), true)? {
            ui.logger().warn("Tag cancelled");
            return Ok(());
        }
    } else if ctx.is_interactive() && !ctx.force() {
        ui.logger().info(&format!("Creating tag: {}", tag_name));
    }

    let Choices {
        suffix,
        sign,
        push,
        remote,
        message,
        prefix,
        ..
    } = choices;
    let prefix = prefix.filter(|p| *p != versioning_model.tag_prefix);

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
    if ctx.dry_run() {
        ui.logger()
            .ok(&format!("{} Would create tag: {}", SUCCESS, tag_name));
    } else {
        ui.logger()
            .ok(&format!("{} Tag created: {}", SUCCESS, tag_name));
    }

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
