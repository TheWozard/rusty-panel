fn main() {
    match rusty_panel::open_first_device() {
        Ok(mut device) => {
            device.handler.register_click_callback(0, || {
                println!("Button 0 clicked");
            });
            device.handler.register_rotate_callback(0, |amount| {
                println!("Button 0 rotated by {}", amount);
            });
            device.open_stream();
        }
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
        }
    }
}
