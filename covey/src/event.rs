//! Actions returned by a plugin.

use std::{fmt, path::PathBuf};

use covey_schema::{hotkey::Hotkey, manifest::Command};

use crate::{Host, Plugin};

/// An action that should be performed by the frontend.
#[derive(Debug)]
pub enum Action {
    Close,
    SetList(List),
    Copy(String),
    SetInput(Input),
    DisplayError(String, String),
}

/// The main text input contents and selection.
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub contents: String,
    /// Range in terms of chars, not bytes
    pub selection: (usize, usize),
}

impl Input {
    pub(crate) fn prefix_with(&mut self, prefix: &str) {
        self.contents.insert_str(0, prefix);
        let prefix_len = prefix.chars().count();

        let (a, b) = self.selection;
        self.selection = (a.saturating_add(prefix_len), b.saturating_add(prefix_len));
    }

    pub(crate) fn from_proto(plugin: &Plugin, il: covey_proto::Input) -> Self {
        let mut input = Self {
            contents: il.query,
            selection: (il.selection.start, il.selection.end),
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
pub struct List {
    pub items: Vec<ListItem>,
    pub style: Option<ListStyle>,
    pub plugin: Plugin,
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("List")
            .field(
                "items",
                &self
                    .items
                    .iter()
                    .take(3)
                    .map(|item| item.title())
                    .collect::<Box<[_]>>(),
            )
            .field("style", &self.style)
            .field("plugin", &self.plugin)
            .finish()
    }
}

impl List {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn from_proto(host: &Host, plugin: &Plugin, proto: covey_proto::List) -> Self {
        let style = proto.style.map(ListStyle::from_proto);
        let list: Vec<_> = proto
            .items
            .into_iter()
            .map(|li| ListItem {
                plugin: Plugin::clone(plugin),
                icon: li
                    .icon
                    .clone()
                    .and_then(|icon| ResolvedIcon::resolve(host, icon)),
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
    pub(crate) fn from_proto(proto: covey_proto::ListStyle) -> Self {
        match proto {
            covey_proto::ListStyle::Rows => Self::Rows,
            covey_proto::ListStyle::Grid => Self::Grid,
            covey_proto::ListStyle::GridWithColumns(columns) => Self::GridWithColumns(columns),
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

    /// The commands that can be activated on this list item as reported by the
    /// plugin.
    ///
    /// Commands will be given in the returned iterator in the same order as the
    /// the commands are defined in the plugin's manifest.
    pub fn available_commands(&self) -> impl Iterator<Item = &Command> {
        self.plugin()
            .manifest()
            .commands
            .iter()
            .filter(|cmd| self.item.available_commands.contains(&cmd.id))
    }

    /// Gets the command that can be activated from the provided hotkey.
    pub fn activated_command_from_hotkey(&self, hotkey: &Hotkey) -> Option<&Command> {
        self.available_commands().find(|cmd| {
            self.plugin()
                .hotkeys_of_cmd(&cmd.id)
                .is_some_and(|hotkeys| hotkeys.contains(&hotkey))
        })
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
    pub local_id: covey_proto::ListItemId,
}

/// Icon with named system icons resolved to a file path.
#[derive(Debug, Clone)]
pub enum ResolvedIcon {
    File(PathBuf),
    Text(String),
}

impl ResolvedIcon {
    pub fn resolve_icon_name(host: &Host, name: &str) -> Option<PathBuf> {
        host.config().app.icon_themes.iter().find_map(|theme| {
            let path = freedesktop_icons::lookup(name)
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
                || path
                    .as_ref()
                    .is_some_and(|path| path.to_str().is_some_and(|str| str.contains(theme)))
            {
                path
            } else {
                None
            }
        })
    }

    pub(crate) fn resolve(host: &Host, proto: covey_proto::ListItemIcon) -> Option<Self> {
        // `freedesktop_icons::lookup` can do filesystem reads, which is blocking.
        // Maybe this function should be async. But this is used on the path of turning
        // responses to actions, which is tricky to turn async.
        //
        // Only new icons will need to perform a filesystem lookup. Most icons should be
        // cached, which is a fast lookup and doesn't block.
        match proto {
            covey_proto::ListItemIcon::Name(name) => {
                Self::resolve_icon_name(host, &name).map(Self::File)
            }
            covey_proto::ListItemIcon::Text(text) => Some(Self::Text(text)),
        }
    }
}
