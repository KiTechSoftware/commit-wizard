#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

THRESHOLD_LINES="${THRESHOLD_LINES:-70}"

(
  cd "${REPO_ROOT}/workspace"
  cargo llvm-cov \
    --workspace \
    --all-features \
    --fail-under-lines "${THRESHOLD_LINES}"
)