use core::fmt;
use std::{
    hash::Hash,
    io::{self, BufRead as _, BufReader, Write as _},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex, Weak, atomic::AtomicU32},
};

use covey_schema::{
    config::PluginEntry,
    hotkey::Hotkey,
    id::{CommandId, PluginId, StringId as _},
    keyed_list::Identify,
    manifest::PluginManifest,
};
use tokio::sync::mpsc;

use crate::{DATA_DIR, event::Message};

/// An integer to distinguish between multiple constructions of the same plugin
/// ID. Should not use pointer equality as an address may be reused when
/// trying to compare [`PluginWeak`].
static PLUGIN_GENERATION: AtomicU32 = AtomicU32::new(0);

/// A plugin ID with a specific, immutable configuration.
///
/// This is ref-counted, and thus cheap to clone. Cloned plugins are [`Eq`] to
/// each other, but a differently constructed plugin even with the same plugin
/// ID are not equal.
///
/// When this plugin is constructed, only the manifest is loaded. The plugin
/// process is not spawned until a request is made. We handle killed plugin
/// processes by re-spawning the process then retrying the request.
#[derive(Clone)]
pub struct Plugin {
    inner: Arc<PluginInner>,
    generation: u32,
}

impl Plugin {
    pub(crate) fn new_read_manifest(
        entry: PluginEntry,
        messages: mpsc::UnboundedSender<Message>,
    ) -> anyhow::Result<Self> {
        let toml = std::fs::read_to_string(manifest_path(entry.id.as_str()))?;
        let manifest: PluginManifest = toml::from_str(&toml)?;

        Ok(Self::new(entry, manifest, messages))
    }

    pub(crate) fn new(
        entry: PluginEntry,
        manifest: PluginManifest,
        messages: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Self {
            inner: Arc::new(PluginInner {
                manifest,
                entry,
                messages,
                process: Mutex::new(None),
            }),
            generation: PLUGIN_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    pub(crate) fn kill_process(&self) {
        // Dropping the ActiveProcess kills the process.
        *self.inner.process.lock().unwrap() = None;
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

    pub(crate) fn query(&self, id: covey_proto::RequestId, text: String) {
        self.send_request_or_display_error(&covey_proto::Request::query(id, text))
    }
    pub(crate) fn activate(
        &self,
        id: covey_proto::RequestId,
        target_id: covey_proto::ActivationTarget,
        command_id: CommandId,
    ) {
        self.send_request_or_display_error(&covey_proto::Request::activate(
            id, target_id, command_id,
        ))
    }

    fn start_process(&self) -> io::Result<ActiveProcess> {
        let bin_path = self.binary_path();
        ActiveProcess::new(
            self.downgrade(),
            &bin_path,
            &self.config_entry().settings,
            self.inner.messages.clone(),
        )
    }

    /// Sends a request to the plugin process, retrying once if the process has
    /// been killed.
    fn send_request_with_retry(&self, request: &covey_proto::Request) -> io::Result<()> {
        // none of this is blocking
        let mut guard = self.inner.process.lock().unwrap();
        match &mut *guard {
            Some(process) => {
                match process.send_request(request) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        // TODO: remove this log, only restart if the right error kind is reached
                        tracing::warn!("failed to write request: {e:#}");
                        tracing::warn!("restarting killed plugin {:?}", self.id());

                        // Do not retry if the request is an activation request. The list item id
                        // would probably not refer to the correct item anymore. The process should
                        // not be dead if the frontend is able to hold a list item produced by that
                        // plugin, so this would only happen if something went wrong with the
                        // plugin.
                        match &request.request {
                            covey_proto::RequestBody::Activate(..) => Err(e),
                            covey_proto::RequestBody::Query(..) => {
                                *process = self.start_process()?;
                                process.send_request(request)?;
                                Ok(())
                            }
                        }
                    }
                }
            }
            None => {
                tracing::info!("initialising plugin {}", self.id());
                let mut process = self.start_process()?;
                // Set the guard even if the request fails for some reason
                let request_result = process.send_request(request);
                *guard = Some(process);
                request_result
            }
        }
    }

    fn send_request_or_display_error(&self, request: &covey_proto::Request) {
        match self.send_request_with_retry(request) {
            Ok(()) => {}
            Err(e) => {
                _ = self.inner.messages.send(Message::PluginResponse(
                    self.clone(),
                    covey_proto::Response::display_error(request.id, format!("{e:#}")),
                ))
            }
        }
    }

    pub fn downgrade(&self) -> PluginWeak {
        PluginWeak {
            id: self.id().clone(),
            generation: self.generation,
            inner: Arc::downgrade(&self.inner),
        }
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

/// A [`Plugin`] with a weak pointer.
#[derive(Clone)]
pub struct PluginWeak {
    id: PluginId,
    generation: u32,
    inner: Weak<PluginInner>,
}

impl PluginWeak {
    pub fn id(&self) -> &PluginId {
        &self.id
    }

    pub fn upgrade(&self) -> Option<Plugin> {
        Some(Plugin {
            inner: self.inner.upgrade()?,
            generation: self.generation,
        })
    }

    pub fn strong_count(&self) -> usize {
        self.inner.strong_count()
    }
}

// Traits - Plugin and PluginWeak should have equivalent implementations.

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Plugin")
            // avoid quotes around the id
            .field(&fmt::from_fn(|f| f.write_str(self.id().as_str())))
            .finish()
    }
}

impl fmt::Debug for PluginWeak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PluginWeak")
            .field(&fmt::from_fn(|f| f.write_str(self.id().as_str())))
            .finish()
    }
}

