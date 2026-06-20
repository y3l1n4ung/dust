#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SCOPE=all

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scope)
      SCOPE="${2:-}"
      shift
      ;;
    --scope=*) SCOPE="${1#--scope=}" ;;
    *)
      echo "Usage: $0 [--scope rust|packages|all]" >&2
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
  echo "+ $*"
  "$@"
}

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

run_dart_pub_get() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Dart pub get: $dir"
    (cd "$dir" && run dart pub get)
  fi
}

run_flutter_pub_get() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Flutter pub get: $dir"
    (cd "$dir" && run flutter pub get)
  fi
}

run_dart_test() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Dart test: $dir"
    (cd "$dir" && run dart test)
  fi
}

run_flutter_test() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Flutter test: $dir"
    (cd "$dir" && run flutter test)
  fi
}

run_dust_check() {
  local root="$1"
  shift || true

  echo "==> Dust check: $root $*"
  run cargo run --quiet -p dust_cli -- check --root "$root" "$@"
}

has_cargo_nextest() {
  cargo nextest --version >/dev/null 2>&1
}

run_rust_tests() {
  if has_cargo_nextest; then
    run cargo nextest run --workspace
  else
    run cargo test --workspace --quiet
  fi
}

DART_TEST_PACKAGES=(
  "packages/dust_dart"
  "packages/dust_db_sqlite3"
)

FLUTTER_TEST_PACKAGES=(
  "packages/dust_flutter"
)

DART_TEST_TARGETS=()
FLUTTER_TEST_TARGETS=()

if [[ "$SCOPE" != rust ]]; then
  DART_TEST_TARGETS=("${DART_TEST_PACKAGES[@]}")
  FLUTTER_TEST_TARGETS=("${FLUTTER_TEST_PACKAGES[@]}")

  if [[ "$SCOPE" == all ]]; then
    DART_TEST_TARGETS+=("examples/product_showcase")
    FLUTTER_TEST_TARGETS+=(
      "examples/benchmark_project"
      "examples/shopping_app"
    )
  fi
fi

if ! has_cmd cargo; then
  echo "error: cargo is required to run Rust tests" >&2
  exit 1
fi

echo "==> Rust tests"
run_rust_tests

if [[ "$SCOPE" == rust ]]; then
  echo "==> Tests complete"
  exit 0
fi

if ! has_cmd dart; then
    echo "warning: dart not found; skipping Dart/Flutter tests" >&2
    exit 0
fi

for package in "${DART_TEST_TARGETS[@]}"; do
  run_dart_pub_get "$package"
done

if has_cmd flutter; then
  for package in "${FLUTTER_TEST_TARGETS[@]}"; do
    run_flutter_pub_get "$package"
  done
else
  for package in "${FLUTTER_TEST_TARGETS[@]}"; do
    echo "warning: flutter not found; skipped tests for $package" >&2
  done
fi

if [[ "$SCOPE" == all ]]; then
  echo "==> Dust freshness checks"
  run_dust_check "examples/product_showcase"
  if has_cmd flutter; then
    run_dust_check "examples/benchmark_project"
    run_dust_check "examples/shopping_app"
    run_dust_check "examples/shopping_app" --db
  else
    echo "warning: flutter not found; skipped Flutter Dust freshness checks" >&2
  fi
fi

for package in "${DART_TEST_TARGETS[@]}"; do
  run_dart_test "$package"
done

if ! has_cmd flutter; then
  exit 0
fi

for package in "${FLUTTER_TEST_TARGETS[@]}"; do
  run_flutter_test "$package"
done

echo "==> Tests complete"
