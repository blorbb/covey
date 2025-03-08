//! Actions returned by a plugin.

use core::fmt;
use std::path::PathBuf;

use az::SaturatingAs as _;

use crate::Plugin;

/// Event returned by a plugin.
pub(crate) enum PluginEvent {
    /// Set the displayed list.
    SetList {
        list: List,
        /// The action number this is associated with.
        ///
        /// If set list is called with an index older than the latest list,
        /// this event will be ignored.
        index: u64,
    },
    Close,
    Copy(String),
    SetInput(Input),
    DisplayError(String),
}

impl fmt::Debug for PluginEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetList { list, index: _ } => f
                .debug_tuple("SetList")
                .field(&format!("{} items", list.len()))
                .finish(),
            Self::Close => write!(f, "Close"),
            Self::Copy(arg0) => f.debug_tuple("Copy").field(arg0).finish(),
            Self::SetInput(arg0) => f.debug_tuple("SetInput").field(arg0).finish(),
            Self::DisplayError(arg0) => f.debug_tuple("DisplayError").field(arg0).finish(),
        }
    }
}

impl PluginEvent {
    pub(crate) fn from_proto_action(plugin: &Plugin, action: covey_proto::action::Action) -> Self {
        use covey_proto::action::Action as PrAction;
        match action {
            PrAction::Close(()) => Self::Close,
            PrAction::Copy(str) => Self::Copy(str),
            PrAction::SetInput(input) => Self::SetInput(Input::from_proto(plugin, input)),
            PrAction::DisplayError(err) => Self::DisplayError(err),
        }
    }
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

    pub(crate) fn from_proto(plugin: &Plugin, il: covey_proto::Input) -> Self {
        let mut input = Self {
            contents: il.query,
            selection: (il.range_lb.saturating_as(), il.range_ub.saturating_as()),
        };
        input.prefix_with(
            plugin
                .prefix()
                .expect("plugin with no prefix should never be queried"),
        );
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

    pub(crate) fn from_proto(
        plugin: &Plugin,
        icon_themes: &[String],
        proto: covey_proto::QueryResponse,
    ) -> Self {
        let style = proto.list_style.map(ListStyle::from_proto);
        let list: Vec<_> = proto
            .items
            .into_iter()
            .map(|li| ListItem {
                plugin: Plugin::clone(plugin),
                icon: li
                    .icon
                    .clone()
                    .and_then(|icon| ResolvedIcon::resolve(icon, icon_themes)),
                item: li,
            })
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
    pub(crate) fn from_proto(proto: covey_proto::query_response::ListStyle) -> Self {
        use covey_proto::query_response::ListStyle as Ls;
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
    item: covey_proto::ListItem,
    icon: Option<ResolvedIcon>,
}

impl ListItem {
    pub fn plugin(&self) -> &Plugin {
        &self.plugin
    }

    pub fn title(&self) -> &str {
        &self.item.title
    }

    pub fn description(&self) -> &str {
        &self.item.description
    }

    pub fn icon(&self) -> Option<&ResolvedIcon> {
        self.icon.as_ref()
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

/// Icon with named system icons resolved to a file path.
#[derive(Debug, Clone)]
pub enum ResolvedIcon {
    File(PathBuf),
    Text(String),
}

impl ResolvedIcon {
    pub(crate) fn resolve(
        proto: covey_proto::list_item::Icon,
        icon_themes: &[String],
    ) -> Option<Self> {
        use covey_proto::list_item::Icon as Proto;
        match proto {
            Proto::Name(name) => icon_themes
                .iter()
                .find_map(|theme| {
                    let path = freedesktop_icons::lookup(&name)
                        .with_theme(theme)
                        .with_size(48)
                        .with_cache()
                        .find()
                        .inspect(|path| {
                            tracing::trace!("found icon {name:?} with theme {theme:?} at {path:?}")
                        });

                    // lookup automatically goes through several backup options, including hicolor
                    // and other paths. do not count an icon as being found if a backup is used.
                    // To check whether a backup is used, see if the path contains the theme name.
                    // But special case hicolor to allow the backup paths to be used.
                    if theme == "hicolor"
                        || path.as_ref().is_some_and(|path| {
                            path.to_str().is_some_and(|str| str.contains(theme))
                        })
                    {
                        path
                    } else {
                        None
                    }
                })
                .map(Self::File),
            Proto::Text(text) => Some(Self::Text(text)),
        }
    }
}
