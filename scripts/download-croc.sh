#!/usr/bin/env bash
# Downloads the croc binary for the current (or overridden) platform.
#
# Environment variables for cross-compilation overrides:
#   CROC_OS      - override uname -s  (e.g. "Linux", "Windows")
#   CROC_ARCH    - override uname -m  (e.g. "aarch64", "x86_64")
#   CROC_VERSION - pinned croc version (default: latest GitHub release)

set -euo pipefail

CROC_VERSION="${CROC_VERSION:-$(curl -sL https://api.github.com/repos/schollz/croc/releases/latest | grep '"tag_name":' | sed 's/.*"v\([^"]*\)".*/\1/')}"
DEST_DIR="$(cd "$(dirname "$0")/../src-tauri/binaries" && pwd)"

echo "Downloading croc v${CROC_VERSION} for current platform..."

OS="${CROC_OS:-$(uname -s)}"
ARCH="${CROC_ARCH:-$(uname -m)}"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  PLATFORM="Linux-64bit" ;;
      aarch64|arm64) PLATFORM="Linux-ARM64" ;;
      armv7l)  PLATFORM="Linux-ARM" ;;
      riscv64) PLATFORM="Linux-riscv64" ;;
      *)
        echo "Unsupported architecture: $ARCH for Linux"
        echo "Available Linux targets: x86_64, aarch64, armv7l"
        echo "You can manually cross-compile croc and place it in: $DEST_DIR"
        exit 1
        ;;
    esac
    EXT="tar.gz"
    ;;
  Darwin|macOS|ios|iOS)
    case "$ARCH" in
      x86_64)  PLATFORM="macOS-64bit" ;;
      aarch64|arm64) PLATFORM="macOS-ARM64" ;;
      *)
        echo "Unsupported architecture: $ARCH for macOS"
        exit 1
        ;;
    esac
    EXT="tar.gz"
    if [ "${CROC_OS:-}" = "ios" ] || [ "${CROC_OS:-}" = "iOS" ]; then
      echo "iOS detected — croc must be cross-compiled. Place the binary manually in: $DEST_DIR"
      echo "  Target triple: aarch64-apple-ios"
      mkdir -p "$DEST_DIR"
      exit 0
    fi
    ;;
  FreeBSD)
    case "$ARCH" in
      x86_64|amd64) PLATFORM="FreeBSD-64bit" ;;
      aarch64|arm64) PLATFORM="FreeBSD-ARM64" ;;
      *)
        echo "Unsupported architecture: $ARCH for FreeBSD"
        exit 1
        ;;
    esac
    EXT="tar.gz"
    ;;
  OpenBSD)
    case "$ARCH" in
      x86_64|amd64) PLATFORM="OpenBSD-64bit" ;;
      *)
        echo "Unsupported architecture: $ARCH for OpenBSD"
        exit 1
        ;;
    esac
    EXT="tar.gz"
    ;;
  NetBSD)
    case "$ARCH" in
      x86_64|amd64) PLATFORM="NetBSD-64bit" ;;
      *)
        echo "Unsupported architecture: $ARCH for NetBSD"
        exit 1
        ;;
    esac
    EXT="tar.gz"
    ;;
  DragonFly)
    PLATFORM="DragonFlyBSD-64bit"
    EXT="tar.gz"
    ;;
  CYGWIN*|MINGW*|MSYS*|Windows)
    case "$ARCH" in
      x86_64|amd64) PLATFORM="Windows-64bit" ;;
      aarch64|arm64)
        echo "Windows ARM64 is not available in croc releases."
        echo "You can manually cross-compile and place the binary in: $DEST_DIR"
        exit 1
        ;;
      *)
        echo "Unsupported architecture: $ARCH for Windows"
        exit 1
        ;;
    esac
    EXT="zip"
    ;;
  Android)
    echo "Android detected — croc must be cross-compiled for Android."
    echo "Place the binary (named 'croc') manually in: $DEST_DIR"
    echo "  Target triples: aarch64-linux-android, armv7-linux-androideabi, x86_64-linux-android"
    mkdir -p "$DEST_DIR"
    exit 0
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "You can manually download or cross-compile croc and place it in: $DEST_DIR"
    exit 1
    ;;
esac

FILENAME="croc_v${CROC_VERSION}_${PLATFORM}.${EXT}"
URL="https://github.com/schollz/croc/releases/download/v${CROC_VERSION}/${FILENAME}"

echo "  Target:  ${PLATFORM}"
echo "  URL:     ${URL}"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading..."
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
mkdir -p "$DEST_DIR"
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
