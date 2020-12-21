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
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::RwLock;
use std::thread::spawn;
use std::time::Duration;

const PANEL_KEY_RESET_TIMEOUT: Duration = Duration::from_millis(1000);
const INTERRUPT_TIMEOUT_DURATION: Duration = Duration::from_secs(10);

const MIN_DESK_HEIGHT_CM: f32 = 65.0;
const MAX_DESK_HEIGHT_CM: f32 = 129.5;

#[derive(Debug)]
pub struct InvalidHeightError {
    height: f32,
    out_of_range: bool,
    not_multiple_of_zero_point_five: bool,
}

impl InvalidHeightError {
    fn new_out_of_range(height: f32) -> InvalidHeightError {
        InvalidHeightError {
            height: height,
            out_of_range: true,
            not_multiple_of_zero_point_five: false,
        }
    }
    fn new_not_multiple_of_zero_point_five(height: f32) -> InvalidHeightError {
        InvalidHeightError {
            height: height,
            out_of_range: false,
            not_multiple_of_zero_point_five: true,
        }
    }
}

impl Display for InvalidHeightError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.out_of_range {
            return write!(
                f,
                "Invalid height: {} - must be between {} and {}",
                self.height, MIN_DESK_HEIGHT_CM, MAX_DESK_HEIGHT_CM
            );
        }

        if self.not_multiple_of_zero_point_five {
            return write!(
                f,
                "Invalid height: {} - must be a multiple of 0.5 cm",
                self.height
            );
        }

        panic!("unrecognized InvalidHeightError cause")
    }
}

impl Error for InvalidHeightError {}

lazy_static! {
    static ref CURRENT_HEIGHT: RwLock<f32> = RwLock::new(0.0);
    static ref TARGET_HEIGHT: RwLock<Option<f32>> = RwLock::new(None);
    static ref CURRENT_PANEL_KEY: RwLock<Option<PanelToDeskMessage>> = RwLock::new(None);
    static ref DESK_DROPPED_BYTE_COUNT: RwLock<usize> = RwLock::new(0);
    static ref DESK_FOUND_FRAME_COUNT: RwLock<usize> = RwLock::new(0);
    static ref PANEL_DROPPED_BYTE_COUNT: RwLock<usize> = RwLock::new(0);
    static ref PANEL_FOUND_FRAME_COUNT: RwLock<usize> = RwLock::new(0);
    static ref INTERRUPT_TX_RX: (
        crossbeam_channel::Sender<()>,
        crossbeam_channel::Receiver<()>,
    ) = unbounded::<()>();
}

pub fn initialize() -> Result<(), Box<dyn Error>> {
    os::initialize()
}

// TODO: make private method, triggered by ctl_rx in run loop
pub fn shutdown() -> Result<(), Box<dyn Error>> {
    os::shutdown()
}

