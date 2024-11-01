use std::{fs, io::Read};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

use crate::CONFIG_DIR;

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
    pub fn read() -> Result<Self> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(CONFIG_DIR.join("config.toml"))?;

        let mut c = String::new();
        file.read_to_string(&mut c)?;
        Ok(toml::from_str(&c)?)
    }
}
