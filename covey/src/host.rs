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
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::Result;
use covey_proto::{covey_request::RequestId, plugin_response::Response};
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
    event::{Action, InternalAction, ListItemId},
};

pub struct RequestSender {
    config: GlobalConfig,
    plugins: KeyedList<Plugin>,
    response_sender: mpsc::UnboundedSender<Action>,
    next_request_id: u64,
}

pub struct ResponseReceiver {
    response_receiver: mpsc::UnboundedReceiver<Action>,
}

impl RequestSender {
    /// Calls a plugin with this query.
    ///
    /// Responses should be handled by calling [`ResponseReceiver::recv_action`].
    #[tracing::instrument(skip(self))]
    pub fn send_query(&mut self, query: String) -> impl Future<Output = ()> + use<> {
        debug!("setting input to {query:?}");

        let request_id = RequestId(self.next_request_id);
        self.next_request_id += 1;
        // let icon_themes = Arc::clone(&self.config.app.icon_themes);
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
                plugin.query(request_id, stripped.to_string()).await;
                // match plugin.query(stripped).await {
                //     Ok(proto_list) => response_sender.send(InternalAction::SetList {
                //         list: List::from_proto(&plugin, &icon_themes, proto_list),
                //         index: request_id,
                //     }),
                //     Err(e) => response_sender.send(InternalAction::DisplayError(format!("{e:#}"))),
                // }
                // .expect("receiver disconnected while sending query response");
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
    ) -> impl Future<Output = ()> + use<> {
        debug!("activating {item:?}");

        let request_id = RequestId(self.next_request_id);
        self.next_request_id += 1;

        async move {
            item.plugin
                .activate(request_id, item.local_id, command_id)
                .await
            // loop {
            //     match stream.message().await {
            //         Ok(Some(value)) => {
            //             let Some(action) = value.action.action else {
            //                 tracing::warn!("plugin {:?} did not provide an action", item.plugin);
            //                 continue;
            //             };

            //             tracing::debug!("received action {action:?}");
            //             response_sender
            //                 .send(InternalAction::from_proto_action(&item.plugin, action))
            //                 .expect("receiver should be alive");
            //         }
            //         Ok(None) => break,
            //         Err(e) => {
            //             response_sender
            //                 .send(InternalAction::DisplayError(e.to_string()))
            //                 .expect("receiver should be alive");
            //         }
            //     };
            // }
        }
    }

    /// Activates a list item using the specified hotkey.
    ///
    /// Figures out the command to run based on the hotkey and plugin configuration.
    /// Returns [`Some`] if the hotkey activated some command, otherwise [`None`].
    #[tracing::instrument(skip(self))]
    pub fn activate_by_hotkey(
        &self,
        item: ListItem,
        hotkey: Hotkey,
    ) -> Option<impl Future<Output = ()> + use<>> {
        let cmd_name = item.activated_command_from_hotkey(&hotkey)?;
        Some(self.activate(item.id(), cmd_name.as_str().to_string()))
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

            if let Some(action) = self.convert_internal_action(internal_action) {
                return action;
            }
        }
    }

    pub fn try_recv_action(&mut self) -> Option<Action> {
        loop {
            let internal_action = self.response_receiver.try_recv().ok()?;

            if let Some(action) = self.convert_internal_action(internal_action) {
                return Some(action);
            }
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

struct ResponseMerger {
    latest_received_query_request_id: AtomicU64,
    action_sender: mpsc::UnboundedSender<Action>,
}

impl ResponseMerger {
    fn send(&self, response: Response, plugin_entry: &PluginEntry) {
        match response.response {
            covey_proto::plugin_response::Body::SetList(list) => {
                // Check if the latest received id < new id. If so, send the action.
                // Otherwise, this response is outdated and we should not update the list.
                let new = response.request_id.0;
                if self
                    .latest_received_query_request_id
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
                        (old < new).then_some(new)
                    })
                    .is_ok()
                {
                    let action =
                        Action::SetList(crate::List::from_proto(plugin, icon_themes, list));
                }
            }
            covey_proto::plugin_response::Body::PerformAction(action) => todo!(),
        }
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
    find_and_insert_plugins_from_fs(&mut global_config);

    let (response_sender, response_receiver) = mpsc::unbounded_channel();

    let plugins = KeyedList::new_lossy(config.plugins.iter().filter_map(|plugin_entry| {
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
    }));

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
            next_request_id: 1,
            latest_received_query_request_id: 0,
        },
        ResponseReceiver { response_receiver },
    )
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

/// Reads the manifests of every plugin listed in the config.
fn load_plugins(
    config: &GlobalConfig,
    responses: mpsc::UnboundedSender<Response>,
) -> KeyedList<Plugin> {
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

    pub fn reload_plugin(&mut self, plugin_id: &PluginId) {
        debug!("reloading plugin {plugin_id:?}");

        let Some(plugin_config) = self.config.plugins.get(plugin_id) else {
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
