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

if [[ "$SCOPE" != packages ]]; then
  ./scripts/rust/test.sh
fi

if [[ "$SCOPE" == rust ]]; then
  echo "==> Tests complete"
  exit 0
fi

./scripts/dart/test.sh
./scripts/flutter/test.sh

if [[ "$SCOPE" == all ]]; then
  ./scripts/dart/test.sh --examples
  ./scripts/flutter/test.sh --examples
fi

echo "==> Tests complete"
