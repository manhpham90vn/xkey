#!/bin/bash
# =============================================================================
# XKey Installation Script
# =============================================================================
#
# This script installs the XKey Vietnamese Telex input method for IBus.
# It performs the following steps:
# 1. Build the project in release mode
# 2. Install the binary to /usr/libexec/
# 3. Install the IBus component XML to /usr/share/ibus/component/
# 4. Restart IBus daemon
# 5. Add xkey to GNOME input sources
# 6. Activate xkey as the current input method
#
# Prerequisites:
# - Rust toolchain (for building)
# - IBus daemon installed and running
# - GNOME desktop environment (for gsettings integration)
#
# Usage:
#   ./linux/install.sh
#
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# =============================================================================
# Helper Functions
# =============================================================================

# Print error message and exit with code 1
die() { echo "ERROR: $*" >&2; exit 1; }

# Load IBus environment variables from the bus file
# IBus stores its connection information in ~/.config/ibus/bus/
load_ibus_env() {
  local busfile=""
  busfile="$(ls -t ~/.config/ibus/bus/* 2>/dev/null | head -n 1 || true)"
  if [[ -n "$busfile" ]]; then
    # Source the bus file to get IBUS_ADDRESS and other env vars
    # shellcheck source=/dev/null
    . "$busfile"
  fi
}

# Ensure D-Bus session environment is properly set
# This is required for communicating with IBus via D-Bus
ensure_dbus_env() {
  export XDG_RUNTIME_DIR="/run/user/$(id -u)"
  export DBUS_SESSION_BUS_ADDRESS="unix:path=${XDG_RUNTIME_DIR}/bus"
}

# Try to determine IBUS_ADDRESS from the running ibus-daemon process
# This is more reliable than reading config files under GNOME
ensure_ibus_address() {
  # Method 1: Read from running ibus-daemon's environment (most reliable)
  local pid
  pid="$(pgrep -n ibus-daemon || true)"
  if [[ -n "${pid}" ]] && [[ -r "/proc/${pid}/environ" ]]; then
    local line
    line="$(tr '\0' '\n' < "/proc/${pid}/environ" | grep '^IBUS_ADDRESS=' || true)"
    if [[ -n "${line}" ]]; then
      export IBUS_ADDRESS="${line#IBUS_ADDRESS=}"
      return 0
    fi
  fi

  # Method 2: Ask the ibus CLI tool (may fail if environment is broken)
  if command -v ibus >/dev/null 2>&1; then
    local addr
    addr="$(ibus address 2>/dev/null || true)"
    if [[ -n "${addr}" ]]; then
      export IBUS_ADDRESS="${addr}"
      return 0
    fi
  fi

  # Not fatal for build/copy, but will affect restart/engine steps
  return 1
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
# Main Installation Process
# =============================================================================

echo "=== XKey Installation ==="
echo ""

# Step 1: Set up environment
ensure_dbus_env
ensure_ibus_address || echo "Warning: Could not determine IBUS_ADDRESS (will still install files)."

# Step 2: Build the project in release mode
echo "[1/6] Building xkey in release mode..."
if ! command -v cargo >/dev/null 2>&1; then
  die "Rust is not installed. Please install it from https://rustup.rs/ before running this script."
fi
cd "$PROJECT_DIR"
cargo build --release

# Step 3: Install the binary
echo "[2/6] Installing binary to /usr/libexec/..."
sudo install -m 0755 "$PROJECT_DIR/target/release/xkey" /usr/libexec/ibus-engine-xkey

# Step 4: Install the IBus component XML
echo "[3/6] Installing component XML to /usr/share/ibus/component/..."
sudo install -D -m 0644 "$SCRIPT_DIR/xkey.xml" /usr/share/ibus/component/xkey.xml

# Step 5: Restart IBus daemon to pick up the new engine
echo "[4/6] Restarting IBus..."
restart_ibus
sleep 2
load_ibus_env

# Verify that the ibus CLI can now see the engine
if ! ibus list-engine >/dev/null 2>&1; then
  echo "Warning: 'ibus' CLI still can't connect. Try running inside GNOME terminal, or relogin."
fi

# Step 6: Configure GNOME input sources
echo "[5/6] Configuring GNOME input sources..."
CURRENT_SOURCES=$(gsettings get org.gnome.desktop.input-sources sources)
if [[ "${CURRENT_SOURCES}" != *"('ibus', 'xkey')"* ]]; then
  # Add xkey to the list of input sources
  NEW_SOURCES=$(echo "${CURRENT_SOURCES}" | sed "s/\]$/, ('ibus', 'xkey')]/")
  gsettings set org.gnome.desktop.input-sources sources "${NEW_SOURCES}"
  echo "Added xkey to GNOME input sources."
else
  echo "xkey is already in GNOME input sources."
fi

# Step 7: Activate xkey as the current input method
echo "[6/6] Activating xkey engine..."
load_ibus_env
if ! ibus engine xkey; then
  echo ""
  echo "Warning: Could not set ibus engine to xkey."
  echo "Troubleshooting commands:"
  echo "  busctl --user list | grep org.freedesktop.IBus"
  echo "  journalctl --user -b | grep -i ibus | tail -n 80"
fi

# =============================================================================
# Installation Complete
# =============================================================================

echo ""
echo "=== Installation complete! ==="
echo ""
echo "XKey has been installed and activated."
echo "Switch input methods using Super+Space or the GNOME top bar."
echo ""
