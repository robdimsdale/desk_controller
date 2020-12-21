#![feature(decl_macro)]

use crossbeam_channel::unbounded;
use rocket::response::status::BadRequest;
use rocket::*;
use rust_pi::*;
use std::error::Error;
use std::thread::spawn;

#[get("/")]
fn index() -> String {
    let (desk_found_frames, desk_dropped_bytes) = rust_pi::desk_frame_counts();
    let (panel_found_frames, panel_dropped_bytes) = rust_pi::panel_frame_counts();
    format!(
        "Current Height: {:?} cm\nTarget Height: {:?} cm\nCurrent Panel Key: {:?}\nDesk - frames found: {:?}, bytes dropped: {:?} ({:?}%)\nPanel - frames found: {:?}, bytes dropped: {:?} ({:?}%)",
        rust_pi::current_height(),
        rust_pi::target_height(),
        rust_pi::current_panel_key(),
        desk_found_frames,
        desk_dropped_bytes,
    100.0*desk_dropped_bytes as f32 / (desk_found_frames*DATA_FRAME_SIZE + desk_dropped_bytes) as f32,
        panel_found_frames,
        panel_dropped_bytes,
        100.0*panel_dropped_bytes as f32 / (panel_found_frames *DATA_FRAME_SIZE+ panel_dropped_bytes) as f32,
    )
}

#[get("/move_desk/<target_height>")]
fn move_desk(target_height: f32) -> Result<(), BadRequest<String>> {
    rust_pi::move_to_height(target_height).map_err(|e| BadRequest(Some(e.to_string())))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (ctl_tx, ctl_rx) = unbounded::<bool>();

    ctrlc::set_handler(move || {
        println!("received kill signal");
        rust_pi::shutdown().expect("Failed to shutdown");

        ctl_tx.send(true).expect("Failed to send shutdown signal");
        // TODO: wait for acknowledgement from run loops

        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    rust_pi::initialize()?;

    spawn(move || {
        rocket::ignite()
            .mount("/", routes![index, move_desk])
            .launch();
    });

    rust_pi::run(ctl_rx)
}
