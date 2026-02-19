use core::fmt;
use std::{hash::Hash, path::PathBuf, sync::Arc};

use anyhow::Result;
use covey_proto::{
    covey_request::{Request, RequestId},
    plugin_response::Response,
};
use covey_schema::{
    config::PluginEntry,
    hotkey::Hotkey,
    id::{CommandId, PluginId, StringId as _},
    keyed_list::Identify,
    manifest::PluginManifest,
};

use crate::DATA_DIR;

/// A ref-counted reference to a plugin instance.
///
/// Comparison traits ([`Eq`], [`Hash`], etc) are in terms of
/// this plugin's name.
///
/// When this plugin is constructed, only the manifest is loaded.
/// The plugin process is not spawned until a request is made.
/// We handle killed plugin processes by re-spawning the process then
/// retrying the request.
#[derive(Clone)]
pub struct Plugin {
    plugin: Arc<implementation::PluginInner>,
}

impl Plugin {
    /// Initialises a plugin from it's configuration and sends responses to
    /// the `response_sender` function.
    pub(crate) fn new(config: PluginEntry, response_sender: Arc<dyn Fn(Response)>) -> Result<Self> {
        Ok(Self {
            plugin: Arc::new(implementation::PluginInner::new(config, response_sender)?),
        })
    }

    pub fn id(&self) -> &PluginId {
        &self.plugin.entry.id
    }

    /// Gets the prefix used to activate this plugin, either the user-defined or
    /// default prefix.
    pub fn prefix(&self) -> Option<&str> {
        self.plugin
            .entry
            .prefix
            .as_ref()
            .or(self.plugin.manifest.default_prefix.as_ref())
            .map(String::as_str)
    }

    /// Get the hotkeys that a command can accept, either from user config
    /// or the default from the manifest.
    pub fn hotkeys_of_cmd(&self, cmd_id: &CommandId) -> Option<&[Hotkey]> {
        Some(
            &**self
                .config_entry()
                .commands
                .get(cmd_id)?
                .hotkeys
                .as_ref()
                .or_else(|| {
                    self.manifest()
                        .commands
                        .get(cmd_id)?
                        .default_hotkeys
                        .as_ref()
                })?,
        )
    }

    pub fn config_entry(&self) -> &PluginEntry {
        &self.plugin.entry
    }

    /// Returns the path to the provided plugin's directory.
    ///
    /// This is in `<data folder>/covey/plugins/<plugin name>`, for example,
    /// `~/.local/share/covey/plugins/my-plugin-name`.
    pub fn data_directory_path(&self) -> PathBuf {
        data_directory_path(self.id().as_str())
    }

    pub fn binary_path(&self) -> PathBuf {
        binary_path(self.id().as_str())
    }

    pub fn manifest_path(&self) -> PathBuf {
        manifest_path(self.id().as_str())
    }

    pub fn manifest(&self) -> &PluginManifest {
        &self.plugin.manifest
    }

    pub(crate) async fn query(&self, request_id: RequestId, query: String) {
        self.plugin
            .send_request_or_display_error(&Request::query(request_id, query))
            .await
    }

    pub(crate) async fn activate(
        &self,
        request_id: RequestId,
        item_id: covey_proto::plugin_response::ListItemId,
        command_id: CommandId,
    ) {
        self.plugin
            .send_request_or_display_error(&Request::activate(request_id, item_id, command_id))
            .await
    }
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Plugin").field(&self.id().as_str()).finish()
    }
}

// COMPARISON TRAITS //
// These MUST be implemented in terms of the name.

impl PartialEq for Plugin {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Plugin {}

impl PartialOrd for Plugin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Plugin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id().cmp(other.id())
    }
}

impl Hash for Plugin {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl Identify for Plugin {
    type Id = PluginId;
    fn id(&self) -> &Self::Id {
        self.id()
    }
}

fn data_directory_path(plugin_name: &str) -> PathBuf {
    DATA_DIR.join("plugins").join(plugin_name)
}

fn binary_path(plugin_name: &str) -> PathBuf {
    data_directory_path(plugin_name).join(plugin_name)
}

fn manifest_path(plugin_name: &str) -> PathBuf {
    data_directory_path(plugin_name).join("manifest.toml")
}

mod implementation {
    use std::{io, path::PathBuf, process::Stdio, sync::Arc};

    use anyhow::{Context as _, Result};
    use covey_proto::{covey_request::Request, plugin_response::Response};
    use covey_schema::{
        config::PluginEntry,
        id::{PluginId, StringId as _},
        manifest::PluginManifest,
    };
    use tokio::{
        io::{AsyncBufReadExt as _, AsyncWriteExt, BufReader},
        process::{Child, Command},
        sync::Mutex,
    };
    use tracing::info;

