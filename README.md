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

## Usage

```bash
cargo build --release
./target/release/rusty-panel
```

## Configuration

rusty-panel uses a toml file for configuration.

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
