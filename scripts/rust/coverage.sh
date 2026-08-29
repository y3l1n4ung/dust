#!/usr/bin/env bash
set -euo pipefail

# Measures Rust workspace line coverage and fails below a floor.
#
# Usage: scripts/rust/coverage.sh [floor-percent]

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

FLOOR="${1:-90}"
REPORT="coverage/rust.lcov"

echo "==> Rust coverage (floor ${FLOOR}%)"
mkdir -p "$(dirname "$REPORT")"

cargo llvm-cov nextest \
  --workspace \
  --all-features \
  --lcov \
  --output-path "$REPORT" \
  --fail-under-lines "$FLOOR"

echo "==> Coverage passed: $REPORT"
