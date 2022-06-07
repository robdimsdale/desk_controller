use crate::protocol::{DeskToPanelMessage, PanelToDeskMessage};
use rand::Rng;
use std::error::Error;
use std::time;

pub fn initialize() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn shutdown() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
    std::thread::sleep(time::Duration::from_secs(3));

    let mut rng = rand::thread_rng();
    let height = rng.gen::<f32>() * 64.0 + 65.0;
    Ok((Some(DeskToPanelMessage::Height(height)), 0))
}

pub fn read_panel() -> Result<(Option<PanelToDeskMessage>, usize), Box<dyn Error>> {
    std::thread::sleep(time::Duration::from_secs(3));
    Ok((Some(PanelToDeskMessage::Three(121.0)), 0))
    // Ok((None, 0))
}

pub fn write_to_panel(_: DeskToPanelMessage) -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn write_to_desk(_: PanelToDeskMessage) -> Result<(), Box<dyn Error>> {
    Ok(())
}
