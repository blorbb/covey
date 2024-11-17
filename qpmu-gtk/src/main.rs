use std::{fs, io::Read, process};

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use install::install_plugin;
use model::Launcher;
use qpmu::{config::Config, plugin::Plugin, CONFIG_DIR, PLUGINS_DIR};
use relm4::RelmApp;
use tracing::{error, info, instrument, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

mod install;
mod model;
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

    // https://stackoverflow.com/a/77485843
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()?
        .add_directive("qpmu=info".parse()?);
    tracing_subscriber::fmt().with_env_filter(filter).init();

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

pub fn load_plugins() -> &'static [Plugin] {
    let config_path = CONFIG_DIR.join("config.toml");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(false)
        .open(config_path)
        .unwrap();

    let mut s = String::new();

    file.read_to_string(&mut s).unwrap();
    let config = Config::read(&s).unwrap();

    let mut v = vec![];
    for plugin in config.plugins {
        match Plugin::new(plugin) {
            Ok(plugin) => v.push(plugin),
            Err(e) => error!("{e}"),
        }
    }

    v.leak()
}
