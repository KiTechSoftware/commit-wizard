//! The crate's README is a copy of the repository README; they must not drift.

use std::path::Path;

#[test]
fn the_crate_readme_matches_the_repository_readme() {
    let crate_readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let repo_readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../README.md");

    // In the published package there is no repository above the crate to compare with.
    let Ok(source) = std::fs::read_to_string(&repo_readme) else {
        return;
    };
    let copy = std::fs::read_to_string(&crate_readme).expect("the crate README exists");

    assert_eq!(
        copy, source,
        "workspace/crates/commit-wizard/README.md differs from README.md; run scripts/sync-readme.sh"
    );
}

#[test]
fn the_readme_has_no_links_that_only_work_on_github() {
    // docs.rs and crates.io render the README away from the repository, so a relative
    // link such as (CONTRIBUTING.md) would be dead there.
    let readme = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .expect("the crate README exists");

    for (number, line) in readme.lines().enumerate() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let after = &rest[start + 2..];
            let target = after.split(')').next().unwrap_or("");
            assert!(
                target.starts_with("http") || target.starts_with('#'),
                "README line {}: relative link `{target}` will break on docs.rs; use a full URL",
                number + 1
            );
            rest = after;
        }
    }
}

#[test]
fn every_readme_code_block_names_its_language() {
    // An untagged fence in crate docs is compiled as a Rust doctest.
    let readme = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md"))
        .expect("the crate README exists");

    let mut inside = false;
    for (number, line) in readme.lines().enumerate() {
        if let Some(info) = line.trim_start().strip_prefix("```") {
            if !inside {
                assert!(
                    !info.trim().is_empty(),
                    "README line {}: a code block without a language becomes a doctest",
                    number + 1
                );
            }
            inside = !inside;
        }
    }
}
