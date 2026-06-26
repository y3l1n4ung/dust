#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

TARGETS=(
  "packages/dust_flutter"
)

if [[ "${1:-}" == "--examples" ]]; then
  TARGETS=(
    "examples/benchmark_project"
    "examples/shopping_app"
  )
elif [[ $# -gt 0 ]]; then
  echo "Usage: $0 [--examples]" >&2
  exit 2
fi

for target in "${TARGETS[@]}"; do
  echo "==> Flutter pub get: $target"
  (cd "$target" && flutter --suppress-analytics pub get)
done

if [[ "${1:-}" == "--examples" ]]; then
  echo "==> Dust build --clean: examples/benchmark_project"
  cargo run --quiet -p dust_cli -- build --clean --root examples/benchmark_project
  echo "==> Dust check: examples/benchmark_project"
  cargo run --quiet -p dust_cli -- check --root examples/benchmark_project
  echo "==> Dust build --clean: examples/shopping_app"
  cargo run --quiet -p dust_cli -- build --clean --root examples/shopping_app
  echo "==> Dust check: examples/shopping_app"
  cargo run --quiet -p dust_cli -- check --root examples/shopping_app
  echo "==> Dust build: examples/shopping_app --db"
  cargo run --quiet -p dust_cli -- build --root examples/shopping_app --db
  echo "==> Dust check: examples/shopping_app --db"
  cargo run --quiet -p dust_cli -- check --root examples/shopping_app --db
fi

for target in "${TARGETS[@]}"; do
  echo "==> Flutter test: $target"
  (cd "$target" && flutter --suppress-analytics test)
done
