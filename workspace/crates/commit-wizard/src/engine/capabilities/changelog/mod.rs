//! Changelog generation: grouping commits into releases and rendering them.
//!
//! Everything here is pure (no git, no filesystem) so it can be tested directly.
//! The use case in `core::usecases::changelog` supplies the commits and writes
//! the result.

use std::collections::BTreeMap;

pub mod links;

use links::Links;

use serde::Serialize;

use crate::engine::{
    capabilities::commit::check::ParsedHeader,
    models::{
        git::CommitSummary,
        policy::{changelog::ChangelogModel, commit::CommitModel},
    },
};

/// One commit as it appears in the changelog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub hash: String,
    #[serde(rename = "type")]
    pub type_name: String,
    pub scope: Option<String>,
    pub subject: String,
    pub breaking: bool,
    pub breaking_note: Option<String>,
    /// Full commit id, for links and machine consumers.
    #[serde(rename = "commit")]
    pub full_hash: String,
    /// Web page for the commit, when links are on.
    pub url: Option<String>,
}

/// Entries that share a heading, such as "Features".
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Section {
    pub title: String,
    pub entries: Vec<Entry>,
}

/// One block of the changelog: a tagged release, or the unreleased changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Release {
    pub label: String,
    pub date: Option<String>,
    pub from: Option<String>,
    pub to: String,
    pub breaking: Vec<Entry>,
    pub sections: Vec<Section>,
    /// Compare (or tag) page for the release, when links are on.
    pub url: Option<String>,
}

