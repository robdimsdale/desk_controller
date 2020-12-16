mod protocol;

#[cfg_attr(all(target_os = "linux", target_arch = "arm"), path = "rpi.rs")]
#[cfg_attr(
    not(all(target_os = "linux", target_arch = "arm")),
    path = "not_rpi.rs"
)]
mod os;

use std::error::Error;

pub use protocol::{DeskToPanelMessage, PanelToDeskMessage};

pub fn initialize() -> Result<(), Box<dyn Error>> {
    os::initialize()
}

pub fn shutdown() -> Result<(), Box<dyn Error>> {
    os::shutdown()
}

pub fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
    os::read_desk()
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
