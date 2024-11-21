use std::{fs, io::Read as _};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{plugin::Plugin, CONFIG_PATH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    plugins: Vec<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    pub(crate) name: String,
    pub(crate) prefix: String,
    #[serde(default)] // empty table if missing
    pub(crate) config: toml::Table,
}

impl Config {
    #[tracing::instrument]
    pub fn load_plugins() -> Result<Vec<Plugin>> {
        info!("loading plugins");

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(false)
            .open(&*CONFIG_PATH)?;

        let mut s = String::new();

        file.read_to_string(&mut s)?;
        debug!("read config {s:?}");
        let config: Self = toml::from_str(&s)?;

        let mut v = vec![];
        for plugin in config.plugins {
            match Plugin::new(plugin) {
                Ok(plugin) => {
                    debug!("found plugin {plugin:?}");
                    v.push(plugin)
                }
                Err(e) => error!("error finding plugin: {e}"),
            }
        }

        info!("found plugins {v:?}");

        Ok(v)
    }
}