impl Release {
    pub fn entry_count(&self) -> usize {
        self.sections.iter().map(|s| s.entries.len()).sum()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BuildOptions {
    /// Keep commits that are not Conventional Commits, under the misc section.
    pub include_non_cc: bool,
}

/// Where a release's commits came from.
#[derive(Debug, Clone)]
pub struct ReleaseSpec {
    pub label: String,
    pub date: Option<String>,
    pub from: Option<String>,
    pub to: String,
}

/// Group `commits` (newest first) into a release.
pub fn build_release(
    spec: ReleaseSpec,
    commits: &[CommitSummary],
    commit_model: &CommitModel,
    changelog: &ChangelogModel,
    options: BuildOptions,
) -> Release {
    let misc_title = changelog
        .misc_section
        .clone()
        .unwrap_or_else(|| "Miscellaneous".to_string());

    // title -> (rank, entries)
    let mut grouped: BTreeMap<String, (i64, Vec<Entry>)> = BTreeMap::new();
    let mut breaking = Vec::new();

    for commit in commits {
        let summary = commit.summary.trim();
        let short = commit.hash.chars().take(7).collect::<String>();

        let (entry, title, rank) = match ParsedHeader::parse(summary) {
            Some(parsed) => {
                let note = breaking_note(commit, &commit_model.breaking_footer_key);
                let is_breaking = parsed.is_breaking || note.is_some();
                let (title, rank) = match section_for(&parsed.type_name, commit_model, changelog) {
                    Some(found) => found,
                    None => (misc_title.clone(), RANK_MISC),
                };
                let entry = Entry {
                    hash: short,
                    type_name: parsed.type_name,
                    scope: parsed.scope,
                    subject: parsed.subject,
                    breaking: is_breaking,
                    breaking_note: note,
                    full_hash: commit.hash.clone(),
                    url: None,
                };
                (entry, title, rank)
            }
            None if options.include_non_cc && !summary.is_empty() && !is_merge(summary) => {
                let entry = Entry {
                    hash: short,
                    type_name: "other".to_string(),
                    scope: None,
                    subject: summary.to_string(),
                    breaking: false,
                    breaking_note: None,
                    full_hash: commit.hash.clone(),
                    url: None,
                };
                (entry, misc_title.clone(), RANK_MISC)
            }
            None => continue,
        };

        if entry.breaking {
            breaking.push(entry.clone());
        }
        let slot = grouped.entry(title).or_insert((rank, Vec::new()));
        slot.0 = slot.0.min(rank);
        slot.1.push(entry);
    }

    // `show_empty_sections`: list every configured section, even with no commits.
    if changelog.show_empty_sections == Some(true) {
        let configured = changelog
            .section_order
            .iter()
            .chain(changelog.sections.keys())
            .map(String::as_str);
        for type_name in configured {
            if let Some((title, rank)) = section_for(type_name, commit_model, changelog) {
                let slot = grouped.entry(title).or_insert((rank, Vec::new()));
                slot.0 = slot.0.min(rank);
            }
        }
    }

    let mut sections: Vec<(i64, String, Vec<Entry>)> = grouped
        .into_iter()
        .map(|(title, (rank, entries))| (rank, title, entries))
        .collect();
    sections.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    Release {
        label: spec.label,
        date: spec.date,
        from: spec.from,
        to: spec.to,
        breaking,
        sections: sections
            .into_iter()
            .map(|(_, title, entries)| Section { title, entries })
            .collect(),
        url: None,
    }
}

/// Point a release and its entries at their pages in the repository.
/// `link_to` is the revision the release ends at, as it should appear in a
/// compare URL (a tag name, or `HEAD` for unreleased changes).
pub fn apply_links(release: &mut Release, links: &Links, link_to: &str) {
    release.url = Some(links.release(release.from.as_deref(), link_to));
    for entry in release.breaking.iter_mut().chain(
        release
            .sections
            .iter_mut()
            .flat_map(|section| section.entries.iter_mut()),
    ) {
        entry.url = Some(links.commit(&entry.full_hash));
    }
}

/// Sorts after every configured section.
const RANK_UNLISTED: i64 = 1_000;
/// The miscellaneous section always comes last.
const RANK_MISC: i64 = 10_000;

/// Heading and sort rank for a commit type, or `None` if the policy does not know it.
fn section_for(
    type_name: &str,
    commit_model: &CommitModel,
    changelog: &ChangelogModel,
) -> Option<(String, i64)> {
    let configured = changelog.sections.get(type_name);
    let type_model = commit_model.find_type(type_name);

    let title = configured
        .and_then(|section| section.title.clone())
        .or_else(|| type_model.map(|t| t.section.clone()))?;

    let rank = configured
        .and_then(|section| section.order)
        .map(i64::from)
        .or_else(|| {
            changelog
                .section_order
                .iter()
                .position(|key| key == type_name)
                .map(|index| index as i64)
        })
        .unwrap_or(RANK_UNLISTED);

    Some((title, rank))
}

fn is_merge(summary: &str) -> bool {
    summary.starts_with("Merge ")
}

/// The text after `BREAKING CHANGE:` in the commit footer, if present.
fn breaking_note(commit: &CommitSummary, footer_key: &str) -> Option<String> {
    let message = commit.full_message.as_deref()?;
    let prefix = format!("{footer_key}:");
    message
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix))
        .map(|note| note.trim().to_string())
        .filter(|note| !note.is_empty())
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Render one release as a Markdown block ending in a blank line.
pub fn render_markdown_release(
    release: &Release,
    model: &ChangelogModel,
    group_by_scope: bool,
) -> String {
    let group_by_scope = group_by_scope || model.group_by.iter().any(|g| g == "scope");

    let label = match &release.url {
        Some(url) => format!("[{}]({url})", release.label),
        None => release.label.clone(),
    };
    let mut out = match &release.date {
        Some(date) => format!("## {label} - {date}\n\n"),
        None => format!("## {label}\n\n"),
    };

    if release.entry_count() == 0 && release.sections.is_empty() {
        out.push_str("_No notable changes._\n\n");
        return out;
    }

    if !release.breaking.is_empty() {
        out.push_str("### Breaking Changes\n\n");
        for entry in &release.breaking {
            out.push_str(&bullet(entry, model.show_scope, true));
        }
        out.push('\n');
    }

    for section in &release.sections {
        out.push_str(&format!("### {}\n\n", section.title));
        if section.entries.is_empty() {
            // An empty section is one line; it does not repeat every configured scope.
            out.push_str(EMPTY_SECTION);
        } else if group_by_scope {
            out.push_str(&render_scoped(&section.entries, model));
        } else {
            for entry in &section.entries {
                out.push_str(&bullet(entry, model.show_scope, false));
            }
        }
        out.push('\n');
    }

    out
}

const EMPTY_SECTION: &str = "_No changes._\n";

fn bullet(entry: &Entry, show_scope: bool, with_note: bool) -> String {
    let scope = match (&entry.scope, show_scope) {
        (Some(scope), true) => format!("**{scope}:** "),
        _ => String::new(),
    };
    let note = match (&entry.breaking_note, with_note) {
        (Some(note), true) => format!(" — {note}"),
        _ => String::new(),
    };
    let hash = match &entry.url {
        Some(url) => format!("[{}]({url})", entry.hash),
        None => entry.hash.clone(),
    };
    format!("- {scope}{} ({hash}){note}\n", entry.subject)
}

/// Unscoped entries first, then one `####` group per scope.
fn render_scoped(entries: &[Entry], model: &ChangelogModel) -> String {
    let mut out = String::new();

    for entry in entries.iter().filter(|e| e.scope.is_none()) {
        out.push_str(&bullet(entry, false, false));
    }

    let mut scopes: Vec<&str> = entries
        .iter()
        .filter_map(|entry| entry.scope.as_deref())
        .collect();
    if model.show_empty_scopes == Some(true) {
        scopes.extend(model.scope_order.iter().map(String::as_str));
    }
    scopes.sort_unstable();
    scopes.dedup();
    scopes.sort_by_key(|scope| {
        model
            .scope_order
            .iter()
            .position(|ordered| ordered == scope)
            .unwrap_or(usize::MAX)
    });

    for scope in scopes {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&format!("#### {scope}\n\n"));
        let mut any = false;
        for entry in entries.iter().filter(|e| e.scope.as_deref() == Some(scope)) {
            any = true;
            out.push_str(&bullet(entry, false, false));
        }
        if !any {
            out.push_str(EMPTY_SECTION);
        }
    }

