use core::fmt;
use std::{path::PathBuf, process::Stdio};

use color_eyre::eyre::{bail, Context, Result};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    sync::OnceCell,
};
use tonic::{transport::Channel, Request};
use tracing::info;

use self::bindings::{plugin_client::PluginClient, QueryRequest, QueryResponse};
use super::{action, ListItem};
use crate::{
    config::PluginConfig,
    plugin::{bindings, Action},
    Input,
};

/// A static reference to a plugin wasm instance.
#[derive(Clone, Copy)]
pub struct Plugin {
    plugin: &'static LazyPlugin,
    prefix: &'static str,
}

impl Plugin {
    /// Initialises a plugin from it's configuration.
    ///
    /// Note that this will leak the plugin and configuration, as they should
    /// be active for the entire program.
    pub fn new(config: PluginConfig) -> Result<Self> {
        Ok(Self {
            plugin: Box::leak(Box::new(LazyPlugin::new(
                config.name,
                config.options.to_string(),
            ))),
            prefix: config.prefix.leak(),
        })
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Runs the plugin until the query is fully completed.
    pub async fn query(&self, query: impl Into<String>) -> Result<Vec<ListItem>> {
        Ok(ListItem::from_many_and_plugin(
            self.plugin
                .get()
                .await?
                .call_query(query.into())
                .await?
                .items,
            *self,
        ))
    }

    pub(super) async fn activate(&self, item: bindings::ListItem) -> Result<Vec<Action>> {
        Ok(action::map_actions(
            *self,
            self.plugin.get().await?.call_activate(item).await?,
        ))
    }

    pub(super) async fn complete(
        &self,
        query: impl Into<String>,
        item: bindings::ListItem,
    ) -> Result<Option<Input>> {
        Ok(self
            .plugin
            .get()
            .await?
            .call_complete(query.into(), item)
            .await?
            .map(|il| Input::from_wit_input(*self, il)))
    }
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Plugin").field(&self.plugin.name).finish()
    }
}

/// A plugin that is not initialised until [`Self::get`] is called.
struct LazyPlugin {
    cell: OnceCell<Result<PluginInner>>,
    name: String,
    options: String,
}

impl LazyPlugin {
    pub fn new(name: String, options: String) -> Self {
        Self {
            cell: OnceCell::new(),
            name,
            options,
        }
    }

    /// Initialises or gets access to the plugin.
    pub async fn get(&self) -> Result<&PluginInner> {
        let plugin = self
            .cell
            .get_or_init(|| async {
                info!("initialising plugin {}", self.name);
                Ok(PluginInner::new(crate::plugin_file_of(&self.name), &self.options).await?)
            })
            .await;

        match plugin {
            Ok(a) => Ok(a),
            Err(e) => bail!("failed to initialise plugin {}: {e}", self.name),
        }
    }
}

/// Internals of a plugin.
///
/// Simple wrapper around `bindings::Plugin` that handles some
/// request-response conversions.
struct PluginInner {
    plugin: PluginClient<Channel>,
}

impl PluginInner {
    async fn new(path: PathBuf, toml: &str) -> Result<Self> {
        let mut process = tokio::process::Command::new(path)
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

        let port: u16 = match first_line.split_once(':') {
            Some(("PORT", port_num)) => port_num
                .trim()
                .parse()
                .context("plugin should print it's connected port number to stdout")?,
            Some(("ERROR", first_err_line)) => {
                _ = process.kill().await;

                // collect error message
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

    async fn call_activate(&self, item: bindings::ListItem) -> Result<Vec<bindings::Action>> {
        Ok(self
            .plugin
            .clone()
            .activate(Request::new(item))
            .await?
            .into_inner()
            .actions)
    }

    async fn call_complete(
        &self,
        query: String,
        item: bindings::ListItem,
    ) -> Result<Option<bindings::InputLine>> {
        Ok(self
            .plugin
            .clone()
            .complete(Request::new(bindings::CompletionRequest {
                query,
                selected: Some(item),
            }))
            .await?
            .into_inner()
            .input)
    }
}
