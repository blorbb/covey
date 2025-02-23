mod event;
mod host;
mod plugin;
mod proto;
mod spawn;

use std::{path::PathBuf, sync::LazyLock};

use covey_config::config::GlobalConfig;
pub use event::{Input, List, ListItem, ListItemId, ListStyle, ResolvedIcon};
pub use host::Host;
pub use plugin::Plugin;

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("config dir must exist")
        .join("covey")
});
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("config.toml"));
pub static DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().expect("data dir must exist").join("covey"));

/// A controller for the UI.
///
/// These methods may not be called on the main thread. Many UI
/// frameworks require updates to be called on the main thread,
/// so you will likely need to use channels to communicate these
/// messages.
pub trait Frontend: Send + 'static {
    /// Close the window.
    fn close(&mut self);

    /// Copy a string to the clipboard.
    fn copy(&mut self, str: String);

    /// Set the UI input to the provided input.
    fn set_input(&mut self, input: Input);

    /// Set the UI results list to the provided list.
    fn set_list(&mut self, list: List);

    /// Reset the frontend with a new configuration.
    fn reload(&mut self, config: GlobalConfig);

    // TODO: refactor this lib to have a custom error type
    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report);
}
