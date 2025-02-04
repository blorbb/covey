use core::fmt;
use std::{hash::Hash, path::PathBuf, sync::Arc};

use color_eyre::eyre::Result;
use covey_config::{
    config::PluginConfig,
    keyed_list::{Key, Keyed},
    manifest::PluginManifest,
};

use crate::{event::Action, proto, Input, List, DATA_DIR};

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
    pub(crate) fn new(config: PluginConfig) -> Result<Self> {
        Ok(Self {
            plugin: Arc::new(implementation::LazyPlugin::new(config)?),
        })
    }

    pub fn id(&self) -> &Key {
        &self.plugin.config.id
    }

    pub fn prefix(&self) -> &str {
        &self.plugin.config.prefix
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

    pub(crate) async fn query(&self, query: impl Into<String>) -> Result<List> {
        Ok(List::from_proto(
            self,
            self.plugin
                .get_and_init()
                .await?
                .call_query(query.into())
                .await?,
        ))
    }

    pub(crate) async fn activate(
        &self,
        selection_id: u64,
        command_name: String,
    ) -> Result<Vec<Action>> {
        Ok(self.map_proto_actions(
            self.plugin
                .get_and_init()
                .await?
                .call_activate(selection_id, command_name)
                .await?,
        ))
    }

    fn map_proto_actions(&self, actions: Vec<proto::Action>) -> Vec<Action> {
        use proto::action::Action as PAction;

        actions
            .into_iter()
            .filter_map(|action| {
                let Some(action) = action.action else {
                    tracing::error!("plugin {self:?} did not provide an action: ignoring");
                    return None;
                };

                Some(match action {
                    PAction::Close(()) => Action::Close,
                    PAction::RunCommand(proto::Command { cmd, args }) => {
                        Action::RunCommand(cmd, args)
                    }
                    PAction::RunShell(str) => Action::RunShell(str),
                    PAction::Copy(str) => Action::Copy(str),
                    PAction::SetInput(input) => Action::SetInput(Input::from_proto(self, input)),
                })
            })
            .collect()
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
        self.id().cmp(&other.id())
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
impl Keyed for Plugin {
    fn key(&self) -> &covey_config::keyed_list::Key {
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
    use covey_config::{config::PluginConfig, manifest::PluginManifest};
    use tokio::{
        io::{AsyncBufReadExt as _, BufReader},
        process::Command,
        sync::{Mutex, OnceCell},
    };
    use tonic::{transport::Channel, Request};
    use tracing::info;

    use super::{
        binary_path, manifest_path,
        proto::{self, plugin_client::PluginClient},
    };

    /// A plugin that is not initialised until [`Self::get_and_init`] is called.
    ///
    /// The manifest is loaded on construction.
    pub(super) struct LazyPlugin {
        cell: OnceCell<PluginInner>,
        // making the manifest sync makes it easier to use in settings
        called_initialise: Mutex<bool>,
        pub(super) manifest: PluginManifest,
        pub(super) config: PluginConfig,
    }

    impl LazyPlugin {
        pub(super) fn new(config: PluginConfig) -> Result<Self> {
            let id = &config.id;
            let path = manifest_path(id.as_str());
            let toml = std::fs::read_to_string(path)
                .context(format!("error opening manifest file of {}", id.as_str()))?;
            let manifest: PluginManifest = toml::from_str(&toml)
                .context(format!("error reading manifest of {}", id.as_str()))?;

            Ok(Self {
                cell: OnceCell::new(),
                called_initialise: Mutex::new(false),
                manifest,
                config,
            })
        }

        /// Gets access to a plugin and ensures it is initialised.
        ///
        /// Locks exclusive access to the plugin while initialising.
        pub(super) async fn get_and_init(&self) -> Result<&PluginInner> {
            let inner = self.get_without_init().await?;

            // ensures that the plugin initialisation function has been called.
            // if already initialised, the lock should be very quickly dropped.
            // otherwise, blocks any other accesses until initialisation
            // either succeeds or fails.
            let mut initialise_guard = self.called_initialise.lock().await;
            if !*initialise_guard {
                let config_json = serde_json::to_string(&self.config.config)?;

                inner
                    .plugin
                    .clone()
                    .initialise(Request::new(proto::InitialiseRequest { json: config_json }))
                    .await
                    .context("plugin initialisation function failed")?;
                *initialise_guard = true;
            }

            Ok(inner)
        }

        async fn get_without_init(&self) -> Result<&PluginInner> {
            self.cell
                .get_or_try_init(|| async {
                    info!("initialising plugin {:?}", self.config.id);
                    let bin_path = binary_path(self.config.id.as_str());
                    PluginInner::new(bin_path).await
                })
                .await
                .context(format!("failed to initialise plugin {:?}", self.config.id))
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
    }

    impl PluginInner {
        /// Starts the plugin binary but does not call initialise.
        async fn new(bin_path: PathBuf) -> Result<Self> {
            // run process and read first line
            let mut process = Command::new(&bin_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("failed to spawn plugin server")?;

            let stdout = process.stdout.take().expect("stdout should be captured");
            let mut stdout = BufReader::new(stdout);

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
                let mut lines =
                    BufReader::new(process.stderr.take().expect("stderr should be captured"))
                        .lines();
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
            Ok(Self { plugin: client })
        }

        pub(super) async fn call_query(&self, query: String) -> Result<proto::QueryResponse> {
            Ok(self
                .plugin
                .clone()
                .query(Request::new(proto::QueryRequest { query }))
                .await?
                .into_inner())
        }

        pub(super) async fn call_activate(
            &self,
            selection_id: u64,
            command_name: String,
        ) -> Result<Vec<proto::Action>> {
            Ok(self
                .plugin
                .clone()
                .activate(Request::new(proto::ActivationRequest {
                    selection_id,
                    command_name,
                }))
                .await?
                .into_inner()
                .actions)
        }
    }
}
