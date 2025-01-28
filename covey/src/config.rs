use std::collections::HashMap;

use covey_manifest::commands::{CommandId, Hotkey};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::Plugin;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct GlobalConfig {
    #[serde(default)]
    pub plugins: IndexMap<String, PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct PluginConfig {
    pub prefix: String,
    #[serde(default)] // empty table if missing
    pub config: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub commands: HashMap<CommandId, Hotkey>,
}

impl GlobalConfig {
    /// Finds and reads the manifests of every plugin.
    pub(crate) fn load_plugins(&self) -> IndexSet<Plugin> {
        self.plugins
            .iter()
            .filter_map(
                |(name, plugin)| match Plugin::new(name.clone(), plugin.clone()) {
                    Ok(plugin) => {
                        debug!("found plugin {plugin:?}");
                        Some(plugin)
                    }
                    Err(e) => {
                        error!("error finding plugin: {e}");
                        None
                    }
                },
            )
            .collect()
    }
}
