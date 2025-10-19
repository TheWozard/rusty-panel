use std::fs;
use std::path::Path;

const DEFAULT_CONFIG: &str = include_str!("../rusty-panel.toml");

fn main() {
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
                eprintln!("Failed to create config directory: {}", e);
                std::process::exit(1);
            }
        }

        match fs::write(path, DEFAULT_CONFIG) {
            Ok(_) => println!("Created default config at: {}", config_path),
            Err(e) => {
                eprintln!("Failed to create default config at {}: {}", config_path, e);
                std::process::exit(1);
            }
        }
    }

    // Load configuration.
    let config = rusty_panel::config::Config::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config from {}: {}", config_path, e);
        std::process::exit(1);
    });

    // Open device and get started.
    match rusty_panel::open_first_device() {
        Ok(mut device) => {
            device.apply_config(config);
            if let Err(e) = device.open_stream() {
                eprintln!("Error during device stream: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
        }
    }
}
