#![feature(decl_macro)]

use crossbeam_channel::unbounded;
use rocket::*;
use std::error::Error;
use std::thread::spawn;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let (ctl_tx, ctl_rx) = unbounded::<bool>();

    ctrlc::set_handler(move || {
        println!("received kill signal");
        desk_controller::shutdown().expect("Failed to shutdown");

        ctl_tx.send(true).expect("Failed to send shutdown signal");
        // TODO: wait for acknowledgement from run loops

        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    desk_controller::initialize()?;

    spawn(move || {
        rocket::ignite()
            .mount(
                "/",
                routes![
                    web::index,
                    web::current_height,
                    web::move_desk,
                    web::clear_target_height
                ],
            )
            .launch();
    });

    desk_controller::run(ctl_rx)
}

mod web {
    use desk_controller::DATA_FRAME_SIZE;
    use rocket::response::status::BadRequest;
    use rocket::*;

    #[get("/")]
    pub fn index() -> String {
        let (desk_found_frames, desk_dropped_bytes) = desk_controller::desk_frame_counts();
        let (panel_found_frames, panel_dropped_bytes) = desk_controller::panel_frame_counts();

        format!(
            "Current Height: {:?} cm\nTarget Height: {:?} cm\nCurrent Panel Key: {:?}\nDesk - frames found: {:?}, bytes dropped: {:?} ({:?}%)\nPanel - frames found: {:?}, bytes dropped: {:?} ({:?}%)",
            desk_controller::current_height(),
            desk_controller::target_height(),
            desk_controller::current_panel_key(),
            desk_found_frames,
            desk_dropped_bytes,
            100.0*desk_dropped_bytes as f32 / (desk_found_frames*DATA_FRAME_SIZE + desk_dropped_bytes) as f32,
            panel_found_frames,
            panel_dropped_bytes,
            100.0*panel_dropped_bytes as f32 / (panel_found_frames *DATA_FRAME_SIZE+ panel_dropped_bytes) as f32,
        )
    }

    #[get("/move_desk/<target_height>")]
    pub fn move_desk(target_height: f32) -> Result<(), BadRequest<String>> {
        desk_controller::move_to_height(target_height).map_err(|e| BadRequest(Some(e.to_string())))
    }

    #[get("/clear_target_height")]
    pub fn clear_target_height() {
        desk_controller::clear_target_height()
    }

    #[get("/current_height")]
    pub fn current_height() -> String {
        format!("{}", desk_controller::current_height())
    }
}
