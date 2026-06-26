#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Rust clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::missing_docs_in_private_items -D missing_docs
