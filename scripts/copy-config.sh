#!/bin/bash

# This script copies the default configuration file to the user's config directory
# if it does not already exist.

set -e

CONFIG_DIR="$HOME/.config/rusty-panel"
DEFAULT_CONFIG="$PWD/rusty-panel.toml"
USER_CONFIG="$CONFIG_DIR/rusty-panel.toml"

# Create config directory if it doesn't exist
mkdir -p "$CONFIG_DIR"

# Copy default config if user config doesn't exist
if [ ! -f "$USER_CONFIG" ]; then
    cp "$DEFAULT_CONFIG" "$USER_CONFIG"
    echo "Copied default config to $USER_CONFIG"
else
    echo "User config already exists at $USER_CONFIG"
fi
