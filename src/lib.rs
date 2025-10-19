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
    const SET_FULL_COLOR: u8 = 0x04;

    pub fn open_stream(self) -> Result<(), HidError> {
        loop {
            let mut buf = [0u8; 64];
            match self.device.read(&mut buf) {
                Ok(bytes_read) if bytes_read >= 3 => {
                    match buf[0] {
                        0x01 => self.handler.rotate(buf[1] as usize, buf[2]),
                        0x02 => self.handler.click(buf[1] as usize),
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

    pub fn set_color(&self) -> Result<(), Error> {
        self.device.write(&[
            self.device_id, PanelDevice::SET_FULL_COLOR, self.color_set_id, 
            0x00, 0xff, 0xff
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