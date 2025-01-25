//! Types which are used by both src-tauri and src (frontend).
//!
//! Add the feature `build` when this is used as a build dependency.
//! This exposes the function [`export_ts_to`], which should be used to
//! generate types for Typescript.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// This must have an equivalent type on the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "build", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Event {
    SetInput {
        contents: String,
        selection: (u16, u16),
    },
    SetList {
        items: Vec<ListItem>,
        style: Option<ListStyle>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "build", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub title: String,
    pub description: String,
    pub icon: Option<Icon>,
    pub id: ListItemId,
    pub available_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "build", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ListItemId {
    pub local_id: u64,
    pub plugin_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "build", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Icon {
    File { path: PathBuf },
    Text { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "build", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ListStyle {
    Rows,
    Grid,
    GridWithColumns { columns: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "build", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "build", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Key {
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Backtick,
    Hyphen,
    Equal,
    Tab,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Apostrophe,
    Enter,
    Comma,
    Period,
    Slash,
}

#[cfg(feature = "build")]
pub fn export_ts_to(path: impl AsRef<std::path::Path>) {
    use ts_rs::TS;
    let path = path.as_ref();

    covey_manifest::PluginManifest::export_all_to(&path).unwrap();
    crate::Event::export_all_to(&path).unwrap();
    crate::Hotkey::export_all_to(&path).unwrap();
}
