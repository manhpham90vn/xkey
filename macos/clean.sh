#!/bin/bash
# =============================================================================
# XKey macOS Uninstallation Script
# =============================================================================
#
# This script removes the XKey Vietnamese Telex input method from macOS.
# It removes the .app bundle from ~/Library/Input Methods/
#
# After uninstallation, log out and log back in to complete removal.
#
# Usage:
#   ./macos/clean.sh
#
# =============================================================================

set -euo pipefail

APP_NAME="XKey.app"
INSTALL_DIR="$HOME/Library/Input Methods"
APP_BUNDLE="$INSTALL_DIR/$APP_NAME"

echo "=== XKey macOS Uninstallation ==="
echo ""

# Step 1: Remove the app bundle
echo "[1/1] Removing $APP_BUNDLE..."
if [[ -d "$APP_BUNDLE" ]]; then
  rm -rf "$APP_BUNDLE"
  echo "Removed: $APP_BUNDLE"
else
  echo "XKey is not installed (nothing to remove)."
fi

# =============================================================================
# Uninstallation Complete
# =============================================================================

echo ""
echo "=== Cleanup complete! ==="
echo ""
echo "XKey has been uninstalled."
echo "Please log out and log back in to complete removal."
echo ""
