use core::fmt;

use super::Plugin;
use crate::plugin::proto;

/// A row in the results list.
#[derive(Clone)]
pub struct ListItem {
    plugin: Plugin,
    item: proto::ListItem,
}

impl ListItem {
    pub(crate) fn new(plugin: Plugin, item: proto::ListItem) -> Self {
        Self { plugin, item }
    }

    pub fn plugin(&self) -> &Plugin {
        &self.plugin
    }

    pub fn title(&self) -> &str {
        &self.item.title
    }

    pub fn description(&self) -> &str {
        &self.item.description
    }

    pub fn icon(&self) -> Option<Icon> {
        self.item.icon.clone().map(Icon::from_proto)
    }

    pub fn id(&self) -> ListItemId {
        ListItemId {
            plugin: Plugin::clone(&self.plugin),
            local_id: self.item.id,
        }
    }
}

impl fmt::Debug for ListItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListItem")
            .field("plugin", &self.plugin())
            .field("title", &self.item.title)
            .field("description", &self.item.description)
            .field("icon", &self.item.icon)
            .finish()
    }
}

/// A list item without rendering details (description, etc).
///
/// Used by the model to call functions on this list item.
///
/// This should usually be constructed by [`ListItem::id`]. However,
/// all fields are public, so it can be constructed elsewhere. This
/// struct does not guarantee that the local ID is known to the plugin.
#[derive(Debug, Clone)]
pub struct ListItemId {
    pub plugin: Plugin,
    /// ID unique within the plugin.
    pub local_id: u64,
}

#[derive(Debug, Clone)]
pub enum Icon {
    Name(String),
    Text(String),
}

impl Icon {
    pub(crate) fn from_proto(proto: proto::list_item::Icon) -> Self {
        use proto::list_item::Icon as Proto;
        match proto {
            Proto::Name(name) => Self::Name(name),
            Proto::Text(text) => Self::Text(text),
        }
    }
}
