//! API for interacting with plugins.

pub(crate) mod bindings;
pub mod event;
mod host;
mod init;
mod wrappers;

pub use wrappers::{Action, ListItem, Plugin};