    out
}

/// The file header (`# Changelog` plus an optional description).
pub fn render_header(model: &ChangelogModel) -> String {
    if !model.header.use_header {
        return String::new();
    }
    let mut out = format!("# {}\n\n", model.header.title);
    if let Some(description) = model
        .header
        .description
        .as_deref()
        .filter(|d| !d.trim().is_empty())
    {
        out.push_str(description.trim());
        out.push_str("\n\n");
    }
    out
}

/// Render releases as a JSON document.
pub fn render_json(releases: &[Release]) -> String {
    #[derive(Serialize)]
    struct Document<'a> {
        releases: &'a [Release],
    }
    let mut text =
        serde_json::to_string_pretty(&Document { releases }).unwrap_or_else(|_| "{}".to_string());
    text.push('\n');
    text
}

// ---------------------------------------------------------------------------
// Updating an existing file
// ---------------------------------------------------------------------------

/// A generated release, ready to be placed in the file.
#[derive(Debug, Clone)]
pub struct RenderedBlock {
    pub label: String,
    pub text: String,
}

/// Combine generated blocks with an existing Markdown changelog.
///
/// - Blocks are matched by label: a regenerated release replaces its old block
///   in place of being duplicated, so running the command twice is safe.
/// - Existing blocks that were not regenerated are kept, below the new ones.
/// - A stale unreleased block is dropped once a real version is generated.
/// - Text above the first release (the file header) is preserved.
pub fn merge_markdown(
    existing: Option<&str>,
    header: &str,
    generated: &[RenderedBlock],
    unreleased_label: &str,
) -> String {
    let (preamble, old_blocks) = match existing {
        Some(text) if !text.trim().is_empty() => split_blocks(text),
        _ => (String::new(), Vec::new()),
    };

    let mut out = if preamble.trim().is_empty() {
        header.to_string()
    } else {
        let mut kept = preamble.trim_end().to_string();
        kept.push_str("\n\n");
        kept
    };

    for block in generated {
        out.push_str(&normalize_block(&block.text));
    }

    let regenerated_a_version = generated
        .iter()
        .any(|block| block.label != unreleased_label);
    for (label, text) in old_blocks {
        let replaced = generated.iter().any(|block| block.label == label);
        let stale_unreleased = regenerated_a_version && label == unreleased_label;
        if !replaced && !stale_unreleased {
            out.push_str(&normalize_block(&text));
        }
    }

    out
}

