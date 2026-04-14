#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

"${SCRIPT_DIR}/cw-fmt.sh"
"${SCRIPT_DIR}/cw-lint.sh"
"${SCRIPT_DIR}/cw-test.sh"
"${SCRIPT_DIR}/cw-test-coverage.sh"
"${SCRIPT_DIR}/cw-deny.sh"

echo "✅ all local checks passed"