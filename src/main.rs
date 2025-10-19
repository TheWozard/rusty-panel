fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Default config path
    let default_config = dirs::home_dir()
        .map(|home| home.join(".config/rusty-panel/rusty-panel.toml"))
        .and_then(|path| path.to_str().map(String::from))
        .unwrap_or_else(|| "rusty-panel.toml".to_string());

    let config_path = args.get(1).unwrap_or(&default_config);
    let config = rusty_panel::config::Config::from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config from {}: {}", config_path, e);
        std::process::exit(1);
    });

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