pub fn run(ctl_rx: crossbeam_channel::Receiver<bool>) -> Result<(), Box<dyn Error>> {
    let (c1_tx, c1_rx) = unbounded::<bool>();
    let (c2_tx, c2_rx) = unbounded::<bool>();
    let (c3_tx, c3_rx) = unbounded::<bool>();
    let (c4_tx, c4_rx) = unbounded::<bool>();

    let (interrupt_tx, interrupt_rx) = INTERRUPT_TX_RX.clone();

    spawn(move || {
        loop {
            // A frame takes about 7 ms to send and the desk sends one frame every 8 ms
            // i.e. it pauses for about one ms between the end of one frame and the start of the next

            select! {
                recv(c3_rx) -> msg => {
                    println!("Run: received val on c3_rx: {:?}. Shutting down.", msg);
                    return
                    // TODO: shutdown
                },
                recv(interrupt_rx) -> _ =>{
                    println!("Run: received interrupt");
                },
                default(INTERRUPT_TIMEOUT_DURATION) => {
                    println!("Run: no interrupt received in {:?}", INTERRUPT_TIMEOUT_DURATION);
                }
            };

            let current_height = current_height();
            println!("Run: current height: {:?}", current_height);

            // TODO: handle situation(s) where desk isn't moving even though we're sending it a key
            // - one situation is if we recently pressed another key
            // - another situation is if we are too close to the target height
            // - can resolve by sending a reset command

            let panel_key = current_panel_key();
            let target_height = target_height();

            let maybe_message_info =
                calculate_panel_to_desk_message(panel_key, target_height, current_height)
                    .expect("failed to calculate panel to desk message");

            if let Some((message, times, reset_target_height)) = maybe_message_info {
                println!("Run: writing {:?} message {:?} times.\n- current_height: {:?}\n- target height: {:?}\n- panel key: {:?}",
                message, times,current_height,target_height,panel_key);

                for _ in 0..times {
                    os::write_to_desk(message).expect("failed to write to desk");
                }

                if reset_target_height {
                    println!("Run: resetting target height to None");
                    set_target_height(None);
                } else {
                    if target_height.is_some() {
                        println!("Run: resetting target_height: {:?}", target_height);
                        interrupt_tx
                            .send(())
                            .expect("failed to send on INTERRUPT_TX_RX");
                    }
                }
            }
        }
    });

    let (write_to_panel_tx, write_to_panel_rx) = unbounded::<DeskToPanelMessage>();

    spawn(move || loop {
        select! {
        recv(c1_rx) -> _=> {
            println!("Received shutdown signal - exiting run (desk->panel) loop");
            return
        },
        default => {
            let (maybe_message,dropped_byte_count) = os::read_desk().expect("failed to read from desk");

            increment_desk_dropped_byte_count(dropped_byte_count);

            if let Some(message) = maybe_message {
                increment_desk_found_frame_count(1);

                match message {
                    DeskToPanelMessage::Height(h) => {
                        set_current_height(h);

                        if h < MIN_DESK_HEIGHT_CM || h > MAX_DESK_HEIGHT_CM {
                            println!(
                                "received abnormal height from desk: {:?} - {:?}",
                                h,
                                message.as_frame()
                            );
                        }
                    }
                    _ => {
                        println!(
                            "received other desk-to-panel message: {:?} - {:?}",
                            message,
                            message.as_frame()
                        );
                    }
                }

                write_to_panel_tx.send(message).expect("failed to send on write_to_panel_tx");
            }
        },
        }
    });

    spawn(move || loop {
        select! {
            recv(c4_rx) -> _=> {
                println!("Received shutdown signal (c4_rx) - exiting run (desk->panel) loop");
                return
            },
            recv(write_to_panel_rx) -> msg =>{
                let message = msg.expect("failed to receive on write_to_panel_rx");
                os::write_to_panel(message).expect("Failed to write to panel");
            },
        }
    });

    let (panel_to_desk_tx, panel_to_desk_rx) = unbounded::<(Option<PanelToDeskMessage>, usize)>();

    // Keep this as a separate loop so that we can have a default timeout in the recv select loop
    spawn(move || loop {
        panel_to_desk_tx
            .send(os::read_panel().expect("Failed to read from panel"))
            .expect("Failed to send on panel_to_desk_tx");
    });

    spawn(move || loop {
        select! {
            recv(c2_rx) -> _=> {
                println!("Received shutdown signal - exiting run (panel->desk) loop");
                return;
            },
            recv(panel_to_desk_rx) -> msg => {
                let (maybe_message,dropped_byte_count) = msg.expect("failed to unpack panel->desk msg");

                increment_panel_dropped_byte_count(dropped_byte_count);

                if let Some(message) = maybe_message {
                    increment_panel_found_frame_count(1);

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

                    set_current_panel_key(maybe_message);
                }
            },
            default(PANEL_KEY_RESET_TIMEOUT) => {
                let current_panel_key = current_panel_key();
                if current_panel_key.is_some(){
                    println!("No panel key received in {:?} - resetting to None (from: {:?})",PANEL_KEY_RESET_TIMEOUT,current_panel_key);
                    set_current_panel_key(None);
                }
            },
        }
    });

    let x = ctl_rx.recv()?;
    c1_tx.send(x)?;
    c2_tx.send(x)?;
    c3_tx.send(x)?;
    c4_tx.send(x)?;

    Ok(())
}

pub fn move_to_height(height_in_cm: f32) -> Result<(), InvalidHeightError> {
    // TODO: validate that height is in range (65.0 to 129.5)
    // TODO: validate that height is a multiple of 0.5

    if height_in_cm < MIN_DESK_HEIGHT_CM || height_in_cm > MAX_DESK_HEIGHT_CM {
        return Err(InvalidHeightError::new_out_of_range(height_in_cm));
    }

    if (height_in_cm * 10.0) as usize % 5 != 0 {
        return Err(InvalidHeightError::new_not_multiple_of_zero_point_five(
            height_in_cm,
        ));
    }

    set_target_height(Some(height_in_cm));

    Ok(())
}

pub fn current_height() -> f32 {
    *CURRENT_HEIGHT.read().unwrap()
}

fn set_current_height(h: f32) {
    *CURRENT_HEIGHT.write().unwrap() = h;
}

pub fn target_height() -> Option<f32> {
    *TARGET_HEIGHT.read().unwrap()
}

fn set_target_height(h: Option<f32>) {
    *TARGET_HEIGHT.write().unwrap() = h;

    let (tx, _) = INTERRUPT_TX_RX.clone();
    tx.send(())
        .expect("failed to send on INTERRUPT_TX_RX (target height)")
}

pub fn current_panel_key() -> Option<PanelToDeskMessage> {
    *CURRENT_PANEL_KEY.read().unwrap()
}

fn set_current_panel_key(key: Option<PanelToDeskMessage>) {
    *CURRENT_PANEL_KEY.write().unwrap() = key;

    let (tx, _) = INTERRUPT_TX_RX.clone();
    tx.send(())
        .expect("failed to send on INTERRUPT_TX_RX (panel key)")
}

pub fn desk_frame_counts() -> (usize, usize) {
    (
        *DESK_FOUND_FRAME_COUNT.read().unwrap(),
        *DESK_DROPPED_BYTE_COUNT.read().unwrap(),
    )
}

