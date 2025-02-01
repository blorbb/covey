//! Types for the user config.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    hotkey::Hotkey,
    keyed_list::{Key, Keyed, KeyedList},
};

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
