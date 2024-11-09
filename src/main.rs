use std::{path::PathBuf, process, sync::LazyLock};

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use install::install_plugin;
use model::Launcher;
use relm4::RelmApp;
use tracing::{info, instrument, Level};

static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("missing config directory")
        .join("qpmu")
});
static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("plugins"));

mod config;
mod install;
mod model;
mod plugin;
mod styles;
mod ui;
pub mod utils;

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
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    relm4::RELM_THREADS
        .set(20)
        .expect("failed to set background threads");
    let args = Args::parse();

    match args.command {
        Some(Command::Install { rest }) => {
            process::exit(install_plugin(&rest)?.code().unwrap_or(1));
        }
        None => new_instance(),
    }
}

#[instrument]
fn new_instance() -> Result<()> {
    info!("starting up app");
    let app = RelmApp::new("r4.qpmu");
    app.run::<Launcher>(());

    Ok(())
}
