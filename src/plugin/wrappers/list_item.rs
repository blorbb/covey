use core::fmt;

use color_eyre::eyre::Result;

use super::Plugin;
use crate::plugin::{bindings, Action, InputLine};

/// A row in the results list.
///
/// Contains an associated plugin.
#[derive(Clone)]
pub struct ListItem {
    plugin: Plugin,
    item: bindings::ListItem,
}

impl ListItem {
    fn new(plugin: Plugin, item: bindings::ListItem) -> Self {
        Self { plugin, item }
    }

    pub(super) fn from_many_and_plugin(
        items: Vec<bindings::ListItem>,
        plugin: Plugin,
    ) -> Vec<Self> {
        items
            .into_iter()
            .map(|item| Self::new(plugin, item))
            .collect()
    }

    pub fn plugin(&self) -> Plugin {
        self.plugin
    }

    pub fn title(&self) -> &str {
        &self.item.title
    }

    pub fn description(&self) -> &str {
        &self.item.description
    }

    pub fn icon(&self) -> Option<&str> {
        self.item.icon.as_deref()
    }

    pub async fn activate(self) -> Result<Vec<Action>> {
        self.plugin.clone().activate(self.as_ref()).await
    }

    pub async fn complete(self, query: &str) -> Result<Option<InputLine>> {
        self.plugin.clone().complete(query, self.as_ref()).await
    }
}

impl fmt::Debug for ListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListItem")
            .field("plugin", &self.plugin())
            .field("title", &self.item.title)
            .field("description", &self.item.description)
            .field("metadata", &self.item.metadata)
            .field("icon", &self.item.icon)
            .finish()
    }
}

impl AsRef<bindings::ListItem> for ListItem {
    fn as_ref(&self) -> &bindings::ListItem {
        &self.item
    }
}
