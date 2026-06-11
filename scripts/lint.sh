#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

run() {
  "$@"
}

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

run_dart_pub_get() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Dart pub get: $dir"
    (cd "$dir" && run dart pub get >/dev/null)
  fi
}

run_flutter_pub_get() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Flutter pub get: $dir"
    (cd "$dir" && run flutter pub get >/dev/null)
  fi
}

run_dart_analyze() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Dart analyze: $dir"
    (cd "$dir" && run dart analyze --fatal-infos)
  fi
}

run_flutter_analyze() {
  local dir="$1"

  if [[ -f "$dir/pubspec.yaml" ]]; then
    echo "==> Flutter analyze: $dir"
    (cd "$dir" && run flutter analyze --no-pub)
  fi
}

run_dust_check() {
  local root="$1"
  shift || true

  echo "==> Dust check: $root $*"
  run cargo run --quiet -p dust_cli -- check --root "$root" "$@"
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

echo "==> Dust freshness checks"
run_dust_check "examples/product_showcase"
run_dust_check "examples/benchmark_project"
run_dust_check "examples/shopping_app"
run_dust_check "examples/shopping_app" --db

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
