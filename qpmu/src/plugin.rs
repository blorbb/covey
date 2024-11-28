//! API for interacting with plugins.

mod action;
pub mod manifest;
#[expect(clippy::module_inception, reason = "private")]
mod plugin;

use core::fmt;

pub use action::Action;
pub use plugin::Plugin;

use crate::ResultList;

#[allow(clippy::pedantic)]
pub(crate) mod proto {
    tonic::include_proto!("plugin");
}

/// Event returned by a plugin.
pub enum PluginEvent {
    /// Set the displayed list.
    SetList { list: ResultList, index: u64 },
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
