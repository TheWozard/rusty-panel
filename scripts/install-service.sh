#!/bin/bash
# Install rusty-panel systemd user service

set -e

USER_SYSTEMD_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="$USER_SYSTEMD_DIR/rusty-panel.service"

# Create systemd user directory if it doesn't exist
mkdir -p "$USER_SYSTEMD_DIR"

# Write service file
echo "Installing service file to $USER_SYSTEMD_DIR/"
cat > "$SERVICE_FILE" << 'EOF'
[Unit]
Description=Rusty Panel HID Device Handler
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/rusty-panel

[Install]
WantedBy=default.target
EOF

# Enable and start the service
echo "Enabling rusty-panel service..."
systemctl --user enable rusty-panel.service
systemctl --user start rusty-panel.service

echo "rusty-panel service installed and started."
