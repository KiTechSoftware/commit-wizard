use std::{fs, path::PathBuf, time::Instant};

use crate::{
    core::{Context, CoreResult},
    engine::{
        capabilities::changelog::{
            BuildOptions, Release, ReleaseSpec, RenderedBlock, apply_links, build_release,
            format_date, links::Links, merge_markdown, render_header, render_json,
            render_markdown_release,
        },
        constants::emoji::SUCCESS,
        error::ErrorCode,
        models::{
            git::CommitSummary,
            policy::{
                changelog::{ChangelogModel, ChangelogOutputFormat},
                commit::CommitModel,
                versioning::{Version, VersioningModel},
            },
        },
    },
};

/// What the user asked for on the command line.
#[derive(Debug, Clone)]
pub struct Options {
    pub all: bool,
    pub from: Option<String>,
    pub to: String,
    pub output: Option<String>,
    pub format: Option<ChangelogOutputFormat>,
    pub group_by_scope: bool,
    pub next_tag: Option<String>,
    pub include_non_cc: bool,
    pub prereleases: bool,
}

/// Where a release's commits come from, before they are read from git.
struct Range {
    label: String,
    date: Option<String>,
    from: Option<String>,
    to: String,
    /// What a compare link should end at: the tag, the version about to be
    /// tagged, or the revision itself.
    link_to: String,
}

pub fn run(ctx: &Context, options: Options) -> CoreResult<()> {
    let ui = ctx.ui();
    let git = ctx.git();
    let start = Instant::now();

    let resolved_config = ctx.config().ok_or_else(|| {
        ErrorCode::ConfigUnreadable
            .error()
            .with_context("context", "Config not resolved")
    })?;
    let commit_model = CommitModel::from_config(resolved_config);
    let changelog = ChangelogModel::from_config(resolved_config);
    let versioning = VersioningModel::from_config(resolved_config);

    if options.all && options.from.is_some() {
        return Err(ErrorCode::InvalidInput
            .error()
            .with_context("reason", "--all and --from cannot be combined"));
    }

    let format = options.format.unwrap_or(changelog.format);
    let target = resolve_output(&options, &changelog, format, ctx.repo_root());

    // --- decide which releases to produce ---------------------------------
    let today = format_date(
        chrono::Utc::now().timestamp(),
        changelog.date_format.as_deref(),
    );
    let next_label = options
        .next_tag
        .as_deref()
        .map(|tag| normalize_label(tag, &versioning.tag_prefix));

    let tags = release_tags(
        git.tags_reachable_from(&options.to)?,
        &versioning.tag_prefix,
        options.prereleases,
    );

    let mut ranges: Vec<Range> = Vec::new();
    if let Some(from) = &options.from {
        ranges.push(Range {
            label: next_label
                .clone()
                .unwrap_or_else(|| format!("{from}..{}", options.to)),
            date: today.clone(),
            from: Some(from.clone()),
            to: options.to.clone(),
            link_to: next_label.clone().unwrap_or_else(|| options.to.clone()),
        });
    } else {
        // Changes since the newest tag, which have not been released yet.
        let latest = tags.first().map(|(tag, _)| tag.clone());
        let pending = git.list_commits(latest.as_deref(), &options.to)?;
        if !pending.is_empty() || next_label.is_some() {
            ranges.push(Range {
                label: next_label
                    .clone()
                    .unwrap_or_else(|| changelog.unreleased_label.clone()),
                date: next_label.as_ref().and(today.clone()),
                from: latest,
                to: options.to.clone(),
                link_to: next_label.clone().unwrap_or_else(|| options.to.clone()),
            });
        }

        if options.all {
            for (index, (tag, _)) in tags.iter().enumerate() {
                let timestamp = git.tag_timestamp(tag)?;
                ranges.push(Range {
                    label: tag.clone(),
                    date: timestamp
                        .and_then(|seconds| format_date(seconds, changelog.date_format.as_deref())),
                    from: tags.get(index + 1).map(|(older, _)| older.clone()),
                    to: tag.clone(),
                    link_to: tag.clone(),
                });
            }
        }
    }

    // --- read commits and build releases ------------------------------------
    let build_options = BuildOptions {
        include_non_cc: options.include_non_cc,
    };
    let links = repo_links(ctx, &changelog);
    let mut releases: Vec<Release> = Vec::new();
    for range in ranges {
        let commits: Vec<CommitSummary> = git
            .list_commits(range.from.as_deref(), &range.to)
            .map_err(|err| {
                err.with_context(
                    "hint",
                    format!(
                        "could not read commits in {}..{}; check that both revisions exist",
                        range.from.as_deref().unwrap_or("<start>"),
                        range.to
                    ),
                )
            })?;
        let mut release = build_release(
            ReleaseSpec {
                label: range.label,
                date: range.date,
                from: range.from,
                to: range.to,
            },
            &commits,
            &commit_model,
            &changelog,
            build_options,
        );
        if let Some(links) = &links {
            apply_links(&mut release, links, &range.link_to);
        }
        releases.push(release);
    }

    // --- render ---------------------------------------------------------------
    let entries: usize = releases.iter().map(Release::entry_count).sum();
    let rendered = match format {
        ChangelogOutputFormat::Json => render_json(&releases),
        ChangelogOutputFormat::Markdown => {
            let blocks: Vec<RenderedBlock> = releases
                .iter()
                .map(|release| RenderedBlock {
                    label: release.label.clone(),
                    text: render_markdown_release(release, &changelog, options.group_by_scope),
                })
                .collect();
            let existing = match &target {
                Target::File(path) if !ctx.force() && path.exists() => {
                    Some(fs::read_to_string(path)?)
                }
                _ => None,
            };
            merge_markdown(
                existing.as_deref(),
                &render_header(&changelog),
                &blocks,
                &changelog.unreleased_label,
            )
        }
    };

    // --- write ----------------------------------------------------------------
    let nothing_to_do = releases.is_empty();
    let mut written = false;
    match &target {
        Target::Stdout => {
            // Only the changelog, so the output can be piped or redirected.
            print!("{rendered}");
            return Ok(());
        }
        Target::File(path) if !ctx.dry_run() && !nothing_to_do => {
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, &rendered)?;
            written = true;
        }
        Target::File(_) => {}
    }

    // --- report ---------------------------------------------------------------
    if nothing_to_do {
        ui.logger().info("No unreleased changes found");
    } else if written {
        ui.logger().ok(&format!(
            "{SUCCESS} Changelog updated: {}",
            target.display()
        ));
    }

    let meta = ui
        .new_output_meta()
        .with_command("changelog".to_string())
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_dry_run(ctx.dry_run());

    let mut content = ui
        .new_output_content()
        .title("Changelog")
        .subtitle("Release notes generated from Conventional Commits")
        .data("output", target.display())
        .data("format", format_name(format))
        .data("releases", releases.len().to_string())
        .data("entries", entries.to_string())
        .data("written", written.to_string());

    if !releases.is_empty() {
        let labels = releases
            .iter()
            .map(|release| release.label.clone())
            .collect::<Vec<_>>()
            .join(", ");
        content = content.data("labels", labels);
    }

    // A dry run writes nothing, so show what would have been written.
    if ctx.dry_run() && !nothing_to_do {
        content = content.section("Preview", rendered, "markdown".to_string());
    }

    ui.print_with_meta(&content, Some(&meta), true)
}

