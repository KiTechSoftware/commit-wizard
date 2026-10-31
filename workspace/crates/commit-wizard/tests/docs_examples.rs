//! Every complete TOML example in the user manual must load, so the docs cannot promise a
//! setting `cw` rejects.

mod common;

use std::path::{Path, PathBuf};

use common::{Repo, text};

fn pages(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("docs directory is readable") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            pages(&path, found);
        } else if path.extension().is_some_and(|e| e == "md") {
            found.push(path);
        }
    }
}

#[test]
fn every_complete_toml_example_in_the_manual_loads() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/pages");
    // The published package has no manual to check.
    if !docs.is_dir() {
        return;
    }

    let mut files = Vec::new();
    pages(&docs, &mut files);
    files.sort();

    let mut checked = 0;
    for file in files {
        let page = std::fs::read_to_string(&file).expect("page is readable");
        for (index, block) in page.split("```toml\n").skip(1).enumerate() {
            let block = block.split("```").next().unwrap_or("");
            // Fragments that show one table are not complete files.
            if !block.lines().any(|l| l.starts_with("version =")) {
                continue;
            }
            let repo = Repo::bare();
            repo.write(".cwizard.toml", block);
            let out = repo.cw(&["config", "list"]);
            let output = text(&out);
            assert!(
                out.status.success() && !output.contains("Parse error"),
                "{} example {} does not load:\n{output}",
                file.display(),
                index + 1
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 10,
        "expected to find the manual's examples, found {checked}"
    );
}
