pub mod rank;
pub mod sql;

mod list;
pub use list::{Icon, List, ListItem, ListStyle};
mod action;
pub use action::Action;
mod input;
pub use input::{Input, SelectionRange};
mod plugin;
pub use plugin::Plugin;
mod context;
pub use context::ActivationContext;
mod hotkey;
pub use hotkey::{Hotkey, Key, Modifiers};
mod server;
pub use server::run_server as main;

mod proto {
    tonic::include_proto!("plugin");
}

pub use anyhow::{self, Result};
