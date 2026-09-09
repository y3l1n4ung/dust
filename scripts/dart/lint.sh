#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

# Read from the root pubspec workspace, so adding a package is one edit.
TARGETS=()
while IFS= read -r package; do
  TARGETS+=("$package")
done < <("$ROOT_DIR/scripts/workspace_packages.sh" dart)

if [[ "${1:-}" == "--examples" ]]; then
  TARGETS=("examples/product_showcase")
elif [[ $# -gt 0 ]]; then
  echo "Usage: $0 [--examples]" >&2
  exit 2
fi

for target in "${TARGETS[@]}"; do
  echo "==> Dart pub get: $target"
  (cd "$target" && dart pub get >/dev/null)
done

if [[ "${1:-}" == "--examples" ]]; then
  for example in examples/product_showcase; do
    echo "==> Dust build --clean: $example"
    cargo run --quiet -p dust_cli -- build --clean --root "$example"
    echo "==> Dust check: $example"
    cargo run --quiet -p dust_cli -- check --root "$example"
  done

fi

for target in "${TARGETS[@]}"; do
  echo "==> Dart analyze: $target"
  (cd "$target" && dart analyze --fatal-infos)
done
