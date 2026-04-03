//! A two-way channel to talk between plugins and the frontend.
//!
//! The frontend should send queries and item activations to talk to
//! the plugins.
//!
//! Plugins will send back action responses, which the frontend should
//! consume with [`Self::recv_action`].

use std::{
    collections::HashMap,
    fs,
    io::{Read as _, Write as _},
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread,
    time::{Duration, Instant},
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
    plugin::PluginWeak,
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
            response_sender,
            plugins,
            // must be greater than the initial `latest_received_query_request_id`
            next_request_id: 1,
            latest_sent_query_request_id: covey_proto::RequestId(0),
            // TODO: make this configurable
            plugin_process_gc: PluginProcessGc::new(Duration::from_hours(24)),
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
    response_sender: mpsc::UnboundedSender<(Plugin, covey_proto::Response)>,
    plugins: KeyedList<Plugin>,
    next_request_id: u64,
    latest_sent_query_request_id: covey_proto::RequestId,
    plugin_process_gc: PluginProcessGc,
}

impl Host {
    /// Calls a plugin with this query.
    ///
    /// Responses should be handled by calling [`ActionReceiver::recv`].
    #[tracing::instrument(skip(self))]
    pub fn send_query(&mut self, query: String) {
        debug!("setting input to {query:?}");

        let request_id = covey_proto::RequestId(self.next_request_id);
        self.latest_sent_query_request_id = request_id;
        self.next_request_id += 1;

        let plugin_with_prefix = self
            .plugins
            .iter()
            .filter(|plugin| !plugin.config_entry().disabled)
            .find_map(|plugin| Some((plugin, query.strip_prefix(plugin.prefix()?)?)));

        match plugin_with_prefix {
            Some((plugin, stripped_query)) => {
                tracing::debug!("querying plugin {plugin:?}");
                self.plugin_process_gc.touch(plugin);
                plugin.query(request_id, stripped_query.to_owned());
            }
            None => {
                tracing::warn!("no plugin activated with query {query}");
                return;
            }
        }
    }

    /// Activates a list item with a specified command.
    ///
    /// Responses should be handled by calling [`ActionReceiver::recv`].
    #[tracing::instrument(skip(self))]
    pub fn activate(&mut self, item: ListItemId, command_id: CommandId) {
        debug!("activating {item:?}");

        let request_id = covey_proto::RequestId(self.next_request_id);
        self.next_request_id += 1;

        let Some(plugin) = self.plugins.get(item.plugin.id()) else {
            return;
        };
        self.plugin_process_gc.touch(plugin);
        plugin.activate(request_id, item.local_id, command_id)
    }

    /// Activates a list item using the specified hotkey.
    ///
    /// Figures out the command to run based on the hotkey and plugin
    /// configuration. Returns [`Some`] if the hotkey activated some command,
    /// otherwise [`None`].
    #[tracing::instrument(skip(self))]
    pub fn activate_by_hotkey(&mut self, item: ListItem, hotkey: Hotkey) -> Option<CommandId> {
        let command = item.activated_command_from_hotkey(hotkey)?;
        self.activate(item.id(), command.id.clone());
        Some(command.id.clone())
    }

    pub fn config(&self) -> &GlobalConfig {
        &self.config
    }

    /// Reloads all plugins with the new configuration.
    ///
    /// Should re-send a query immediately after reloading.
    #[tracing::instrument(skip_all)]
    pub fn reload(&mut self, config: GlobalConfig) {
        debug!("reloading");
        self.config = config;
        self.plugins = load_plugins_from_config(&self.config, &self.response_sender);
        // TODO: spawn this in another task and handle errors properly
        Self::write_config(&self.config).expect("TODO");
    }

