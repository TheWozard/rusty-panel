# Rusty Panel

A Rust application for handling HID input from PC-Panel

> This currently only supports PC-Panel Mini.

## Prerequisites

- Rust toolchain (install from [rustup.rs](https://rustup.rs))

## Setup

To access the PC Panel without sudo, you need to set up udev rules:

```bash
sudo ./scripts/hidraw-rules.sh
```

This script will:
- Create a udev rule at `/etc/udev/rules.d/70-rusty-panel.rules`

## Installation

Install the binary to your system:

```bash
cargo install --path .
```

This installs `rusty-panel` to `~/.cargo/bin/rusty-panel`.

### Auto-start on Login

For KDE Plasma, GNOME, and most other desktop environments, use the XDG autostart method:

```bash
./scripts/install-autostart.sh
```

This script will:
- Create a desktop entry at `~/.config/autostart/rusty-panel.desktop`

### Configuration

rusty-panel uses a TOML configuration file. By default, it looks for the config at:
- `~/.config/rusty-panel/rusty-panel.toml`

You can specify a custom config path:
```bash
rusty-panel /path/to/config.toml
```

### Device Settings

| Field | Type | Description |
|-------|------|-------------|
| `color` | string | Device LED color in hex format (`#RRGGBB` or `#RRGGBBAA`) |

### Button Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | number | Yes | Button identifier (0-3 for PC-Panel Mini) |
| `on_click` | string | No | Shell command to execute when button is pressed |
| `on_rotate` | string | No | Shell command to execute when dial is rotated (use `{amount}` placeholder) |
| `range` | string | No | Range for rotation command in format `min-max` (default, `0-100`). Resolution cannot exceed 255 |
