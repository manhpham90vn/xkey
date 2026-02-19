#!/bin/bash
# =============================================================================
# XKey macOS Installation Script
# =============================================================================
#
# This script installs the XKey Vietnamese Telex input method on macOS.
# It performs the following steps:
# 1. Build the project in release mode
# 2. Create the .app bundle structure
# 3. Install to ~/Library/Input Methods/
#
# Prerequisites:
# - Rust toolchain (for building)
# - macOS
#
# Usage:
#   ./macos/install.sh
#
# After installation, log out and log back in, then add XKey via:
#   System Settings > Keyboard > Input Sources > + > Vietnamese > XKey
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="XKey.app"
INSTALL_DIR="$HOME/Library/Input Methods"
APP_BUNDLE="$INSTALL_DIR/$APP_NAME"

# Print error message and exit
die() { echo "ERROR: $*" >&2; exit 1; }

echo "=== XKey macOS Installation ==="
echo ""

# Step 1: Build the project in release mode
echo "[1/3] Building xkey in release mode..."
if ! command -v cargo >/dev/null 2>&1; then
  die "Rust is not installed. Please install it from https://rustup.rs/"
fi
cd "$PROJECT_DIR"
cargo build --release

# Step 2: Create the .app bundle
echo "[2/3] Creating app bundle..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary
cp target/release/xkey "$APP_BUNDLE/Contents/MacOS/xkey"

# Copy Info.plist
cp "$SCRIPT_DIR/Info.plist" "$APP_BUNDLE/Contents/Info.plist"

# Create PkgInfo
echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

echo "[3/3] Installed to: $APP_BUNDLE"
echo ""
echo "=== Installation complete! ==="
echo ""
echo "To activate XKey:"
echo "  1. Log out and log back in (or restart)"
echo "  2. Go to System Settings > Keyboard > Input Sources"
echo "  3. Click '+' > Vietnamese > XKey Vietnamese Telex"
echo "  4. Switch input methods using Ctrl+Space or globe key"
echo ""
