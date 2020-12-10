#[cfg(target_arch = "arm")]
use rppal::uart::{Parity, Uart};
use rust_pi::*;
use std::error::Error;
use std::thread;
use std::time::Duration;

const DESK_UART: &str = "/dev/ttyAMA3";
const PANEL_UART: &str = "/dev/ttyAMA2";
// const DESK_UART: &str = "/dev/ttyUSB1";
// const PANEL_UART: &str = "/dev/ttyUSB0";

#[cfg(target_arch = "arm")]
pub fn read_desk() -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    let mut uart_desk = Uart::with_path(DESK_UART, 9600, Parity::None, 8, 1)?;
    read_uart(&mut uart_desk)
}

#[cfg(target_arch = "arm")]
pub fn read_panel() -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    let mut uart_panel = Uart::with_path(PANEL_UART, 9600, Parity::None, 8, 1)?;
    read_uart(&mut uart_panel)
}

#[cfg(target_arch = "arm")]
pub fn read_uart(uart: &mut Uart) -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    uart.set_read_mode(1, Duration::from_millis(100))?;

    let mut buffer = [0u8; DATA_FRAME_SIZE];

    let mut dropped_frame_count = 0;
    loop {
        if uart.read(&mut buffer)? > 0 {
            if validate_frame(&buffer.to_vec()) {
                return Ok((Some(buffer.to_vec()), dropped_frame_count));
            } else {
                dropped_frame_count += 1;
            }
        } else {
            return Ok((None, 0));
        }
    }
}

#[cfg(target_arch = "arm")]
pub fn write_to_uart(
    uart: &mut Uart,
    frame: &mut DataFrame,
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
pub fn write_to_panel(rx_message: DeskToPanelMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to panel: {:?}", times, rx_message);

    let mut uart = Uart::with_path(PANEL_UART, 9600, Parity::None, 8, 1)?;
    write_to_uart(&mut uart, &mut rx_message.as_frame(), times)
}

#[cfg(target_arch = "arm")]
pub fn write_to_desk(tx_message: PanelToDeskMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to desk: {:?}", times, tx_message);

    let mut uart = Uart::with_path(DESK_UART, 9600, Parity::None, 8, 1)?;
    write_to_uart(&mut uart, &mut tx_message.as_frame(), times)
}
