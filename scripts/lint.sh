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
)

FLUTTER_PACKAGES=(
  "packages/dust_flutter"
)

DART_TARGETS=()
FLUTTER_TARGETS=()

if [[ "$SCOPE" != rust ]]; then
  DART_TARGETS=("${DART_PACKAGES[@]}")
  FLUTTER_TARGETS=("${FLUTTER_PACKAGES[@]}")

  if [[ "$SCOPE" == all ]]; then
    DART_TARGETS+=("examples/product_showcase")
    FLUTTER_TARGETS+=(
      "examples/benchmark_project"
      "examples/shopping_app"
    )
  fi
fi

if [[ "$SCOPE" != rust ]] && has_cmd dart; then
  for package in "${DART_TARGETS[@]}"; do
    run_dart_pub_get "$package"
  done

  if has_cmd flutter; then
    for package in "${FLUTTER_TARGETS[@]}"; do
      run_flutter_pub_get "$package"
    done
  else
    for package in "${FLUTTER_TARGETS[@]}"; do
      echo "warning: flutter not found; skipped analysis for $package" >&2
    done
  fi
fi

run ./scripts/format.sh --check --scope "$SCOPE"

if ! has_cmd cargo; then
  echo "error: cargo is required to lint Rust code" >&2
  exit 1
fi

echo "==> Rust clippy"
run cargo clippy --workspace --all-targets --all-features -- -D warnings

if [[ "$SCOPE" == rust ]]; then
  echo "==> Lint complete"
  exit 0
fi

if ! has_cmd dart; then
    echo "warning: dart not found; skipping Dart/Flutter analysis" >&2
    exit 0
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

for package in "${DART_TARGETS[@]}"; do
  run_dart_analyze "$package"
done

if ! has_cmd flutter; then
  exit 0
fi

for package in "${FLUTTER_TARGETS[@]}"; do
  run_flutter_analyze "$package"
done

echo "==> Lint complete"
