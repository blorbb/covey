use core::fmt;

use color_eyre::eyre::{ContextCompat, Result};
use tokio::fs;

use super::action;
use crate::{
    config::PluginConfig,
    plugin::{proto, Action},
    Details, Input, ResultList, DATA_DIR,
};

/// A static reference to a plugin instance.
///
/// This can be constructed using [`Config::load_plugins`].
///
/// [`Config::load_plugins`]: crate::config::Config::load_plugins
#[derive(Clone, Copy)]
pub struct Plugin {
    plugin: &'static implementation::LazyPlugin,
}

impl Plugin {
    /// Initialises a plugin from it's configuration.
    ///
    /// Note that this will leak the plugin and configuration, as they should
    /// be active for the entire program.
    pub fn new(config: PluginConfig) -> Result<Self> {
        Ok(Self {
            plugin: Box::leak(Box::new(implementation::LazyPlugin::new(config))),
        })
    }

    pub fn name(&self) -> &str {
        &self.plugin.config.name
    }

    pub fn prefix(&self) -> &str {
        &self.plugin.config.prefix
    }

    pub async fn details(&self) -> Result<Details> {
        self.plugin
            .details()
            .await
            .map(|details| Details::from_proto(*self, details))
    }

    pub(crate) async fn query(&self, query: impl Into<String>) -> Result<ResultList> {
        Ok(ResultList::from_proto(
            *self,
            self.plugin
                .get_and_init()
                .await?
                .call_query(query.into())
                .await?,
        ))
    }

    pub(crate) async fn activate(&self, selection_id: u64) -> Result<Vec<Action>> {
        Ok(action::map_actions(
            *self,
            self.plugin
                .get_and_init()
                .await?
                .call_activate(selection_id)
                .await?,
        ))
    }

    pub(crate) async fn alt_activate(&self, selection_id: u64) -> Result<Vec<Action>> {
        Ok(action::map_actions(
            *self,
            self.plugin
                .get_and_init()
                .await?
                .call_alt_activate(selection_id)
                .await?,
        ))
    }

    pub(crate) async fn hotkey_activate(
        &self,
        selection_id: u64,
        hotkey: proto::Hotkey,
    ) -> Result<Vec<Action>> {
        Ok(action::map_actions(
            *self,
            self.plugin
                .get_and_init()
                .await?
                .call_hotkey_activate(selection_id, hotkey)
                .await?,
        ))
    }

    pub(crate) async fn complete(&self, selection_id: u64) -> Result<Option<Input>> {
        Ok(self
            .plugin
            .get_and_init()
            .await?
            .call_complete(selection_id)
            .await?
            .map(|il| Input::from_proto(*self, il)))
    }
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Plugin").field(&self.name()).finish()
    }
}

/// Gets the connection URL for a given plugin.
///
/// The database file will be created first.
async fn sqlite_connection_url(plugin_name: &str) -> Result<String> {
    let path = DATA_DIR.join(format!("{plugin_name}.db"));
    assert!(path.is_absolute());

    // make the file
    fs::create_dir_all(&*DATA_DIR).await?;
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .await?;

    // https://docs.rs/sqlx/latest/sqlx/sqlite/struct.SqliteConnectOptions.html
    let connection_string = format!(
        "sqlite://{}",
        path.to_str()
            .context("plugin data path must be a UTF-8 string")?
    );

    Ok(connection_string)
}

mod implementation {
    use std::{path::PathBuf, process::Stdio};

    use color_eyre::eyre::{bail, Context as _, Result};
    use tokio::{
        io::{AsyncBufReadExt as _, BufReader},
        process::Command,
        sync::{Mutex, OnceCell},
    };
    use tonic::{transport::Channel, Request};
    use tracing::info;

    use super::sqlite_connection_url;
    use crate::{
        config::PluginConfig,
        plugin::proto::{self, plugin_client::PluginClient},
        PLUGINS_DIR,
    };

    /// A plugin that is not initialised until [`Self::get_and_init`] is called.
    pub(super) struct LazyPlugin {
        cell: OnceCell<Result<PluginInner>>,
        called_initialise: Mutex<bool>,
        pub(super) config: PluginConfig,
    }

    impl LazyPlugin {
        pub(super) fn new(config: PluginConfig) -> Self {
            Self {
                cell: OnceCell::new(),
                called_initialise: Mutex::new(false),
                config,
            }
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
                let db_url = sqlite_connection_url(&self.config.name).await?;
                let config_toml = self.config.config.to_string();

                inner
                    .plugin
                    .clone()
                    .initialise(Request::new(proto::InitialiseRequest {
                        toml: config_toml,
                        sqlite_url: db_url,
                    }))
                    .await
                    .context("plugin initialisation function failed")?;
                *initialise_guard = true;
            }

            Ok(inner)
        }

        async fn get_without_init(&self) -> Result<&PluginInner> {
            let plugin = self
                .cell
                .get_or_init(|| async {
                    info!("initialising plugin {}", self.config.name);

                    let bin_path = PLUGINS_DIR.join(&self.config.name);

                    Ok(PluginInner::new(bin_path).await?)
                })
                .await;

            match plugin {
                Ok(a) => Ok(a),
                Err(e) => bail!("failed to initialise plugin {}: {e}", self.config.name),
            }
        }

        pub(super) async fn details(&self) -> Result<proto::DetailsResponse> {
            Ok(self
                .get_without_init()
                .await?
                .plugin
                .clone()
                .details(Request::new(()))
                .await?
                .into_inner())
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
            let mut process = Command::new(bin_path)
                .stdout(Stdio::piped())
                .spawn()
                .context("failed to spawn plugin server")?;

            let stdout = process.stdout.take().expect("stdout should be captured");
            let mut stdout = BufReader::new(stdout);

            let mut first_line = String::new();
            stdout.read_line(&mut first_line).await.context(
                "failed to read port or error from plugin: plugin should print to stdout",
            )?;

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

        pub(super) async fn call_activate(&self, selection_id: u64) -> Result<Vec<proto::Action>> {
            Ok(self
                .plugin
                .clone()
                .activate(Request::new(proto::ActivationRequest { selection_id }))
                .await?
                .into_inner()
                .actions)
        }

        pub(super) async fn call_alt_activate(
            &self,
            selection_id: u64,
        ) -> Result<Vec<proto::Action>> {
            Ok(self
                .plugin
                .clone()
                .alt_activate(Request::new(proto::ActivationRequest { selection_id }))
                .await?
                .into_inner()
                .actions)
        }

        pub(super) async fn call_hotkey_activate(
            &self,
            selection_id: u64,
            hotkey: proto::Hotkey,
        ) -> Result<Vec<proto::Action>> {
            Ok(self
                .plugin
                .clone()
                .hotkey_activate(Request::new(proto::HotkeyActivationRequest {
                    request: proto::ActivationRequest { selection_id },
                    hotkey,
                }))
                .await?
                .into_inner()
                .actions)
        }

        pub(super) async fn call_complete(
            &self,
            selection_id: u64,
        ) -> Result<Option<proto::Input>> {
            Ok(self
                .plugin
                .clone()
                .complete(Request::new(proto::ActivationRequest { selection_id }))
                .await?
                .into_inner()
                .input)
        }
    }
}
