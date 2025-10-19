use hidapi::{HidApi, HidDevice, HidError};
use std::io::{Error, ErrorKind};

pub mod config;

// Function to get the first available HID device matching known VID/PID pairs
pub fn open_first_device() -> Result<PanelDevice, HidError> {
    let api = HidApi::new()?;
    for device_info in api.device_list() {
        match (device_info.vendor_id(), device_info.product_id()) {
            (0x0483, 0xa3c4) // PC Panel Mini
            => {
                let device = device_info.open_device(&api)?;
                return Ok(PanelDevice { 
                    device, 
                    device_id: 0x06,
                    color_set_id: 0x05,
                    handler: PanelHandler::new(4) 
                });
            }
            _ => continue,
        }
    }
    Err(Error::new(
        ErrorKind::NotFound,
        "no compatible HID device found",
    ).into())
}

pub struct PanelDevice {
    device: HidDevice,
    device_id: u8,
    color_set_id: u8,

    pub handler: PanelHandler,
}

impl PanelDevice {
    pub fn apply_config(&mut self, config: config::Config) {
        let _ = self.set_color(config.device.parse_color().unwrap());

        for button_cfg in config.buttons {
            if let Some(_) = &button_cfg.on_click {
                self.handler.register_click_callback(button_cfg.id, move || {
                    println!("Button {} clicked, running command: {}", button_cfg.id, button_cfg.on_click.as_ref().unwrap());
                });
            }
            if let Some(_) = &button_cfg.on_rotate {
                self.handler.register_rotate_callback(button_cfg.id, move |amount| {
                    println!("Button {} rotated by {}, running command: {}", button_cfg.id, amount, button_cfg.on_rotate.as_ref().unwrap());
                });
            }
        }
    }

    pub fn open_stream(self) -> Result<(), HidError> {
        loop {
            let mut buf = [0u8; 64];
            match self.device.read(&mut buf) {
                Ok(bytes_read) if bytes_read >= 3 => {
                    match (buf[0], buf[1], buf[2]) {
                        (0x01, button_id, rotation) => self.handler.rotate(button_id as usize, rotation),
                        (0x02, button_id, 0x01) => self.handler.click(button_id as usize), // Button press
                        (0x02, _, 0x00) => {}, // Button release
                        _ => {},
                    }
                }
                Ok(_) => continue, // No data read
                Err(e) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("error reading from device: {}", e),
                    ).into());
                }
            }
        }
    }

    const SET_FULL_COLOR: u8 = 0x04;

    pub fn set_color(&self, color: (u8, u8, u8)) -> Result<(), Error> {
        self.device.write(&[
            self.device_id, PanelDevice::SET_FULL_COLOR, self.color_set_id,
            color.0, color.1, color.2
        ]).map(|_| ()).map_err(|result| {
            Error::new(ErrorKind::Other, format!("failed to set color: {}", result))
        })
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