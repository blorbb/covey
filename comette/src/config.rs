use std::{fs, io::Read as _};

use color_eyre::eyre::Result;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{plugin::Plugin, CONFIG_PATH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    #[serde(default)]
    pub(super) plugins: IndexMap<String, PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PluginConfig {
    pub(crate) prefix: String,
    #[serde(default)] // empty table if missing
    pub(crate) config: toml::Table,
}

impl GlobalConfig {
    /// Reads the plugin manifests of every plugin.
    #[tracing::instrument(skip_all)]
    pub fn load(self) -> IndexSet<Plugin> {
        info!("loading plugins from config");
        self.plugins
            .into_iter()
            .filter_map(|(name, plugin)| match Plugin::new(name, plugin) {
                Ok(plugin) => {
                    debug!("found plugin {plugin:?}");
                    Some(plugin)
                }
                Err(e) => {
                    error!("error finding plugin: {e}");
                    None
                }
            })
            .collect()
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
}
