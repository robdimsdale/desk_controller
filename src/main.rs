use rppal::uart::{Parity, Uart};
use std::error::Error;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let mut uart = Uart::new(9600, Parity::None, 8, 1)?;

    uart.set_read_mode(1, Duration::default())?;

    const DATA_FRAME_SIZE: usize = 7;

    let mut first_val = [0u8; 1];
    let mut buffer = [0u8; DATA_FRAME_SIZE - 1];
    loop {
        // Fill the buffer variable with any incoming data.
        if uart.read(&mut first_val)? > 0 {
            if first_val[0] != 104 {
                println!("skipped byte: {:?}", first_val);
            } else {
                // Fill the buffer variable with any incoming data.
                if uart.read(&mut buffer)? > 0 {
                    println!("Received bytes: {:?}{:?}", first_val, buffer);
                }
            }
        }
    }
}
