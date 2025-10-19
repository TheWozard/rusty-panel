#!/bin/bash
# Install rusty-panel autostart for KDE/GNOME and other desktop environments
# This method uses the XDG autostart directory which is supported by most desktop environments

set -e

AUTOSTART_DIR="$HOME/.config/autostart"
DESKTOP_FILE="$AUTOSTART_DIR/rusty-panel.desktop"
BINARY_PATH="$HOME/.cargo/bin/rusty-panel"
mkdir -p "$AUTOSTART_DIR"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Warning: rusty-panel binary not found at $BINARY_PATH"
    echo "Make sure to install rusty-panel first with: cargo install --path ."
fi

# Write desktop file
echo "Installing autostart file to $AUTOSTART_DIR/"
cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Type=Application
Name=Rusty Panel
Comment=HID Device Handler for PC Panel
Exec=$BINARY_PATH
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
X-KDE-autostart-after=panel
Terminal=false
EOF
chmod +x "$DESKTOP_FILE"

# Success
echo "rusty-panel autostart installed successfully."