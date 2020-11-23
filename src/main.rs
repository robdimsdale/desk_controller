#![feature(decl_macro)]
use crossbeam_channel::unbounded;
use rocket::*;
use rppal::uart::{Parity, Uart};
use std::convert::TryInto;
use std::error::Error;
use std::thread;
use std::thread::spawn;
use std::time::Duration;

const DATA_FRAME_SIZE: usize = 7;
const DATA_FRAME_START: u8 = 104u8;
const DATA_FRAME_END: u8 = 22u8;

const TX_UP_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 1u8, 0u8, 0u8, 2u8, DATA_FRAME_END];
const TX_DOWN_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 2u8, 0u8, 0u8, 3u8, DATA_FRAME_END];
const TX_ONE_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 6u8, 0u8, 0u8, 7u8, DATA_FRAME_END];
const TX_TWO_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 7u8, 0u8, 0u8, 8u8, DATA_FRAME_END];
const TX_THREE_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 8u8, 0u8, 0u8, 9u8, DATA_FRAME_END];
const TX_NOKEY_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 3u8, 0u8, 0u8, 4u8, DATA_FRAME_END];
const TX_UNKNOWN_FRAME: [u8; DATA_FRAME_SIZE] =
    [DATA_FRAME_START, 1u8, 1u8, 1u8, 1u8, 1u8, DATA_FRAME_END];
const TX_NONE_FRAME: [u8; DATA_FRAME_SIZE] = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];

type DataFrame = Vec<u8>;

#[derive(Debug, PartialEq)]
enum TxMessage {
    Up,
    Down,
    One,
    Two,
    Three,
    NoKey,
    Unknown,
    None,
}

impl TxMessage {
    fn as_frame(&self) -> DataFrame {
        match *self {
            TxMessage::Up => TX_UP_FRAME.to_vec(),
            TxMessage::Down => TX_DOWN_FRAME.to_vec(),
            TxMessage::One => TX_ONE_FRAME.to_vec(),
            TxMessage::Two => TX_TWO_FRAME.to_vec(),
            TxMessage::Three => TX_THREE_FRAME.to_vec(),
            TxMessage::NoKey => TX_NOKEY_FRAME.to_vec(),
            TxMessage::Unknown => TX_UNKNOWN_FRAME.to_vec(),
            TxMessage::None => TX_NONE_FRAME.to_vec(),
        }
    }

    fn from_frame(buf: &DataFrame) -> TxMessage {
        match buf.as_slice().try_into().unwrap() {
            TX_UP_FRAME => TxMessage::Up,
            TX_DOWN_FRAME => TxMessage::Down,
            TX_ONE_FRAME => TxMessage::One,
            TX_TWO_FRAME => TxMessage::Two,
            TX_THREE_FRAME => TxMessage::Three,
            TX_NOKEY_FRAME => TxMessage::NoKey,
            TX_NONE_FRAME => TxMessage::None,
            _ => TxMessage::Unknown,
        }
    }
}

#[get("/")]
fn index() -> String {
    let height = current_height().unwrap();
    format!("Current Height: {:?} cm", height)
}

#[get("/move_desk/<target_height>")]
fn move_desk(target_height: f32) -> String {
    // move_to_height_cm(target_height).unwrap();

    let height = current_height().unwrap();
    format!("Current Height: {:?} cm", height)
}

fn current_height() -> Result<f32, Box<dyn Error>> {
    let (buffer, _) = read_desk()?;

    read_height_from_buffer(&buffer)
}

// fn move_to_height_cm(target_height: f32) -> Result<(), Box<dyn Error>> {
//     loop {
//         let current_height = current_height()?;

//         if current_height == target_height {
//             write_to_desk(TxMessage::NoKey, 200);
//             println!("At target height of {:?} - returning", target_height);
//             return Ok(());
//         }

//         if current_height < target_height {
//             println!(
//                 "Moving up. Current height: {:?}, target height: {:?}",
//                 current_height, target_height
//             );
//             write_to_desk(TxMessage::Up, 100);
//         } else {
//             println!(
//                 "Moving down. Current height: {:?}, target height: {:?}",
//                 current_height, target_height
//             );
//             write_to_desk(TxMessage::Down, 200);
//         }
//     }
// }

