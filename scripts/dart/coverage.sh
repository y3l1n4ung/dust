#!/usr/bin/env bash
set -euo pipefail

# Measures line coverage for one Dart package and fails below a floor.
#
# A coverage number nobody enforces drifts down one uncovered branch at a time,
# and the drop is invisible in review. This turns it into a failing check.
#
# Usage: scripts/dart/coverage.sh <package-path> [floor-percent]

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

TARGET="${1:-}"
FLOOR="${2:-100}"

if [[ -z "$TARGET" ]]; then
  echo "Usage: $0 <package-path> [floor-percent]" >&2
  exit 2
fi

echo "==> Dart coverage: $TARGET (floor ${FLOOR}%)"

(
  cd "$TARGET"
  rm -rf .coverage
  dart test --coverage=.coverage
  dart run coverage:format_coverage \
    --lcov \
    --in=.coverage \
    --out=.coverage/lcov.info \
    --report-on=lib
)

python3 scripts/check_lcov_floor.py "$TARGET/.coverage/lcov.info" "$FLOOR"

echo "==> Coverage passed"
