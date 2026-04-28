#!/usr/bin/env bash
set -euo pipefail

REPO="${DUST_REPO:-y3l1n4ung/dust}"
VERSION="${DUST_VERSION:-latest}"
INSTALL_DIR="${DUST_INSTALL_DIR:-$HOME/.local/bin}"
INSTALL_BIN="$INSTALL_DIR/dust"
TMP_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

detect_asset() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin)
      case "$arch" in
        arm64|aarch64) echo "dust-aarch64-apple-darwin.tar.gz" ;;
        x86_64) echo "dust-x86_64-apple-darwin.tar.gz" ;;
        *) echo "unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    Linux)
      case "$arch" in
        arm64|aarch64) echo "dust-aarch64-unknown-linux-gnu.tar.gz" ;;
        x86_64) echo "dust-x86_64-unknown-linux-gnu.tar.gz" ;;
        *) echo "unsupported architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "unsupported operating system: $os" >&2
      exit 1
      ;;
  esac
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    echo "sha256sum or shasum is required to verify release checksum" >&2
    exit 1
  fi
}

ASSET="$(detect_asset)"
if [[ "$VERSION" == "latest" ]]; then
  BASE_URL="https://github.com/$REPO/releases/latest/download"
else
  BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
fi
URL="$BASE_URL/$ASSET"
CHECKSUM_URL="$BASE_URL/SHA256SUMS.txt"

echo "downloading $URL"
curl -fsSL "$URL" -o "$TMP_DIR/$ASSET"
curl -fsSL "$CHECKSUM_URL" -o "$TMP_DIR/SHA256SUMS.txt"

EXPECTED_SHA="$(awk -v asset="$ASSET" '$2 == asset { print $1 }' "$TMP_DIR/SHA256SUMS.txt" | head -n 1)"
if [[ -z "$EXPECTED_SHA" ]]; then
  echo "could not find checksum for $ASSET" >&2
  exit 1
fi

ACTUAL_SHA="$(sha256_file "$TMP_DIR/$ASSET")"
ACTUAL_SHA_LOWER="$(printf '%s' "$ACTUAL_SHA" | tr '[:upper:]' '[:lower:]')"
EXPECTED_SHA_LOWER="$(printf '%s' "$EXPECTED_SHA" | tr '[:upper:]' '[:lower:]')"
if [[ "$ACTUAL_SHA_LOWER" != "$EXPECTED_SHA_LOWER" ]]; then
  echo "checksum mismatch for $ASSET" >&2
  echo "expected: $EXPECTED_SHA" >&2
  echo "actual:   $ACTUAL_SHA" >&2
  exit 1
fi
echo "verified $ASSET"

tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"

TARGET_BIN="$(find "$TMP_DIR" -type f -name dust | head -n 1)"
if [[ -z "$TARGET_BIN" ]]; then
  echo "could not find dust binary in release archive" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
cp "$TARGET_BIN" "$INSTALL_BIN"
chmod +x "$INSTALL_BIN"

echo "installed dust -> $INSTALL_BIN"
