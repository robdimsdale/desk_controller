#![feature(decl_macro)]
use crossbeam_channel::unbounded;
use rocket::*;
use rppal::uart::{Parity, Uart};
use std::error::Error;
use std::thread;
use std::thread::spawn;
use std::time::Duration;

use rust_pi::*;

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
    if let (Some(frame), _) = read_desk()? {
        return read_height_cm_from_frame(&frame);
    }

    Ok(0.0)
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
    // spawn(move || {
    //     rocket::ignite()
    //         .mount("/", routes![index, move_desk])
    //         .launch();
    // });

    let mut uart_desk_read = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;

    uart_desk_read.set_read_mode(1, Duration::new(1, 0))?;

    let mut buf_desk_to_panel = [0u8; DATA_FRAME_SIZE];

    let (desk_to_panel_tx, desk_to_panel_rx) = unbounded::<[u8; DATA_FRAME_SIZE]>();

    spawn(move || loop {
        let frame = desk_to_panel_rx
            .recv()
            .expect("Failed to receive on desk_to_panel_rx");

        // println!("Received on desk_to_panel_rx: {:?}", received_frame);
        let message = RxMessage::from_frame(&frame.to_vec());

        match message {
            RxMessage::Height(h) => {
                if h < 6.50 || h > 129.5 {
                    println!("desk-to-panel abnormal height: {:?} - {:?}", h, frame);
                }
            }
            _ => {
                println!("desk-to-panel message: {:?} - {:?}", message, frame);
            }
        }

        write_to_panel(message, 1).expect("Failed to write to panel uart");
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
                // println!("Sending on desk_to_panel_tx: {:?}", buf_desk_to_panel);
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
        if let (Some(frame), _) = read_panel()? {
            let message = TxMessage::from_frame(&frame);

            match message {
                TxMessage::NoKey => {}
                _ => {
                    println!("panel-to-desk message: {:?} - {:?}", message, frame);
                }
            }

            // Write 10x messages to account for dropping ~90% of frames
            write_to_desk(message, 10)?;
        }
    }

    // thread::sleep(Duration::new(60, 0));

    // Ok(())
}

fn read_desk() -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    let mut uart_desk = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;
    read_uart(&mut uart_desk)
}

fn read_panel() -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    let mut uart_panel = Uart::with_path("/dev/ttyAMA2", 9600, Parity::None, 8, 1)?;
    read_uart(&mut uart_panel)
}

fn read_uart(uart: &mut Uart) -> Result<(Option<DataFrame>, usize), Box<dyn Error>> {
    uart.set_read_mode(1, Duration::from_millis(100))?;

    let mut buffer = [0u8; DATA_FRAME_SIZE];

    let mut dropped_frame_count = 0;
    loop {
        if uart.read(&mut buffer)? > 0 {
            if buffer[0] == DATA_FRAME_START && buffer[DATA_FRAME_SIZE - 1] == DATA_FRAME_END {
                return Ok((Some(buffer.to_vec()), dropped_frame_count));
            } else {
                dropped_frame_count += 1;
            }
        } else {
            return Ok((None, 0));
        }
    }
}

fn write_to_uart(
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

fn write_to_panel(rx_message: RxMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to panel: {:?}", times, rx_message);

    let mut uart = Uart::with_path("/dev/ttyAMA2", 9600, Parity::None, 8, 1)?;
    write_to_uart(&mut uart, &mut rx_message.as_frame(), times)
}

fn write_to_desk(tx_message: TxMessage, times: usize) -> Result<(), Box<dyn Error>> {
    // println!("Writing {:?} times to desk: {:?}", times, tx_message);

    let mut uart = Uart::with_path("/dev/ttyAMA1", 9600, Parity::None, 8, 1)?;
    write_to_uart(&mut uart, &mut tx_message.as_frame(), times)
}
