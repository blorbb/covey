use std::{path::PathBuf, process, sync::LazyLock};

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use install::install_plugin;
use model::Launcher;
use qpmu::{config::Config, plugin::Plugin};
use relm4::RelmApp;
use tokio::{fs, io::AsyncReadExt};
use tracing::{error, info, instrument, Level};

mod install;
mod model;
mod styles;
mod ui;

static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("missing config directory")
        .join("qpmu")
});
static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("plugins"));

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
    let plugins = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(load_plugins());
    info!("finished setting up plugins");

    let app = RelmApp::new("r4.qpmu");
    app.run::<Launcher>(plugins);

    Ok(())
}

pub async fn load_plugins() -> &'static [Plugin] {
    let config_path = CONFIG_DIR.join("config.toml");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(false)
        .open(config_path)
        .await
        .unwrap();

    let mut s = String::new();
    file.read_to_string(&mut s).await.unwrap();
    let config = Config::read(&s).await.unwrap();

    let mut v = vec![];
    for plugin in config.plugins {
        let name = plugin.name.replace('-', "_");
        info!("initialising plugin {name}");
        let path = format!("{name}.wasm");
        match Plugin::new(plugin, fs::read(PLUGINS_DIR.join(path)).await.unwrap()).await {
            Ok(plugin) => v.push(plugin),
            Err(e) => error!("{e}"),
        }
    }

    v.leak()
}
