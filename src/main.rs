use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use config::Config;
use install::install_plugin;
use model::Launcher;
use plugins::Plugin;
use relm4::RelmApp;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{fs, process};

static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("missing config directory")
        .join("qpmu")
});
static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("plugins"));

static PLUGINS: LazyLock<Vec<Plugin>> = LazyLock::new(|| {
    let plugins = &*PLUGINS_DIR;
    if !plugins.is_dir() {
        fs::create_dir_all(plugins).expect("could not create qpmu/plugins directory");
    }

    let config = Config::read().unwrap();
    config
        .plugins
        .iter()
        .inspect(|p| eprintln!("loading plugin {}", p.name))
        .filter_map(|p| {
            Plugin::from_config(p.clone())
                .inspect_err(|e| eprintln!("{e}"))
                .ok()
        })
        .collect()
});

mod config;
mod install;
mod model;
mod plugins;
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
