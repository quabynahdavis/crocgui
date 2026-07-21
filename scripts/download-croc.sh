#!/usr/bin/env bash
set -euo pipefail

CROC_VERSION="${CROC_VERSION:-$(curl -sL https://api.github.com/repos/schollz/croc/releases/latest | grep '"tag_name":' | sed 's/.*"v\([^"]*\)".*/\1/')}"
DEST_DIR="$(cd "$(dirname "$0")/../src-tauri/binaries" && pwd)"

echo "Downloading croc v${CROC_VERSION} for current platform..."

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  PLATFORM="Linux-64bit" ;;
      aarch64) PLATFORM="Linux-ARM64" ;;
      armv7l)  PLATFORM="Linux-ARM" ;;
      *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
    esac
    EXT="tar.gz"
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  PLATFORM="macOS-64bit" ;;
      arm64)   PLATFORM="macOS-ARM64" ;;
      *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
    esac
    EXT="tar.gz"
    ;;
  FreeBSD)
    PLATFORM="FreeBSD-64bit"
    EXT="tar.gz"
    ;;
  DragonFly)
    PLATFORM="DragonFlyBSD-64bit"
    EXT="tar.gz"
    ;;
  CYGWIN*|MINGW*|MSYS*)
    PLATFORM="Windows-64bit"
    EXT="zip"
    ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

FILENAME="croc_v${CROC_VERSION}_${PLATFORM}.${EXT}"
URL="https://github.com/schollz/croc/releases/download/v${CROC_VERSION}/${FILENAME}"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading ${URL}..."
curl -sL "$URL" -o "$TMPDIR/$FILENAME"

echo "Extracting..."
case "$EXT" in
  tar.gz)
    tar -xzf "$TMPDIR/$FILENAME" -C "$TMPDIR"
    ;;
  zip)
    unzip -q "$TMPDIR/$FILENAME" -d "$TMPDIR"
    ;;
esac

# croc binary is at the root of the archive
if [ -f "$TMPDIR/croc" ]; then
  cp "$TMPDIR/croc" "$DEST_DIR/croc"
elif [ -f "$TMPDIR/croc.exe" ]; then
  cp "$TMPDIR/croc.exe" "$DEST_DIR/croc.exe"
else
  # Try to find it
  find "$TMPDIR" -name "croc*" -type f -executable -exec cp {} "$DEST_DIR/" \;
fi

chmod +x "$DEST_DIR/croc" 2>/dev/null || true

echo "Done! Binary saved to ${DEST_DIR}/"
ls -la "$DEST_DIR/"
