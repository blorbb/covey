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

#[cfg(feature = "build")]
pub fn export_ts_to(path: impl AsRef<std::path::Path>) {
    use ts_rs::TS;
    let path = path.as_ref();

    qpmu_manifest::PluginManifest::export_all_to(&path).unwrap();
    crate::Event::export_all_to(&path).unwrap();
}
