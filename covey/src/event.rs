//! Actions returned by a plugin.

use core::fmt;

use az::SaturatingAs as _;

use crate::{proto, Plugin};

/// Event returned by a plugin.
pub(crate) enum PluginEvent {
    /// Set the displayed list.
    SetList { list: List, index: u64 },
    /// Run a sequence of actions.
    Run(Vec<Action>),
}

impl fmt::Debug for PluginEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetList { list, .. } => f
                .debug_tuple("PluginEvent::SetList")
                .field(&format!("{} items", list.len()))
                .finish(),
            Self::Run(actions) => f.debug_tuple("PluginEvent::Run").field(actions).finish(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Action {
    Close,
    RunCommand(String, Vec<String>),
    RunShell(String),
    Copy(String),
    SetInput(Input),
}

/// The main text input contents and selection.
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub contents: String,
    /// Range in terms of chars, not bytes
    pub selection: (u16, u16),
}

impl Input {
    pub(crate) fn prefix_with(&mut self, prefix: &str) {
        self.contents.insert_str(0, prefix);
        let prefix_len =
            u16::try_from(prefix.chars().count()).expect("prefix should not be insanely long");

        let (a, b) = self.selection;
        self.selection = (a.saturating_add(prefix_len), b.saturating_add(prefix_len));
    }

    pub(crate) fn from_proto(plugin: &Plugin, il: proto::Input) -> Self {
        let mut input = Self {
            contents: il.query,
            selection: (il.range_lb.saturating_as(), il.range_ub.saturating_as()),
        };
        input.prefix_with(plugin.prefix());
        input
    }
}

/// A list of results to show provided by a plugin.
#[derive(Debug)]
pub struct List {
    pub items: Vec<ListItem>,
    pub style: Option<ListStyle>,
    pub plugin: Plugin,
}

impl List {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn from_proto(plugin: &Plugin, proto: proto::QueryResponse) -> Self {
        let style = proto.list_style.map(ListStyle::from_proto);
        let list: Vec<_> = proto
            .items
            .into_iter()
            .map(|li| ListItem::new(Plugin::clone(plugin), li))
            .collect();
        Self {
            style,
            items: list,
            plugin: plugin.clone(),
        }
    }
}

/// The style to display the list provided by a plugin.
#[derive(Debug, Clone, Copy)]
pub enum ListStyle {
    Rows,
    Grid,
    GridWithColumns(u32),
}

impl ListStyle {
    pub(crate) fn from_proto(proto: proto::query_response::ListStyle) -> Self {
        use proto::query_response::ListStyle as Ls;
        match proto {
            Ls::Rows(()) => Self::Rows,
            Ls::Grid(()) => Self::Grid,
            Ls::GridWithColumns(columns) => Self::GridWithColumns(columns),
        }
    }
}

/// A single result provided by a plugin.
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

    pub fn available_commands(&self) -> &[String] {
        &self.item.available_commands
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
