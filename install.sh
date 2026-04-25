#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_BIN="$ROOT_DIR/target/release/dust"
INSTALL_DIR="${DUST_INSTALL_DIR:-$HOME/.local/bin}"
INSTALL_BIN="$INSTALL_DIR/dust"

if [[ ! -x "$TARGET_BIN" ]]; then
  echo "building dust..."
  cargo build --release -p dust_cli --manifest-path "$ROOT_DIR/Cargo.toml"
fi

mkdir -p "$INSTALL_DIR"
cp "$TARGET_BIN" "$INSTALL_BIN"
chmod +x "$INSTALL_BIN"

echo "installed dust -> $INSTALL_BIN"
