#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use covey_egui::cli;
use eframe::egui;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

// https://github.com/emilk/egui/blob/main/examples/serial_windows/src/main.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://stackoverflow.com/a/77485843
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .unwrap()
        .add_directive("covey=debug".parse().unwrap());
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let options = eframe::NativeOptions {
        run_and_return: true,
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    // https://github.com/emilk/egui/blob/main/examples/external_eventloop/src/main.rs
    // let mut event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll);
    // https://docs.rs/tray-icon/0.21.2/tray_icon/
    // can't use a custom UserEvent with the event loop when using eframe::create_native
    // need to use some other kind of channel anyways to tell this app to open from another process

    let rx = match cli::listener()? {
        Some(rx) => rx,
        // Another instance is already open, quit.
        None => return Ok(()),
    };

    tracing::info!("Starting window");
    eframe::run_native(
        "covey",
        options.clone(),
        Box::new(|_cc| Ok(Box::new(MyApp { rx: rx.clone() }))),
    )?;

    loop {
        tracing::info!("closed window");

        // If the app received an exit signal while open, stop
        match rx.last_handled_msg() {
            Some(cli::Message::Exit) => break,
            // Ignore an open that was already handled
            // (ignored bc its already open or just the last open message)
            Some(cli::Message::Open) => {}
            None => {}
        }

        match rx.recv() {
            cli::Message::Open => {}
            cli::Message::Exit => break,
        }

        tracing::info!("Starting window");
        eframe::run_native(
            "covey",
            options.clone(),
            Box::new(|_cc| Ok(Box::new(MyApp { rx: rx.clone() }))),
        )?;
    }

    tracing::info!("exiting");
    Ok(())
}

struct MyApp {
    rx: cli::Receiver,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.rx.try_recv() {
                Some(cli::Message::Exit) => {
                    tracing::info!("received exit message");
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
                }
                // Trying to open while already open -> do nothing
                Some(cli::Message::Open) => {}
                None => {}
            }

            let label_text =
                "When this window is closed the next will be opened after a short delay";
            ui.label(label_text);

            // https://github.com/emilk/egui/issues/3655#issuecomment-3239209608
            // TODO: try the above
            if ui.button("Close").clicked() {
                tracing::info!("Pressed Close button");
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}