fn main() -> Result<(), Box<dyn Error>> {
    // rocket::ignite()
    //     .mount("/", routes![index, move_desk])
    //     .launch();

    // Ok(())

    let mut uart_desk_read = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;
    let mut uart_panel_write = Uart::with_path("/dev/ttyAMA2", 9600, Parity::None, 8, 1)?;

    uart_desk_read.set_read_mode(1, Duration::new(1, 0))?;

    let mut buf_desk_to_panel = [0u8; DATA_FRAME_SIZE];

    let (desk_to_panel_tx, desk_to_panel_rx) = unbounded::<[u8; DATA_FRAME_SIZE]>();

    spawn(move || loop {
        uart_panel_write
            .write(
                &mut desk_to_panel_rx
                    .recv()
                    .expect("Failed to receive on desk_to_panel_rx"),
            )
            .expect("Failed to write to panel uart");

        // It takes a bit over one millisecond to transfer each byte
        // (Blocking doesn't seem to work)
        // So we have to sleep for at least 7 and a bit milliseconds
        // to avoid sending overlapping frames.
        thread::sleep(Duration::from_millis(8));
    });

    // Spawn the thread and move ownership of the sending half into the new thread. This can also be
    // cloned if needed since there can be multiple producers.
    spawn(move || loop {
        if uart_desk_read
            .read(&mut buf_desk_to_panel)
            .expect("Failed to read from desk uart")
            > 0
        {
            if buf_desk_to_panel[0] == DATA_FRAME_START
                && buf_desk_to_panel[DATA_FRAME_SIZE - 1] == DATA_FRAME_END
            {
                println!("Sending on desk_to_panel_tx");
                desk_to_panel_tx
                    .send(buf_desk_to_panel)
                    .expect("Failed to send on desk_to_panel_tx");
            }
        }
    });

    // let mut data_frame_count = 0;
    // let mut dropped_frame_count = 0;
    // let mut current_height = 0.0;
    // loop {
    //     let (frame, dropped_count) = read_uart()?;
    //     data_frame_count += 1;
    //     dropped_frame_count += dropped_count;

    //     let height = read_height(frame)?;
    //     if current_height != height {
    //         current_height = height;
    //         println!("Current height: {:?} cm", current_height);
    //         println!(
    //             "Consumed frames: {:?}, dropped frames: {:?}",
    //             data_frame_count, dropped_frame_count
    //         );
    //     }
    // }

    // let mut uart_desk = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;

    // println!("Sending Up key 300 times");
    // write_to_desk(&mut uart_desk, TxMessage::Up, 300)?;

    // println!("Sending NoKey 100 times");
    // write_to_desk(&mut uart_desk, TxMessage::NoKey, 100)?;

    // println!("Sleeping for 0.5 seconds");
    // thread::sleep(Duration::from_millis(500));

    // println!("Sending Down key 300 times");
    // write_to_desk(&mut uart_desk, TxMessage::Down, 300)?;

    // println!("Sending NoKey 100 times");
    // write_to_desk(&mut uart_desk, TxMessage::NoKey, 100)?;

    // let mut uart_desk_write = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;

    loop {
        let (frame, _) = read_panel()?;
        let tx_message = TxMessage::from_frame(&frame);

        if tx_message != TxMessage::None {
            write_to_desk(tx_message, 10)?;
        }
    }

    // thread::sleep(Duration::new(60, 0));

    // Ok(())
}

fn read_desk() -> Result<(DataFrame, usize), Box<dyn Error>> {
    let mut uart_desk = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;
    read_uart(&mut uart_desk)
}

fn read_panel() -> Result<(DataFrame, usize), Box<dyn Error>> {
    let mut uart_panel = Uart::with_path("/dev/ttyAMA2", 9600, Parity::None, 8, 1)?;
    read_uart(&mut uart_panel)
}

fn read_uart(uart: &mut Uart) -> Result<(DataFrame, usize), Box<dyn Error>> {
    uart.set_read_mode(1, Duration::from_millis(100))?;

    let mut buffer = [0u8; DATA_FRAME_SIZE];

    let mut dropped_frame_count = 0;
    loop {
        if uart.read(&mut buffer)? > 0 {
            if buffer[0] == DATA_FRAME_START && buffer[DATA_FRAME_SIZE - 1] == DATA_FRAME_END {
                return Ok((buffer.to_vec(), dropped_frame_count));
            } else {
                dropped_frame_count += 1;
            }
        } else {
            return Ok((buffer.to_vec(), 0));
        }
    }
}

fn read_height_from_buffer(buf: &DataFrame) -> Result<f32, Box<dyn Error>> {
    Ok((256 * buf[3] as isize + buf[4] as isize + 650) as f32 / 10.0)
}

fn write_to_desk(tx_message: TxMessage, times: usize) -> Result<(), Box<dyn Error>> {
    let mut uart = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;

    for i in 0..times {
        let bytes_written_count = uart.write(&mut tx_message.as_frame())?;

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
