#[cfg(target_arch = "arm")]
use rppal::gpio::Gpio;
#[cfg(target_arch = "arm")]
use rppal::uart::{Parity, Uart};

use crate::protocol;
use crate::protocol::{DeskToPanelMessage, PanelToDeskMessage, DATA_FRAME_SIZE};
use std::error::Error;
use std::thread;
use std::time::Duration;

const DESK_UART: &str = "/dev/ttyAMA3";
const PANEL_UART: &str = "/dev/ttyAMA2";
// const DESK_UART: &str = "/dev/ttyUSB1";
// const PANEL_UART: &str = "/dev/ttyUSB0";

// Gpio uses BCM pin numbering. BCM GPIO 22 is tied to physical pin 15.
const GPIO_LED: u8 = 22;

#[cfg(target_arch = "arm")]
pub fn initialize() -> Result<(), Box<dyn Error>> {
    println!("Turning on LED at GPIO {}.", GPIO_LED,);

    let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();

    pin.set_high();

    Ok(())
}

#[cfg(target_arch = "arm")]
pub fn shutdown() -> Result<(), Box<dyn Error>> {
    println!("Turning off LED at GPIO {}.", GPIO_LED,);

    let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();

    pin.set_low();
    drop(pin);

    let current_state = Gpio::new()?.get(GPIO_LED)?.read();
    println!("New state of LED at GPIO {}: {}.", GPIO_LED, current_state);

    Ok(())
}

#[cfg(target_arch = "arm")]
pub fn read_desk() -> Result<(Option<DeskToPanelMessage>, usize), Box<dyn Error>> {
    let mut uart_desk = Uart::with_path(DESK_UART, 9600, Parity::None, 8, 1)?;
    let (maybe_frame, dropped_frame_count) = read_uart(&mut uart_desk)?;
    if let Some(frame) = maybe_frame {
        Ok((
            Some(DeskToPanelMessage::from_frame(&frame)),
            dropped_frame_count,
        ))
    } else {
        Ok((None, dropped_frame_count))
    }
}

#[cfg(target_arch = "arm")]
pub fn read_panel() -> Result<(Option<PanelToDeskMessage>, usize), Box<dyn Error>> {
    let mut uart_panel = Uart::with_path(PANEL_UART, 9600, Parity::None, 8, 1)?;
    let (maybe_frame, dropped_frame_count) = read_uart(&mut uart_panel)?;
    if let Some(frame) = maybe_frame {
        Ok((
            Some(PanelToDeskMessage::from_frame(&frame)),
            dropped_frame_count,
        ))
    } else {
        Ok((None, dropped_frame_count))
    }
}

#[cfg(target_arch = "arm")]
fn read_uart(uart: &mut Uart) -> Result<(Option<protocol::DataFrame>, usize), Box<dyn Error>> {
    uart.set_read_mode(1, Duration::from_millis(100))?;

    let mut buffer = [0u8; 1];
    let mut frame = [0u8; DATA_FRAME_SIZE];
    let mut frame_index = 0;

    let mut dropped_byte_count = 0;
    loop {
        if uart.read(&mut buffer)? > 0 {
            let b = buffer[0];

            println!("byte: {:?}, frame_index: {:?}", b, frame_index);

            frame[frame_index] = b;

            if frame_index == 0 {
                if !protocol::is_start_byte(b) {
                    dropped_byte_count += 1;
                    println!("Not starting byte: {:?} - dropping", b);
                    continue;
                }
            }

            if frame_index == DATA_FRAME_SIZE - 1 {
                println!("Validating frame: {:?}", &frame.to_vec());
                if protocol::validate_frame(&frame.to_vec()) {
                    println!(
                        "Returning valid frame: {:?} - dropped byte count: {:?}",
                        &frame.to_vec(),
                        dropped_byte_count
                    );
                    return Ok((Some(frame.to_vec()), dropped_byte_count / DATA_FRAME_SIZE));
                } else {
                    println!("Invalid frame: {:?}", &frame.to_vec());
                    dropped_byte_count += DATA_FRAME_SIZE;
                    frame_index = 0;
                    continue;
                }
            }

            frame_index += 1;
        } else {
            return Ok((None, 0));
        }
    }
}

#[cfg(target_arch = "arm")]
fn write_to_uart(
    uart: &mut Uart,
    frame: &mut protocol::DataFrame,
    times: usize,
) -> Result<(), Box<dyn Error>> {
    for i in 0..times {
        let bytes_written_count = uart.write(frame)?;

        if bytes_written_count != DATA_FRAME_SIZE {
            println!(
                "Wrote {:?} bytes - Expected to write: {:?}",
                bytes_written_count, DATA_FRAME_SIZE
            );
        }

        // It takes a bit over one millisecond to transfer each byte
        // (Blocking doesn't seem to work)
        // So we have to sleep for at least 7 and a bit milliseconds
        // to avoid sending overlapping frames.
        thread::sleep(Duration::from_millis(8));
    }

    Ok(())
}

#[cfg(target_arch = "arm")]
pub fn write_to_panel(message: DeskToPanelMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to panel: {:?}", times, rx_message);

    let mut uart = Uart::with_path(PANEL_UART, 9600, Parity::None, 8, 1)?;
    write_to_uart(&mut uart, &mut message.as_frame(), times)
}

#[cfg(target_arch = "arm")]
pub fn write_to_desk(message: PanelToDeskMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to desk: {:?}", times, tx_message);

    let mut uart = Uart::with_path(DESK_UART, 9600, Parity::None, 8, 1)?;
    write_to_uart(&mut uart, &mut message.as_frame(), times)
}
