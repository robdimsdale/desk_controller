use crate::protocol::{DeskToPanelMessage, PanelToDeskMessage};

use std::error::Error;

pub fn initialize() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn shutdown() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
    Ok((Some(DeskToPanelMessage::Height(123.5)), 0))
}

pub fn read_panel() -> Result<(Option<PanelToDeskMessage>, usize), Box<dyn Error>> {
    Ok((None, 0))
}

pub fn write_to_panel(message: DeskToPanelMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!(
    //     "Not actually writing {:?} times to panel: {:?}",
    //     times, message
    // );
    Ok(())
}

pub fn write_to_desk(message: PanelToDeskMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!(
    //     "Not actually writing {:?} times to desk: {:?}",
    //     times, message
    // );
    Ok(())
}