fn increment_desk_found_frame_count(u: usize) {
    *DESK_FOUND_FRAME_COUNT.write().unwrap() += u;
}

fn increment_desk_dropped_byte_count(u: usize) {
    *DESK_DROPPED_BYTE_COUNT.write().unwrap() += u;
}

pub fn panel_frame_counts() -> (usize, usize) {
    (
        *PANEL_FOUND_FRAME_COUNT.read().unwrap(),
        *PANEL_DROPPED_BYTE_COUNT.read().unwrap(),
    )
}

fn increment_panel_found_frame_count(u: usize) {
    *PANEL_FOUND_FRAME_COUNT.write().unwrap() += u;
}

fn increment_panel_dropped_byte_count(u: usize) {
    *PANEL_DROPPED_BYTE_COUNT.write().unwrap() += u;
}

fn calculate_panel_to_desk_message(
    received_panel_key: Option<PanelToDeskMessage>,
    target_height: Option<f32>,
    current_height: f32,
) -> Result<Option<(PanelToDeskMessage, usize, bool)>, Box<dyn Error>> {
    match received_panel_key {
        Some(PanelToDeskMessage::NoKey) => {
            if target_height.is_none() {
                return Ok(Some((PanelToDeskMessage::NoKey, 1, false)));
            }
        }
        Some(key) => return Ok(Some((key, 1, false))),
        None => {
            // continue
        }
    }

    let target_height = match target_height {
        None => {
            return Ok(None);
        }
        Some(t) => t,
    };

    // if target_height.is_none() {
    //     return Ok(None);
    // }
    // let target_height = target_height.unwrap();

    // TODO: test error windows. Also is there a better way to solve this problem?
    let error_window = 0.5;

    if current_height >= target_height - error_window
        && current_height <= target_height + error_window
    {
        return Ok(Some((PanelToDeskMessage::NoKey, 200, true)));
    }

    if current_height < target_height - error_window {
        return Ok(Some((PanelToDeskMessage::Up, 1, false)));
    }

    if current_height > target_height + error_window {
        return Ok(Some((PanelToDeskMessage::Down, 1, false)));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_panel_to_desk_message_no_key_no_target_height() -> Result<(), Box<dyn Error>>
    {
        let current_height = 70.0;
        assert_eq!(
            calculate_panel_to_desk_message(None, None, current_height)?,
            None,
        );
        Ok(())
    }

    #[test]
    fn test_calculate_panel_to_desk_message_no_key_target_greater_than_current(
    ) -> Result<(), Box<dyn Error>> {
        let target_height = Some(100.0);
        let current_height = 70.0;
        assert_eq!(
            calculate_panel_to_desk_message(None, target_height, current_height)?,
            Some((PanelToDeskMessage::Up, 1, false))
        );
        Ok(())
    }

    #[test]
    fn test_calculate_panel_to_desk_message_no_key_target_less_than_current(
    ) -> Result<(), Box<dyn Error>> {
        let target_height = Some(60.0);
        let current_height = 70.0;
        assert_eq!(
            calculate_panel_to_desk_message(None, target_height, current_height)?,
            Some((PanelToDeskMessage::Down, 1, false))
        );
        Ok(())
    }

    #[test]
    fn test_calculate_panel_to_desk_message_no_key_target_equal_to_current(
    ) -> Result<(), Box<dyn Error>> {
        let target_height = Some(70.0);
        let current_height = 70.0;
        assert_eq!(
            calculate_panel_to_desk_message(None, target_height, current_height)?,
            Some((PanelToDeskMessage::NoKey, 200, true))
        );
        Ok(())
    }

    #[test]
    fn test_calculate_panel_to_desk_message_current_key_nokey_target_greater_than_current(
    ) -> Result<(), Box<dyn Error>> {
        let target_height = Some(100.0);
        let current_height = 70.0;
        let current_panel_key = Some(PanelToDeskMessage::NoKey);
        assert_eq!(
            calculate_panel_to_desk_message(current_panel_key, target_height, current_height)?,
            Some((PanelToDeskMessage::Up, 1, false))
        );
        Ok(())
    }

    #[test]
    fn test_calculate_panel_to_desk_message_current_key_nokey_no_target(
    ) -> Result<(), Box<dyn Error>> {
        let target_height = None;
        let current_height = 70.0;
        let current_panel_key = Some(PanelToDeskMessage::NoKey);
        assert_eq!(
            calculate_panel_to_desk_message(current_panel_key, target_height, current_height)?,
            Some((PanelToDeskMessage::NoKey, 1, false))
        );
        Ok(())
    }

    #[test]
    fn test_calculate_panel_to_desk_message_current_key_other_target_greater_current(
    ) -> Result<(), Box<dyn Error>> {
        let target_height = Some(100.0);
        let current_height = 70.0;
        let current_panel_key = Some(PanelToDeskMessage::Two(120.0));
        assert_eq!(
            calculate_panel_to_desk_message(current_panel_key, target_height, current_height)?,
            Some((PanelToDeskMessage::Two(120.0), 1, false))
        );
        Ok(())
    }
}
