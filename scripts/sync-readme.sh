#!/usr/bin/env bash
# The repository README is the source. The crate carries a copy, because a
# crate can only package files inside its own directory, and that copy is what
# crates.io and docs.rs show. A test fails if the two differ.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

cp "${REPO_ROOT}/README.md" "${REPO_ROOT}/workspace/crates/commit-wizard/README.md"
echo "copied README.md into workspace/crates/commit-wizard/"
