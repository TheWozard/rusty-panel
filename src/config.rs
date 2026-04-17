use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

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
    pub range: Option<String>,
}

pub struct ConfigWatcher {
    rx: mpsc::UnboundedReceiver<()>,
    path: PathBuf,
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    pub async fn wait_for_change(&mut self) {
        self.rx.recv().await;
        while self.rx.try_recv().is_ok() {}
    }

    pub fn reload(&self) -> Result<Config, Box<dyn std::error::Error>> {
        Config::load(&self.path)
    }
}

impl Config {
    pub(crate) fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<(Self, ConfigWatcher), Box<dyn std::error::Error>> {
        let config = Self::load(&path)?;
        let (tx, rx) = mpsc::unbounded_channel::<()>();
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if result.is_ok() {
                    let _ = tx.send(());
                }
            },
            notify::Config::default(),
        )?;
        watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;
        let watcher = ConfigWatcher {
            rx,
            path: path.as_ref().to_path_buf(),
            _watcher: watcher,
        };
        Ok((config, watcher))
    }

    pub fn from_str(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(toml::from_str(s)?)
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

impl ButtonConfig {
    // Parse the range string into (range, offset) tuple. This is derived from min and max values.
    pub fn parse_range_offset(&self) -> Result<(u16, u16), Box<dyn std::error::Error>> {
        let min_max = self.parse_min_max()?;
        Ok((min_max.1 - min_max.0, min_max.0))
    }

    // Parse the range string into a tuple of (min, max) values. Supports formats like "min-max" or just "max".
    pub fn parse_min_max(&self) -> Result<(u16, u16), Box<dyn std::error::Error>> {
        if let Some(range_str) = &self.range {
            let parts: Vec<&str> = range_str.split('-').collect();
            match parts.len() {
                1 => return Ok((0, parts[0].parse::<u16>()?)),
                2 => return Ok((parts[0].parse::<u16>()?, parts[1].parse::<u16>()?)),
                _ => return Err(format!("Invalid range format: {}", range_str).into()),
            }
        }
        Ok((0, 100))
    }
}
