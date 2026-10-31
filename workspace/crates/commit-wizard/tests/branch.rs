//! `cw branch`: list, switch, create and prune.

mod common;

use common::{Repo, text};

/// A repo on `main` with a pushed first commit.
fn repo() -> Repo {
    let repo = Repo::new();
    repo.git(&["push", "-q", "-u", "origin", "main"]);
    repo
}

/// Branch `feature/<name>` with one commit, pushed, then back on main. With `merge`
/// the branch is merged into main (a merge commit), as a pull request would.
fn feature(repo: &Repo, name: &str, merge: bool) {
    let branch = format!("feature/{name}");
    repo.git(&["switch", "-q", "-c", &branch]);
    repo.commit(&format!("feat: {name}"));
    repo.git(&["push", "-q", "-u", "origin", &branch]);
    repo.git(&["switch", "-q", "main"]);
    if merge {
        repo.git(&[
            "merge",
            "-q",
            "--no-ff",
            &branch,
            "-m",
            &format!("Merge {name}"),
        ]);
        repo.git(&["push", "-q", "origin", "main"]);
    }
}

fn delete_on_remote(repo: &Repo, name: &str) {
    repo.git(&[
        "push",
        "-q",
        "origin",
        "--delete",
        &format!("feature/{name}"),
    ]);
}