/// Render a file from scratch.
pub fn compose_markdown(header: &str, generated: &[RenderedBlock]) -> String {
    merge_markdown(None, header, generated, "")
}

fn normalize_block(text: &str) -> String {
    format!("{}\n\n", text.trim_end())
}

/// Split a changelog into the text before the first `## ` heading and its release blocks.
fn split_blocks(text: &str) -> (String, Vec<(String, String)>) {
    let mut preamble = String::new();
    let mut blocks: Vec<(String, String)> = Vec::new();

    for line in text.lines() {
        if line.starts_with("## ") {
            blocks.push((block_label(line), String::new()));
        }
        match blocks.last_mut() {
            Some((_, body)) => {
                body.push_str(line);
                body.push('\n');
            }
            None => {
                preamble.push_str(line);
                preamble.push('\n');
            }
        }
    }

    (preamble, blocks)
}

/// `## [1.2.0] - 2026-01-01` and `## 1.2.0` both have the label `1.2.0`.
fn block_label(heading: &str) -> String {
    let title = heading.trim_start_matches('#').trim();
    // `[v1.2.0](https://...) - date`: the label is the link text.
    if let Some(rest) = title.strip_prefix('[')
        && let Some((text, _)) = rest.split_once(']')
    {
        return text.trim().to_string();
    }
    let label = title.split(" - ").next().unwrap_or(title).trim();
    label.trim_matches(|c| c == '[' || c == ']').to_string()
}

// ---------------------------------------------------------------------------
// Dates
// ---------------------------------------------------------------------------

/// Turn a configured date format into a `strftime` pattern.
///
/// Config files use Go-style reference layouts (`2006-01-02`), as in the docs;
/// patterns that already contain `%` are used as they are.
pub fn strftime_pattern(configured: Option<&str>) -> String {
    let Some(format) = configured.map(str::trim).filter(|f| !f.is_empty()) else {
        return "%Y-%m-%d".to_string();
    };
    if format.contains('%') {
        return format.to_string();
    }

    let mut out = format.to_string();
    for (go, strftime) in [
        ("2006", "%Y"),
        ("January", "%B"),
        ("Jan", "%b"),
        ("Monday", "%A"),
        ("Mon", "%a"),
        ("01", "%m"),
        ("02", "%d"),
        ("15", "%H"),
        ("04", "%M"),
        ("05", "%S"),
    ] {
        out = out.replace(go, strftime);
    }
    out
}

