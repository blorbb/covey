use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub plugins: Vec<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub prefix: String,
    #[serde(default)] // empty table if missing
    pub options: toml::Table,
}

impl Config {
    pub async fn read(file_contents: &str) -> Result<Self> {
        Ok(toml::from_str(file_contents)?)
    }
}
