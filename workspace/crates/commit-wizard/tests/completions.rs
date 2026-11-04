//! `cw completions` prints a script and needs no repository.

use assert_cmd::Command;

#[test]
fn prints_a_script_for_each_shell() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let dir = tempfile::tempdir().unwrap();
        let out = Command::cargo_bin("cw")
            .unwrap()
            .current_dir(dir.path())
            .args(["completions", shell])
            .output()
            .unwrap();
        assert!(out.status.success(), "{shell}");
        let script = String::from_utf8_lossy(&out.stdout);
        assert!(script.contains("cw"), "{shell}");
        assert!(
            script.contains("hooks") && script.contains("release"),
            "{shell}"
        );
    }
}
