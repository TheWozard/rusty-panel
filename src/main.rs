fn main() {
    let config = rusty_panel::config::Config::from_file("rusty-panel.toml").unwrap_or_else(|e| {
        eprintln!("Failed to load config: {}", e);
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
