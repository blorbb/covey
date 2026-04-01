//! Types for the user config.

use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    hotkey::{Hotkey, KeyCode},
    id::{CommandId, PluginId},
    keyed_list::{Identify, KeyedList},
    style::UserStyle,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct GlobalConfig {
    #[serde(default)]
    pub app: AppSettings,
    #[serde(default)]
    pub style: UserStyle,
    #[serde(default)]
    pub plugins: KeyedList<PluginEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case", default)]
// every field must have a default annotation
// so that a partially filled config still works
pub struct AppSettings {
    /// Hotkey to re-initialise the current plugin.
    ///
    /// Default is Ctrl+R.
    #[serde(default = "default_reload_hotkey")]
    pub reload_hotkey: Hotkey,
    /// List of system icon themes to use when rendering a named icon from a
    /// plugin.
    ///
    /// Icons will try to be loaded from top to bottom.
    #[serde(default = "default_icon_themes")]
    pub icon_themes: Arc<[String]>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            reload_hotkey: default_reload_hotkey(),
            icon_themes: default_icon_themes(),
        }
    }
}

fn default_reload_hotkey() -> Hotkey {
    Hotkey {
        key: KeyCode::R,
        ctrl: true,
        alt: false,
        shift: false,
        meta: false,
    }
}

fn default_icon_themes() -> Arc<[String]> {
    Arc::from([String::from("hicolor")])
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct PluginEntry {
    pub id: PluginId,
    /// Disables this plugin.
    ///
    /// This is `false` by default. The plugin will also be disabled if no
    /// prefix is defined (no user prefix or default prefix).
    #[serde(default)]
    pub disabled: bool,
    /// Prefix to select this plugin.
    ///
    /// This or a default prefix must be defined. If both aren't defined, this
    /// plugin will be disabled.
    pub prefix: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub settings: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub commands: BTreeMap<CommandId, CommandSettings>,
}

impl Identify for PluginEntry {
    type Id = PluginId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl PluginEntry {
    pub fn new(plugin_id: PluginId) -> Self {
        // make sure these match with the serde default annotations!
        Self {
            id: plugin_id,
            disabled: false,
            prefix: None,
            settings: serde_json::Map::new(),
            commands: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct CommandSettings {
    // This should be Option<Vec> instead of an empty/default vec
    // to distinguish between a command with no hotkeys, and
    // a command without user-set hotkeys (use plugin defaults).
    pub hotkeys: Option<Vec<Hotkey>>,
}
