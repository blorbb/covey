use core::fmt;
use std::{hash::Hash, path::PathBuf, sync::Arc};

use color_eyre::eyre::Result;
use covey_schema::{
    config::PluginEntry,
    keyed_list::{Id, Identify},
    manifest::PluginManifest,
};

use crate::DATA_DIR;

/// A ref-counted reference to a plugin instance.
///
/// This can be constructed using [`GlobalConfig::load`].
///
/// Comparison traits ([`Eq`], [`Hash`], etc) are in terms of
/// this plugin's name. [`Equivalent`] is also implemented to
/// look up this plugin based on it's name.
///
/// [`GlobalConfig::load`]: crate::config::GlobalConfig::load
#[derive(Clone)]
pub struct Plugin {
    plugin: Arc<implementation::LazyPlugin>,
}

impl Plugin {
    /// Initialises a plugin from it's configuration.
    pub(crate) fn new(config: PluginEntry) -> Result<Self> {
        Ok(Self {
            plugin: Arc::new(implementation::LazyPlugin::new(config)?),
        })
    }

    pub fn id(&self) -> &Id {
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

    pub(crate) async fn query(
        &self,
        query: impl Into<String>,
    ) -> Result<covey_proto::QueryResponse> {
        self.plugin
            .get_and_init()
            .await?
            .call_query(query.into())
            .await
    }

    pub(crate) async fn activate(
        &self,
        selection_id: u64,
        command_name: String,
    ) -> Result<tonic::Streaming<covey_proto::ActivationResponse>> {
        Ok(self
            .plugin
            .get_and_init()
            .await?
            .call_activate(selection_id, command_name)
            .await?)
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

// Allow looking up a plugin in a hash set by it's name.
// Implement `Equivalent` instead of `Borrow` as plugins should be used
// in an indexmap. It also doesn't completely fit the `Borrow` contract.
impl Identify for Plugin {
    fn id(&self) -> &covey_schema::keyed_list::Id {
        self.id()
    }
}

// Do not implement serde traits. Can be serialized as a string but it can't
// be properly deserialized.

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
    use std::{path::PathBuf, process::Stdio};

    use color_eyre::eyre::{Context as _, Result};
    use covey_proto::plugin_client::PluginClient;
    use covey_schema::{config::PluginEntry, manifest::PluginManifest};
    use tokio::{
        io::{AsyncBufReadExt as _, BufReader},
        process::{Child, Command},
        sync::OnceCell,
    };
    use tonic::{Request, Streaming, transport::Channel};
    use tracing::info;

    use super::{binary_path, manifest_path};

    /// A plugin that is not initialised until [`Self::get_and_init`] is called.
    ///
    /// The manifest is loaded on construction.
    pub(super) struct LazyPlugin {
        cell: OnceCell<PluginInner>,
        pub(super) manifest: PluginManifest,
        pub(super) entry: PluginEntry,
    }

    impl LazyPlugin {
        pub(super) fn new(config: PluginEntry) -> Result<Self> {
            let id = &config.id;
            let path = manifest_path(id.as_str());
            let toml = std::fs::read_to_string(path)
                .context(format!("error opening manifest file of {}", id.as_str()))?;
            let manifest: PluginManifest = toml::from_str(&toml)
                .context(format!("error reading manifest of {}", id.as_str()))?;

            Ok(Self {
                cell: OnceCell::new(),
                manifest,
                entry: config,
            })
        }

        /// Gets access to a plugin and ensures it is initialised.
        ///
        /// Locks exclusive access to the plugin while initialising.
        pub(super) async fn get_and_init(&self) -> Result<&PluginInner> {
            self.cell
                .get_or_try_init(|| async {
                    info!("initialising plugin {:?}", self.entry.id);
                    let config_json = serde_json::to_string(&self.entry.settings)?;
                    let bin_path = binary_path(self.entry.id.as_str());
                    let plugin = PluginInner::new(bin_path).await?;

                    plugin
                        .call_initialise(config_json)
                        .await
                        .context("plugin initialisation function failed")?;
                    color_eyre::eyre::Ok(plugin)
                })
                .await
                .context(format!("failed to initialise plugin {:?}", self.entry.id))
        }
    }

    /// Internals of a plugin.
    ///
    /// Simple wrapper that handles some request-response conversions.
    ///
    /// This should only be returned to the [`super::Plugin`] in an
    /// initialised state.
    pub(super) struct PluginInner {
        plugin: PluginClient<Channel>,
        // killed on drop, need to hold it so that it's dropped when this struct is dropped.
        _process: Child,
    }

    impl PluginInner {
        /// Starts the plugin binary but does not call initialise.
        async fn new(bin_path: PathBuf) -> Result<Self> {
            // run process and read first line
            let mut process = Command::new(&bin_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()
                .context("failed to spawn plugin server")?;

            let stdout = process.stdout.take().expect("stdout should be captured");
            let stderr = process.stderr.take().expect("stderr should be captured");
            let mut stdout = BufReader::new(stdout);
            let stderr = BufReader::new(stderr);

            let mut first_line = String::new();
            stdout.read_line(&mut first_line).await.context(
                "failed to read port or error from plugin: plugin should print to stdout",
            )?;

            tokio::spawn(async move {
                let mut lines = stdout.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!("plugin: {line}");
                }
            });
            tokio::spawn(async move {
                let mut lines = stderr.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    tracing::info!("plugin: {line}");
                }
            });

            let port: u16 = first_line
                .trim()
                .parse()
                .context("plugin should print it's connected port number to stdout")?;

            let client = PluginClient::connect(format!("http://[::1]:{port}"))
                .await
                .context(format!("failed to connect to plugin server on port {port}"))?;

            info!("finished initialising plugin binary");
            Ok(Self {
                plugin: client,
                _process: process,
            })
        }

        pub(super) async fn call_initialise(&self, config_json: String) -> Result<()> {
            self.plugin
                .clone()
                .initialise(Request::new(covey_proto::InitialiseRequest {
                    json: config_json,
                }))
                .await?;
            Ok(())
        }

        pub(super) async fn call_query(&self, query: String) -> Result<covey_proto::QueryResponse> {
            Ok(self
                .plugin
                .clone()
                .query(Request::new(covey_proto::QueryRequest { query }))
                .await?
                .into_inner())
        }

        pub(super) async fn call_activate(
            &self,
            selection_id: u64,
            command_name: String,
        ) -> Result<Streaming<covey_proto::ActivationResponse>> {
            Ok(self
                .plugin
                .clone()
                .activate(Request::new(covey_proto::ActivationRequest {
                    selection_id,
                    command_name,
                }))
                .await?
                .into_inner())
        }
    }
}
