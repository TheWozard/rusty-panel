use hidapi::{HidApi, HidDevice, HidError};
use std::io::{Error, ErrorKind};

// Function to get the first available HID device matching known VID/PID pairs
pub fn open_first_device() -> Result<PanelDevice, HidError> {
    let api = HidApi::new()?;
    for device_info in api.device_list() {
        match (device_info.vendor_id(), device_info.product_id()) {
            (0x0483, 0xa3c4) // PC Panel Mini
            => {
                let device = device_info.open_device(&api)?;
                return Ok(PanelDevice { device, handler: PanelHandler::new(4) });
            }
            _ => continue,
        }
    }
    Err(Error::new(
        ErrorKind::NotFound,
        "No compatible HID device found",
    ).into())
}

pub struct PanelDevice {
    device: HidDevice,
    pub handler: PanelHandler,
}

impl PanelDevice {
    pub fn open_stream(self) {
        loop {
            let mut buf = [0u8; 3];
            match self.device.read(&mut buf) {
                Ok(bytes_read) if bytes_read > 0 => {
                    match buf.get(0) {
                        Some(&0x01) => self.handler.rotate(
                            buf.get(1).cloned().unwrap_or(0) as usize,
                            buf.get(2).cloned().unwrap_or(0),
                        ),
                        Some(&0x02) => self.handler.click(
                            buf.get(1).cloned().unwrap_or(0) as usize,
                        ),
                        _ => {},
                    }
                }
                Ok(_) => continue, // No data read
                Err(e) => {
                    eprintln!("\nError reading from device: {}", e);
                    break;
                }
            }
        }
    }
}

pub struct PanelHandler {
    click_callback_lookup: Vec<Option<Box<dyn Fn()>>>,
    rotate_callback_lookup: Vec<Option<Box<dyn Fn(u8)>>>,
}

impl PanelHandler {
    pub fn new(buttons: usize) -> Self {
        let mut result = Self { 
            click_callback_lookup: Vec::with_capacity(buttons),
            rotate_callback_lookup: Vec::with_capacity(buttons),
        };
        result.click_callback_lookup.resize_with(buttons, || None);
        result.rotate_callback_lookup.resize_with(buttons, || None);
        result
    }

    pub fn register_click_callback<F>(&mut self, button: usize, callback: F)
    where
        F: Fn() + 'static,
    {
        if button < self.click_callback_lookup.len() {
            self.click_callback_lookup[button] = Some(Box::new(callback));
        }
    }

    pub fn register_rotate_callback<F>(&mut self, button: usize, callback: F)
    where
        F: Fn(u8) + 'static,
    {
        if button < self.rotate_callback_lookup.len() {
            self.rotate_callback_lookup[button] = Some(Box::new(callback));
        }
    }

    pub fn click(&self, button: usize) {
        if let Some(result) = self.click_callback_lookup.get(button) {
            if let Some(callback) = result {
                callback();
            }
        }
    }

    pub fn rotate(&self, button: usize, amount: u8) {
        if let Some(result) = self.rotate_callback_lookup.get(button) {
            if let Some(callback) = result {
                callback(amount);
            }
        }
    }
}