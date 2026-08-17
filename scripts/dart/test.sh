#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

TARGETS=(
  "packages/dust_dart"
  "packages/dust_db_sqlite3"
  "packages/dust_server"
)

if [[ "${1:-}" == "--examples" ]]; then
  TARGETS=("examples/product_showcase" "examples/notes_mvc" "examples/todo_server")
elif [[ $# -gt 0 ]]; then
  echo "Usage: $0 [--examples]" >&2
  exit 2
fi

for target in "${TARGETS[@]}"; do
  echo "==> Dart pub get: $target"
  (cd "$target" && dart pub get)
done

if [[ "${1:-}" == "--examples" ]]; then
  for example in examples/product_showcase examples/notes_mvc examples/todo_server; do
    echo "==> Dust build --clean: $example"
    cargo run --quiet -p dust_cli -- build --clean --root "$example"
    echo "==> Dust check: $example"
    cargo run --quiet -p dust_cli -- check --root "$example"
  done

  for example in examples/notes_mvc examples/todo_server; do
    echo "==> Dust build --db: $example"
    cargo run --quiet -p dust_cli -- build --root "$example" --db
    echo "==> Dust check --db: $example"
    cargo run --quiet -p dust_cli -- check --root "$example" --db
  done
fi

for target in "${TARGETS[@]}"; do
  echo "==> Dart test: $target"
  (cd "$target" && dart test)
done
