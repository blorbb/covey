#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use covey_egui::{App, cli};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

// https://github.com/emilk/egui/blob/main/examples/serial_windows/src/main.rs

// Need to use the multi-threaded runtime for some reason
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://stackoverflow.com/a/77485843
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .unwrap()
        .add_directive("covey=debug".parse().unwrap());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // https://github.com/emilk/egui/blob/main/examples/external_eventloop/src/main.rs
    //
    // ```
    // let mut event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll);
    // ```
    //
    // https://docs.rs/tray-icon/0.21.2/tray_icon/
    // Can't use a custom UserEvent with the event loop when using
    // eframe::create_native. Need to use some other kind of channel anyways to
    // tell this app to open from another process

    let (settings, rx) = match cli::listener()? {
        Some((settings, rx)) => (settings, rx),
        // Another instance is already open, quit.
        None => return Ok(()),
    };

    tracing::info!("Starting window");
    let mut app = App::new(&rx, settings)?;
    app.open()?;

    loop {
        tracing::info!("closed window");

        // If the app received an exit signal while open, stop
        match rx.last_handled_msg() {
            Some(cli::Message::Exit) => break,
            // Ignore an open that was already handled
            // (ignored bc its already open or just the last open message)
            Some(cli::Message::Open | cli::Message::OpenAndStay) => {}
            None => {}
        }

        match rx.recv() {
            cli::Message::Open => {}
            cli::Message::OpenAndStay => {
                app.gui_settings.close_on_blur = false;
            }
            cli::Message::Exit => break,
        }

        tracing::info!("Starting window");
        app.open()?;
    }

    tracing::info!("exiting");
    Ok(())
}
