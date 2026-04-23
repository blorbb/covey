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
#[derive(Debug)]
#[non_exhaustive]
pub struct List {
    pub sections: Vec<ListSection>,
    pub(crate) activation_target: ActivationTarget,
    pub(crate) request_id: covey_proto::RequestId,
}

impl List {
    pub fn plugin(&self) -> &Plugin {
        &self.activation_target.plugin
    }

    pub fn get_item(&self, idx: usize) -> Option<&ListItem> {
        let mut items_passed = 0;
        for section in &self.sections {
            if items_passed + section.items.len() <= idx {
                items_passed += section.items.len();
            } else {
                return section.items.get(idx - items_passed);
            }
        }

        None
    }

    pub fn total_len(&self) -> usize {
        self.sections
            .iter()
            .map(|section| section.items.len())
            .sum()
    }

    pub fn is_response_of_latest_query(&self, host: &Host) -> bool {
        host.query_request_id_is_latest(self.request_id)
    }

    pub fn activation_target(&self) -> &ActivationTarget {
        &self.activation_target
    }
}

#[non_exhaustive]
pub struct ListSection {
    pub title: String,
    pub items: Vec<ListItem>,
}

impl fmt::Debug for ListSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListSection")
            .field("title", &self.title)
            .field("items", &&self.items[..3.min(self.items.len())])
            .finish()
    }
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
