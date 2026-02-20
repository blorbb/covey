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
};

use anyhow::Result;
use covey_schema::{
    config::{GlobalConfig, PluginEntry},
    hotkey::Hotkey,
    id::{CommandId, PluginId},
    keyed_list::KeyedList,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    CONFIG_DIR, CONFIG_PATH, ListItem, PLUGINS_DIR, Plugin,
    event::{Action, ListItemId},
};

pub fn channel() -> Result<(Host, ActionReceiver)> {
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
    find_and_insert_plugins_from_fs(&mut global_config);

    let (response_sender, response_receiver) = mpsc::unbounded_channel();
    let (action_sender, action_receiver) = mpsc::unbounded_channel();

    let plugins = load_plugins_from_config(&global_config, &response_sender);
    info!("found plugins: {plugins:?}");

    Ok((
        Host {
            config: global_config,
            other_actions: action_sender,
            sender: RequestSender {
                plugins,
                sender: response_sender,
                // must be greater than the initial `latest_received_query_request_id`
                next_request_id: 1,
            },
        },
        ActionReceiver {
            receiver: response_receiver,
            other_actions: action_receiver,
            latest_received_query_request_id: 0,
        },
    ))
}

pub struct Host {
    config: GlobalConfig,
    other_actions: mpsc::UnboundedSender<Action>,
    sender: RequestSender,
}

impl Host {
    pub fn send_query(&mut self, query: String) -> impl Future<Output = ()> + use<> + Send + Sync {
        self.sender.send_query(query)
    }

    pub fn activate(
        &mut self,
        item: ListItemId,
        command_id: CommandId,
    ) -> impl Future<Output = ()> + use<> + Send + Sync {
        self.sender.activate(item, command_id)
    }

    pub fn activate_by_hotkey(
        &mut self,
        item: ListItem,
        hotkey: Hotkey,
    ) -> Option<impl Future<Output = ()> + use<>> {
        self.sender.activate_by_hotkey(item, hotkey)
    }

    pub fn config(&self) -> &GlobalConfig {
        &self.config
    }

    /// Reloads all plugins with the new configuration.
    #[tracing::instrument(skip_all)]
    pub fn reload(&mut self, config: GlobalConfig) {
        debug!("reloading");
        self.config = config;
        self.sender.plugins = load_plugins_from_config(&self.config, &self.sender.sender);
        // TODO: spawn this in another task and handle errors properly
        Self::write_config(&self.config).expect("TODO");
    }

    /// Reloads a specific existing plugin, re-reading its manifest.
    pub fn reload_plugin(&mut self, plugin_id: &PluginId) {
        debug!("reloading plugin {plugin_id:?}");

        let replace_result = self.sender.plugins.replace(plugin_id, |plugin| {
            Plugin::new_read_manifest(plugin.config_entry().clone(), self.sender.sender.clone())
        });

        match replace_result {
            covey_schema::keyed_list::ReplaceResult::IdNotFound => {
                self.send_error(
                    "Failed to reload plugin",
                    format!("could not find config of plugin {plugin_id}",),
                );
            }
            covey_schema::keyed_list::ReplaceResult::ReplaceError(e) => {
                self.send_error("Failed to reload plugin", format!("{e:#}"));
            }
            covey_schema::keyed_list::ReplaceResult::DifferentId => {
                panic!("reloaded plugin should have same plugin id");
            }
            covey_schema::keyed_list::ReplaceResult::Replaced => {}
        }
    }

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

    fn send_error(&self, title: impl Into<String>, description: impl Into<String>) {
        _ = self
            .other_actions
            .send(Action::DisplayError(title.into(), description.into()))
    }
}

struct RequestSender {
    plugins: KeyedList<Plugin>,
    sender: mpsc::UnboundedSender<(Plugin, covey_proto::Response)>,
    next_request_id: u64,
}

pub struct ActionReceiver {
    receiver: mpsc::UnboundedReceiver<(Plugin, covey_proto::Response)>,
    other_actions: mpsc::UnboundedReceiver<Action>,
    latest_received_query_request_id: u64,
}

