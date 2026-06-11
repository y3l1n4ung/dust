#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

CHECK_MODE=false

case "${1:-}" in
  "") ;;
  --check|-c) CHECK_MODE=true ;;
  *)
    echo "Usage: $0 [--check]" >&2
    exit 2
    ;;
esac

run() {
  "$@"
}

run_dart_format() {
  local dir="$1"

  if [[ ! -d "$dir" ]]; then
    echo "warning: $dir not found; skipping" >&2
    return
  fi

  if [[ ! -f "$dir/pubspec.yaml" ]]; then
    echo "warning: $dir has no pubspec.yaml; skipping" >&2
    return
  fi

  echo "==> Dart format: $dir"

  (
    cd "$dir"
    if [[ "$CHECK_MODE" == true ]]; then
      run find . -name "*.dart" \
        ! -path "*/.dart_tool/*" \
        ! -name "*.g.dart" \
        ! -name "*.freezed.dart" \
        ! -path "*/generated/*" \
        ! -path "*/generated_models/*" \
        -exec dart format --output=none --set-exit-if-changed {} +
    else
      run find . -name "*.dart" \
        ! -path "*/.dart_tool/*" \
        ! -name "*.g.dart" \
        ! -name "*.freezed.dart" \
        ! -path "*/generated/*" \
        ! -path "*/generated_models/*" \
        -exec dart format {} +
    fi
  )
}

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to format Rust code" >&2
  exit 1
fi

echo "==> Rust format"
if [[ "$CHECK_MODE" == true ]]; then
  run cargo fmt --all -- --check
else
  run cargo fmt --all
fi

if ! command -v dart >/dev/null 2>&1; then
  echo "warning: dart not found; skipping Dart/Flutter formatting" >&2
  exit 0
fi

echo "==> Dart/Flutter package format"
run_dart_format "packages/dust_dart"
run_dart_format "packages/dust_db_sqlite3"
run_dart_format "packages/dust_flutter"
run_dart_format "examples/product_showcase"
run_dart_format "examples/benchmark_project"
run_dart_format "examples/shopping_app"

echo "==> Format complete"
