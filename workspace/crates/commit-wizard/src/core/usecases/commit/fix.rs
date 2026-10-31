//! `cw fix`: correct commit messages that break the project's rules.
//!
//! Only messages change. Each commit keeps its tree, author and dates, so the
//! files are untouched; the commits from the first fixed one up to HEAD get new
//! hashes, and the branch is moved to them. The old tip is saved under
//! `refs/cw/fix-backup/` first, so the change can be undone.

use std::collections::HashMap;
use std::time::Instant;

use scriba::{SelectOption, SelectRequest, Table};

use crate::{
    core::{Context, CoreResult},
    engine::{
        PromptTrait,
        capabilities::commit::{
            check::{CommitValidator, ValidatedCommit, validate_commits},
            fix::{Reword, reworded_message, suggested_subject, suggested_type, with_header},
        },
        error::ErrorCode,
        models::{
            git::CommitSummary,
            policy::commit::{CommitModel, ScopeRequirement},
        },
    },
};

/// What was asked for on the command line.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Check from here (exclusive) up to HEAD instead of the unpublished commits.
    pub from: Option<String>,
    /// Only look at the newest N commits (0 or unset = all).
    pub tail: Option<u32>,
    /// `HASH=HEADER` pairs giving the corrected first line of a commit.
    pub rewords: Vec<String>,
}

