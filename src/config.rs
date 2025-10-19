use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub device: DeviceConfig,
    pub buttons: Vec<ButtonConfig>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceConfig {
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct ButtonConfig {
    pub id: usize,
    pub on_click: Option<String>,
    pub on_rotate: Option<String>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration from a TOML string
    pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = toml::from_str(s)?;
        Ok(config)
    }
}

impl DeviceConfig {
    /// Parse the hex color string into RGB components. Supports optional alpha channel.
    pub fn parse_color(&self) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
        let hex = self.color.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err(format!("Invalid color format: {}", self.color).into());
        }
        let r = u16::from_str_radix(&hex[0..2], 16)?;
        let g = u16::from_str_radix(&hex[2..4], 16)?;
        let b = u16::from_str_radix(&hex[4..6], 16)?;
        let a = if hex.len() == 8 {
            u16::from_str_radix(&hex[6..8], 16)?
        } else {
            255
        };
        Ok((
            (r * a / 255) as u8,
            (g * a / 255) as u8,
            (b * a / 255) as u8,
        ))
    }
}
