#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

(
  cd "${REPO_ROOT}/workspace"
  cargo clippy --workspace --all-features --all-targets -- -D warnings
)