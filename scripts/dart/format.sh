#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

CHECK_MODE=false
# Read from the root pubspec workspace, so adding a package is one edit.
TARGETS=()
while IFS= read -r package; do
  TARGETS+=("$package")
done < <("$ROOT_DIR/scripts/workspace_packages.sh" dart)

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
