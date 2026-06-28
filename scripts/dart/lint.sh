#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

TARGETS=(
  "packages/dust_dart"
  "packages/dust_db_sqlite3"
)

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
  echo "==> Dust build --clean: examples/product_showcase"
  cargo run --quiet -p dust_cli -- build --clean --root examples/product_showcase
  echo "==> Dust check: examples/product_showcase"
  cargo run --quiet -p dust_cli -- check --root examples/product_showcase
fi

for target in "${TARGETS[@]}"; do
  echo "==> Dart analyze: $target"
  (cd "$target" && dart analyze --fatal-infos)
done
