use std::process;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use install::install_plugin;
use model::Launcher;
use relm4::RelmApp;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

mod install;
mod model;
mod styles;
mod ui;
mod hotkey;
mod tray_icon;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Install { rest: Vec<String> },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    // https://stackoverflow.com/a/77485843
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()?
        .add_directive("qpmu=info".parse()?);
    tracing_subscriber::fmt().with_env_filter(filter).init();

    relm4::RELM_THREADS
        .set(2)
        .expect("failed to set background threads");
    let args = Args::parse();

    match args.command {
        Some(Command::Install { rest }) => {
            process::exit(install_plugin(&rest)?.code().unwrap_or(1));
        }
        None => new_instance(),
    }
}

fn new_instance() -> Result<()> {
    info!("starting up app");

    let app = RelmApp::new("blorbb.qpmu");
    app.run::<Launcher>(());

    Ok(())
}
