//! End-to-end tests for `cw commit --ai`, with a fake `copilot` on PATH.

mod common;

#[cfg(unix)]
mod unix {
    use super::common::{Repo, text};
    use std::{os::unix::fs::PermissionsExt, process::Output};

    fn enable_ai(repo: &Repo, commit: bool) {
        let config = repo.path().join(".cwizard.toml");
        let body = std::fs::read_to_string(&config)
            .unwrap()
            .replace("[ai]\nenabled = false", "[ai]\nenabled = true")
            .replace("commit = false", &format!("commit = {commit}"));
        std::fs::write(&config, body).unwrap();
        repo.git(&["add", "-A"]);
        repo.commit("chore: enable ai");
    }

    /// Run `cw` with a `copilot` that prints `answer`.
    fn cw_with_answer(repo: &Repo, answer: &str, args: &[&str]) -> Output {
        let bin = tempfile::tempdir().unwrap();
        let script = bin.path().join("copilot");
        std::fs::write(&script, format!("#!/bin/sh\ncat <<'EOF'\n{answer}\nEOF\n")).unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = format!(
            "{}:{}",
            bin.path().display(),
            std::env::var("PATH").unwrap()
        );
        std::process::Command::new(env!("CARGO_BIN_EXE_cw"))
            .args(args)
            .current_dir(repo.path())
            .env("PATH", path)
            .env("HOME", repo.path())
            .env("NO_COLOR", "1")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    }

    fn stage_a_file(repo: &Repo) {
        std::fs::write(repo.path().join("a.txt"), "hello\n").unwrap();
        repo.git(&["add", "a.txt"]);
    }

    #[test]
    fn ai_writes_the_commit() {
        let repo = Repo::new();
        enable_ai(&repo, true);
        stage_a_file(&repo);
        let out = cw_with_answer(
            &repo,
            "feat(files): add a greeting\n\nAdds a.txt.",
            &["commit", "--ai", "--yes"],
        );
        assert!(out.status.success(), "{}", text(&out));
        assert_eq!(
            repo.git(&["log", "-1", "--format=%s"]),
            "feat(files): add a greeting"
        );
        assert!(
            repo.git(&["log", "-1", "--format=%b"])
                .contains("Adds a.txt.")
        );
    }

    #[test]
    fn flags_win_over_the_ai() {
        let repo = Repo::new();
        enable_ai(&repo, true);
        stage_a_file(&repo);
        let out = cw_with_answer(
            &repo,
            "feat: from the ai",
            &["commit", "--ai", "--yes", "-t", "fix"],
        );
        assert!(out.status.success(), "{}", text(&out));
        assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "fix: from the ai");
    }

    #[test]
    fn ai_is_refused_unless_enabled() {
        let repo = Repo::new();
        stage_a_file(&repo);
        let out = cw_with_answer(&repo, "feat: x", &["commit", "--ai", "--yes"]);
        assert!(!out.status.success(), "{}", text(&out));
        assert!(text(&out).contains("[ai]"), "{}", text(&out));
    }

    #[test]
    fn a_reply_that_is_not_a_commit_message_fails_cleanly() {
        let repo = Repo::new();
        enable_ai(&repo, true);
        stage_a_file(&repo);
        let out = cw_with_answer(&repo, "Sure! Here you go", &["commit", "--ai", "--yes"]);
        assert!(!out.status.success(), "{}", text(&out));
        assert_eq!(repo.git(&["log", "-1", "--format=%s"]), "chore: enable ai");
    }

    #[test]
    fn nothing_staged_is_an_error() {
        let repo = Repo::new();
        enable_ai(&repo, true);
        let out = cw_with_answer(&repo, "feat: x", &["commit", "--ai", "--yes"]);
        assert!(!out.status.success(), "{}", text(&out));
    }
}
