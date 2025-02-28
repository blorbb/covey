//! Contains types and helpers for all configuration files in covey.
//!
//! All types implement serde's [`Serialize`] and [`Deserialize`] traits.
//! The feature `"ts-rs"` enables deriving [`ts_rs::TS`] as well.
//!
//! "manifest" is the term used for the `manifest.toml` required for each plugin.
//! The manifests give details about the plugin, define the possible commands,
//! and provide a schema that can be used to configure the plugin.
//!
//! "config" is the term used for the global `config.toml` of the covey app.
//! This is the user-defined configuration for the app that follows the plugin
//! manifest's schema.
//!
//! [`Serialize`]: serde::Serialize
//! [`Deserialize`]: serde::Deserialize

pub mod config;
#[doc(hidden)]
pub mod generate;
pub mod hotkey;
pub mod keyed_list;
pub mod manifest;