    use super::{binary_path, manifest_path};

    pub(super) struct PluginInner {
        process: Mutex<Option<PluginProcess>>,
        pub(super) manifest: PluginManifest,
        pub(super) entry: PluginEntry,
        response_sender: Arc<dyn Fn(Response)>,
    }

    impl PluginInner {
        pub(super) fn new(config: PluginEntry, responses: Arc<dyn Fn(Response)>) -> Result<Self> {
            let id = &config.id;
            let path = manifest_path(id.as_str());
            let toml = std::fs::read_to_string(path)
                .context(format!("error opening manifest file of {}", id.as_str()))?;
            let manifest: PluginManifest = toml::from_str(&toml)
                .context(format!("error reading manifest of {}", id.as_str()))?;

            Ok(Self {
                process: Mutex::new(None),
                manifest,
                entry: config,
                response_sender: responses,
            })
        }

        async fn new_process(&self) -> io::Result<PluginProcess> {
            let bin_path = binary_path(self.entry.id.as_str());
            PluginProcess::new(
                self.entry.id.clone(),
                bin_path,
                &self.entry.settings,
                Arc::clone(&self.response_sender),
            )
            .await
        }

        /// Sends a request to the plugin process, retrying once if the process has been killed.
        async fn send_request_with_retry(&self, request: &Request) -> io::Result<()> {
            let mut guard = self.process.lock().await;
            match &mut *guard {
                Some(process) => {
                    match process.send_request(request).await {
                        Ok(()) => Ok(()),
                        Err(e) => {
                            // TODO: remove this log, only restart if the right error kind is reached
                            tracing::warn!("failed to write request: {e:#}");
                            tracing::warn!("restarting killed plugin {:?}", self.entry.id);
                            *process = self.new_process().await?;
                            process.send_request(request).await?;
                            Ok(())
                        }
                    }
                }
                None => {
                    info!("initialising plugin {:?}", self.entry.id);
                    let mut process = self.new_process().await?;
                    // Set the guard even if the request fails for some reason
                    let request_result = process.send_request(request).await;
                    *guard = Some(process);
                    request_result
                }
            }
        }

        pub(super) async fn send_request_or_display_error(&self, request: &Request) {
            match self.send_request_with_retry(request).await {
                Ok(()) => {}
                Err(e) => {
                    _ = (self.response_sender)(Response::display_error(
                        request.id,
                        format!("{e:#}"),
                    ))
                }
            }
        }
    }

    struct PluginProcess {
        // killed on drop, need to hold it so that it's dropped when this struct is dropped.
        _process: Child,
        child_stdin: tokio::process::ChildStdin,
    }

    impl PluginProcess {
        /// Starts the plugin process.
        async fn new(
            plugin_id: PluginId,
            bin_path: PathBuf,
            initialization_settings: &serde_json::Map<String, serde_json::Value>,
            response_sender: Arc<dyn Fn(Response)>,
        ) -> io::Result<Self> {
            let initialization_settings = serde_json::to_string(initialization_settings)
                .expect("plugin init settings should be serializable");

            let mut process = Command::new(&bin_path)
                .arg(initialization_settings)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .kill_on_drop(true)
                .spawn()?;

            let stdout = process.stdout.take().expect("stdout should be captured");
            let stderr = process.stderr.take().expect("stderr should be captured");
            let stdin = process.stdin.take().expect("stdin should be captured");
            let stderr = BufReader::new(stderr);
            let stdout = BufReader::new(stdout);

            // Forward stderr as logs.
            tokio::spawn({
                let plugin_id = plugin_id.clone();
                async move {
                    let mut lines = stderr.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        tracing::info!("plugin {id}: {line}", id = plugin_id.as_str());
                    }
                }
            });

            // Forward child stdout to the response_sender channel.
            // Any unrecognised lines will be forwarded as logs, but as a warning.
            // Plugins should not be printing logs to stdout.
            tokio::task::spawn_local(async move {
                let mut lines = stdout.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    match serde_json::from_str::<Response>(&line) {
                        Ok(response) => response_sender(response),
                        Err(_) => {
                            tracing::warn!("plugin {id} (stdout): {line}", id = plugin_id.as_str())
                        }
                    }
                }
            });

            // Must work with child stdin directly instead of a channel so that attempted
            // requests can fail if the child has been killed.

            Ok(Self {
                _process: process,
                child_stdin: stdin,
            })
        }

        /// Tries to send the request to the process. Does not retry on failure.
        async fn send_request(&mut self, request: &Request) -> io::Result<()> {
            let mut json = serde_json::to_string(request).expect("request should be serializable");
            json.push('\n');
            self.child_stdin.write_all(json.as_bytes()).await?;
            self.child_stdin.flush().await?;
            Ok(())
        }
    }
}
