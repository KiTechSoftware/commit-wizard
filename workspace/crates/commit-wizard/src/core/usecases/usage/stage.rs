use std::{path::PathBuf, time::Instant};

use crate::{
    core::{Context, CoreResult},
    engine::{
        ErrorCode, PromptTrait,
        capabilities::usage::stage::{
            expand_target_paths, plan_transitions, preselected_ids, staged_counts, view_paths,
        },
        constants::emoji::{FILES, INSPECT},
    },
};

use scriba::{MultiSelectOption, MultiSelectRequest, SelectOption, SelectRequest};

pub fn run(
    ctx: &Context,
    paths: Vec<PathBuf>,
    all: bool,
    exclude_staged: bool,
    unstage: bool,
) -> CoreResult<()> {
    let ui = ctx.ui();
    let git = ctx.git();
    let start = Instant::now();

    let path_strings = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    // ---------- Fast path: explicit paths ----------
    if !path_strings.is_empty() {
        let changes = git.list_changes()?;
        let targets = expand_target_paths(&path_strings, &changes);

        let duration_ms = start.elapsed().as_millis() as u64;

        if targets.is_empty() {
            let content = ui
                .new_output_content()
                .title("Stage")
                .subtitle(if unstage {
                    "No matching staged/changed paths to unstage"
                } else {
                    "No matching changed paths to stage"
                })
                .data("requested_count", path_strings.len().to_string())
                .data("matched_count", "0")
                .data("unstage", unstage.to_string())
                .data("dry_run", ctx.dry_run().to_string());

            let meta = ui
                .new_output_meta()
                .with_duration_ms(duration_ms)
                .with_timestamp(chrono::Utc::now().to_string())
                .with_command("stage".to_string())
                .with_dry_run(ctx.dry_run());

            return ui.print_with_meta(&content, Some(&meta), true);
        }

        if ctx.dry_run() {
            let mut content = ui
                .new_output_content()
                .title(if unstage {
                    "Unstage Preview"
                } else {
                    "Stage Preview"
                })
                .subtitle("Dry run: no git index changes were made")
                .data("matched_count", targets.len().to_string())
                .data("unstage", unstage.to_string());

            content = content.section("Paths", targets.join("\n"), "text".to_string());

            let meta = ui
                .new_output_meta()
                .with_duration_ms(duration_ms)
                .with_timestamp(chrono::Utc::now().to_string())
                .with_command("stage".to_string())
                .with_dry_run(true);

            return ui.print_with_meta(&content, Some(&meta), true);
        }

        if unstage {
            git.unstage_paths(&targets)?;
        } else {
            git.stage_paths(&targets)?;
        }

        let mut content = ui
            .new_output_content()
            .title(if unstage {
                "Paths Unstaged"
            } else {
                "Paths Staged"
            })
            .subtitle("Git index updated successfully")
            .data("matched_count", targets.len().to_string())
            .data("unstage", unstage.to_string());

        content = content.section("Paths", targets.join("\n"), "text".to_string());

        let meta = ui
            .new_output_meta()
            .with_duration_ms(duration_ms)
            .with_timestamp(chrono::Utc::now().to_string())
            .with_command("stage".to_string())
            .with_dry_run(false);

        return ui.print_with_meta(&content, Some(&meta), true);
    }

    // ---------- Fast path: --all ----------
    if all && !unstage {
        let duration_ms = start.elapsed().as_millis() as u64;

        if ctx.dry_run() {
            let content = ui
                .new_output_content()
                .title("Stage Preview")
                .subtitle("Dry run: would stage all files")
                .data("all", "true")
                .data("unstage", "false");

            let meta = ui
                .new_output_meta()
                .with_duration_ms(duration_ms)
                .with_timestamp(chrono::Utc::now().to_string())
                .with_command("stage".to_string())
                .with_dry_run(true);

            return ui.print_with_meta(&content, Some(&meta), true);
        }

        git.stage_all()?;

        let content = ui
            .new_output_content()
            .title("All Files Staged")
            .subtitle("Git index updated successfully")
            .data("all", "true")
            .data("unstage", "false");

        let meta = ui
            .new_output_meta()
            .with_duration_ms(duration_ms)
            .with_timestamp(chrono::Utc::now().to_string())
            .with_command("stage".to_string())
            .with_dry_run(false);

        return ui.print_with_meta(&content, Some(&meta), true);
    }

    if all && unstage {
        let duration_ms = start.elapsed().as_millis() as u64;

        if ctx.dry_run() {
            let content = ui
                .new_output_content()
                .title("Unstage Preview")
                .subtitle("Dry run: would unstage all files")
                .data("all", "true")
                .data("unstage", "true");

            let meta = ui
                .new_output_meta()
                .with_duration_ms(duration_ms)
                .with_timestamp(chrono::Utc::now().to_string())
                .with_command("stage".to_string())
                .with_dry_run(true);

            return ui.print_with_meta(&content, Some(&meta), true);
        }

        git.unstage_all()?;

        let content = ui
            .new_output_content()
            .title("All Files Unstaged")
            .subtitle("Git index updated successfully")
            .data("all", "true")
            .data("unstage", "true");

        let meta = ui
            .new_output_meta()
            .with_duration_ms(duration_ms)
            .with_timestamp(chrono::Utc::now().to_string())
            .with_command("stage".to_string())
            .with_dry_run(false);

        return ui.print_with_meta(&content, Some(&meta), true);
    }
    // ---------- Interactive mode ----------
    if !ctx.is_interactive() {
        return Err(ErrorCode::UserInteractionRequired.error().with_context(
            "reason",
            "interactive staging requires either interactive mode, explicit paths, or --all",
        ));
    }

    let changes = git.list_changes()?;
    let visible_paths = view_paths(&changes, exclude_staged, unstage);

    let duration_ms = start.elapsed().as_millis() as u64;

    if visible_paths.is_empty() {
        let content = ui
            .new_output_content()
            .title("Stage")
            .subtitle("No modified, untracked, or staged files")
            .data("visible_count", "0")
            .data("exclude_staged", exclude_staged.to_string())
            .data("unstage", unstage.to_string());

        let meta = ui
            .new_output_meta()
            .with_duration_ms(duration_ms)
            .with_timestamp(chrono::Utc::now().to_string())
            .with_command("stage".to_string())
            .with_dry_run(ctx.dry_run());

        return ui.print_with_meta(&content, Some(&meta), true);
    }

    let (staged_in_view, unstaged_in_view) = staged_counts(&visible_paths, &changes);

    ui.logger().heading("Stage");
    ui.logger().info(&format!(
        "{} {} file(s) in view: {} staged, {} unstaged",
        FILES,
        visible_paths.len(),
        staged_in_view,
        unstaged_in_view
    ));

    let preselected = preselected_ids(&visible_paths, &changes)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();

    let request = MultiSelectRequest {
        message: if exclude_staged {
            "Select files to stage (currently unstaged):".to_string()
        } else if unstage {
            "Select files to UNSTAGE (currently staged):".to_string()
        } else {
            "Select files to stage or unstage:".to_string()
        },
        options: visible_paths
            .iter()
            .map(|path| MultiSelectOption {
                id: path.clone(),
                label: path.clone(),
                description: None,
                selected: preselected.contains(path),
            })
            .collect(),
        page_size: None,
    };

    let selected = ui.multiselect(&request)?;

    if !selected.is_empty()
        && ui.confirm("Would you like to preview diffs for selected files?", false)?
    {
        let mut files = selected.clone();
        files.sort();

        loop {
            let mut options = files
                .iter()
                .map(|path| SelectOption {
                    id: path.clone(),
                    label: path.clone(),
                    description: Some("diff".to_string()),
                })
                .collect::<Vec<_>>();

            options.push(SelectOption {
                id: "__done__".to_string(),
                label: "Done viewing diffs".to_string(),
                description: None,
            });

            let response = ui.select(&SelectRequest {
                message: "Select a file to view diff".to_string(),
                options,
                page_size: None,
            })?;

            if response == "__done__" {
                break;
            }

            match git.diff_unstaged(&response)? {
                Some(patch) => {
                    let output = ui
                        .new_output_content()
                        .title(format!("{} Diff: {}", INSPECT, response))
                        .code(Some("diff".to_string()), ui.git_diff(patch.trim()));

                    ui.print(&output)?;
                    let _ = ui.confirm("Press Enter to continue", true)?;
                }
                None => {
                    ui.logger()
                        .info(&format!("No unstaged changes for '{}'", response));
                }
            }
        }
    }

    let (to_stage, to_unstage) = plan_transitions(&visible_paths, &selected, &changes);

    let mut content = ui
        .new_output_content()
        .title(if ctx.dry_run() {
            "Stage Preview"
        } else {
            "Stage Result"
        })
        .subtitle(if ctx.dry_run() {
            "Dry run: no git index changes were made"
        } else {
            "Git index updated successfully"
        })
        .data("visible_count", visible_paths.len().to_string())
        .data("staged_in_view", staged_in_view.to_string())
        .data("unstaged_in_view", unstaged_in_view.to_string())
        .data("to_stage_count", to_stage.len().to_string())
        .data("to_unstage_count", to_unstage.len().to_string())
        .data("exclude_staged", exclude_staged.to_string())
        .data("unstage_mode", unstage.to_string());

    if !to_stage.is_empty() {
        content = content.section("Will Stage", to_stage.join("\n"), "text".to_string());
    }

    if !to_unstage.is_empty() {
        content = content.section("Will Unstage", to_unstage.join("\n"), "text".to_string());
    }

    if !ctx.dry_run() {
        if !to_stage.is_empty() {
            git.stage_paths(&to_stage)?;
        }

        if !to_unstage.is_empty() {
            git.unstage_paths(&to_unstage)?;
        }

        if to_stage.is_empty() && to_unstage.is_empty() {
            content = content.subtitle("No changes to staging");
        }
    }

    let meta = ui
        .new_output_meta()
        .with_duration_ms(duration_ms)
        .with_timestamp(chrono::Utc::now().to_string())
        .with_command("stage".to_string())
        .with_dry_run(ctx.dry_run());

    ui.print_with_meta(&content, Some(&meta), true)
}
