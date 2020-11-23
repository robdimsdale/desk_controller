use rppal::uart::{Parity, Uart};
use std::error::Error;
use std::thread;
use std::time::Duration;

const DATA_FRAME_SIZE: usize = 7;
const DATA_FRAME_START: u8 = 104u8;
const DATA_FRAME_END: u8 = 22u8;

enum TxMessage {
    Up,
    Down,
    One,
    Two,
    Three,
    NoKey,
}

impl TxMessage {
    fn value(&self) -> [u8; DATA_FRAME_SIZE] {
        match *self {
            TxMessage::Up => [DATA_FRAME_START, 1u8, 1u8, 0u8, 0u8, 2u8, DATA_FRAME_END],
            TxMessage::Down => [DATA_FRAME_START, 1u8, 2u8, 0u8, 0u8, 3u8, DATA_FRAME_END],
            TxMessage::One => [DATA_FRAME_START, 1u8, 6u8, 0u8, 0u8, 7u8, DATA_FRAME_END],
            TxMessage::Two => [DATA_FRAME_START, 1u8, 7u8, 0u8, 0u8, 8u8, DATA_FRAME_END],
            TxMessage::Three => [DATA_FRAME_START, 1u8, 8u8, 0u8, 0u8, 9u8, DATA_FRAME_END],
            TxMessage::NoKey => [DATA_FRAME_START, 1u8, 3u8, 0u8, 0u8, 4u8, DATA_FRAME_END],
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // loop {
    //     let height = read_height(receive_serial()?)?;
    //     println!("Current height: {:?} cm", height);
    // }

    println!("Sending Up key 500 times");
    send_message_serial(TxMessage::Up, 500)?;

    println!("Sending NoKey 500 times");
    send_message_serial(TxMessage::NoKey, 500)?;

    println!("Sending Down key 500 times");
    send_message_serial(TxMessage::Down, 500)?;

    println!("Sending NoKey 500 times");
    send_message_serial(TxMessage::NoKey, 500)?;

    Ok(())
}

fn receive_serial() -> Result<Vec<u8>, Box<dyn Error>> {
    let mut uart = Uart::new(9600, Parity::None, 8, 1)?;

    uart.set_read_mode(1, Duration::new(1, 0))?;

    let mut buffer = [0u8; DATA_FRAME_SIZE];
    loop {
        if uart.read(&mut buffer)? > 0 && buffer[0] == 104 {
            return Ok(buffer.to_vec());
        }
    }
}

fn read_height(buf: Vec<u8>) -> Result<f32, Box<dyn Error>> {
    Ok((256 * buf[3] as isize + buf[4] as isize + 650) as f32 / 10.0)
}

fn send_message_serial(tx_message: TxMessage, times: usize) -> Result<(), Box<dyn Error>> {
    let mut uart = Uart::new(9600, Parity::None, 8, 1)?;

    for i in 0..times {
        let bytes_written_count = uart.write(&mut tx_message.value())?;

        println!("Wrote {:?} bytes", bytes_written_count);

        if bytes_written_count != DATA_FRAME_SIZE {
            println!("Expected to write: {:?}", DATA_FRAME_SIZE);
        }

        // It takes a bit over one millisecond to transfer each byte
        // (Blocking doesn't seem to work)
        // So we have to sleep for at least 7 and a bit milliseconds
        // to avoid sending overlapping frames.
        thread::sleep(Duration::from_millis(8));
    }

    Ok(())
}
