use std::{fs, io::Read as _, mem};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{plugin::Plugin, CONFIG_PATH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub(super) plugins: Vec<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PluginConfig {
    pub(crate) name: String,
    pub(crate) prefix: String,
    #[serde(default)] // empty table if missing
    pub(crate) config: toml::Table,
}

impl Config {
    #[tracing::instrument(skip_all)]
    pub fn load(self) -> Vec<Plugin> {
        info!("loading plugins from config");
        let mut v = vec![];
        for plugin in self.plugins {
            match Plugin::new(plugin) {
                Ok(plugin) => {
                    debug!("found plugin {plugin:?}");
                    v.push(plugin);
                }
                Err(e) => error!("error finding plugin: {e}"),
            }
        }

        info!("found plugins {v:?}");
        v
    }

    #[tracing::instrument]
    pub fn from_file() -> Result<Self> {
        info!("loading config from {:?}", &*CONFIG_PATH);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(false)
            .open(&*CONFIG_PATH)?;

        let mut s = String::new();

        file.read_to_string(&mut s)?;
        debug!("read config {s:?}");
        Ok(toml::from_str(&s)?)
    }

    /// Modifies the `plugins` list to be ordered the same as the provided
    /// list of plugins, preserving existing configuration.
    pub fn reorder_plugins<'plugin>(
        &mut self,
        new_order: impl IntoIterator<Item = &'plugin Plugin>,
    ) {
        let new_plugin_configs: Vec<PluginConfig> = new_order
            .into_iter()
            .map(|plugin| {
                self.plugins
                    .iter_mut()
                    .find_map(|existing| {
                        (&existing.name == plugin.name()).then(|| mem::take(existing))
                    })
                    // insert a new config
                    .unwrap_or_else(|| PluginConfig {
                        name: plugin.name().to_string(),
                        prefix: plugin.prefix().to_string(),
                        config: toml::Table::default(),
                    })
            })
            .collect();
        self.plugins = new_plugin_configs;
    }
}
