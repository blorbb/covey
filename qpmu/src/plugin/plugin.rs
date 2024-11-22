use core::fmt;
use std::{path::PathBuf, process::Stdio};

use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    sync::OnceCell,
};
use tonic::{transport::Channel, Request};
use tracing::info;

use self::proto::{plugin_client::PluginClient, QueryRequest, QueryResponse};
use super::action;
use crate::{
    config::PluginConfig,
    plugin::{proto, Action},
    Input, ResultList, DATA_DIR, PLUGINS_DIR,
};

/// A static reference to a plugin instance.
///
/// This can be constructed using [`Config::load_plugins`].
///
/// [`Config::load_plugins`]: crate::config::Config::load_plugins
#[derive(Clone, Copy)]
pub struct Plugin {
    plugin: &'static LazyPlugin,
}

impl Plugin {
    /// Initialises a plugin from it's configuration.
    ///
    /// Note that this will leak the plugin and configuration, as they should
    /// be active for the entire program.
    pub fn new(config: PluginConfig) -> Result<Self> {
        Ok(Self {
            plugin: Box::leak(Box::new(LazyPlugin::new(config))),
        })
    }

    pub fn name(&self) -> &str {
        &self.plugin.config.name
    }

    pub fn prefix(&self) -> &str {
        &self.plugin.config.prefix
    }

    pub(crate) async fn query(&self, query: impl Into<String>) -> Result<ResultList> {
        Ok(ResultList::from_proto(
            *self,
            self.plugin.get().await?.call_query(query.into()).await?,
        ))
    }

    pub(crate) async fn activate(
        &self,
        query: impl Into<String>,
        item: proto::ListItem,
    ) -> Result<Vec<Action>> {
        Ok(action::map_actions(
            *self,
            self.plugin
                .get()
                .await?
                .call_activate(query.into(), item)
                .await?,
        ))
    }

    pub(crate) async fn alt_activate(
        &self,
        query: impl Into<String>,
        item: proto::ListItem,
    ) -> Result<Vec<Action>> {
        Ok(action::map_actions(
            *self,
            self.plugin
                .get()
                .await?
                .call_alt_activate(query.into(), item)
                .await?,
        ))
    }

    pub(crate) async fn hotkey_activate(
        &self,
        query: impl Into<String>,
        item: proto::ListItem,
        hotkey: proto::Hotkey,
    ) -> Result<Vec<Action>> {
        Ok(action::map_actions(
            *self,
            self.plugin
                .get()
                .await?
                .call_hotkey_activate(query.into(), item, hotkey)
                .await?,
        ))
    }

    pub(crate) async fn complete(
        &self,
        query: impl Into<String>,
        item: proto::ListItem,
    ) -> Result<Option<Input>> {
        Ok(self
            .plugin
            .get()
            .await?
            .call_complete(query.into(), item)
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

/// A plugin that is not initialised until [`Self::get`] is called.
struct LazyPlugin {
    cell: OnceCell<Result<PluginInner>>,
    config: PluginConfig,
}

impl LazyPlugin {
    pub fn new(config: PluginConfig) -> Self {
        Self {
            cell: OnceCell::new(),
            config,
        }
    }

    /// Initialises or gets access to the plugin.
    pub async fn get(&self) -> Result<&PluginInner> {
        let plugin = self
            .cell
            .get_or_init(|| async {
                info!("initialising plugin {}", self.config.name);

                let bin_path = PLUGINS_DIR.join(&self.config.name);
                let db_url = sqlite_connection_url(&self.config.name).await?;
                let config_toml = self.config.config.to_string();

                Ok(PluginInner::new(bin_path, &db_url, &config_toml).await?)
            })
            .await;

        match plugin {
            Ok(a) => Ok(a),
            Err(e) => bail!("failed to initialise plugin {}: {e}", self.config.name),
        }
    }
}

/// Internals of a plugin.
///
/// Simple wrapper that handles some request-response conversions.
struct PluginInner {
    plugin: PluginClient<Channel>,
}

impl PluginInner {
    async fn new(bin_path: PathBuf, db_url: &str, toml: &str) -> Result<Self> {
        // run process and read first line
        let mut process = Command::new(bin_path)
            .arg(db_url)
            .arg(toml)
            .stdout(Stdio::piped())
            .spawn()
            .context("failed to spawn plugin server")?;

        let stdout = process.stdout.take().expect("stdout should be captured");
        let mut stdout = BufReader::new(stdout);

        let mut first_line = String::new();
        stdout
            .read_line(&mut first_line)
            .await
            .context("failed to read port or error from plugin: plugin should print to stdout")?;

        // stdout should either be:
        // PORT:12345 if the server was successfully created
        // ERROR:... if an error occurred during initialisation
        let port: u16 = match first_line.split_once(':') {
            Some(("PORT", port_num)) => port_num
                .trim()
                .parse()
                .context("plugin should print it's connected port number to stdout")?,
            Some(("ERROR", first_err_line)) => {
                _ = process.kill().await;

                // collect the entire error message, not just first line
                let mut err = String::from(first_err_line);
                stdout.read_to_string(&mut err).await?;
                bail!("Error initialising process:\n{err}")
            }
            Some(_) | None => {
                _ = process.kill().await;
                bail!("invalid stdout of plugin process")
            }
        };

        let client = PluginClient::connect(format!("http://[::1]:{port}"))
            .await
            .context(format!("failed to connect to plugin server on port {port}"))?;

        info!("finished initialising plugin");
        Ok(Self { plugin: client })
    }

    async fn call_query(&self, query: String) -> Result<QueryResponse> {
        Ok(self
            .plugin
            .clone()
            .query(Request::new(QueryRequest { query }))
            .await?
            .into_inner())
    }

    async fn call_activate(
        &self,
        query: String,
        item: proto::ListItem,
    ) -> Result<Vec<proto::Action>> {
        Ok(self
            .plugin
            .clone()
            .activate(Request::new(proto::ActivationRequest {
                query,
                selected: item,
            }))
            .await?
            .into_inner()
            .actions)
    }

    async fn call_alt_activate(
        &self,
        query: String,
        item: proto::ListItem,
    ) -> Result<Vec<proto::Action>> {
        Ok(self
            .plugin
            .clone()
            .alt_activate(Request::new(proto::ActivationRequest {
                query,
                selected: item,
            }))
            .await?
            .into_inner()
            .actions)
    }

    async fn call_hotkey_activate(
        &self,
        query: String,
        item: proto::ListItem,
        hotkey: proto::Hotkey,
    ) -> Result<Vec<proto::Action>> {
        Ok(self
            .plugin
            .clone()
            .hotkey_activate(Request::new(proto::HotkeyActivationRequest {
                request: proto::ActivationRequest {
                    query,
                    selected: item,
                },
                hotkey,
            }))
            .await?
            .into_inner()
            .actions)
    }

    async fn call_complete(
        &self,
        query: String,
        item: proto::ListItem,
    ) -> Result<Option<proto::Input>> {
        Ok(self
            .plugin
            .clone()
            .complete(Request::new(proto::ActivationRequest {
                query,
                selected: item,
            }))
            .await?
            .into_inner()
            .input)
    }
}
