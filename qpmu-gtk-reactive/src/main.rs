mod reactive;
mod styles;
mod ui;
mod hotkey;

use any_spawner::Executor;
use color_eyre::eyre::Result;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkApplicationExt, WidgetExt};
use reactive_graph::owner::{on_cleanup, Owner};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // https://stackoverflow.com/a/77485843
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()?
        .add_directive("qpmu=debug".parse()?)
        .add_directive("reactive_graph=trace".parse()?);
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_env_filter(filter)
        .init();

    new_instance();
    Ok(())
}

fn new_instance() {
    info!("starting up app");

    gtk::init().unwrap();
    Executor::init_glib().unwrap();

    let owner = Owner::new();
    owner.set();
    on_cleanup(|| eprintln!("CLEANUP"));

    let app = gtk::Application::builder()
        .application_id("blorbb.qpmu")
        .build();

    app.connect_startup(move |app| {
        let window = ui::root();
        app.add_window(&window);
        eprintln!("window");
    });
    app.connect_activate(move |app| {
        if let Some(window) = app.active_window() {
            window.set_visible(true);
            eprintln!("activated");
        }
    });
    app.connect_shutdown(move |_| {
        owner.cleanup();
    });
    app.run();
}
