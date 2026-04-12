#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use covey_egui::{App, AppControlFlow, cli};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

    let Some((settings, cli_rx)) = cli::listener()? else {
        // Another instance is already open, quit.
        return Ok(());
    };

    tracing::info!("Starting window");
    let mut app = App::new(cli_rx, settings)?;
    app.open()?;

    'main: loop {
        tracing::info!("closed window");

        // If the app received an exit signal while open, stop
        match app.cli.last_handled_msg() {
            Some(cli::Action::Exit) => break 'main,
            // Ignore an open that was already handled
            // (ignored bc its already open or just the last open message)
            Some(cli::Action::Open | cli::Action::OpenAndStay) => {}
            None => {}
        }

        tracing::info!("continuing listening in background");
        'background_tasks: loop {
            let mut dummy_rendering_state = covey_egui::RenderingState {
                new_cursor_selection: None,
                list_selection_changed: false,
                window_is_focused: false,
            };

            let control_flow = tokio::select! {
                action = app.plugin_actions.recv() => {
                    tracing::debug!("received plugin action {action:?}");
                    app.handle_plugin_action(None, action, &mut dummy_rendering_state)
                }
                action = app.cli.recv() => {
                    tracing::debug!("received cli action {action:?}");
                    app.handle_cli_action(None, action)
                }
            };

            match control_flow {
                AppControlFlow::Continue => {}
                AppControlFlow::OpenGui => break 'background_tasks,
                AppControlFlow::CloseGui => {}
                AppControlFlow::ExitProcess => break 'main,
            }
        }

        tracing::info!("Starting window");
        app.open()?;
    }

    tracing::info!("exiting");
    Ok(())
}
