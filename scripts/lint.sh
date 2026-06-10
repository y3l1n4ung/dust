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

run_dart_analyze() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Dart analyze: $dir"
    if has_cmd flutter; then
      (cd "$dir" && run flutter analyze)
    else
      (cd "$dir" && run dart analyze)
    fi
  fi
}

run_flutter_analyze() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Flutter analyze: $dir"
    (cd "$dir" && run flutter analyze)
  fi
}

DART_PACKAGES=(
  "packages/dust_dart"
  "packages/dust_db_sqlite3"
  "examples/product_showcase"
)

FLUTTER_PACKAGES=(
  "packages/dust_flutter"
  "examples/benchmark_project"
  "examples/shopping_app"
)

run ./scripts/format.sh --check

if ! has_cmd cargo; then
  echo "error: cargo is required to lint Rust code" >&2
  exit 1
fi

echo "==> Rust clippy"
run cargo clippy --workspace --all-targets --all-features -- -D warnings

if ! has_cmd dart; then
  echo "warning: dart not found; skipping Dart/Flutter analysis" >&2
  exit 0
fi

echo "==> Dust build: examples"
run cargo run -p dust_cli -- build --root examples/product_showcase
(cd examples/benchmark_project && run ./generate.sh --count 10)
run cargo run -p dust_cli -- build --root examples/benchmark_project
run cargo run -p dust_cli -- build --root examples/shopping_app

for package in "${DART_PACKAGES[@]}"; do
  run_dart_pub_get "$package"
  run_dart_analyze "$package"
done

if ! has_cmd flutter; then
  for package in "${FLUTTER_PACKAGES[@]}"; do
    echo "warning: flutter not found; skipped analysis for $package" >&2
  done
  exit 0
fi

for package in "${FLUTTER_PACKAGES[@]}"; do
  run_flutter_pub_get "$package"
  run_flutter_analyze "$package"
done

echo "==> Lint complete"