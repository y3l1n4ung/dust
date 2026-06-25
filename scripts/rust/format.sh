#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Rust format"

if [[ "${1:-}" == "--check" ]]; then
  cargo fmt --all -- --check
elif [[ $# -eq 0 ]]; then
  cargo fmt --all
else
  echo "Usage: $0 [--check]" >&2
  exit 2
fi
