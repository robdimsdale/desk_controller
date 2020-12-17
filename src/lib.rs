mod protocol;

#[cfg_attr(all(target_os = "linux", target_arch = "arm"), path = "rpi.rs")]
#[cfg_attr(
    not(all(target_os = "linux", target_arch = "arm")),
    path = "not_rpi.rs"
)]
mod os;

#[macro_use]
extern crate lazy_static;

pub use protocol::{DeskToPanelMessage, PanelToDeskMessage, DATA_FRAME_SIZE};
use std::error::Error;
use std::sync::Mutex;

lazy_static! {
    static ref CURRENT_HEIGHT: Mutex<f32> = Mutex::new(0.0);
    static ref CURRENT_PANEL_KEY: Mutex<Option<PanelToDeskMessage>> = Mutex::new(None);
    static ref DESK_DROPPED_BYTE_COUNT: Mutex<usize> = Mutex::new(0);
    static ref DESK_FOUND_FRAME_COUNT: Mutex<usize> = Mutex::new(0);
    static ref PANEL_DROPPED_BYTE_COUNT: Mutex<usize> = Mutex::new(0);
    static ref PANEL_FOUND_FRAME_COUNT: Mutex<usize> = Mutex::new(0);
}

pub fn initialize() -> Result<(), Box<dyn Error>> {
    os::initialize()
}

pub fn shutdown() -> Result<(), Box<dyn Error>> {
    os::shutdown()
}

pub fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
    let (maybe_message, dropped_byte_count) = os::read_desk()?;

    if let Some(message) = maybe_message {
        *DESK_FOUND_FRAME_COUNT.lock().unwrap() += 1;
        if let DeskToPanelMessage::Height(h) = message {
            *CURRENT_HEIGHT.lock().unwrap() = h;
        }
    }
    *DESK_DROPPED_BYTE_COUNT.lock().unwrap() += dropped_byte_count;

    Ok((maybe_message, dropped_byte_count))
}
pub fn read_panel() -> Result<(Option<PanelToDeskMessage>, usize), Box<dyn Error>> {
    let (maybe_message, dropped_byte_count) = os::read_panel()?;

    *CURRENT_PANEL_KEY.lock().unwrap() = maybe_message;
    if let Some(_) = maybe_message {
        *PANEL_FOUND_FRAME_COUNT.lock().unwrap() += 1;
    }
    *PANEL_DROPPED_BYTE_COUNT.lock().unwrap() += dropped_byte_count;

    Ok((maybe_message, dropped_byte_count))
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

pub fn current_panel_key() -> Option<PanelToDeskMessage> {
    *CURRENT_PANEL_KEY.lock().unwrap()
}

pub fn desk_frame_counts() -> (usize, usize) {
    (
        *DESK_FOUND_FRAME_COUNT.lock().unwrap(),
        *DESK_DROPPED_BYTE_COUNT.lock().unwrap(),
    )
}

pub fn panel_frame_counts() -> (usize, usize) {
    (
        *PANEL_FOUND_FRAME_COUNT.lock().unwrap(),
        *PANEL_DROPPED_BYTE_COUNT.lock().unwrap(),
    )
}
