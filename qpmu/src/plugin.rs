//! API for interacting with plugins.

pub mod event;
mod wrappers;

pub use wrappers::{Action, ListItem, Plugin};

pub(crate) mod proto {
    tonic::include_proto!("plugin");
}
