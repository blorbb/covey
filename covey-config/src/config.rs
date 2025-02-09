//! Types for the user config.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    hotkey::{Hotkey, KeyCode},
    keyed_list::{Id, Identify, KeyedList},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub plugins: KeyedList<PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct AppConfig {
    /// Hotkey to re-initialise the current plugin.
    ///
    /// Default is Ctrl+R.
    reload_hotkey: Hotkey,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            reload_hotkey: Hotkey {
                key: KeyCode::R,
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct PluginConfig {
    pub id: Id,
    pub prefix: String,
    #[serde(default)] // empty table if missing
    pub config: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub commands: HashMap<Id, Hotkey>,
}

impl Identify for PluginConfig {
    fn id(&self) -> &Id {
        &self.id
    }
}
