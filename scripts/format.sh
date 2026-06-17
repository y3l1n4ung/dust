#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

CHECK_MODE=false
SCOPE=all

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check|-c) CHECK_MODE=true ;;
    --scope)
      SCOPE="${2:-}"
      shift
      ;;
    --scope=*) SCOPE="${1#--scope=}" ;;
    *)
      echo "Usage: $0 [--check] [--scope rust|packages|all]" >&2
      exit 2
      ;;
  esac
  shift
done

case "$SCOPE" in
  rust|packages|all) ;;
  *)
    echo "error: unsupported scope `$SCOPE`; expected rust, packages, or all" >&2
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

if [[ "$SCOPE" == rust ]]; then
  echo "==> Format complete"
  exit 0
fi

echo "==> Dart/Flutter package format"
FORMAT_DIRS=(
  "packages/dust_dart"
  "packages/dust_db_sqlite3"
  "packages/dust_flutter"
)

if [[ "$SCOPE" == all ]]; then
  FORMAT_DIRS+=(
    "examples/product_showcase"
    "examples/benchmark_project"
    "examples/shopping_app"
  )
fi

for dir in "${FORMAT_DIRS[@]}"; do
  run_dart_format "$dir"
done

echo "==> Format complete"
