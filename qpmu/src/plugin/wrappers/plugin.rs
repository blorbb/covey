use core::fmt;

use color_eyre::eyre::{bail, eyre, Result};
use tokio::sync::{Mutex, MutexGuard, OnceCell};
use tracing::info;
use wasmtime::{component::ResourceAny, Store};

use super::{
    super::bindings::{DeferredResult, InputLine, QueryResult},
    ListItem,
};
use crate::{
    config::PluginConfig,
    plugin::{bindings, host, init, Action},
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
    pub fn new(config: PluginConfig, binary: Vec<u8>) -> Result<Self> {
        Ok(Self {
            plugin: Box::leak(Box::new(LazyPlugin::new(
                binary,
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
    ///
    /// Returns `Ok(None)` if any of the results are `QueryResult::Skip`.
    pub async fn complete_query(&self, query: &str) -> Result<Option<Vec<ListItem>>> {
        let mut result = self.plugin.get().await?.call_query(query).await?;
        loop {
            match result {
                QueryResult::SetList(vec) => {
                    return Ok(Some(ListItem::from_many_and_plugin(vec, *self)))
                }
                QueryResult::Defer(deferred_action) => {
                    let deferred_result = deferred_action.run().await;
                    result = self
                        .plugin
                        .get()
                        .await?
                        .call_handle_deferred(query, &deferred_result)
                        .await
                        .unwrap();
                }
                QueryResult::Skip => return Ok(None),
            }
        }
    }

    pub(super) async fn activate(&self, item: &bindings::ListItem) -> Result<Vec<Action>> {
        Ok(self
            .plugin
            .get()
            .await?
            .call_activate(item)
            .await?
            .into_iter()
            .map(|action| Action::from_wit_action(*self, action))
            .collect())
    }

    pub(super) async fn complete(
        &self,
        query: &str,
        item: &bindings::ListItem,
    ) -> Result<Option<InputLine>> {
        self.plugin.get().await?.call_complete(query, item).await
    }
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Plugin").field(&self.plugin.name).finish()
    }
}

/// A plugin that is not initialised until [`Self::get`] is called.
struct LazyPlugin {
    cell: OnceCell<Result<Mutex<PluginInner>>>,
    name: String,
    options: String,
    binary: Vec<u8>,
}

impl LazyPlugin {
    pub fn new(binary: Vec<u8>, name: String, options: String) -> Self {
        Self {
            cell: OnceCell::new(),
            binary,
            name,
            options,
        }
    }

    /// Initialises or gets access to the plugin.
    pub async fn get(&self) -> Result<MutexGuard<PluginInner>> {
        let plugin = self
            .cell
            .get_or_init(|| async {
                info!("initialising plugin {}", self.name);
                Ok(Mutex::new(
                    PluginInner::new(&self.binary, &self.options).await?,
                ))
            })
            .await;

        match plugin {
            Ok(a) => Ok(a.lock().await),
            Err(e) => bail!("failed to initialise plugin {}: {e}", self.name),
        }
    }
}

/// Internals of a plugin.
///
/// Simple wrapper around `bindings::Plugin` and the plugin handler resource
/// so that calling functions doesn't need to pass in a `Store` or `ResourceAny`.
struct PluginInner {
    plugin: bindings::Plugin,
    store: Store<host::State>,
    resource: ResourceAny,
}

impl PluginInner {
    async fn new(binary: &[u8], toml: &str) -> Result<Self> {
        let (plugin, mut store) = init::initialise_plugin(binary)
            .await
            .map_err(|e| eyre!(e))?;

        let resource = plugin
            .qpmu_plugin_handler()
            .handler()
            .call_constructor(&mut store, toml)
            .await
            .map_err(|e| eyre!(e))?;

        Ok(Self {
            plugin,
            store,
            resource,
        })
    }

    async fn call_query(&mut self, input: &str) -> Result<QueryResult> {
        self.plugin
            .qpmu_plugin_handler()
            .handler()
            .call_query(&mut self.store, self.resource, input)
            .await
            .unwrap()
            .map_err(|e| eyre!(e))
    }

    async fn call_handle_deferred(
        &mut self,
        query: &str,
        result: &DeferredResult,
    ) -> Result<QueryResult> {
        self.plugin
            .qpmu_plugin_handler()
            .handler()
            .call_handle_deferred(&mut self.store, self.resource, query, result)
            .await
            .unwrap()
            .map_err(|e| eyre!(e))
    }

    async fn call_activate(&mut self, item: &bindings::ListItem) -> Result<Vec<bindings::Action>> {
        self.plugin
            .qpmu_plugin_handler()
            .handler()
            .call_activate(&mut self.store, self.resource, item)
            .await
            .unwrap()
            .map_err(|e| eyre!(e))
    }

    async fn call_complete(
        &mut self,
        query: &str,
        item: &bindings::ListItem,
    ) -> Result<Option<bindings::InputLine>> {
        self.plugin
            .qpmu_plugin_handler()
            .handler()
            .call_complete(&mut self.store, self.resource, query, item)
            .await
            .unwrap()
            .map_err(|e| eyre!(e))
    }
}
