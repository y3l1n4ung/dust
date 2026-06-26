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

run_formatter() {
  if [[ "$CHECK_MODE" == true ]]; then
    "$1" --check
  else
    "$1"
  fi
}

if [[ "$SCOPE" != packages ]]; then
  run_formatter ./scripts/rust/format.sh
fi

if [[ "$SCOPE" == rust ]]; then
  echo "==> Format complete"
  exit 0
fi

run_formatter ./scripts/dart/format.sh
run_formatter ./scripts/flutter/format.sh

if [[ "$SCOPE" == all ]]; then
  if [[ "$CHECK_MODE" == true ]]; then
    ./scripts/dart/format.sh --check --examples
    ./scripts/flutter/format.sh --check --examples
  else
    ./scripts/dart/format.sh --examples
    ./scripts/flutter/format.sh --examples
  fi
fi

echo "==> Format complete"