fn short(hash: &str) -> &str {
    &hash[..8.min(hash.len())]
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
    let policy = CommitModel::from_config(resolved_config);

    if git.operation_in_progress()? {
        return Err(ErrorCode::HistoryRewriteUnsupported.error().with_context(
            "reason",
            "a merge, rebase or cherry-pick is in progress; finish or abort it first",
        ));
    }

    // --- which commits to look at -------------------------------------------
    let mut commits = match &options.from {
        Some(from) => git.list_commits(Some(from), "HEAD")?,
        None => git.unpublished_commits("HEAD")?,
    };
    if let Some(tail) = options.tail.filter(|n| *n > 0) {
        commits.truncate(tail as usize);
    }
    let total = commits.len();
    let by_hash: HashMap<String, CommitSummary> = commits
        .iter()
        .map(|c| (c.hash.clone(), c.clone()))
        .collect();

    // Oldest first, the order a person would fix them in.
    let report = validate_commits(commits, &policy);
    let mut invalid: Vec<ValidatedCommit> =
        report.commits.into_iter().filter(|c| !c.valid).collect();
    invalid.reverse();

    if invalid.is_empty() {
        ui.logger().ok(&format!(
            "All {total} commit(s) follow the rules; nothing to fix"
        ));
        let meta = meta(ctx, start, "fix");
        let content = ui
            .new_output_content()
            .title("Nothing to fix")
            .data("checked", total.to_string())
            .data("invalid", "0");
        return ui.print_with_meta(&content, Some(&meta), true);
    }

    ui.logger().warn(&format!(
        "{} of {total} commit(s) break the rules",
        invalid.len()
    ));
    for commit in &invalid {
        ui.logger()
            .info(&format!("  {}  {}", short(&commit.hash), commit.summary));
        for violation in &commit.violations {
            ui.logger()
                .info(&format!("      - {}", violation.message()));
        }
    }

    // --- published commits are not ours to rewrite ------------------------
    let invalid_hashes: Vec<String> = invalid.iter().map(|c| c.hash.clone()).collect();
    let published = git.published_among(&invalid_hashes)?;
    if !published.is_empty() && !ctx.force() {
        return Err(ErrorCode::HistoryPublished
            .error()
            .with_context(
                "commits",
                published.iter().map(|h| short(h)).collect::<Vec<_>>().join(", "),
            )
            .with_context(
                "reason",
                "these commits are already on a remote; rewriting them breaks everyone who pulled them",
            )
            .with_context("hint", "pass --force if you are sure, then push with --force-with-lease"));
    }

    // --- decide the new messages --------------------------------------------
    let validator = CommitValidator::new(&policy);
    let mut decisions: HashMap<String, String> = HashMap::new();

    for spec in &options.rewords {
        let (prefix, header) = spec.split_once('=').ok_or_else(|| {
            ErrorCode::InvalidInput
                .error()
                .with_context("value", spec.as_str())
                .with_context(
                    "reason",
                    "expected HASH=HEADER, e.g. 1a2b3c4d=\"fix: handle empty input\"",
                )
        })?;
        let prefix = prefix.trim();
        let matches: Vec<&ValidatedCommit> = invalid
            .iter()
            .filter(|c| prefix.len() >= 4 && c.hash.starts_with(prefix))
            .collect();
        let target = match matches.as_slice() {
            [one] => *one,
            [] => {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("commit", prefix)
                    .with_context(
                        "reason",
                        "no commit that breaks the rules starts with that hash",
                    ));
            }
            _ => {
                return Err(ErrorCode::InvalidInput
                    .error()
                    .with_context("commit", prefix)
                    .with_context("reason", "that hash is ambiguous; give more characters"));
            }
        };
        let original = full_message(&by_hash, &target.hash);
        let message = with_header(&original, header);
        let problems = validator.validate_message(&message);
        if !problems.is_empty() {
            return Err(ErrorCode::InvalidInput
                .error()
                .with_context("commit", short(&target.hash))
                .with_context("message", header.trim())
                .with_context(
                    "reason",
                    problems
                        .iter()
                        .map(|p| p.message())
                        .collect::<Vec<_>>()
                        .join("; "),
                ));
        }
        decisions.insert(target.hash.clone(), message);
    }

    let guided = ctx.is_interactive() && !ctx.auto_yes();
    if guided && decisions.len() < invalid.len() {
        for commit in &invalid {
            if decisions.contains_key(&commit.hash) {
                continue;
            }
            match ask_about(
                ctx,
                &policy,
                &validator,
                commit,
                &full_message(&by_hash, &commit.hash),
            )? {
                Answer::Reword(message) => {
                    decisions.insert(commit.hash.clone(), message);
                }
                Answer::Skip => {}
                Answer::Stop => break,
            }
        }
    } else if decisions.is_empty() && !ctx.dry_run() {
        return Err(ErrorCode::UserInteractionRequired
            .error()
            .with_context("command", "fix")
            .with_context(
                "hint",
                "run on a terminal to be guided, or pass --reword HASH=HEADER for each commit (--dry-run only lists them)",
            ));
    }

    if decisions.is_empty() {
        ui.logger().info("Nothing was rewritten");
        return Ok(());
    }

    // --- what would happen ----------------------------------------------------
    let keys: Vec<String> = decisions.keys().cloned().collect();
    let span = git.rewrite_span(&keys)?;
    if !span.signed.is_empty() && !ctx.force() {
        return Err(ErrorCode::HistoryRewriteUnsupported
            .error()
            .with_context(
                "commits",
                span.signed
                    .iter()
                    .map(|h| short(h))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
            .with_context("reason", "these signed commits would lose their signature")
            .with_context("hint", "pass --force to rewrite them anyway"));
    }

    let mut ordered: Vec<&ValidatedCommit> = invalid
        .iter()
        .filter(|c| decisions.contains_key(&c.hash))
        .collect();
    ordered.sort_by_key(|c| span.chain.iter().position(|h| *h == c.hash));
    let plan = Table::new(
        vec!["commit".into(), "before".into(), "after".into()],
        ordered
            .iter()
            .map(|c| {
                let after = decisions[&c.hash].lines().next().unwrap_or("").to_string();
                vec![short(&c.hash).to_string(), c.summary.clone(), after]
            })
            .collect(),
    );

    let rehashed = span.chain.len();
    let note = format!(
        "{} message(s) will change; {rehashed} commit(s) get new hashes (files are untouched)",
        decisions.len()
    );

    if ctx.dry_run() {
        let content = ui
            .new_output_content()
            .title("Fix preview")
            .subtitle(note)
            .data("rewritten", "false")
            .table(Some("Changes".to_string()), plan);
        return ui.print_with_meta(&content, Some(&meta(ctx, start, "fix")), true);
    }

    if guided {
        for row in ordered.iter() {
            let after = decisions[&row.hash].lines().next().unwrap_or("");
            ui.logger().info(&format!(
                "  {}  {}  ->  {after}",
                short(&row.hash),
                row.summary
            ));
        }
        let question = format!(
            "Rewrite {} message(s)? {rehashed} commit(s) get new hashes and the old tip is kept as a backup",
            decisions.len()
        );
        if !ui.confirm(&question, true)? {
            ui.logger().warn("Nothing was rewritten");
            return Ok(());
        }
    }

    // --- rewrite ---------------------------------------------------------------
    git.head_branch_ref()?;
    let done = git.rewrite_messages(&decisions)?;

    ui.logger().ok(&format!(
        "Reworded {} commit(s); {} now have new hashes",
        done.reworded, done.rehashed
    ));
    let rewrote_published = decisions.keys().any(|h| published.contains(h));
    if rewrote_published {
        ui.logger()
            .warn("Published commits were rewritten: push with `git push --force-with-lease`");
    }

    let content = ui
        .new_output_content()
        .title("Commits fixed")
        .subtitle(note)
        .data("rewritten", "true")
        .data("reworded", done.reworded.to_string())
        .data("rehashed", done.rehashed.to_string())
        .data("old_tip", done.old_tip.clone())
        .data("new_tip", done.new_tip.clone())
        .data("backup_ref", done.backup_ref.clone())
        .data(
            "undo",
            format!("git update-ref {} {}", done.branch_ref, done.old_tip),
        )
        .table(Some("Changes".to_string()), plan);
    ui.print_with_meta(&content, Some(&meta(ctx, start, "fix")), true)
}

