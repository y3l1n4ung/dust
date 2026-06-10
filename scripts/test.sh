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
    if has_cmd flutter; then
      (cd "$dir" && run flutter pub get)
    else
      (cd "$dir" && run dart pub get)
    fi
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
    if has_cmd flutter; then
      (cd "$dir" && run flutter test)
    else
      (cd "$dir" && run dart test)
    fi
  fi
}

run_flutter_test() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Flutter test: $dir"
    (cd "$dir" && run flutter test)
  fi
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
run cargo test --workspace

if ! has_cmd dart; then
  echo "warning: dart not found; skipping Dart/Flutter tests" >&2
  exit 0
fi

echo "==> Dust build: examples"
run cargo run -p dust_cli -- build --root examples/product_showcase
(cd examples/benchmark_project && run ./generate.sh --count 10)
run cargo run -p dust_cli -- build --root examples/benchmark_project
run cargo run -p dust_cli -- build --root examples/shopping_app

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