#!/bin/bash

set -e

echo "Building xkey in release mode..."
cargo build --release

echo "Installing binary to /usr/libexec/..."
sudo cp target/release/xkey /usr/libexec/ibus-engine-xkey

echo "Installing component XML to /usr/share/ibus/component/..."
sudo mkdir -p /usr/share/ibus/component
sudo cp xkey.xml /usr/share/ibus/component/xkey.xml

echo "Restarting IBus..."
ibus restart || echo "Warning: Could not restart IBus. Please run 'ibus restart' manually."

sleep 2

echo "Configuring GNOME input sources..."
CURRENT_SOURCES=$(gsettings get org.gnome.desktop.input-sources sources)
if [[ "$CURRENT_SOURCES" != *"'ibus', 'xkey'"* ]]; then
    NEW_SOURCES=$(echo "$CURRENT_SOURCES" | sed "s/\]$/, ('ibus', 'xkey')\]/")
    gsettings set org.gnome.desktop.input-sources sources "$NEW_SOURCES"
    echo "Added xkey to GNOME input sources."
else
    echo "xkey is already in GNOME input sources."
fi

echo "Activating xkey engine..."
ibus engine xkey || echo "Warning: Could not set ibus engine to xkey."

echo "Installation complete!"
echo "XKey has been installed and activated."
