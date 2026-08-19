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

python3 - "$TARGET" "$FLOOR" <<'PY'
import sys

target, floor = sys.argv[1], float(sys.argv[2])
total = covered = 0
uncovered = {}
current = None

with open(f'{target}/.coverage/lcov.info') as report:
    for line in report:
        line = line.rstrip('\n')
        if line.startswith('SF:'):
            current, missing = line[3:], []
        elif line.startswith('DA:'):
            number, count = line[3:].split(',')
            total += 1
            if int(count) > 0:
                covered += 1
            else:
                missing.append(number)
        elif line == 'end_of_record' and missing:
            uncovered[current] = missing

if total == 0:
    sys.exit('no coverage data; did the suite run?')

percent = covered * 100 / total
print(f'{covered}/{total} lines = {percent:.2f}%')

for path, missing in uncovered.items():
    print(f'  uncovered {path}: {", ".join(missing)}')

if percent + 1e-9 < floor:
    sys.exit(f'coverage {percent:.2f}% is below the {floor:.2f}% floor')
PY

echo "==> Coverage passed"
