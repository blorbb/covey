mod event;
mod from_proto;
mod host;
mod plugin;

use std::{path::PathBuf, sync::LazyLock};

pub use covey_schema;
pub use event::{Action, ActivationTarget, Input, List, ListItem, ListStyle, ResolvedIcon};
pub use host::{ActionReceiver, Host, channel};
pub use plugin::{Plugin, PluginWeak};

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("config dir must exist")
        .join("covey")
});
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("config.toml"));
pub static DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().expect("data dir must exist").join("covey"));
pub static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("plugins"));
