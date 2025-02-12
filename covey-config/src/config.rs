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
// every field must have a default annotation
// so that a partially filled config still works
pub struct AppConfig {
    /// Hotkey to re-initialise the current plugin.
    ///
    /// Default is Ctrl+R.
    #[serde(default = "default_reload_hotkey")]
    pub reload_hotkey: Hotkey,
    /// List of icon themes to use when rendering a named icon from a plugin.
    ///
    /// Icons will try to be loaded from top to bottom.
    #[serde(default = "default_icon_themes")]
    pub icon_themes: Vec<IconTheme>,
}

/// A theme to try render a named icon with.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub struct IconTheme {
    pub kind: IconThemeKind,
    /// Name of the icon theme within the kind.
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[serde(rename_all = "kebab-case")]
pub enum IconThemeKind {
    /// Icon theme from the operating system.
    ///
    /// On linux, "hicolor" is usually the default system theme.
    System,
    /// Icon theme from [iconify-icon](https://icon-sets.iconify.design/).
    ///
    /// The associated name should be the prefix of the icon set, before the colon.
    ///
    /// E.g. Phosphor icons has prefix "ph", Material Design Icons has prefix "mdi".
    IconifyIcon,
}

impl Default for AppConfig {
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

fn default_icon_themes() -> Vec<IconTheme> {
    vec![IconTheme {
        kind: IconThemeKind::System,
        name: "hicolor".to_string(),
    }]
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
