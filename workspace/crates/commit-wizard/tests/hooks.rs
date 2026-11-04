//! End-to-end tests for `cw hooks` and `cw check --message-file`.

mod common;

use common::{Repo, text};

fn hook_path(repo: &Repo, name: &str) -> std::path::PathBuf {
    repo.path().join(".git/hooks").join(name)
}

#[test]
fn install_writes_both_hooks_by_default() {
    let repo = Repo::new();
    let out = repo.cw(&["hooks", "install"]);
    assert!(out.status.success(), "{}", text(&out));
    for name in ["commit-msg", "pre-push"] {
        let body = std::fs::read_to_string(hook_path(&repo, name)).unwrap();
        assert!(body.contains("managed by commit-wizard"), "{name}");
    }
    let status = repo.cw(&["hooks", "status"]);
    assert!(
        text(&status).contains("installed (cw)"),
        "{}",
        text(&status)
    );
}

#[test]
fn install_refuses_to_replace_a_foreign_hook_without_force() {
    let repo = Repo::new();
    let path = hook_path(&repo, "commit-msg");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "#!/bin/sh\necho mine\n").unwrap();

    let out = repo.cw(&["hooks", "install"]);
    assert!(!out.status.success(), "{}", text(&out));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "#!/bin/sh\necho mine\n"
    );
    // Nothing was half-installed.
    assert!(!hook_path(&repo, "pre-push").exists());

    let forced = repo.cw(&["hooks", "install", "--force"]);
    assert!(forced.status.success(), "{}", text(&forced));
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("managed by commit-wizard")
    );
}

#[test]
fn uninstall_removes_only_cw_hooks() {
    let repo = Repo::new();
    assert!(
        repo.cw(&["hooks", "install", "--hook", "commit-msg"])
            .status
            .success()
    );
    let foreign = hook_path(&repo, "pre-push");
    std::fs::write(&foreign, "#!/bin/sh\n").unwrap();

    let out = repo.cw(&["hooks", "uninstall"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(!hook_path(&repo, "commit-msg").exists());
    assert!(foreign.exists());
}

#[test]
fn dry_run_installs_nothing() {
    let repo = Repo::new();
    let out = repo.cw(&["hooks", "install", "--dry-run"]);
    assert!(out.status.success(), "{}", text(&out));
    assert!(!hook_path(&repo, "commit-msg").exists());
}

#[test]
fn check_message_file_accepts_and_rejects() {
    let repo = Repo::new();
    let good = repo.path().join("good.txt");
    std::fs::write(
        &good,
        "feat: add parser\n\n# Please enter the commit message\n",
    )
    .unwrap();
    let out = repo.cw(&["check", "--message-file", good.to_str().unwrap()]);
    assert!(out.status.success(), "{}", text(&out));

    let bad = repo.path().join("bad.txt");
    std::fs::write(&bad, "not conventional\n").unwrap();
    let out = repo.cw(&["check", "--message-file", bad.to_str().unwrap()]);
    assert!(!out.status.success(), "{}", text(&out));
}

#[cfg(unix)]
#[test]
fn installed_commit_msg_hook_blocks_a_bad_commit() {
    let repo = Repo::new();
    assert!(
        repo.cw(&["hooks", "install", "--hook", "commit-msg"])
            .status
            .success()
    );

    // The hook calls `cw` from PATH, so put the test binary's directory first.
    let bin_dir = std::path::Path::new(env!("CARGO_BIN_EXE_cw"))
        .parent()
        .unwrap();
    let path = format!("{}:{}", bin_dir.display(), std::env::var("PATH").unwrap());
    let run = |message: &str| {
        std::process::Command::new("git")
            .args(["commit", "-q", "--allow-empty", "-m", message])
            .current_dir(repo.path())
            .env("PATH", &path)
            .env("HOME", repo.path())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .output()
            .unwrap()
    };
    assert!(!run("nonsense").status.success());
    assert!(run("fix: a real change").status.success());
}
