mod protocol;

#[cfg_attr(all(target_os = "linux", target_arch = "arm"), path = "rpi.rs")]
#[cfg_attr(
    not(all(target_os = "linux", target_arch = "arm")),
    path = "not_rpi.rs"
)]
mod os;

#[macro_use]
extern crate lazy_static;

pub use protocol::{DeskToPanelMessage, PanelToDeskMessage};
use std::error::Error;
use std::sync::Mutex;

lazy_static! {
    static ref CURRENT_HEIGHT: Mutex<f32> = Mutex::new(0.0);
}

pub fn initialize() -> Result<(), Box<dyn Error>> {
    os::initialize()
}

pub fn shutdown() -> Result<(), Box<dyn Error>> {
    os::shutdown()
}

pub fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
    let (maybe_message, dropped_frame_count) = os::read_desk()?;

    if let Some(DeskToPanelMessage::Height(h)) = maybe_message {
        *CURRENT_HEIGHT.lock().unwrap() = h;
    }

    Ok((maybe_message, dropped_frame_count))
}
pub fn read_panel() -> Result<(Option<PanelToDeskMessage>, usize), Box<dyn Error>> {
    os::read_panel()
}

pub fn write_to_panel(message: DeskToPanelMessage, times: usize) -> Result<(), Box<dyn Error>> {
    os::write_to_panel(message, times)
}

pub fn write_to_desk(message: PanelToDeskMessage, times: usize) -> Result<(), Box<dyn Error>> {
    os::write_to_desk(message, times)
}

pub fn current_height() -> f32 {
    *CURRENT_HEIGHT.lock().unwrap()
}
