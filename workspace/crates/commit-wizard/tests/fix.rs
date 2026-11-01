//! `cw fix`: correct commit messages without touching anything else.

mod common;

use common::{Repo, text};

fn commit_file(repo: &Repo, name: &str, message: &str) {
    repo.write(name, name);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
}

fn commit_with_body(repo: &Repo, name: &str, subject: &str, body: &str) {
    repo.write(name, name);
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", subject, "-m", body]);
}

/// chore: init, then four commits, two of which break the rules.
///
///   feat: good one | added a button | feat: fine | wip stuff
fn with_bad_commits() -> Repo {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "feat: good one");
    commit_file(&repo, "b.txt", "added a button");
    commit_file(&repo, "c.txt", "feat: fine");
    commit_file(&repo, "d.txt", "wip stuff");
    repo
}

fn hash_of(repo: &Repo, subject: &str) -> String {
    repo.git(&["log", "--format=%H", "--grep", subject, "-1"])
}

/// Everything about each commit except its message and hash.
fn snapshot(repo: &Repo) -> Vec<String> {
    repo.git(&["log", "--reverse", "--format=%T|%an|%ae|%aI|%cn|%cI"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn subjects_oldest_first(repo: &Repo) -> Vec<String> {
    repo.git(&["log", "--reverse", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn hashes_oldest_first(repo: &Repo) -> Vec<String> {
    repo.git(&["log", "--reverse", "--format=%H"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn fix_both(repo: &Repo) -> std::process::Output {
    let button = hash_of(repo, "added a button");
    let wip = hash_of(repo, "wip stuff");
    repo.cw(&[
        "fix",
        "--reword",
        &format!("{}=feat: add a button", &button[..10]),
        "--reword",
        &format!("{}=chore: tidy up", &wip[..10]),
    ])
}

#[test]
fn a_clean_history_has_nothing_to_fix() {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "feat: fine");
    let before = hashes_oldest_first(&repo);

    let out = repo.cw(&["fix"]);

    assert!(out.status.success(), "{}", text(&out));
    assert!(text(&out).contains("nothing to fix"), "{}", text(&out));
    assert_eq!(hashes_oldest_first(&repo), before);
}

#[test]
fn dry_run_lists_the_offenders_and_their_reasons_without_changing_anything() {
    let repo = with_bad_commits();
    let before = hashes_oldest_first(&repo);

    let out = repo.cw(&["fix", "--dry-run"]);

    assert!(out.status.success(), "{}", text(&out));
    let output = text(&out);
    assert!(
        output.contains("2 of 5 commit(s) break the rules"),
        "{output}"
    );
    assert!(
        output.contains("added a button") && output.contains("wip stuff"),
        "{output}"
    );
    assert!(
        output.contains("Header does not match conventional commit format"),
        "{output}"
    );
    assert_eq!(hashes_oldest_first(&repo), before);
}

#[test]
fn rewording_changes_only_messages_never_files_authors_or_dates() {
    let repo = with_bad_commits();
    let before = snapshot(&repo);

    let out = fix_both(&repo);

    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(
        snapshot(&repo),
        before,
        "trees, authors and dates must be identical"
    );
    assert_eq!(
        subjects_oldest_first(&repo),
        [
            "chore: init",
            "feat: good one",
            "feat: add a button",
            "feat: fine",
            "chore: tidy up"
        ]
    );
    assert!(
        repo.git(&["status", "--porcelain"]).is_empty(),
        "working tree changed"
    );
}

#[test]
fn commits_before_the_first_fix_keep_their_hashes_and_later_ones_get_new_ones() {
    let repo = with_bad_commits();
    let before = hashes_oldest_first(&repo);

    assert!(fix_both(&repo).status.success());
    let after = hashes_oldest_first(&repo);

    assert_eq!(after.len(), before.len());
    assert_eq!(
        after[..2],
        before[..2],
        "commits before the first fix must not change"
    );
    for (new, old) in after[2..].iter().zip(&before[2..]) {
        assert_ne!(new, old, "a commit after the first fix kept its hash");
    }
}

#[test]
fn the_old_tip_is_saved_and_the_change_can_be_undone() {
    let repo = with_bad_commits();
    let old_tip = repo.git(&["rev-parse", "HEAD"]);

    let out = fix_both(&repo);
    assert!(out.status.success(), "{}", text(&out));

    let backups = repo.git(&[
        "for-each-ref",
        "refs/cw/fix-backup",
        "--format=%(objectname)",
    ]);
    assert_eq!(backups, old_tip);
    assert!(
        text(&out).contains(&format!("git update-ref refs/heads/main {old_tip}")),
        "{}",
        text(&out)
    );

    // Following the printed undo command restores the original history.
    repo.git(&["update-ref", "refs/heads/main", &old_tip]);
    assert!(subjects_oldest_first(&repo).contains(&"wip stuff".to_string()));
}

#[test]
fn the_body_and_footers_of_a_reworded_commit_are_kept() {
    let repo = Repo::new();
    commit_with_body(
        &repo,
        "a.txt",
        "Fixed the parser",
        "It crashed on empty input.\n\nSigned-off-by: A <a@example.com>",
    );
    let hash = hash_of(&repo, "Fixed the parser");

    let out = repo.cw(&[
        "fix",
        "--reword",
        &format!("{}=fix: handle empty input", &hash[..10]),
    ]);

    assert!(out.status.success(), "{}", text(&out));
    let message = repo.git(&["log", "-1", "--format=%B"]);
    assert!(
        message.starts_with("fix: handle empty input\n"),
        "{message}"
    );
    assert!(message.contains("It crashed on empty input."), "{message}");
    assert!(
        message.contains("Signed-off-by: A <a@example.com>"),
        "{message}"
    );
}

#[test]
fn a_correction_that_is_still_wrong_is_rejected_and_nothing_changes() {
    let repo = with_bad_commits();
    let before = hashes_oldest_first(&repo);
    let button = hash_of(&repo, "added a button");

    let out = repo.cw(&[
        "fix",
        "--reword",
        &format!("{}=still not conventional", &button[..10]),
    ]);

    assert!(!out.status.success());
    assert!(
        text(&out).contains("Header does not match"),
        "{}",
        text(&out)
    );
    assert_eq!(hashes_oldest_first(&repo), before);
}

#[test]
fn one_bad_correction_stops_the_whole_run_before_anything_is_rewritten() {
    let repo = with_bad_commits();
    let before = hashes_oldest_first(&repo);
    let button = hash_of(&repo, "added a button");
    let wip = hash_of(&repo, "wip stuff");

    let out = repo.cw(&[
        "fix",
        "--reword",
        &format!("{}=feat: add a button", &button[..10]),
        "--reword",
        &format!("{}=nonsense", &wip[..10]),
    ]);

    assert!(!out.status.success());
    assert_eq!(
        hashes_oldest_first(&repo),
        before,
        "the valid correction must not be applied alone"
    );
}

#[test]
fn a_hash_that_matches_nothing_or_is_too_short_is_an_error() {
    let repo = with_bad_commits();
    for spec in ["ffffffff=feat: x", "abc=feat: x", "no-equals-sign"] {
        let out = repo.cw(&["fix", "--reword", spec]);
        assert!(!out.status.success(), "{spec}: {}", text(&out));
    }
}

#[test]
fn without_a_terminal_it_asks_for_reword_flags() {
    let repo = with_bad_commits();
    let out = repo.cw(&["fix"]);

    assert!(!out.status.success());
    assert!(text(&out).contains("--reword"), "{}", text(&out));
}

#[test]
fn tail_limits_how_far_back_it_looks() {
    let repo = with_bad_commits();
    // Only the newest commit ("wip stuff") is in range.
    let out = repo.cw(&["fix", "--dry-run", "--tail", "1"]);

    let output = text(&out);
    assert!(
        output.contains("1 of 1 commit(s) break the rules"),
        "{output}"
    );
    assert!(
        output.contains("wip stuff") && !output.contains("added a button"),
        "{output}"
    );
}

// --- published history -------------------------------------------------------

fn publish(repo: &Repo) {
    repo.git(&["push", "-q", "origin", "main"]);
}

#[test]
fn commits_already_on_a_remote_are_not_offered_by_default() {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "Fixed the parser");
    publish(&repo);
    commit_file(&repo, "b.txt", "wip stuff");

    let out = repo.cw(&["fix", "--dry-run"]);

    let output = text(&out);
    assert!(
        output.contains("1 of 1 commit(s) break the rules"),
        "{output}"
    );
    assert!(
        output.contains("wip stuff") && !output.contains("Fixed the parser"),
        "{output}"
    );
}

#[test]
fn a_published_commit_is_refused_unless_forced() {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "Fixed the parser");
    publish(&repo);
    let before = hashes_oldest_first(&repo);
    let hash = hash_of(&repo, "Fixed the parser");
    let reword = format!("{}=fix: handle the parser", &hash[..10]);

    let refused = repo.cw(&["fix", "--from", "HEAD~1", "--reword", &reword]);
    assert!(!refused.status.success());
    assert!(
        text(&refused).contains("already on a remote"),
        "{}",
        text(&refused)
    );
    assert_eq!(hashes_oldest_first(&repo), before);

    let forced = repo.cw(&["--force", "fix", "--from", "HEAD~1", "--reword", &reword]);
    assert!(forced.status.success(), "{}", text(&forced));
    assert!(
        text(&forced).contains("force-with-lease"),
        "{}",
        text(&forced)
    );
    assert_eq!(repo.subjects()[0], "fix: handle the parser");
}

// --- histories that cannot be rewritten safely ---------------------------------

#[test]
fn a_detached_head_is_refused() {
    let repo = with_bad_commits();
    repo.git(&["checkout", "-q", "--detach"]);
    let wip = hash_of(&repo, "wip stuff");

    let out = repo.cw(&["fix", "--reword", &format!("{}=chore: tidy up", &wip[..10])]);

    assert!(!out.status.success());
    assert!(text(&out).contains("detached"), "{}", text(&out));
}

#[test]
fn a_merge_commit_in_the_range_is_refused() {
    let repo = Repo::new();
    commit_file(&repo, "a.txt", "added a button");
    repo.git(&["checkout", "-q", "-b", "side"]);
    commit_file(&repo, "side.txt", "feat: on the side");
    repo.git(&["checkout", "-q", "main"]);
    commit_file(&repo, "b.txt", "feat: on main");
    repo.git(&["merge", "-q", "--no-ff", "side", "-m", "Merge branch side"]);
    let before = hashes_oldest_first(&repo);
    let button = hash_of(&repo, "added a button");

    let out = repo.cw(&[
        "fix",
        "--reword",
        &format!("{}=feat: add a button", &button[..10]),
    ]);

    assert!(!out.status.success());
    assert!(text(&out).contains("merge commit"), "{}", text(&out));
    assert_eq!(hashes_oldest_first(&repo), before);
}

#[test]
fn an_unfinished_merge_or_rebase_is_refused() {
    let repo = with_bad_commits();
    let head = repo.git(&["rev-parse", "HEAD"]);
    std::fs::write(repo.path().join(".git/MERGE_HEAD"), format!("{head}\n")).unwrap();
    let wip = hash_of(&repo, "wip stuff");

    let out = repo.cw(&["fix", "--reword", &format!("{}=chore: tidy up", &wip[..10])]);

    assert!(!out.status.success());
    assert!(text(&out).contains("in progress"), "{}", text(&out));
}

#[test]
fn signed_commits_are_refused_unless_forced() {
    let mut repo = Repo::new();
    if !repo.enable_signing() {
        eprintln!("skipping: gpg unavailable");
        return;
    }
    repo.write("a.txt", "a");
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-S", "-m", "added a button"]);
    let hash = hash_of(&repo, "added a button");
    let reword = format!("{}=feat: add a button", &hash[..10]);

    let refused = repo.cw(&["fix", "--reword", &reword]);
    assert!(!refused.status.success());
    assert!(
        text(&refused).contains("lose their signature"),
        "{}",
        text(&refused)
    );

    let forced = repo.cw(&["--force", "fix", "--reword", &reword]);
    assert!(forced.status.success(), "{}", text(&forced));
    assert_eq!(repo.subjects()[0], "feat: add a button");
}
