use std::{fs, path::PathBuf, process, sync::LazyLock};

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use install::install_plugin;
use model::Launcher;
use qpmu::{config::Config, plugin::Plugin};
use relm4::RelmApp;
use tracing::{info, instrument, Level};

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
    let plugins = tokio::runtime::Runtime::new().unwrap().block_on(load_plugins());

    let app = RelmApp::new("r4.qpmu");
    app.run_async::<Launcher>(plugins);

    Ok(())
}

pub async fn load_plugins() -> &'static [Plugin] {
    let config_path = CONFIG_DIR.join("config.toml");
    let config = Config::read(config_path).await.unwrap();

    let mut v = vec![];
    for plugin in config.plugins {
        let name = plugin.name.replace('-', "_");
        info!("initialising plugin {name}");
        let path = format!("{name}.wasm");
        let plugin = Plugin::new(plugin, fs::read(PLUGINS_DIR.join(path)).unwrap())
            .await
            .unwrap();
        v.push(plugin);
    }

    v.leak()
}
