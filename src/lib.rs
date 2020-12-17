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
use std::time::Duration;

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
    let (desk_to_panel_tx, desk_to_panel_rx) = unbounded::<DeskToPanelMessage>();
    let (panel_to_desk_tx, panel_to_desk_rx) = unbounded::<PanelToDeskMessage>();

    spawn(move || loop {
        if let (Some(message), _) = read_desk().expect("Failed to read from desk") {
            desk_to_panel_tx
                .send(message)
                .expect("Failed to send on desk_to_panel_tx");
        }
    });

    spawn(move || loop {
        if let (Some(message), _) = read_panel().expect("Failed to read from panel") {
            panel_to_desk_tx
                .send(message)
                .expect("Failed to send on desk_to_panel_tx");
        }
    });

    loop {
        // At most one of these two receive operations will be executed.
        select! {
            recv(ctl_rx) -> _=> {
                println!("Received shutdown signal - exiting run loop");
                return Ok(());
            },
            recv(desk_to_panel_rx) -> msg => {
                let message = msg?;
                match message {
                    DeskToPanelMessage::Height(h) => {
                        if h < 6.50 || h > 129.5 {
                            println!(
                                "desk-to-panel abnormal height: {:?} - {:?}",
                                h,
                                message.as_frame()
                            );
                        }


                        println!(
                            "desk-to-panel height message: {:?} - {:?}",
                            message,
                            message.as_frame()
                        );
                    }
                    _ => {
                        println!(
                            "other desk-to-panel message: {:?} - {:?}",
                            message,
                            message.as_frame()
                        );
                    }
                }

                write_to_panel(message, 1).expect("Failed to write to panel");
            },
            recv(panel_to_desk_rx) -> msg => {
                let message = msg?;
                match message {
                    PanelToDeskMessage::NoKey => {}
                    _ => {
                        println!(
                            "panel-to-desk message: {:?} - {:?}",
                            message,
                            message.as_frame()
                        );
                    }
                }

                // Write 10x messages to account for dropping ~90% of frames
                write_to_desk(message, 10)?;
            },
            default(Duration::from_secs(1)) => println!("no messages for 1 second"),
        }
    }
}

fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
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
fn read_panel() -> Result<(Option<PanelToDeskMessage>, usize), Box<dyn Error>> {
    let (maybe_message, dropped_byte_count) = os::read_panel()?;

    *CURRENT_PANEL_KEY.lock().unwrap() = maybe_message;
    if let Some(_) = maybe_message {
        *PANEL_FOUND_FRAME_COUNT.lock().unwrap() += 1;
    }
    *PANEL_DROPPED_BYTE_COUNT.lock().unwrap() += dropped_byte_count;

    Ok((maybe_message, dropped_byte_count))
}

fn write_to_panel(message: DeskToPanelMessage, times: usize) -> Result<(), Box<dyn Error>> {
    os::write_to_panel(message, times)
}

fn write_to_desk(message: PanelToDeskMessage, times: usize) -> Result<(), Box<dyn Error>> {
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
