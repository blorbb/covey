use std::collections::HashMap;

use covey_config::{
    keyed_list::{Keyed, Key, KeyedList},
    manifest::Hotkey,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::Plugin;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct GlobalConfig {
    #[serde(default)]
    pub plugins: KeyedList<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct PluginConfig {
    pub id: Key,
    pub prefix: String,
    #[serde(default)] // empty table if missing
    pub config: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub commands: HashMap<Key, Hotkey>,
}

impl Keyed for PluginConfig {
    fn key(&self) -> &Key {
        &self.id
    }
}

impl GlobalConfig {
    /// Finds and reads the manifests of every plugin.
    pub(crate) fn load_plugins(&self) -> KeyedList<Plugin> {
        KeyedList::new_lossy(self.plugins.iter().filter_map(|config| {
            match Plugin::new(config.clone()) {
                Ok(plugin) => {
                    debug!("found plugin {plugin:?}");
                    Some(plugin)
                }
                Err(e) => {
                    error!("error finding plugin: {e}");
                    None
                }
            }
        }))
    }
}
