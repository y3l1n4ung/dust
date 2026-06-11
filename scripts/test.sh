#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

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

DART_TEST_PACKAGES=(
  "packages/dust_dart"
  "packages/dust_db_sqlite3"
  "examples/product_showcase"
)

FLUTTER_TEST_PACKAGES=(
  "packages/dust_flutter"
  "examples/benchmark_project"
  "examples/shopping_app"
)

if ! has_cmd cargo; then
  echo "error: cargo is required to run Rust tests" >&2
  exit 1
fi

echo "==> Rust tests"
run cargo test --workspace --quiet

if ! has_cmd dart; then
  echo "warning: dart not found; skipping Dart/Flutter tests" >&2
  exit 0
fi

echo "==> Dust freshness checks"
run_dust_check "examples/product_showcase"
run_dust_check "examples/benchmark_project"
run_dust_check "examples/shopping_app"
run_dust_check "examples/shopping_app" --db

for package in "${DART_TEST_PACKAGES[@]}"; do
  run_dart_pub_get "$package"
  run_dart_test "$package"
done

if ! has_cmd flutter; then
  for package in "${FLUTTER_TEST_PACKAGES[@]}"; do
    echo "warning: flutter not found; skipped tests for $package" >&2
  done
  exit 0
fi

for package in "${FLUTTER_TEST_PACKAGES[@]}"; do
  run_flutter_pub_get "$package"
  run_flutter_test "$package"
done

echo "==> Tests complete"