fn local_branches(repo: &Repo) -> Vec<String> {
    repo.git(&["branch", "--format=%(refname:short)"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn row_for(output: &str, branch: &str) -> String {
    output
        .lines()
        .find(|line| line.contains('│') && line.contains(branch))
        .unwrap_or_else(|| panic!("no row for {branch} in:\n{output}"))
        .to_string()
}

// --- list ---------------------------------------------------------------------

#[test]
fn list_shows_branches_what_they_follow_and_where_they_stand() {
    let repo = repo();
    feature(&repo, "live", false);
    feature(&repo, "done", true);
    delete_on_remote(&repo, "done");
    repo.git(&["fetch", "--prune"]);
    repo.git(&["switch", "-q", "-c", "scratch"]);

    let out = repo.cw(&["branch", "list"]);
    assert!(out.status.success(), "{}", text(&out));
    let output = text(&out);
    assert!(row_for(&output, "feature/live").contains("up to date"));
    assert!(row_for(&output, "feature/done").contains("remote deleted"));
    let scratch = row_for(&output, "scratch");
    assert!(scratch.contains('*'), "{scratch}");
    assert!(!scratch.contains("up to date"), "{scratch}");
}

#[test]
fn list_counts_commits_ahead_and_behind() {
    let repo = repo();
    repo.commit("feat: local only");
    let out = repo.cw(&["branch", "list"]);
    assert!(
        row_for(&text(&out), "main").contains("ahead 1"),
        "{}",
        text(&out)
    );
}

#[test]
fn list_remote_and_all() {
    let repo = repo();
    feature(&repo, "live", false);

    let remote = text(&repo.cw(&["branch", "list", "--remote"]));
    assert!(remote.contains("origin/feature/live"), "{remote}");
    assert!(!remote.contains("│ feature/live "), "{remote}");

    let all = text(&repo.cw(&["branch", "list", "--all"]));
    assert!(
        all.contains("origin/feature/live") && all.contains("│ feature/live"),
        "{all}"
    );
}

#[test]
fn bare_branch_lists_when_there_is_no_terminal() {
    let repo = repo();
    let out = repo.cw(&["branch"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(row_for(&text(&out), "main").contains('*'));
}

// --- switch -------------------------------------------------------------------

#[test]
fn switch_checks_out_a_branch() {
    let repo = repo();
    feature(&repo, "live", false);
    let out = repo.cw(&["branch", "switch", "feature/live"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        repo.git(&["branch", "--show-current"]).trim(),
        "feature/live"
    );
}

#[test]
fn switch_picks_up_a_branch_that_exists_only_on_the_remote() {
    let repo = repo();
    feature(&repo, "remote-only", false);
    repo.git(&["branch", "-q", "-D", "feature/remote-only"]);
    let out = repo.cw(&["branch", "switch", "feature/remote-only"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        repo.git(&["branch", "--show-current"]).trim(),
        "feature/remote-only"
    );
    assert_eq!(
        repo.git(&[
            "rev-parse",
            "--abbrev-ref",
            "feature/remote-only@{upstream}"
        ])
        .trim(),
        "origin/feature/remote-only"
    );
}

#[test]
fn switch_without_a_name_or_terminal_says_what_to_pass() {
    let repo = repo();
    let out = repo.cw(&["branch", "switch"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("switch NAME"), "{}", text(&out));
}

#[test]
fn switching_to_a_missing_branch_fails_and_stays_put() {
    let repo = repo();
    let out = repo.cw(&["branch", "switch", "nope"]);
    assert!(!out.status.success());
    assert_eq!(repo.git(&["branch", "--show-current"]).trim(), "main");
}

#[test]
fn switch_dry_run_stays_put() {
    let repo = repo();
    feature(&repo, "live", false);
    let out = repo.cw(&["branch", "switch", "feature/live", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.git(&["branch", "--show-current"]).trim(), "main");
}

// --- new ----------------------------------------------------------------------

#[test]
fn new_builds_the_name_from_type_ticket_and_description() {
    let repo = repo();
    let out = repo.cw(&[
        "branch",
        "new",
        "Add login page",
        "--type",
        "feature",
        "--ticket",
        "PROJ-123",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        repo.git(&["branch", "--show-current"]).trim(),
        "feature/PROJ-123-add-login-page"
    );
}

#[test]
fn new_without_a_type_or_ticket_uses_just_the_description() {
    let repo = repo();
    let out = repo.cw(&["branch", "new", "Quick fix"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.git(&["branch", "--show-current"]).trim(), "quick-fix");
}

#[test]
fn new_can_be_named_after_the_ticket_alone() {
    let repo = repo();
    let out = repo.cw(&["branch", "new", "--type", "bugfix", "--ticket", "#42"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.git(&["branch", "--show-current"]).trim(), "bugfix/#42");
}

#[test]
fn a_name_with_a_slash_is_used_as_it_is() {
    let repo = repo();
    let out = repo.cw(&["branch", "new", "team/Exact_Name"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        repo.git(&["branch", "--show-current"]).trim(),
        "team/Exact_Name"
    );

    let clash = repo.cw(&["branch", "new", "a/b", "--type", "feature"]);
    assert!(!clash.status.success());
}

#[test]
fn new_can_stay_on_the_current_branch_and_start_elsewhere() {
    let repo = repo();
    let first = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.commit("feat: second");
    let out = repo.cw(&["branch", "new", "old base", "--from", &first, "--no-switch"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(repo.git(&["branch", "--show-current"]).trim(), "main");
    assert_eq!(repo.git(&["rev-parse", "old-base"]).trim(), first);
}

#[test]
fn new_refuses_an_existing_or_invalid_name() {
    let repo = repo();
    assert!(
        repo.cw(&["branch", "new", "x", "--type", "feature"])
            .status
            .success()
    );
    repo.git(&["switch", "-q", "main"]);
    let again = repo.cw(&["branch", "new", "x", "--type", "feature"]);
    assert!(!again.status.success());
    assert!(text(&again).contains("already exists"), "{}", text(&again));

    let bad = repo.cw(&["branch", "new", "bad..name/x"]);
    assert!(!bad.status.success());
}

#[test]
fn new_without_a_name_or_ticket_says_what_to_pass() {
    let repo = repo();
    let out = repo.cw(&["branch", "new"]);
    assert!(!out.status.success());
    assert!(text(&out).contains("--ticket"), "{}", text(&out));
}

#[test]
fn new_checks_the_ticket_against_the_projects_pattern() {
    let repo = repo();
    repo.write(
        ".cwizard.toml",
        "version = 1\n[commit.ticket]\nrequired = true\npattern = \"[A-Z]+-[0-9]+\"\n",
    );

    let bad = repo.cw(&["branch", "new", "x", "--ticket", "nope"]);
    assert!(!bad.status.success());
    assert!(
        text(&bad).contains("commit.ticket.pattern"),
        "{}",
        text(&bad)
    );

    let missing = repo.cw(&["branch", "new", "x"]);
    assert!(!missing.status.success());
    assert!(text(&missing).contains("--ticket"), "{}", text(&missing));

    let ok = repo.cw(&[
        "branch", "new", "x", "--ticket", "AB-1", "--type", "feature",
    ]);
    assert!(ok.status.success(), "{}", text(&ok));
}

#[test]
fn a_branch_named_with_a_ticket_feeds_cw_commit() {
    let repo = repo();
    repo.write(
        ".cwizard.toml",
        "version = 1\n[commit.ticket]\nrequired = true\nsource = \"branch\"\npattern = \"[A-Z]+-[0-9]+\"\n",
    );
    let out = repo.cw(&[
        "branch", "new", "login", "--type", "feature", "--ticket", "PROJ-9",
    ]);
    assert!(out.status.success(), "{}", text(&out));
    repo.write("a.txt", "a");
    repo.git(&["add", "a.txt"]);
    let commit = repo.cw(&["commit", "-t", "feat", "-m", "add login"]);
    assert!(commit.status.success(), "{}", text(&commit));
    assert_eq!(repo.subjects()[0], "feat: PROJ-9: add login");
}

#[test]
fn new_dry_run_creates_nothing() {
    let repo = repo();
    let out = repo.cw(&["branch", "new", "x", "--type", "feature", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(local_branches(&repo), ["main"]);
}

// --- prune --------------------------------------------------------------------

#[test]
fn prune_deletes_merged_branches_whose_remote_is_gone() {
    let repo = repo();
    feature(&repo, "done", true);
    feature(&repo, "live", false);
    delete_on_remote(&repo, "done");

    let out = repo.cw(&["branch", "prune", "--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(
        text(&out).contains("git branch feature/done"),
        "restore hint: {}",
        text(&out)
    );
    assert_eq!(local_branches(&repo), ["feature/live", "main"]);
}

#[test]
fn prune_keeps_unmerged_work_unless_forced() {
    let repo = repo();
    feature(&repo, "squashed", false); // never merged into main
    delete_on_remote(&repo, "squashed");

    let kept = repo.cw(&["branch", "prune", "--yes"]);
    assert!(kept.status.success(), "{}", text(&kept));
    assert!(text(&kept).contains("--force"), "{}", text(&kept));
    assert!(local_branches(&repo).contains(&"feature/squashed".to_string()));

    let forced = repo.cw(&["branch", "prune", "--yes", "--force"]);
    assert!(forced.status.success(), "{}", text(&forced));
    assert_eq!(local_branches(&repo), ["main"]);
}

#[test]
fn prune_without_a_terminal_needs_yes_or_dry_run() {
    let repo = repo();
    feature(&repo, "done", true);
    delete_on_remote(&repo, "done");

    let refused = repo.cw(&["branch", "prune"]);
    assert!(!refused.status.success());
    assert!(text(&refused).contains("--yes"), "{}", text(&refused));
    assert!(local_branches(&repo).contains(&"feature/done".to_string()));

    let preview = repo.cw(&["branch", "prune", "--dry-run"]);
    assert!(preview.status.success(), "{}", text(&preview));
    assert!(
        text(&preview).contains("will be deleted"),
        "{}",
        text(&preview)
    );
    assert!(local_branches(&repo).contains(&"feature/done".to_string()));
}

#[test]
fn prune_never_touches_the_current_or_a_protected_branch() {
    let repo = repo();
    feature(&repo, "done", true);
    delete_on_remote(&repo, "done");
    repo.git(&["switch", "-q", "feature/done"]);

    let out = repo.cw(&["branch", "prune", "--yes", "--force"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("checked out"), "{}", text(&out));
    assert!(local_branches(&repo).contains(&"feature/done".to_string()));

    // main follows a remote branch that still exists, and is protected anyway.
    assert!(local_branches(&repo).contains(&"main".to_string()));
}

#[test]
fn prune_with_nothing_gone_says_so() {
    let repo = repo();
    feature(&repo, "live", false);
    let out = repo.cw(&["branch", "prune", "--yes"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("Nothing to Prune"), "{}", text(&out));
}

#[test]
fn prune_learns_about_deleted_remote_branches_by_fetching() {
    let repo = repo();
    feature(&repo, "done", true);
    // Delete it inside the remote itself, so this clone's tracking ref goes stale
    // and only a fetch can find out.
    let remote = repo
        .git(&["remote", "get-url", "origin"])
        .trim()
        .to_string();
    let status = std::process::Command::new("git")
        .args(["--git-dir", &remote, "branch", "-D", "feature/done"])
        .status()
        .unwrap();
    assert!(status.success());

    let offline = repo.cw(&["branch", "prune", "--no-fetch", "--yes"]);
    assert!(
        text(&offline).contains("Nothing to Prune"),
        "{}",
        text(&offline)
    );
    assert!(local_branches(&repo).contains(&"feature/done".to_string()));

    let online = repo.cw(&["branch", "prune", "--yes"]);
    assert!(online.status.success(), "{}", text(&online));
    assert!(!local_branches(&repo).contains(&"feature/done".to_string()));
}
