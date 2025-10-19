fn main() {
    let mut handler = rusty_panel::PanelHandler::new(4);
    handler.register_click_callback(0, || {
        println!("Button 0 clicked");
    });
    handler.register_rotate_callback(0, |amount| {
        println!("Button 0 rotated by {}", amount);
    });

    match rusty_panel::open_first_device() {
        Ok(device) => {
            device.open_stream(handler);
        }
        Err(e) => {
            eprintln!("Failed to open device: {}", e);
        }
    }
}
