#![feature(decl_macro)]

use crossbeam_channel::unbounded;
use rocket::*;
use rust_pi::*;
use std::error::Error;
use std::thread::spawn;

#[get("/")]
fn index() -> String {
    let height = rust_pi::current_height();
    format!("Current Height: {:?} cm", height)
}

#[get("/move_desk/<target_height>")]
fn move_desk(target_height: f32) -> String {
    // move_to_height_cm(target_height).unwrap();

    let height = rust_pi::current_height();
    format!("Current Height: {:?} cm", height)
}

fn main() -> Result<(), Box<dyn Error>> {
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        rust_pi::shutdown().expect("Failed to shutdown");

        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    rust_pi::initialize()?;

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

        rust_pi::write_to_panel(message, 1).expect("Failed to write to panel");
    });

    // Spawn the thread and move ownership of the sending half into the new thread. This can also be
    // cloned if needed since there can be multiple producers.
    spawn(move || loop {
        if let (Some(message), _) = rust_pi::read_desk().expect("Failed to read from desk") {
            // println!("Sending on desk_to_panel_tx: {:?}", message)b;
            desk_to_panel_tx
                .send(message)
                .expect("Failed to send on desk_to_panel_tx");
        }
    });

    loop {
        if let (Some(message), _) = rust_pi::read_panel()? {
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
            rust_pi::write_to_desk(message, 10)?;
        }
    }
}
