#![feature(decl_macro)]

#[cfg_attr(all(target_os = "linux", target_arch = "arm"), path = "rpi.rs")]
#[cfg_attr(
    not(all(target_os = "linux", target_arch = "arm")),
    path = "not_rpi.rs"
)]
mod os;

use crossbeam_channel::unbounded;
use rocket::*;
use rust_pi::*;
use std::error::Error;
use std::thread::spawn;

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
    match os::read_desk()? {
        (Some(frame), _) => match DeskToPanelMessage::from_frame(&frame) {
            DeskToPanelMessage::Height(h) => return Ok(h),
            _ => return Ok(0.0),
        },
        _ => return Ok(0.0),
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    spawn(move || {
        rocket::ignite()
            .mount("/", routes![index, move_desk])
            .launch();
    });

    let (desk_to_panel_tx, desk_to_panel_rx) = unbounded::<DeskToPanelMessage>();

    spawn(move || loop {
        let message = desk_to_panel_rx
            .recv()
            .expect("Failed to receive on desk_to_panel_rx");

        match message {
            DeskToPanelMessage::Height(h) => {
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
                    "desk-to-panel message: {:?} - {:?}",
                    message,
                    message.as_frame()
                );
            }
        }

        os::write_to_panel(message, 1).expect("Failed to write to panel");
    });

    // Spawn the thread and move ownership of the sending half into the new thread. This can also be
    // cloned if needed since there can be multiple producers.
    spawn(move || loop {
        if let (Some(frame), _) = os::read_desk().expect("Failed to read from desk") {
            let message = DeskToPanelMessage::from_frame(&frame);
            // println!("Sending on desk_to_panel_tx: {:?}", message);
            desk_to_panel_tx
                .send(message)
                .expect("Failed to send on desk_to_panel_tx");
        }
    });

    loop {
        if let (Some(frame), _) = os::read_panel()? {
            let message = PanelToDeskMessage::from_frame(&frame);

            match message {
                PanelToDeskMessage::NoKey => {}
                _ => {
                    println!("panel-to-desk message: {:?} - {:?}", message, frame);
                }
            }

            // Write 10x messages to account for dropping ~90% of frames
            os::write_to_desk(message, 10)?;
        }
    }
}