/// Where commits and compare views live on the web, if that is known: the
/// configured address, else the one the `origin` remote implies.
fn repo_links(ctx: &Context, changelog: &ChangelogModel) -> Option<Links> {
    if !changelog.links_enabled {
        return None;
    }
    match &changelog.links_url {
        Some(url) => Some(Links::new(url)),
        None => ctx
            .git()
            .remote_url("origin")
            .and_then(|remote| Links::from_remote(&remote)),
    }
}

enum Target {
    Stdout,
    File(PathBuf),
}

impl Target {
    fn display(&self) -> String {
        match self {
            Target::Stdout => "-".to_string(),
            Target::File(path) => path.display().to_string(),
        }
    }
}

fn resolve_output(
    options: &Options,
    changelog: &ChangelogModel,
    format: ChangelogOutputFormat,
    repo_root: &std::path::Path,
) -> Target {
    if options.output.as_deref() == Some("-") {
        return Target::Stdout;
    }

    let mut path = PathBuf::from(options.output.as_deref().unwrap_or(&changelog.output));
    // The configured default is a Markdown file; JSON should not overwrite it.
    if options.output.is_none()
        && format == ChangelogOutputFormat::Json
        && path.extension().is_some_and(|ext| ext == "md")
    {
        path.set_extension("json");
    }

    Target::File(if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    })
}

fn format_name(format: ChangelogOutputFormat) -> &'static str {
    match format {
        ChangelogOutputFormat::Markdown => "markdown",
        ChangelogOutputFormat::Json => "json",
    }
}

/// `1.2.0` and `v1.2.0` should name the same release.
fn normalize_label(tag: &str, prefix: &str) -> String {
    match Version::parse(tag, prefix) {
        Some(version) => version.format(prefix),
        None => tag.to_string(),
    }
}

/// Version tags, newest first. Prereleases roll into the next stable release
/// unless asked for.
fn release_tags(tags: Vec<String>, prefix: &str, prereleases: bool) -> Vec<(String, Version)> {
    let mut versions: Vec<(String, Version)> = tags
        .into_iter()
        .filter_map(|tag| Version::parse(&tag, prefix).map(|version| (tag, version)))
        .filter(|(_, version)| prereleases || !version.is_prerelease())
        .collect();
    versions.sort_by(|a, b| b.1.cmp(&a.1));
    versions
}
