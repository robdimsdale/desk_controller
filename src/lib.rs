mod protocol;

#[cfg_attr(all(target_os = "linux", target_arch = "arm"), path = "rpi.rs")]
#[cfg_attr(
    not(all(target_os = "linux", target_arch = "arm")),
    path = "not_rpi.rs"
)]
mod os;

#[macro_use]
extern crate lazy_static;

pub use crate::protocol::DATA_FRAME_SIZE;
use crate::protocol::{DeskToPanelMessage, PanelToDeskMessage};
use crossbeam_channel::{select, unbounded};
use std::error::Error;
use std::sync::Mutex;
use std::thread::spawn;

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

pub fn run(ctl_rx: crossbeam_channel::Receiver<bool>) -> Result<(), Box<dyn Error>> {
    let (desk_to_panel_tx, desk_to_panel_rx) = unbounded::<(Option<DeskToPanelMessage>, usize)>();
    let (panel_to_desk_tx, panel_to_desk_rx) = unbounded::<(Option<PanelToDeskMessage>, usize)>();

    let (c1_tx, c1_rx) = unbounded::<bool>();
    let (c2_tx, c2_rx) = unbounded::<bool>();

    spawn(move || loop {
        desk_to_panel_tx
            .send(os::read_desk().expect("failed to read from desk"))
            .expect("Failed to send on desk_to_panel_tx");
    });

    spawn(move || loop {
        panel_to_desk_tx
            .send(os::read_panel().expect("Failed to read from panel"))
            .expect("Failed to send on panel_to_desk_tx");
    });

    spawn(move || loop {
        select! {
            recv(c1_rx) -> _=> {
                println!("Received shutdown signal - exiting run (desk->panel) loop");
                return
            },
            recv(desk_to_panel_rx) -> msg => {
                let (maybe_message,dropped_byte_count) = msg.expect("failed to unpack desk->panel msg");

                *DESK_DROPPED_BYTE_COUNT.lock().unwrap() += dropped_byte_count;

                if let Some(message) = maybe_message {
                    *DESK_FOUND_FRAME_COUNT.lock().unwrap() += 1;

                    if let DeskToPanelMessage::Height(h) = message {
                        *CURRENT_HEIGHT.lock().unwrap() = h;
                    }

                    match message {
                        DeskToPanelMessage::Height(h) => {
                            *CURRENT_HEIGHT.lock().unwrap() = h;

                            if h < 6.50 || h > 129.5 {
                                println!(
                                    "desk-to-panel abnormal height: {:?} - {:?}",
                                    h,
                                    message.as_frame()
                                );
                            }
                        }
                        _ => {
                            println!(
                                "other desk-to-panel message: {:?} - {:?}",
                                message,
                                message.as_frame()
                            );
                        }
                    }

                    os::write_to_panel(message,).expect("Failed to write to panel");
                }
            },
        }
    });

    spawn(move || loop {
        select! {
            recv(c2_rx) -> _=> {
                println!("Received shutdown signal - exiting run (panel->desk) loop");
                return;
            },
            recv(panel_to_desk_rx) -> msg => {
                let (maybe_message,dropped_byte_count) = msg.expect("failed to unpack panel->desk msg");

                *PANEL_DROPPED_BYTE_COUNT.lock().unwrap() += dropped_byte_count;
                *CURRENT_PANEL_KEY.lock().unwrap() = maybe_message;

                if let Some(message) = maybe_message {
                    *PANEL_FOUND_FRAME_COUNT.lock().unwrap() += 1;

                    match message{
                        PanelToDeskMessage::NoKey => {}
                        _ => {
                            println!(
                                "panel-to-desk message: {:?} - {:?}",
                                message,
                                message.as_frame()
                            );
                        },
                    }

                    os::write_to_desk(message).expect("failed to write to desk");
                }
            },
        }
    });

    let x = ctl_rx.recv()?;
    c1_tx.send(x)?;
    c2_tx.send(x)?;

    Ok(())
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
