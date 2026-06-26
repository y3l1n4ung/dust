#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

dart --disable-analytics >/dev/null

CHECK_MODE=false
TARGETS=(
  "packages/dust_dart"
  "packages/dust_db_sqlite3"
)

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check) CHECK_MODE=true ;;
    --examples) TARGETS=("examples/product_showcase") ;;
    *)
      echo "Usage: $0 [--check] [--examples]" >&2
      exit 2
      ;;
  esac
  shift
done

for target in "${TARGETS[@]}"; do
  echo "==> Dart format: $target"
  if [[ "$CHECK_MODE" == true ]]; then
    (
      cd "$target"
      find . -name "*.dart" \
        ! -path "*/.dart_tool/*" \
        ! -name "*.g.dart" \
        ! -name "*.freezed.dart" \
        ! -path "*/generated/*" \
        ! -path "*/generated_models/*" \
        -exec dart format --output=none --set-exit-if-changed {} +
    )
  else
    (
      cd "$target"
      find . -name "*.dart" \
        ! -path "*/.dart_tool/*" \
        ! -name "*.g.dart" \
        ! -name "*.freezed.dart" \
        ! -path "*/generated/*" \
        ! -path "*/generated_models/*" \
        -exec dart format {} +
    )
  fi
done
