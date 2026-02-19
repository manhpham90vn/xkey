#!/bin/bash
# =============================================================================
# XKey Uninstallation Script
# =============================================================================
#
# This script removes the XKey Vietnamese Telex input method from the system.
# It performs the following steps:
# 1. Remove the binary from /usr/libexec/
# 2. Remove the IBus component XML from /usr/share/ibus/component/
# 3. Remove xkey from GNOME input sources
# 4. Restart IBus daemon
#
# Usage:
#   ./linux/clean.sh
#
# =============================================================================

set -euo pipefail

# =============================================================================
# Helper Functions
# =============================================================================

# Ensure D-Bus session environment is properly set
ensure_dbus_env() {
  export XDG_RUNTIME_DIR="/run/user/$(id -u)"
  export DBUS_SESSION_BUS_ADDRESS="unix:path=${XDG_RUNTIME_DIR}/bus"
}

# Restart the IBus daemon
# Tries systemd first (for GNOME), falls back to ibus CLI
restart_ibus() {
  # Method 1: Try systemd user service (standard on GNOME)
  if systemctl --user status org.freedesktop.IBus.session.GNOME.service >/dev/null 2>&1; then
    systemctl --user restart org.freedesktop.IBus.session.GNOME.service || true
    return 0
  fi
  
  # Method 2: Fall back to ibus CLI tool
  ibus restart || true
}

# =============================================================================
# Main Uninstallation Process
# =============================================================================

echo "=== XKey Uninstallation ==="
echo ""

# Set up environment
ensure_dbus_env

# Step 1: Remove installed files
echo "[1/3] Removing binary and component files..."
sudo rm -f /usr/share/ibus/component/xkey.xml /usr/libexec/ibus-engine-xkey
echo "Removed:"
echo "  - /usr/libexec/ibus-engine-xkey"
echo "  - /usr/share/ibus/component/xkey.xml"

# Step 2: Remove xkey from GNOME input sources
echo ""
echo "[2/3] Reconfiguring GNOME input sources..."
CURRENT_SOURCES=$(gsettings get org.gnome.desktop.input-sources sources)
if [[ "${CURRENT_SOURCES}" == *"('ibus', 'xkey')"* ]]; then
  # Remove xkey from the list using multiple sed patterns to handle different positions
  NEW_SOURCES=$(echo "${CURRENT_SOURCES}" \
    | sed "s/, ('ibus', 'xkey')//g" \
    | sed "s/('ibus', 'xkey'), //g" \
    | sed "s/('ibus', 'xkey')//g")
  gsettings set org.gnome.desktop.input-sources sources "${NEW_SOURCES}"
  echo "Removed xkey from GNOME input sources."
else
  echo "xkey is not in GNOME input sources (nothing to remove)."
fi

# Step 3: Restart IBus to apply changes
echo ""
echo "[3/3] Restarting IBus..."
restart_ibus

# =============================================================================
# Uninstallation Complete
# =============================================================================

echo ""
echo "=== Cleanup complete! ==="
echo ""
echo "XKey has been uninstalled from your system."
echo ""