fn meta(ctx: &Context, start: Instant, command: &str) -> scriba::Meta {
    ctx.ui()
        .new_output_meta()
        .with_command(command.to_string())
        .with_dry_run(ctx.dry_run())
        .with_duration_ms(start.elapsed().as_millis() as u64)
        .with_timestamp(chrono::Utc::now().to_string())
}

fn full_message(by_hash: &HashMap<String, CommitSummary>, hash: &str) -> String {
    by_hash
        .get(hash)
        .map(|c| c.full_message.clone().unwrap_or_else(|| c.summary.clone()))
        .unwrap_or_default()
}

enum Answer {
    Reword(String),
    Skip,
    Stop,
}

/// Walk through one offending commit and work out its corrected message.
fn ask_about(
    ctx: &Context,
    policy: &CommitModel,
    validator: &CommitValidator,
    commit: &ValidatedCommit,
    original: &str,
) -> CoreResult<Answer> {
    let ui = ctx.ui();
    ui.logger()
        .info(&format!("\n{}  {}", short(&commit.hash), commit.summary));
    for violation in &commit.violations {
        ui.logger().info(&format!("    - {}", violation.message()));
    }

    let action = ui
        .select(&SelectRequest::new(
            "What would you like to do with this commit?".to_string(),
            vec![
                SelectOption::new("reword", "Reword it")
                    .description("Pick a type, scope and subject".to_string()),
                SelectOption::new("skip", "Skip it")
                    .description("Leave this one as it is".to_string()),
                SelectOption::new("stop", "Stop here")
                    .description("Keep what you have decided so far".to_string()),
            ],
        ))
        .map_err(|_| {
            ErrorCode::InvalidInput
                .error()
                .with_context("field", "action")
        })?;
    match action.as_str() {
        "skip" => return Ok(Answer::Skip),
        "stop" => return Ok(Answer::Stop),
        _ => {}
    }

    loop {
        let reword = ask_reword(ctx, policy, commit)?;
        let message = reworded_message(original, &reword, policy);
        let problems = validator.validate_message(&message);
        if problems.is_empty() {
            return Ok(Answer::Reword(message));
        }
        ui.logger().warn("That message still breaks the rules:");
        for problem in &problems {
            ui.logger().info(&format!("    - {}", problem.message()));
        }
    }
}

fn ask_reword(ctx: &Context, policy: &CommitModel, commit: &ValidatedCommit) -> CoreResult<Reword> {
    let ui = ctx.ui();

    // The type the wording hints at goes first, so Enter takes it.
    let guess = suggested_type(&commit.summary, policy);
    let mut types: Vec<_> = policy.types.iter().collect();
    if let Some(guess) = &guess
        && let Some(position) = types.iter().position(|t| &t.key == guess)
    {
        let found = types.remove(position);
        types.insert(0, found);
    }
    let commit_type = ui
        .select(&SelectRequest::new(
            "Which type is it?".to_string(),
            types
                .iter()
                .map(|t| {
                    let label = if guess.as_deref() == Some(t.key.as_str()) {
                        format!("{} (suggested)", t.key)
                    } else {
                        t.key.clone()
                    };
                    SelectOption::new(t.key.clone(), label)
                        .description(t.description.clone().unwrap_or_default())
                })
                .collect(),
        ))
        .map_err(|_| {
            ErrorCode::InvalidInput
                .error()
                .with_context("field", "type")
        })?;

    let scope = match policy.scope_requirement {
        ScopeRequirement::Disabled => None,
        ScopeRequirement::Optional => {
            let value = ui.text("Scope", None, Some("Leave empty for none"))?;
            Some(value.trim().to_string()).filter(|s| !s.is_empty())
        }
        ScopeRequirement::Required => {
            let value = ui.text("Scope", None, Some("Required by the project's rules"))?;
            Some(value.trim().to_string()).filter(|s| !s.is_empty())
        }
    };

    let subject = ui.text(
        "Short subject",
        Some(&suggested_subject(&commit.summary)),
        Some("Imperative tone, e.g. 'add parser'"),
    )?;

    let breaking = ui.confirm("Is this a breaking change?", false)?;
    let breaking_note = if breaking {
        let hint = if policy.breaking_footer_required {
            "Required by the project's rules"
        } else {
            "Optional; leave empty to skip the footer"
        };
        Some(ui.text("Describe the breaking change", None, Some(hint))?)
            .filter(|note| !note.trim().is_empty())
    } else {
        None
    };

    Ok(Reword {
        commit_type,
        scope,
        subject: subject.trim().to_string(),
        breaking,
        breaking_note,
    })
}
