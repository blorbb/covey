use core::fmt;
use std::{
    hash::Hash,
    io,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Weak},
};

use covey_schema::{
    config::PluginEntry,
    hotkey::Hotkey,
    id::{CommandId, PluginId, StringId as _},
    keyed_list::Identify,
    manifest::PluginManifest,
};
use tokio::{
    io::{AsyncBufReadExt as _, AsyncWriteExt as _, BufReader},
    process::{Child, ChildStdin, Command},
    sync::{Mutex, mpsc},
};

use crate::DATA_DIR;

/// Comparison traits ([`Eq`], [`Hash`], etc) are in terms of
/// this plugin's name.
///
/// When this plugin is constructed, only the manifest is loaded.
/// The plugin process is not spawned until a request is made.
/// We handle killed plugin processes by re-spawning the process then
/// retrying the request.
#[derive(Clone)]
pub struct Plugin {
    inner: Arc<PluginInner>,
}

impl Plugin {
    pub(crate) fn new_read_manifest(
        entry: PluginEntry,
        response_sender: mpsc::UnboundedSender<(Plugin, covey_proto::Response)>,
    ) -> anyhow::Result<Self> {
        let toml = std::fs::read_to_string(manifest_path(entry.id.as_str()))?;
        let manifest: PluginManifest = toml::from_str(&toml)?;

        Ok(Self::new(entry, manifest, response_sender))
    }

    pub(crate) fn new(
        entry: PluginEntry,
        manifest: PluginManifest,
        response_sender: mpsc::UnboundedSender<(Plugin, covey_proto::Response)>,
    ) -> Self {
        Self {
            inner: Arc::new(PluginInner {
                manifest,
                entry,
                response_sender,
                process: Mutex::new(None),
            }),
        }
    }

    pub fn id(&self) -> &PluginId {
        &self.inner.entry.id
    }

    /// Gets the prefix used to activate this plugin, either the user-defined or
    /// default prefix.
    pub fn prefix(&self) -> Option<&str> {
        self.inner
            .entry
            .prefix
            .as_ref()
            .or(self.inner.manifest.default_prefix.as_ref())
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
        &self.inner.entry
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
        &self.inner.manifest
    }

    pub(crate) async fn query(&self, id: covey_proto::RequestId, text: String) {
        self.send_request_or_display_error(&covey_proto::Request::query(id, text))
            .await
    }
    pub(crate) async fn activate(
        &self,
        id: covey_proto::RequestId,
        item_id: covey_proto::ListItemId,
        command_id: CommandId,
    ) {
        self.send_request_or_display_error(&covey_proto::Request::activate(id, item_id, command_id))
            .await
    }

    async fn start_process(&self) -> io::Result<ActiveProcess> {
        let bin_path = self.binary_path();
        ActiveProcess::new(
            Arc::downgrade(&self.inner),
            bin_path,
            &self.config_entry().settings,
            self.inner.response_sender.clone(),
        )
        .await
    }

    /// Sends a request to the plugin process, retrying once if the process has been killed.
    async fn send_request_with_retry(&self, request: &covey_proto::Request) -> io::Result<()> {
        let mut guard = self.inner.process.lock().await;
        match &mut *guard {
            Some(process) => {
                match process.send_request(request).await {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        // TODO: remove this log, only restart if the right error kind is reached
                        tracing::warn!("failed to write request: {e:#}");
                        tracing::warn!("restarting killed plugin {:?}", self.id());
                        *process = self.start_process().await?;
                        process.send_request(request).await?;
                        Ok(())
                    }
                }
            }
            None => {
                tracing::info!("initialising plugin {}", self.id());
                let mut process = self.start_process().await?;
                // Set the guard even if the request fails for some reason
                let request_result = process.send_request(request).await;
                *guard = Some(process);
                request_result
            }
        }
    }

    async fn send_request_or_display_error(&self, request: &covey_proto::Request) {
        match self.send_request_with_retry(request).await {
            Ok(()) => {}
            Err(e) => {
                _ = self.inner.response_sender.send((
                    self.clone(),
                    covey_proto::Response::display_error(request.id, format!("{e:#}")),
                ))
            }
        }
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

struct PluginInner {
    manifest: PluginManifest,
    entry: PluginEntry,
    response_sender: mpsc::UnboundedSender<(Plugin, covey_proto::Response)>,
    process: Mutex<Option<ActiveProcess>>,
}

impl Drop for PluginInner {
    fn drop(&mut self) {
        tracing::info!("dropped plugin {}", self.entry.id)
    }
}

struct ActiveProcess {
    // killed on drop, need to hold it so that it's dropped when this struct is dropped.
    _process: Child,
    child_stdin: ChildStdin,
}

impl ActiveProcess {
    pub(super) async fn new(
        plugin: Weak<PluginInner>,
        bin_path: PathBuf,
        initialization_settings: &serde_json::Map<String, serde_json::Value>,
        response_sender: mpsc::UnboundedSender<(Plugin, covey_proto::Response)>,
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
            let plugin = Weak::clone(&plugin);
            async move {
                let mut lines = stderr.lines();
                while let Ok(Some(line)) = lines.next_line().await
                    && let Some(inner) = plugin.upgrade()
                {
                    let plugin = Plugin { inner };
                    tracing::info!("plugin {id}: {line}", id = plugin.id());
                }
            }
        });

        // Forward child stdout to the response_sender channel.
        // Any unrecognised lines will be forwarded as logs, but as a warning.
        // Plugins should not be printing logs to stdout.
        tokio::spawn(async move {
            let mut lines = stdout.lines();
            while let Ok(Some(line)) = lines.next_line().await
                && let Some(inner) = plugin.upgrade()
            {
                let plugin = Plugin { inner };
                match serde_json::from_str::<covey_proto::Response>(&line) {
                    Ok(response) => _ = response_sender.send((plugin, response)),
                    Err(_) => {
                        tracing::warn!("plugin {id} (stdout): {line}", id = plugin.id())
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
    pub(super) async fn send_request(&mut self, request: &covey_proto::Request) -> io::Result<()> {
        let mut json = serde_json::to_string(request).expect("request should be serializable");
        json.push('\n');
        self.child_stdin.write_all(json.as_bytes()).await?;
        self.child_stdin.flush().await?;
        Ok(())
    }
}
