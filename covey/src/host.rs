//! A two-way channel to talk between plugins and the frontend.
//!
//! The frontend should send queries and item activations to talk to
//! the plugins.
//!
//! Plugins will send back action responses, which the frontend should
//! consume with [`Self::recv_action`].

use std::{
    fs,
    future::Future,
    io::{Read as _, Write as _},
    mem,
    sync::Arc,
};

use color_eyre::eyre::Result;
use covey_schema::{
    config::{GlobalConfig, PluginEntry},
    keyed_list::{Id, KeyedList},
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    CONFIG_DIR, CONFIG_PATH, List, PLUGINS_DIR, Plugin,
    event::{Action, InternalAction, ListItemId},
};

pub struct RequestSender {
    config: GlobalConfig,
    plugins: KeyedList<Plugin>,
    response_sender: mpsc::UnboundedSender<InternalAction>,
    /// Every query has an incrementing dispatch number that plugins respond with,
    /// to identify which query a response was from.
    next_dispatch_index: u64,
}

pub struct ResponseReceiver {
    response_receiver: mpsc::UnboundedReceiver<InternalAction>,
    /// The last dispatch number received from any plugin response.
    ///
    /// If we receive a response with a dispatch number smaller than this,
    /// it should be ignored as it is outdated.
    latest_received_dispatch: u64,
}

impl RequestSender {
    /// Calls a plugin with this query.
    ///
    /// Responses should be handled by calling [`ResponseReceiver::recv_action`].
    #[tracing::instrument(skip(self))]
    pub fn send_query(&mut self, query: String) -> impl Future<Output = ()> + use<> {
        debug!("setting input to {query:?}");

        let response_sender = self.response_sender.clone();
        let dispatch_index = self.next_dispatch_index;
        self.next_dispatch_index += 1;
        let icon_themes = Arc::clone(&self.config.app.icon_themes);
        let plugins = self.plugins.clone();

        async move {
            for plugin in plugins
                .into_iter()
                .filter(|plugin| !plugin.config_entry().disabled)
            {
                let Some(stripped) = plugin
                    .prefix()
                    .and_then(|prefix| query.strip_prefix(prefix))
                else {
                    continue;
                };

                debug!("querying plugin {plugin:?}");

                match plugin.query(stripped).await {
                    Ok(proto_list) => response_sender.send(InternalAction::SetList {
                        list: List::from_proto(&plugin, &icon_themes, proto_list),
                        index: dispatch_index,
                    }),
                    Err(e) => response_sender.send(InternalAction::DisplayError(format!("{e:#}"))),
                }
                .expect("receiver disconnected while sending query response");
                return;
            }

            tracing::warn!("no plugin activated with query {query}");
        };
    }

    /// Activates a list item with a specified command.
    ///
    /// Responses should be handled by calling [`ResponseReceiver::recv_action`].
    #[tracing::instrument(skip(self))]
    pub fn activate(
        &self,
        item: ListItemId,
        command_name: String,
    ) -> impl Future<Output = ()> + use<> {
        debug!("activating {item:?}");

        let response_sender = self.response_sender.clone();
        async move {
            let mut stream = match item.plugin.activate(item.local_id, command_name).await {
                Ok(stream) => stream,
                Err(e) => {
                    response_sender
                        .send(InternalAction::DisplayError(format!("{e:#}")))
                        .expect("receiver should be alive");
                    return;
                }
            };

            loop {
                match stream.message().await {
                    Ok(Some(value)) => {
                        let Some(action) = value.action.action else {
                            tracing::warn!("plugin {:?} did not provide an action", item.plugin);
                            continue;
                        };

                        tracing::debug!("received action {action:?}");
                        response_sender
                            .send(InternalAction::from_proto_action(&item.plugin, action))
                            .expect("receiver should be alive");
                    }
                    Ok(None) => break,
                    Err(e) => {
                        response_sender
                            .send(InternalAction::DisplayError(e.to_string()))
                            .expect("receiver should be alive");
                    }
                };
            }
        }
    }
}

impl ResponseReceiver {
    /// Receives an action by a plugin.
    pub async fn recv_action(&mut self) -> Action {
        loop {
            let internal_action = self
                .response_receiver
                .recv()
                .await
                .expect("senders should still exist");

            match internal_action {
                InternalAction::SetList { list, index } => {
                    if index <= self.latest_received_dispatch {
                        continue;
                    }
                    self.latest_received_dispatch = index;
                    return Action::SetList(list);
                }
                InternalAction::Close => return Action::Close,
                InternalAction::Copy(str) => return Action::Copy(str),
                InternalAction::SetInput(input) => return Action::SetInput(input),
                InternalAction::DisplayError(err) => {
                    return Action::DisplayError("Plugin failed".to_owned(), err);
                }
            };
        }
    }
}

