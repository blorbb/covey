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
    id::{CommandId, PluginId, StringId},
    keyed_list::{Identify as _, KeyedList},
};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    CONFIG_DIR, CONFIG_PATH, ListItem, PLUGINS_DIR, Plugin,
    event::{Action, ListItemId},
    plugin::PluginProcess,
};

pub struct RequestSender {
    config: GlobalConfig,
    plugin_processes: Arc<KeyedList<Arc<PluginProcess>>>,
    merger: Arc<ResponseMerger>,
    next_request_id: u64,
}

pub struct ResponseReceiver {
    action_receiver: mpsc::UnboundedReceiver<Action>,
}

impl RequestSender {
    /// Calls a plugin with this query.
    ///
    /// Responses should be handled by calling [`ResponseReceiver::recv_action`].
    #[tracing::instrument(skip(self))]
    pub fn send_query(&mut self, query: String) -> impl Future<Output = ()> + use<> + Send + Sync {
        debug!("setting input to {query:?}");

        let request_id = RequestId(self.next_request_id);
        self.next_request_id += 1;
        // let icon_themes = Arc::clone(&self.config.app.icon_themes);
        let plugins = Arc::clone(&self.plugin_processes);

        async move {
            for plugin in plugins
                .iter()
                .filter(|plugin| !plugin.metadata().config_entry().disabled)
            {
                let Some(stripped) = plugin
                    .metadata()
                    .prefix()
                    .and_then(|prefix| query.strip_prefix(prefix))
                else {
                    continue;
                };

                debug!("querying plugin {}", plugin.id().as_str());
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
        let plugins = Arc::clone(&self.plugin_processes);

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

impl ResponseReceiver {
    /// Receives an action by a plugin.
    pub async fn recv_action(&mut self) -> Action {
        self.action_receiver.recv().await.unwrap()
    }

    pub fn try_recv_action(&mut self) -> Option<Action> {
        match self.action_receiver.try_recv() {
            Ok(action) => Some(action),
            Err(mpsc::error::TryRecvError::Empty) => None,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                panic!("action sender should never be dropped")
            }
        }
    }
}

// extra getters
impl RequestSender {
    pub fn config(&self) -> &GlobalConfig {
        &self.config
    }
}

struct ResponseMerger {
    latest_received_query_request_id: AtomicU64,
    action_sender: mpsc::UnboundedSender<Action>,
    // Extra config stuff
    icon_themes: RwLock<Arc<[String]>>,
}

impl ResponseMerger {
    fn send_error(&self, title: impl Into<String>, description: impl Into<String>) {
        _ = self
            .action_sender
            .send(Action::DisplayError(title.into(), description.into()))
    }

    fn send(&self, plugin: &Plugin, response: Response) {
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
                    let action = Action::SetList(crate::List::from_proto(
                        plugin,
                        &self.icon_themes.read(),
                        list,
                    ));
                    _ = self.action_sender.send(action);
                }
            }
            covey_proto::plugin_response::Body::PerformAction(action) => {
                let action = match action {
                    covey_proto::plugin_response::Action::Close => Action::Close,
                    covey_proto::plugin_response::Action::Copy(str) => Action::Copy(str),
                    covey_proto::plugin_response::Action::SetInput(input) => {
                        Action::SetInput(crate::Input::from_proto(plugin, input))
                    }
                    covey_proto::plugin_response::Action::DisplayError(err) => {
                        Action::DisplayError(format!("Plugin {} failed", plugin.id().as_str()), err)
                    }
                };
                _ = self.action_sender.send(action)
            }
        }
    }

    fn plugin_to_process(self: Arc<Self>, plugin: Plugin) -> Arc<PluginProcess> {
        Arc::new(PluginProcess::new(
            plugin.clone(),
            Arc::new(move |response| self.send(&plugin, response)),
        ))
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

    let (action_sender, action_receiver) = mpsc::unbounded_channel();

    let merger = Arc::new(ResponseMerger {
        latest_received_query_request_id: AtomicU64::new(0),
        action_sender,
        icon_themes: RwLock::new(Arc::clone(&global_config.app.icon_themes)),
    });

    let plugins = load_plugins_from_config(&global_config);
    info!("found plugins: {plugins:?}");
    let plugin_processes = map_plugins_to_processes(plugins, Arc::clone(&merger));

    Ok((
        RequestSender {
            config: global_config,
            plugin_processes: Arc::new(plugin_processes),
            merger,
            // must be greater than the initial `latest_received_query_request_id`
            next_request_id: 1,
        },
        ResponseReceiver { action_receiver },
    ))
}

fn load_plugins_from_config(config: &GlobalConfig) -> KeyedList<Plugin> {
    KeyedList::new_lossy(config.plugins.iter().filter_map(|plugin_entry| {
        match Plugin::new_read_manifest(plugin_entry.clone()) {
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

fn map_plugins_to_processes(
    plugins: KeyedList<Plugin>,
    merger: Arc<ResponseMerger>,
) -> KeyedList<Arc<PluginProcess>> {
    plugins
        .map_same_id(|plugin| Arc::clone(&merger).plugin_to_process(plugin))
        .expect("plugin process has same id as plugin")
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

// extra file-io methods
impl RequestSender {
    /// Reloads all plugins with the new configuration.
    #[tracing::instrument(skip_all)]
    pub fn reload(&mut self, config: GlobalConfig) {
        debug!("reloading");
        self.config = config;
        *self.merger.icon_themes.write() = Arc::clone(&self.config.app.icon_themes);
        self.plugin_processes = Arc::new(map_plugins_to_processes(
            load_plugins_from_config(&self.config),
            Arc::clone(&self.merger),
        ));
        // TODO: spawn this in another task and handle errors properly
        Self::write_config(&self.config).expect("TODO");
    }

    pub fn reload_plugin(&mut self, plugin_id: &PluginId) {
        debug!("reloading plugin {plugin_id:?}");

        let Some(plugin_config) = self.config.plugins.get(plugin_id) else {
            self.merger.send_error(
                "Failed to reload plugin",
                format!("could not find config of plugin {}", plugin_id.as_str()),
            );
            return;
        };

        let new_plugins = self.plugin_processes.iter().filter_map(|plugin| {
            if plugin.id() == plugin_id {
                match Plugin::new_read_manifest(plugin_config.clone()) {
                    Ok(plugin) => Some(Arc::clone(&self.merger).plugin_to_process(plugin)),
                    Err(e) => {
                        self.merger
                            .send_error("Failed to reload plugin", format!("{e:#}"));
                        None
                    }
                }
            } else {
                Some(Arc::clone(plugin))
            }
        });

        self.plugin_processes =
            Arc::new(KeyedList::new(new_plugins).expect("new keyed list should have same keys"));
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
