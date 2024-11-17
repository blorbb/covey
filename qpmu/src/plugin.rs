//! API for interacting with plugins.

mod action;
mod plugin;

pub use action::Action;
pub use plugin::Plugin;

use crate::ListItem;

pub(crate) mod proto {
    tonic::include_proto!("plugin");
}

/// Event returned by a plugin.
#[derive(Debug)]
pub enum PluginEvent {
    /// Set the displayed list.
    SetList { list: Vec<ListItem>, index: u64 },
    /// Run a sequence of actions.
    Run(Vec<Action>),
}
