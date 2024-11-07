use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use install::install_plugin;
use model::Launcher;
use relm4::RelmApp;
use std::path::PathBuf;
use std::process;
use std::sync::LazyLock;

static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("missing config directory")
        .join("qpmu")
});
static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("plugins"));

mod config;
mod install;
mod model;
mod plugins;
mod styles;
mod ui;

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
    let args = Args::parse();

    match args.command {
        Some(Command::Install { rest }) => {
            process::exit(install_plugin(&rest)?.code().unwrap_or(1));
        }
        None => new_instance(),
    }
}

fn new_instance() -> Result<()> {
    let app = RelmApp::new("r4.qpmu");
    app.run::<Launcher>(());

    Ok(())
}
