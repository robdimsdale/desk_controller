use crate::protocol::{DeskToPanelMessage, PanelToDeskMessage};
use rand::Rng;
use std::error::Error;

pub fn initialize() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn shutdown() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let height = rng.gen::<f32>() * 64.0 + 65.0;
    Ok((Some(DeskToPanelMessage::Height(height)), 0))
}

pub fn read_panel() -> Result<(Option<PanelToDeskMessage>, usize), Box<dyn Error>> {
    Ok((Some(PanelToDeskMessage::Three(121.0)), 0))
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
