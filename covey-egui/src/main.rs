#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::io;

use color_eyre::eyre::Result;
use eframe::egui;
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName,
    tokio::Stream,
    traits::tokio::{Listener, Stream as _},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

// https://github.com/emilk/egui/blob/main/examples/serial_windows/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let name = "covey.sock".to_ns_name::<GenericNamespaced>()?;
    let listener = match ListenerOptions::new().name(name.clone()).create_tokio() {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            tracing::info!("address in use");
            // connect to the existing socket and ask it to open
            let (mut rx, mut tx) = Stream::connect(name).await?.split();

            tx.write_all(b"open").await?;
            drop(tx);

            // wait for a response just to confirm
            rx.read_to_end(&mut Vec::new()).await?;
            drop(rx);

            return Ok(());
        }
        x => x?,
    };

    loop {
        tracing::info!("Starting window");

        eframe::run_native(
            "First Window",
            options.clone(),
            Box::new(|_cc| Ok(Box::new(MyApp { has_next: true }))),
        )?;

        let (mut rx, tx) = listener.accept().await?.split();

        let mut request = String::new();
        rx.read_to_string(&mut request).await?;
        drop(rx);

        match &*request {
            "open" => {}
            "exit" => break,
            _ => tracing::error!("unknown request {request:?}"),
        }

        // complete request, nothing to send
        drop(tx);
    }

    tracing::info!("exiting");
    Ok(())
}

struct MyApp {
    pub(crate) has_next: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let label_text = if self.has_next {
                "When this window is closed the next will be opened after a short delay"
            } else {
                "This is the last window. Program will end when closed"
            };
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
