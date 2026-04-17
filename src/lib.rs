use hidapi::HidApi;
use std::{
    fs::File,
    io::{self, Error, ErrorKind, Read, Write},
    os::unix::fs::OpenOptionsExt,
};
use tokio::io::unix::AsyncFd;

pub mod config;
mod sleep;

pub fn open_first_device() -> Result<PanelDevice, Box<dyn std::error::Error>> {
    let api = HidApi::new()?;
    for device_info in api.device_list() {
        match (device_info.vendor_id(), device_info.product_id()) {
            (0x0483, 0xa3c4) => {
                let path = device_info.path().to_str()?;
                let file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(path)?;
                return Ok(PanelDevice {
                    device: AsyncFd::new(file)?,
                    device_id: 0x06,
                    color_set_id: 0x05,
                    handler: PanelHandler::new(4),
                    color: None,
                });
            }
            _ => continue,
        }
    }
    Err(Error::new(ErrorKind::NotFound, "no compatible HID device found").into())
}

pub struct PanelDevice {
    device: AsyncFd<File>,
    device_id: u8,
    color_set_id: u8,
    handler: PanelHandler,
    color: Option<(u8, u8, u8)>,
}

impl PanelDevice {
    pub fn apply_config(&mut self, config: config::Config) -> Result<(), Box<dyn std::error::Error>> {
        self.handler.clear();
        let color = config.device.parse_color()?;
        let _ = self.set_color(color);
        self.color = Some(color);

        for button_cfg in config.buttons {
            let (range, offset) = button_cfg.parse_range_offset()?;
            if let Some(command) = button_cfg.on_click {
                self.handler.register_click_command(button_cfg.id, command.to_string());
            }
            if let Some(command) = button_cfg.on_rotate {
                self.handler.register_rotate_command(button_cfg.id, command.to_string(), range, offset);
            }
        }

        Ok(())
    }

    pub async fn open_stream(&mut self, mut watcher: config::ConfigWatcher) -> Result<(), io::Error> {
        let mut wake = sleep::WakeWatcher::new().await;
        loop {
            let mut buf = [0u8; 64];
            tokio::select! {
                result = self.read_event(&mut buf) => {
                    match result? {
                        n if n >= 3 => match (buf[0], buf[1], buf[2]) {
                            (0x01, button_id, rotation) => self.handler.rotate(button_id as usize, rotation),
                            (0x02, button_id, 0x01) => self.handler.click(button_id as usize),
                            (0x02, _, 0x00) => {}
                            _ => log::debug!("unknown input report: {:?}", &buf[..n]),
                        },
                        _ => {}
                    }
                }
                _ = watcher.wait_for_change() => {
                    match watcher.reload() {
                        Ok(config) => {
                            log::info!("Config changed, reloading");
                            if let Err(e) = self.apply_config(config) {
                                log::error!("Failed to apply reloaded config: {}", e);
                            }
                        }
                        Err(e) => log::error!("Failed to reload config: {}", e),
                    }
                }
                _ = wake.wait() => {
                    if let Some(color) = self.color {
                        let _ = self.set_color(color);
                    }
                }
            }
        }
    }

    async fn read_event(&self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let mut guard = self.device.readable().await?;
            match guard.try_io(|inner: &AsyncFd<_>| inner.get_ref().read(buf)) {
                Ok(result) => return result,
                Err(_would_block) => continue,
            }
        }
    }

    const SET_FULL_COLOR: u8 = 0x04;

    pub fn set_color(&self, color: (u8, u8, u8)) -> Result<(), io::Error> {
        (&*self.device.get_ref()).write_all(&[
            self.device_id, Self::SET_FULL_COLOR, self.color_set_id,
            color.0, color.1, color.2,
        ])
    }
}

pub struct PanelHandler {
    click_callback_lookup: Vec<Option<Box<dyn Fn()>>>,
    rotate_callback_lookup: Vec<Option<Box<dyn FnMut(u8)>>>,
}

impl PanelHandler {
    pub fn clear(&mut self) {
        for slot in &mut self.click_callback_lookup {
            *slot = None;
        }
        for slot in &mut self.rotate_callback_lookup {
            *slot = None;
        }
    }

    pub fn new(buttons: usize) -> Self {
        let mut result = Self {
            click_callback_lookup: Vec::with_capacity(buttons),
            rotate_callback_lookup: Vec::with_capacity(buttons),
        };
        result.click_callback_lookup.resize_with(buttons, || None);
        result.rotate_callback_lookup.resize_with(buttons, || None);
        result
    }

    pub fn register_click_command(&mut self, button: usize, command: String) {
        if button < self.click_callback_lookup.len() {
            let executable = TemplatedCommand::new(command);
            self.click_callback_lookup[button] = Some(Box::new(move || {
                executable.execute();
            }));
        }
    }

    pub fn register_rotate_command(&mut self, button: usize, command: String, range: u16, offset: u16) {
        if button < self.rotate_callback_lookup.len() {
            let mut executable = TemplatedCommand::new(command);
            self.rotate_callback_lookup[button] = Some(Box::new(move |amount| {
                executable.execute_with_amount(((amount as u16 * range) / 0xff) + offset);
            }));
        }
    }

    pub fn click(&self, button: usize) {
        log::debug!("Click button {}", button);
        if let Some(Some(callback)) = self.click_callback_lookup.get(button) {
            callback();
        }
    }

    pub fn rotate(&mut self, button: usize, amount: u8) {
        log::debug!("Rotate button {}: {}", button, amount);
        if let Some(Some(callback)) = self.rotate_callback_lookup.get_mut(button) {
            callback(amount);
        }
    }
}

struct TemplatedCommand {
    command: String,
    prev_amount: Option<u16>,
}

impl TemplatedCommand {
    pub fn new(command: String) -> Self {
        Self { command, prev_amount: None }
    }

    pub fn execute(&self) {
        let command = self.command.clone();
        log::debug!("Executing: {}", &command);
        tokio::spawn(async move {
            if let Err(e) = tokio::process::Command::new("sh").arg("-c").arg(&command).status().await {
                log::error!("Error executing command '{}': {}", command, e);
            }
        });
    }

    pub fn execute_with_amount(&mut self, amount: u16) {
        if self.prev_amount == Some(amount) {
            return;
        }
        self.prev_amount = Some(amount);
        let full_command = self.command.replace("{amount}", &amount.to_string());
        log::debug!("Executing: {}", &full_command);
        tokio::spawn(async move {
            if let Err(e) = tokio::process::Command::new("sh").arg("-c").arg(&full_command).status().await {
                log::error!("Error executing command '{}': {}", full_command, e);
            }
        });
    }
}
