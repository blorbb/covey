use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::Plugin;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    #[serde(default)]
    pub(crate) plugins: IndexMap<String, PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub(crate) struct PluginConfig {
    pub(crate) prefix: String,
    #[serde(default)] // empty table if missing
    pub(crate) config: toml::Table,
}

impl GlobalConfig {
    /// Finds and reads the manifests of every plugin.
    pub(crate) fn load_plugins(self) -> IndexSet<Plugin> {
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
}
