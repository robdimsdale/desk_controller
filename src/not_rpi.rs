use rust_pi::*;
use std::error::Error;

pub fn initialize() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn deinitialize() -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn read_desk() -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    Ok((Some(DeskToPanelMessage::Height(123.5).as_frame()), 0))
}

pub fn read_panel() -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    Ok((None, 0))
}

pub fn write_to_panel(rx_message: DeskToPanelMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to panel: {:?}", times, rx_message);
    Ok(())
}

pub fn write_to_desk(tx_message: PanelToDeskMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to desk: {:?}", times, tx_message);
    Ok(())
}
