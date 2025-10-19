use std::fs;
use std::path::Path;

const DEFAULT_CONFIG: &str = include_str!("../rusty-panel.toml");

fn main() {
    // Initialize logger (controlled by RUST_LOG environment variable)
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    // Setup args and defaults.
    let default_config = dirs::home_dir()
        .map(|home| home.join(".config/rusty-panel/rusty-panel.toml"))
        .and_then(|path| path.to_str().map(String::from))
        .unwrap_or_else(|| "rusty-panel.toml".to_string());
    let config_path = args.get(1).unwrap_or(&default_config);

    // Create config file with embedded DEFAULT_CONFIG if it doesn't exist.
    let path = Path::new(config_path);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                log::error!("Failed to create config directory for default config: {}", e);
                std::process::exit(1);
            }
        }
        match fs::write(path, DEFAULT_CONFIG) {
            Ok(_) => log::info!("Created default config at {}", config_path),
            Err(e) => {
                log::error!("Failed to create default config at {}: {}", config_path, e);
                std::process::exit(1);
            }
        }
    }

    // Load configuration.
    log::info!("Starting rusty-panel with configuration: {}", config_path);
    let config = rusty_panel::config::Config::from_file(config_path).unwrap_or_else(|e| {
        log::error!("Failed to load config from {}: {}", config_path, e);
        std::process::exit(1);
    });

    // Open device and get started.
    match rusty_panel::open_first_device() {
        Ok(mut device) => {
            if let Err(e) = device.apply_config(config) {
                log::error!("Error applying configuration: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = device.open_stream() {
                log::error!("Error during device stream: {}", e);
            }
        }
        Err(e) => {
            log::error!("Failed to open device: {}", e);
        }
    }
}
