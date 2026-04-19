use std::collections::BTreeSet;

use crate::engine::models::git::Change;

pub fn expand_target_paths(requested_paths: &[String], changes: &[Change]) -> Vec<String> {
    let mut targets = BTreeSet::new();

    for want in requested_paths {
        for change in changes {
            if matches_path(&change.path, want) {
                targets.insert(change.path.clone());
            }
        }
    }

    targets.into_iter().collect()
}

pub fn view_paths(changes: &[Change], exclude_staged: bool, unstage: bool) -> Vec<String> {
    let staged_set = staged_path_set(changes);

    let mut paths = changes
        .iter()
        .map(|c| c.path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if exclude_staged {
        paths.retain(|p| !staged_set.contains(p));
    } else if unstage {
        paths.retain(|p| staged_set.contains(p));
    }

    paths
}

pub fn staged_counts(paths: &[String], changes: &[Change]) -> (usize, usize) {
    let staged_set = staged_path_set(changes);
    let staged = paths.iter().filter(|p| staged_set.contains(*p)).count();
    let unstaged = paths.len().saturating_sub(staged);
    (staged, unstaged)
}

pub fn preselected_ids(paths: &[String], changes: &[Change]) -> Vec<String> {
    let staged_set = staged_path_set(changes);

    paths
        .iter()
        .filter(|p| staged_set.contains(*p))
        .cloned()
        .collect()
}

pub fn plan_transitions(
    visible_paths: &[String],
    selected_ids: &[String],
    changes: &[Change],
) -> (Vec<String>, Vec<String>) {
    let staged_set = staged_path_set(changes);
    let selected = selected_ids.iter().cloned().collect::<BTreeSet<_>>();

    let mut to_stage = Vec::new();
    let mut to_unstage = Vec::new();

    for path in visible_paths {
        let is_selected = selected.contains(path);
        let is_staged = staged_set.contains(path);

        if is_selected && !is_staged {
            to_stage.push(path.clone());
        } else if !is_selected && is_staged {
            to_unstage.push(path.clone());
        }
    }

    (to_stage, to_unstage)
}

fn staged_path_set(changes: &[Change]) -> BTreeSet<String> {
    changes
        .iter()
        .filter(|c| c.staged)
        .map(|c| c.path.clone())
        .collect()
}

fn matches_path(file: &str, want: &str) -> bool {
    let file = normalize_path(file);
    let want = normalize_path(want);

    file == want || file.starts_with(&(want + "/"))
}

fn normalize_path(value: &str) -> String {
    value.replace('\\', "/").trim_matches('/').to_string()
}
