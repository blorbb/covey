//! Actions returned by a plugin.

use std::{fmt, path::PathBuf};

use covey_schema::{hotkey::Hotkey, manifest::Command};

use crate::{Host, Plugin};

/// An internal message that needs to be processed to present an [`Action`] to
/// the user.
pub(crate) enum Message {
    Action(Action),
    PluginResponse(Plugin, covey_proto::Response),
}

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

/// A list of results to show provided by a plugin.
#[non_exhaustive]
pub struct List {
    pub items: Vec<ListItem>,
    pub(crate) activation_target: ActivationTarget,
    pub(crate) request_id: covey_proto::RequestId,
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("List")
            .field(
                "items",
                &self
                    .items
                    .iter()
                    .take(3)
                    .map(ListItem::title)
                    .collect::<Box<[_]>>(),
            )
            .field("plugin", &self.plugin())
            .field("activation_target", &self.activation_target)
            .field("request_id", &self.request_id)
            .finish_non_exhaustive()
    }
}

impl List {
    pub fn plugin(&self) -> &Plugin {
        &self.activation_target.plugin
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn is_response_of_latest_query(&self, host: &Host) -> bool {
        host.query_request_id_is_latest(self.request_id)
    }

    pub fn activation_target(&self) -> &ActivationTarget {
        &self.activation_target
    }
}

/// The style to display the list provided by a plugin.
#[derive(Debug, Clone, Copy)]
pub enum ListStyle {
    Rows,
    Grid,
    GridWithColumns(u32),
}

/// A single result provided by a plugin.
#[derive(Debug, Clone)]
pub struct ListItem {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) icon: Option<Icon>,
    pub(crate) activation_target: ActivationTarget,
}

impl ListItem {
    pub fn plugin(&self) -> &Plugin {
        &self.activation_target.plugin
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn icon(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    pub fn activation_target(&self) -> &ActivationTarget {
        &self.activation_target
    }
}

#[derive(Debug, Clone)]
pub struct ActivationTarget {
    pub(crate) plugin: Plugin,
    /// ID unique within the plugin.
    pub(crate) local_target_id: covey_proto::ActivationTarget,
    pub(crate) commands: Vec<covey_proto::CommandId>,
}

impl ActivationTarget {
    /// The commands that can be activated on this list item as reported by the
    /// plugin.
    ///
    /// Commands will be given in the returned iterator in the same order as the
    /// the commands are defined in the plugin's manifest.
    pub fn available_commands(&self) -> impl Iterator<Item = &Command> {
        self.plugin
            .manifest()
            .commands
            .iter()
            .filter(|cmd| self.commands.contains(&cmd.id))
    }

    /// Gets the command that can be activated from the provided hotkey.
    pub fn activated_command_from_hotkey(&self, hotkey: Hotkey) -> Option<&Command> {
        self.available_commands().find(|cmd| {
            self.plugin
                .hotkeys_of_cmd(&cmd.id)
                .is_some_and(|hotkeys| hotkeys.contains(&hotkey))
        })
    }

    pub fn plugin(&self) -> &Plugin {
        &self.plugin
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Icon(pub(crate) covey_proto::ListItemIcon);

impl Icon {
    /// New icon that will be searched in the file system.
    pub fn new_named(name: String) -> Self {
        Self(covey_proto::ListItemIcon::Name(name))
    }

    pub fn resolve(&self, host: &Host) -> Result<ResolvedIcon, ResolveIconError> {
        host.resolve_icon(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolveIconError {
    Loading,
    NotFound,
}

impl fmt::Display for ResolveIconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveIconError::Loading => write!(f, "icon loading"),
            ResolveIconError::NotFound => write!(f, "icon not found"),
        }
    }
}

impl std::error::Error for ResolveIconError {}

/// Icon with named system icons resolved to a file path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResolvedIcon {
    File(PathBuf),
    Text(String),
}
