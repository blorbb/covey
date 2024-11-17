//! API for interacting with plugins.

pub(crate) mod bindings;
pub mod event;
mod wrappers;

pub use wrappers::{Action, ListItem, Plugin};
