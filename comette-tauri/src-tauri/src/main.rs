// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_eyre::eyre::Result;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // https://stackoverflow.com/a/77485843
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()?
        .add_directive("comette=debug".parse()?);
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tauri::async_runtime::set(tokio::runtime::Handle::current());
    comette_tauri::run();
    Ok(())
}
