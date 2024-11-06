use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context, Result};
use config::Config;
use install::install_plugin;
use model::Launcher;
use plugins::Plugin;
use relm4::RelmApp;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{fs, process};

const SOCKET_ADDR: &str = "127.0.0.1:7547";

static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("missing config directory")
        .join("qpmu")
});
static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("plugins"));

mod config;
mod install;
mod plugins;
mod ui;
mod model;

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
        None => try_run_instance(),
    }
}

fn try_run_instance() -> Result<()> {
    match TcpListener::bind(SOCKET_ADDR) {
        Ok(listener) => new_instance(listener)?,
        Err(_) => {
            // another instance running
            println!("activating other instance");
            TcpStream::connect(SOCKET_ADDR)?
                .write_all(b"1")
                .context("error writing to stream")?;
        }
    }

    Ok(())
}

fn new_instance(listener: TcpListener) -> Result<()> {
    let plugins = &*PLUGINS_DIR;
    if !plugins.is_dir() {
        fs::create_dir_all(plugins).context("could not create qpmu/plugins directory")?;
    }

    let config = Config::read()?;
    let plugins: Vec<_> = config
        .plugins
        .iter()
        .inspect(|p| eprintln!("loading plugin {}", p.name))
        .filter_map(|p| {
            Plugin::from_config(p.clone())
                .inspect_err(|e| eprintln!("{e}"))
                .ok()
        })
        .collect();

    let app = RelmApp::new("r4.qpmu");
    app.run::<Launcher>(plugins);

    Ok(())
}