    /// Reloads a specific existing plugin, re-reading its manifest.
    ///
    /// Should re-send a query immediately after reloading.
    pub fn reload_plugin(&mut self, plugin_id: &PluginId) {
        debug!("reloading plugin {plugin_id}");

        let replace_result = self.plugins.replace(plugin_id, |plugin| {
            Plugin::new_read_manifest(plugin.config_entry().clone(), self.response_sender.clone())
        });

        match replace_result {
            covey_schema::keyed_list::ReplaceResult::IdNotFound => {
                self.send_error(
                    "Failed to reload plugin",
                    format!("could not find config of plugin {plugin_id}"),
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

    pub(crate) fn query_request_id_is_latest(&self, id: covey_proto::RequestId) -> bool {
        debug_assert!(
            self.latest_sent_query_request_id.0 >= id.0,
            "found {id:?} when latest should be {:?}",
            self.latest_sent_query_request_id
        );

        self.latest_sent_query_request_id.0 == id.0
    }
}

pub struct ActionReceiver {
    receiver: mpsc::UnboundedReceiver<(Plugin, covey_proto::Response)>,
    other_actions: mpsc::UnboundedReceiver<Action>,
    latest_received_query_request_id: u64,
}

impl ActionReceiver {
    /// Receives an action by a plugin.
    ///
    /// Cancel safe.
    #[tracing::instrument(skip_all)]
    pub async fn recv(&mut self, host: &Host) -> Action {
        // TODO: receive from other_actions too

        loop {
            tracing::trace!(items_in_queue = self.receiver.len());

            let (plugin, response) = self
                .receiver
                .recv()
                .await
                .expect("host should contain corresponding sender");

            if let Some(action) = self.response_to_action(host, &plugin, response) {
                return action;
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn try_recv(&mut self, host: &Host) -> Option<Action> {
        tracing::trace!(items_in_queue = self.receiver.len());

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
        tracing::trace!(?plugin, ?response, "received plugin response");

        match response.response {
            covey_proto::ResponseBody::SetList(list) => {
                // Check if the latest received id < new id. If so, send the action.
                // Otherwise, this response is outdated and we should not update the list.
                let new = response.request_id.0;
                if self.latest_received_query_request_id < new {
                    self.latest_received_query_request_id = new;
                    Some(Action::SetList(crate::List::from_proto(
                        host,
                        plugin,
                        list,
                        response.request_id,
                    )))
                } else {
                    tracing::trace!("ignoring list response due to outdated request id");
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

/// Finds extra plugins from the plugin directory and inserts it into the
/// config.
fn find_and_insert_plugins_from_fs(config: &mut GlobalConfig) {
    let Ok(dirs) = fs::read_dir(&*PLUGINS_DIR) else {
        warn!("failed to read plugins dir");
        return;
    };

    // each directory in the plugins directory should be the plugin's id
    let plugin_ids = dirs
        .filter_map(Result::ok)
        .flat_map(|plugin_dir| plugin_dir.file_name().into_string())
        .inspect(|plugin_id| debug!("discovered plugin {plugin_id} from fs"))
        .map(|plugin_id| PluginId::new(&plugin_id));
    config.plugins.extend_lossy(plugin_ids.map(|plugin_id| {
        let mut entry = PluginEntry::new(plugin_id);
        entry.disabled = true; // require explicitly enabling a new plugin
        entry
    }));
}

/// Automatically kills plugin processes after a period of time if they haven't
/// been queried/activated.
///
/// Never kills the most recently queried plugin to avoid the user trying to
/// activate a list item of a killed plugin process.
struct PluginProcessGc {
    last_touched_times: Arc<Mutex<HashMap<PluginWeak, Instant>>>,
    stop_signal: Arc<AtomicBool>,
}

impl PluginProcessGc {
    /// Kills plugin processes after _at least_ `timeout` has passed.
    ///
    /// The exact time that the processes are killed is not precise and may be a
    /// while after `timeout`.
    fn new(timeout: Duration) -> Self {
        let last_touched_times = Arc::new(Mutex::new(HashMap::<PluginWeak, Instant>::new()));
        let stop_signal = Arc::new(AtomicBool::new(false));

        let this = Self {
            last_touched_times: Arc::clone(&last_touched_times),
            stop_signal: Arc::clone(&stop_signal),
        };

        thread::spawn(move || {
            loop {
                // Check the hash map periodically.
                // Simpler than storing futures with exact timeouts and allows the hashmap to be
                // cleared if the plugin has been dropped anyways.
                thread::sleep(Duration::from_mins(1));

                if stop_signal.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let mut last_touched_times = last_touched_times.lock().unwrap();
                let Some(most_recent_query) = last_touched_times.values().max().copied() else {
                    // empty map, don't need to do anything
                    continue;
                };

                last_touched_times.retain(|plugin, query_time| match plugin.upgrade() {
                    Some(plugin) => {
                        if query_time.elapsed() > timeout && *query_time != most_recent_query {
                            plugin.kill_process();
                            false
                        } else {
                            true
                        }
                    }
                    // plugin is already gone
                    None => false,
                });
            }
        });

        this
    }

    fn touch(&self, plugin: &Plugin) {
        self.last_touched_times
            .lock()
            .unwrap()
            .insert(plugin.downgrade(), Instant::now());
    }
}

impl Drop for PluginProcessGc {
    fn drop(&mut self) {
        self.stop_signal
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
