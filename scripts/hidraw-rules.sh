#!/bin/bash

# This script sets up udev rules for hidraw devices used by rusty-panel.
# Must be run with sudo: sudo ./scripts/hidraw-rules.sh

set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Error: This script must be run with sudo"
    echo "Usage: sudo $0"
    exit 1
fi

# Define the udev rules file path
UDEV_RULES_FILE="/etc/udev/rules.d/70-rusty-panel.rules"

# Create the udev rules file
echo "Creating udev rules file at $UDEV_RULES_FILE"
tee $UDEV_RULES_FILE <<EOF
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="0483", ATTRS{idProduct}=="a3c4", TAG+="uaccess"
EOF

# Reload udev rules
udevadm control --reload-rules
udevadm trigger

# Success
echo "Udev rules for rusty-panel have been set up."
