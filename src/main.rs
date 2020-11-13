use rppal::uart::{Parity, Uart};
use std::error::Error;
use std::time::Duration;

const DATA_FRAME_SIZE: usize = 7;

fn main() -> Result<(), Box<dyn Error>> {
    let mut uart = Uart::new(9600, Parity::None, 8, 1)?;

    uart.set_read_mode(1, Duration::default())?;

    let mut buffer = [0u8; DATA_FRAME_SIZE];
    loop {
        // Fill the buffer variable with any incoming data.
        if uart.read(&mut buffer)? > 0 {
            if buffer[0] == 104 {
                println!("Received bytes: {:?}", buffer);
            }
        }
    }
}