// extra getters
impl RequestSender {
    pub fn config(&self) -> &GlobalConfig {
        &self.config
    }

    /// Ordered set of all enabled plugins.
    #[tracing::instrument(skip(self))]
    pub fn plugins(&self) -> &KeyedList<Plugin> {
        &self.plugins
    }
}

pub fn channel() -> Result<(RequestSender, ResponseReceiver)> {
    info!("reading config from file: {:?}", &*CONFIG_PATH);

    fs::create_dir_all(&*CONFIG_DIR)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(false)
        .open(&*CONFIG_PATH)?;

    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let mut global_config: GlobalConfig = toml::from_str(&s)?;
    find_plugins_from_fs(&mut global_config);
    let plugins = load_plugins(&global_config);

    info!("found plugins: {plugins:?}");

    Ok(channel_with_plugins(global_config, plugins))
}

fn channel_with_plugins(
    config: GlobalConfig,
    plugins: KeyedList<Plugin>,
) -> (RequestSender, ResponseReceiver) {
    let (response_sender, response_receiver) = mpsc::unbounded_channel();
    (
        RequestSender {
            config,
            plugins,
            response_sender,
            next_dispatch_index: 1,
        },
        ResponseReceiver {
            response_receiver,
            latest_received_dispatch: 0,
        },
    )
}

/// Finds extra plugins from the plugin directory and inserts it into the config.
fn find_plugins_from_fs(config: &mut GlobalConfig) {
    let Ok(dirs) = fs::read_dir(&*PLUGINS_DIR) else {
        warn!("failed to read plugins dir");
        return;
    };

    // each directory in the plugins directory should be the plugin's id
    let plugin_ids = dirs
        .filter_map(Result::ok)
        .flat_map(|plugin_dir| plugin_dir.file_name().into_string())
        .inspect(|plugin_id| debug!("discovered plugin {plugin_id:?} from fs"));
    config.plugins.extend_lossy(plugin_ids.map(|plugin_id| {
        let mut entry = PluginEntry::new(Id::new(&plugin_id));
        entry.disabled = true; // require explicitly enabling a new plugin
        entry
    }));
}

/// Reads the manifests of every plugin listed in the config.
fn load_plugins(config: &GlobalConfig) -> KeyedList<Plugin> {
    KeyedList::new_lossy(config.plugins.iter().filter_map(|plugin_entry| {
        match Plugin::new(plugin_entry.clone()) {
            Ok(plugin) => {
                debug!("found plugin {plugin:?}");
                Some(plugin)
            }
            Err(e) => {
                error!("error loading plugin: {e}");
                None
            }
        }
    }))
}

// extra file-io methods
impl RequestSender {
    /// Reloads all plugins with the new configuration.
    #[tracing::instrument(skip_all)]
    pub fn reload(&mut self, config: GlobalConfig) {
        debug!("reloading");
        self.plugins = load_plugins(&config);
        // TODO: spawn this in another task and handle errors properly
        Self::write_config(&config).expect("TODO");
        self.config = config;
    }

    pub fn reload_plugin(&mut self, plugin_id: &Id) {
        debug!("reloading plugin {plugin_id:?}");

        let Some(plugin_config) = self.config.plugins.get(plugin_id.as_str()) else {
            self.response_sender
                .send(InternalAction::DisplayError(format!(
                    "Failed to reload plugin\ncould not find config of plugin {plugin_id:?}"
                )))
                .expect("receiver must remain open");
            return;
        };

        let new_plugins = mem::take(&mut self.plugins)
            .into_iter()
            .filter_map(|plugin| {
                if plugin.id() == plugin_id {
                    match Plugin::new(plugin_config.clone()) {
                        Ok(plugin) => Some(plugin),
                        Err(e) => {
                            self.response_sender
                                .send(InternalAction::DisplayError(format!(
                                    "Failed to reload plugin\n{e:#}"
                                )))
                                .expect("receiver must remain open");
                            None
                        }
                    }
                } else {
                    Some(plugin)
                }
            });

        self.plugins = KeyedList::new(new_plugins).expect("new keyed list should have same keys");
    }

    /// Writes the config to the [`CONFIG_PATH`].
    ///
    /// # Errors
    /// Returns an error if there was an IO or serialization issue.
    fn write_config(config: &GlobalConfig) -> Result<()> {
        // stringify here to avoid truncating the file then erroring
        let toml_str = toml::to_string_pretty(config)?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&*CONFIG_PATH)?;

        file.write_all(toml_str.as_bytes())?;

        Ok(())
    }
}