/// Format Unix seconds with the configured date format.
pub fn format_date(seconds: i64, configured: Option<&str>) -> Option<String> {
    let moment = chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, 0)?;
    Some(moment.format(&strftime_pattern(configured)).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::models::policy::changelog::ChangelogSectionModel;

    fn commit(summary: &str) -> CommitSummary {
        CommitSummary {
            hash: "0123456789abcdef".to_string(),
            summary: summary.to_string(),
            full_message: Some(summary.to_string()),
        }
    }

    fn commit_with_footer(summary: &str, footer: &str) -> CommitSummary {
        CommitSummary {
            hash: "fedcba9876543210".to_string(),
            summary: summary.to_string(),
            full_message: Some(format!("{summary}\n\nbody\n\n{footer}")),
        }
    }

    fn spec(label: &str) -> ReleaseSpec {
        ReleaseSpec {
            label: label.to_string(),
            date: Some("2026-01-02".to_string()),
            from: None,
            to: "HEAD".to_string(),
        }
    }

    fn build(commits: &[CommitSummary], options: BuildOptions) -> Release {
        build_release(
            spec("v1.0.0"),
            commits,
            &CommitModel::default(),
            &ChangelogModel::default(),
            options,
        )
    }

    fn titles(release: &Release) -> Vec<&str> {
        release.sections.iter().map(|s| s.title.as_str()).collect()
    }

    #[test]
    fn groups_commits_by_type_section_in_configured_order() {
        let release = build(
            &[
                commit("fix: handle empty input"),
                commit("docs: explain flags"),
                commit("feat: add parser"),
            ],
            BuildOptions::default(),
        );

        // section_order puts feat before fix before docs.
        assert_eq!(release.entry_count(), 3);
        let order: Vec<&str> = release
            .sections
            .iter()
            .map(|s| s.entries[0].type_name.as_str())
            .collect();
        assert_eq!(order, ["feat", "fix", "docs"]);
    }

    #[test]
    fn non_conventional_commits_are_skipped_unless_requested() {
        let commits = [commit("feat: keep me"), commit("random message")];

        assert_eq!(build(&commits, BuildOptions::default()).entry_count(), 1);

        let included = build(
            &commits,
            BuildOptions {
                include_non_cc: true,
            },
        );
        assert_eq!(included.entry_count(), 2);
        assert_eq!(titles(&included).last(), Some(&"Miscellaneous"));
    }

    #[test]
    fn merge_commits_are_never_listed() {
        let release = build(
            &[commit("Merge branch 'x' into y")],
            BuildOptions {
                include_non_cc: true,
            },
        );
        assert_eq!(release.entry_count(), 0);
    }

    #[test]
    fn unknown_types_land_in_the_misc_section_instead_of_vanishing() {
        let release = build(&[commit("wibble: odd type")], BuildOptions::default());
        assert_eq!(titles(&release), ["Miscellaneous"]);
    }

    #[test]
    fn breaking_changes_come_from_the_bang_and_the_footer() {
        let release = build(
            &[
                commit("feat!: drop the old api"),
                commit_with_footer("fix: tweak", "BREAKING CHANGE: config moved"),
                commit("feat: harmless"),
            ],
            BuildOptions::default(),
        );

        assert_eq!(release.breaking.len(), 2);
        assert_eq!(
            release.breaking[1].breaking_note.as_deref(),
            Some("config moved")
        );

        let text = render_markdown_release(&release, &ChangelogModel::default(), false);
        assert!(text.contains("### Breaking Changes"));
        assert!(text.contains("— config moved"));
    }

    #[test]
    fn config_titles_and_order_override_the_commit_type_section() {
        let mut changelog = ChangelogModel::default();
        changelog.sections.insert(
            "fix".to_string(),
            ChangelogSectionModel {
                title: Some("Squashed Bugs".to_string()),
                order: Some(-5),
            },
        );

        let release = build_release(
            spec("v1.0.0"),
            &[commit("feat: a"), commit("fix: b")],
            &CommitModel::default(),
            &changelog,
            BuildOptions::default(),
        );

        // order -5 sorts the fix section before feat, under its configured title.
        assert_eq!(release.sections[0].title, "Squashed Bugs");
    }

    #[test]
    fn markdown_has_heading_date_scope_and_short_hash() {
        let release = build(&[commit("feat(core): add parser")], BuildOptions::default());
        let text = render_markdown_release(&release, &ChangelogModel::default(), false);

        assert!(text.starts_with("## v1.0.0 - 2026-01-02\n\n"));
        assert!(text.contains("- **core:** add parser (0123456)"));
    }

    #[test]
    fn grouping_by_scope_adds_scope_headings() {
        let release = build(
            &[
                commit("feat(core): a"),
                commit("feat(cli): b"),
                commit("feat: c"),
            ],
            BuildOptions::default(),
        );
        let text = render_markdown_release(&release, &ChangelogModel::default(), true);

        assert!(text.contains("#### cli"));
        assert!(text.contains("#### core"));
        // Unscoped entries come before the scope groups.
        assert!(text.find("- c (").unwrap() < text.find("#### cli").unwrap());
    }

    #[test]
    fn empty_release_says_so() {
        let release = build(&[], BuildOptions::default());
        let text = render_markdown_release(&release, &ChangelogModel::default(), false);
        assert!(text.contains("_No notable changes._"));
    }

    #[test]
    fn json_lists_releases_with_typed_entries() {
        let release = build(&[commit("feat(core): add parser")], BuildOptions::default());
        let json: serde_json::Value = serde_json::from_str(&render_json(&[release])).unwrap();

        let entry = &json["releases"][0]["sections"][0]["entries"][0];
        assert_eq!(json["releases"][0]["label"], "v1.0.0");
        assert_eq!(entry["type"], "feat");
        assert_eq!(entry["scope"], "core");
    }

    fn block(label: &str, line: &str) -> RenderedBlock {
        RenderedBlock {
            label: label.to_string(),
            text: format!("## {label}\n\n- {line}\n"),
        }
    }

    #[test]
    fn new_releases_go_above_existing_ones_and_keep_the_header() {
        let existing = "# Changelog\n\nNotes.\n\n## v1.0.0 - 2025-01-01\n\n- old (abc)\n";
        let merged = merge_markdown(
            Some(existing),
            "# ignored\n\n",
            &[block("v1.1.0", "new")],
            "Unreleased",
        );

        assert!(merged.starts_with("# Changelog\n\nNotes.\n\n## v1.1.0"));
        assert!(merged.find("v1.1.0").unwrap() < merged.find("v1.0.0").unwrap());
        assert!(!merged.contains("ignored"));
    }

    #[test]
    fn regenerating_a_release_replaces_it_instead_of_duplicating() {
        let existing = "# Changelog\n\n## v1.0.0 - 2025-01-01\n\n- stale (abc)\n";
        let merged = merge_markdown(
            Some(existing),
            "",
            &[block("v1.0.0", "fresh")],
            "Unreleased",
        );

        assert_eq!(merged.matches("## v1.0.0").count(), 1);
        assert!(merged.contains("fresh") && !merged.contains("stale"));
    }

    #[test]
    fn running_twice_gives_the_same_file() {
        let blocks = [block("Unreleased", "x")];
        let once = merge_markdown(None, "# Changelog\n\n", &blocks, "Unreleased");
        let twice = merge_markdown(Some(&once), "# Changelog\n\n", &blocks, "Unreleased");
        assert_eq!(once, twice);
    }

    #[test]
    fn a_stale_unreleased_block_is_dropped_when_a_version_is_cut() {
        let existing = "# Changelog\n\n## Unreleased\n\n- pending (abc)\n";
        let merged = merge_markdown(
            Some(existing),
            "",
            &[block("v2.0.0", "released")],
            "Unreleased",
        );

        assert!(merged.contains("## v2.0.0"));
        assert!(!merged.contains("## Unreleased"));
    }

    #[test]
    fn bracketed_labels_in_existing_files_still_match() {
        let existing = "## [v1.0.0] - 2025-01-01\n\n- old (abc)\n";
        let merged = merge_markdown(
            Some(existing),
            "",
            &[block("v1.0.0", "fresh")],
            "Unreleased",
        );
        assert_eq!(merged.matches("v1.0.0").count(), 1);
    }

    #[test]
    fn go_style_date_layouts_become_strftime() {
        assert_eq!(strftime_pattern(None), "%Y-%m-%d");
        assert_eq!(strftime_pattern(Some("2006-01-02")), "%Y-%m-%d");
        assert_eq!(strftime_pattern(Some("02 Jan 2006")), "%d %b %Y");
        assert_eq!(strftime_pattern(Some("%d/%m/%Y")), "%d/%m/%Y");
        assert_eq!(
            format_date(1_767_312_000, Some("2006-01-02")).as_deref(),
            Some("2026-01-02")
        );
    }

    #[test]
    fn show_empty_sections_lists_configured_sections_without_commits() {
        let changelog = ChangelogModel {
            show_empty_sections: Some(true),
            ..ChangelogModel::default()
        };

        let release = build_release(
            spec("v1.0.0"),
            &[commit("feat: only this")],
            &CommitModel::default(),
            &changelog,
            BuildOptions::default(),
        );
        let text = render_markdown_release(&release, &changelog, false);

        // Every configured type's section appears; the empty ones say so.
        assert!(text.contains("### Features") && text.contains("only this"));
        assert!(text.contains("### Bug Fixes\n\n_No changes._"), "{text}");
        assert!(release.sections.len() > 1);
    }

    #[test]
    fn empty_sections_stay_hidden_by_default() {
        let release = build(&[commit("feat: only this")], BuildOptions::default());
        let text = render_markdown_release(&release, &ChangelogModel::default(), false);
        assert!(!text.contains("_No changes._"), "{text}");
        assert_eq!(release.sections.len(), 1);
    }

    #[test]
    fn show_empty_scopes_adds_configured_scopes_that_have_no_entries() {
        let changelog = ChangelogModel {
            show_empty_scopes: Some(true),
            scope_order: vec!["api".to_string(), "ui".to_string()],
            ..ChangelogModel::default()
        };

        let release = build_release(
            spec("v1.0.0"),
            &[commit("feat(ui): button")],
            &CommitModel::default(),
            &changelog,
            BuildOptions::default(),
        );
        let text = render_markdown_release(&release, &changelog, true);

        assert!(text.contains("#### api\n\n_No changes._"), "{text}");
        assert!(text.contains("#### ui\n\n- button"), "{text}");
        // api is configured first, so it is listed before ui.
        assert!(text.find("#### api").unwrap() < text.find("#### ui").unwrap());
    }

    #[test]
    fn scope_headings_are_not_invented_without_show_empty_scopes() {
        let changelog = ChangelogModel {
            scope_order: vec!["api".to_string(), "ui".to_string()],
            ..ChangelogModel::default()
        };

        let release = build_release(
            spec("v1.0.0"),
            &[commit("feat(ui): button")],
            &CommitModel::default(),
            &changelog,
            BuildOptions::default(),
        );
        let text = render_markdown_release(&release, &changelog, true);
        assert!(!text.contains("#### api"), "{text}");
    }

    fn linked_release() -> Release {
        let mut release = build_release(
            ReleaseSpec {
                label: "v1.1.0".to_string(),
                date: Some("2026-01-02".to_string()),
                from: Some("v1.0.0".to_string()),
                to: "v1.1.0".to_string(),
            },
            &[
                commit("feat(core): add parser"),
                commit_with_footer("fix: tweak", "BREAKING CHANGE: moved"),
            ],
            &CommitModel::default(),
            &ChangelogModel::default(),
            BuildOptions::default(),
        );
        apply_links(
            &mut release,
            &Links::new("https://github.com/acme/app"),
            "v1.1.0",
        );
        release
    }

    #[test]
    fn links_point_the_heading_at_the_compare_view_and_hashes_at_commits() {
        let text = render_markdown_release(&linked_release(), &ChangelogModel::default(), false);

        assert!(
            text.starts_with(
                "## [v1.1.0](https://github.com/acme/app/compare/v1.0.0...v1.1.0) - 2026-01-02"
            ),
            "{text}"
        );
        assert!(
            text.contains("([0123456](https://github.com/acme/app/commit/0123456789abcdef))"),
            "{text}"
        );
        // Breaking entries link too.
        assert!(
            text.contains("(https://github.com/acme/app/commit/fedcba9876543210)"),
            "{text}"
        );
    }

    #[test]
    fn the_first_release_links_to_its_tag_page() {
        let mut release = build(&[commit("feat: a")], BuildOptions::default());
        apply_links(
            &mut release,
            &Links::new("https://github.com/acme/app"),
            "v1.0.0",
        );
        assert_eq!(
            release.url.as_deref(),
            Some("https://github.com/acme/app/tree/v1.0.0")
        );
    }

    #[test]
    fn json_carries_the_urls_and_full_commit_ids() {
        let json: serde_json::Value =
            serde_json::from_str(&render_json(&[linked_release()])).unwrap();
        let release = &json["releases"][0];

        assert_eq!(
            release["url"],
            "https://github.com/acme/app/compare/v1.0.0...v1.1.0"
        );
        let entry = &release["sections"][0]["entries"][0];
        assert_eq!(entry["commit"].as_str().unwrap().len(), 16);
        assert!(entry["url"].as_str().unwrap().contains("/commit/"));
    }

    #[test]
    fn a_linked_heading_is_matched_by_its_label() {
        let linked =
            "# Changelog\n\n## [v1.0.0](https://x/compare/a...b) - 2025-01-01\n\n- old (abc)\n";
        let merged = merge_markdown(Some(linked), "", &[block("v1.0.0", "fresh")], "Unreleased");

        assert_eq!(merged.matches("## ").count(), 1, "{merged}");
        assert!(
            merged.contains("fresh") && !merged.contains("old"),
            "{merged}"
        );
    }

    #[test]
    fn without_links_nothing_changes_in_the_output() {
        let release = build(&[commit("feat: a")], BuildOptions::default());
        let text = render_markdown_release(&release, &ChangelogModel::default(), false);
        assert!(!text.contains("]("), "{text}");
        assert_eq!(release.url, None);
    }
}