impl PartialEq for Plugin {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation
    }
}

impl PartialEq for PluginWeak {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation
    }
}

impl Eq for Plugin {}
impl Eq for PluginWeak {}

impl PartialOrd for Plugin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd for PluginWeak {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Plugin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Ordering by plugin ID first might be useful
        self.id()
            .cmp(other.id())
            .then(self.generation.cmp(&other.generation))
    }
}

impl Ord for PluginWeak {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id()
            .cmp(other.id())
            .then(self.generation.cmp(&other.generation))
    }
}

impl Hash for Plugin {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.generation.hash(state);
    }
}

impl Hash for PluginWeak {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.generation.hash(state);
    }
}

impl Identify for Plugin {
    type Id = PluginId;
    fn id(&self) -> &Self::Id {
        self.id()
    }
}

impl Identify for PluginWeak {
    type Id = PluginId;
    fn id(&self) -> &Self::Id {
        self.id()
    }
}

struct PluginInner {
    manifest: PluginManifest,
    entry: PluginEntry,
    messages: mpsc::UnboundedSender<Message>,
    process: Mutex<Option<ActiveProcess>>,
}

impl Drop for PluginInner {
    fn drop(&mut self) {
        tracing::info!("dropped plugin {}", self.entry.id);
    }
}

struct ActiveProcess {
    process: Child,
    child_stdin: ChildStdin,
}

impl ActiveProcess {
    /// This is _not blocking_.
    pub(super) fn new(
        plugin_weak: PluginWeak,
        bin_path: &Path,
        initialization_settings: &serde_json::Map<String, serde_json::Value>,
        messages: mpsc::UnboundedSender<Message>,
    ) -> io::Result<Self> {
        let initialization_settings = serde_json::to_string(initialization_settings)
            .expect("plugin init settings should be serializable");

        let mut process = Command::new(bin_path)
            .arg(initialization_settings)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        let stdout = process.stdout.take().expect("stdout should be captured");
        let stderr = process.stderr.take().expect("stderr should be captured");
        let stdin = process.stdin.take().expect("stdin should be captured");
        let stderr = BufReader::new(stderr);
        let stdout = BufReader::new(stdout);

        // Forward stderr as logs.
        std::thread::spawn({
            let plugin_weak = plugin_weak.clone();
            move || {
                let mut lines = stderr.lines();
                while let Some(Ok(line)) = lines.next()
                    && let Some(plugin) = plugin_weak.upgrade()
                {
                    tracing::info!("plugin {id}: {line}", id = plugin.id());
                }

                tracing::info!("stopped reading plugin {:?} stderr", plugin_weak.id());
            }
        });

        // Forward child stdout to the messages channel.
        // Any unrecognised lines will be forwarded as logs, but as a warning.
        // Plugins should not be printing logs to stdout.
        std::thread::spawn(move || {
            let mut lines = stdout.lines();
            while let Some(Ok(line)) = lines.next()
                && let Some(plugin) = plugin_weak.upgrade()
            {
                let Ok(response) = serde_json::from_str::<covey_proto::Response>(&line) else {
                    tracing::warn!("plugin {id} (stdout): {line}", id = plugin.id());
                    continue;
                };

                tracing::trace!("plugin {id} (stdout): {line}", id = plugin.id());
                match messages.send(Message::PluginResponse(plugin.clone(), response)) {
                    Ok(()) => {}
                    Err(e) => {
                        tracing::error!(?plugin, "failed to send response through channel: {e:#}");
                        return;
                    }
                }
            }

            tracing::info!("stopped reading plugin {:?} stdout", plugin_weak.id());
        });

        // Must work with child stdin directly instead of a channel so that attempted
        // requests can fail if the child has been killed.

        Ok(Self {
            process,
            child_stdin: stdin,
        })
    }

    /// Tries to send the request to the process. Does not retry on failure.
    pub(super) fn send_request(&mut self, request: &covey_proto::Request) -> io::Result<()> {
        let mut json = serde_json::to_string(request).expect("request should be serializable");
        json.push('\n');
        self.child_stdin.write_all(json.as_bytes())?;
        self.child_stdin.flush()?;
        Ok(())
    }
}

impl Drop for ActiveProcess {
    fn drop(&mut self) {
        // This also stops the stdout/err forwarding threads as the readers are closed.
        match self.process.kill() {
            Ok(()) => {}
            Err(e) => tracing::error!("failed to kill plugin process: {e:#}"),
        }
    }
}
