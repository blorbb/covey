use std::collections::HashMap;

use covey_manifest::{
    commands::Hotkey,
    ordered_map::{HasId, Id, OrderedMap},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::Plugin;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct GlobalConfig {
    #[serde(default)]
    pub plugins: OrderedMap<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct PluginConfig {
    pub id: Id,
    pub prefix: String,
    #[serde(default)] // empty table if missing
    pub config: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub commands: HashMap<Id, Hotkey>,
}

impl HasId for PluginConfig {
    fn id(&self) -> &Id {
        &self.id
    }
}

impl GlobalConfig {
    /// Finds and reads the manifests of every plugin.
    pub(crate) fn load_plugins(&self) -> OrderedMap<Plugin> {
        OrderedMap::new_lossy(self.plugins.iter().filter_map(|config| {
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
