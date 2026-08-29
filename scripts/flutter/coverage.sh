#!/usr/bin/env bash
set -euo pipefail

# Measures line coverage for one Flutter package and fails below a floor.
#
# Usage: scripts/flutter/coverage.sh <package-path> [floor-percent]

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

TARGET="${1:-}"
FLOOR="${2:-100}"

if [[ -z "$TARGET" ]]; then
  echo "Usage: $0 <package-path> [floor-percent]" >&2
  exit 2
fi

echo "==> Flutter coverage: $TARGET (floor ${FLOOR}%)"

(
  cd "$TARGET"
  flutter test --coverage
)

python3 scripts/check_lcov_floor.py "$TARGET/coverage/lcov.info" "$FLOOR"

echo "==> Coverage passed"
