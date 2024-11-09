use std::path::Path;

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncReadExt as _};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub plugins: Vec<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub prefix: String,
}

impl Config {
    // TODO: make this sans-io
    pub async fn read(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(path)
            .await?;

        let mut c = String::new();
        file.read_to_string(&mut c).await?;
        Ok(toml::from_str(&c)?)
    }
}