impl RequestSender {
    /// Calls a plugin with this query.
    ///
    /// Responses should be handled by calling [`ResponseReceiver::recv_action`].
    #[tracing::instrument(skip(self))]
    pub fn send_query(&mut self, query: String) -> impl Future<Output = ()> + use<> + Send + Sync {
        debug!("setting input to {query:?}");

        let request_id = covey_proto::RequestId(self.next_request_id);
        self.next_request_id += 1;
        let plugins = self.plugins.clone();

        async move {
            for plugin in plugins
                .iter()
                .filter(|plugin| !plugin.config_entry().disabled)
            {
                let Some(stripped) = plugin
                    .prefix()
                    .and_then(|prefix| query.strip_prefix(prefix))
                else {
                    continue;
                };

                debug!("querying plugin {}", plugin.id());
                plugin.query(request_id, stripped.to_string()).await;
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
        &mut self,
        item: ListItemId,
        command_id: CommandId,
    ) -> impl Future<Output = ()> + use<> + Send + Sync {
        debug!("activating {item:?}");

        let request_id = covey_proto::RequestId(self.next_request_id);
        self.next_request_id += 1;
        let plugins = self.plugins.clone();

        async move {
            let Some(plugin) = plugins.get(item.plugin.id()) else {
                return;
            };
            plugin.activate(request_id, item.local_id, command_id).await
        }
    }

    /// Activates a list item using the specified hotkey.
    ///
    /// Figures out the command to run based on the hotkey and plugin configuration.
    /// Returns [`Some`] if the hotkey activated some command, otherwise [`None`].
    #[tracing::instrument(skip(self))]
    pub fn activate_by_hotkey(
        &mut self,
        item: ListItem,
        hotkey: Hotkey,
    ) -> Option<impl Future<Output = ()> + use<>> {
        let command_id = item.activated_command_from_hotkey(&hotkey)?;
        Some(self.activate(item.id(), command_id.clone()))
    }
}

impl ActionReceiver {
    /// Receives an action by a plugin.
    pub async fn recv_action(&mut self, host: &Host) -> Action {
        loop {
            let Some((plugin, response)) = self.receiver.recv().await else {
                continue;
            };
            let Some(action) = self.response_to_action(host, &plugin, response) else {
                continue;
            };
            return action;
        }
    }

    pub fn try_recv_action(&mut self, host: &Host) -> Option<Action> {
        let plugin_action = self
            .receiver
            .try_recv()
            .ok()
            .and_then(|(plugin, response)| self.response_to_action(host, &plugin, response));

        match plugin_action {
            Some(action) => Some(action),
            None => self.other_actions.try_recv().ok(),
        }
    }

    fn response_to_action(
        &mut self,
        host: &Host,
        plugin: &Plugin,
        response: covey_proto::Response,
    ) -> Option<Action> {
        match response.response {
            covey_proto::ResponseBody::SetList(list) => {
                // Check if the latest received id < new id. If so, send the action.
                // Otherwise, this response is outdated and we should not update the list.
                let new = response.request_id.0;
                if self.latest_received_query_request_id < new {
                    self.latest_received_query_request_id = new;
                    Some(Action::SetList(crate::List::from_proto(
                        plugin,
                        &host.config.app.icon_themes,
                        list,
                    )))
                } else {
                    None
                }
            }
            covey_proto::ResponseBody::PerformAction(action) => Some(match action {
                covey_proto::PluginAction::Close => Action::Close,
                covey_proto::PluginAction::Copy(str) => Action::Copy(str),
                covey_proto::PluginAction::SetInput(input) => {
                    Action::SetInput(crate::Input::from_proto(plugin, input))
                }
                covey_proto::PluginAction::DisplayError(err) => {
                    Action::DisplayError(format!("Plugin {} failed", plugin.id()), err)
                }
            }),
        }
    }
}

fn load_plugins_from_config(
    config: &GlobalConfig,
    response_sender: &mpsc::UnboundedSender<(Plugin, covey_proto::Response)>,
) -> KeyedList<Plugin> {
    KeyedList::new_lossy(config.plugins.iter().filter_map(|plugin_entry| {
        match Plugin::new_read_manifest(plugin_entry.clone(), response_sender.clone()) {
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

/// Finds extra plugins from the plugin directory and inserts it into the config.
fn find_and_insert_plugins_from_fs(config: &mut GlobalConfig) {
    let Ok(dirs) = fs::read_dir(&*PLUGINS_DIR) else {
        warn!("failed to read plugins dir");
        return;
    };

    // each directory in the plugins directory should be the plugin's id
    let plugin_ids = dirs
        .filter_map(Result::ok)
        .flat_map(|plugin_dir| plugin_dir.file_name().into_string())
        .inspect(|plugin_id| debug!("discovered plugin {plugin_id:?} from fs"))
        .map(|plugin_id| PluginId::new(&plugin_id));
    config.plugins.extend_lossy(plugin_ids.map(|plugin_id| {
        let mut entry = PluginEntry::new(plugin_id);
        entry.disabled = true; // require explicitly enabling a new plugin
        entry
    }));
}
