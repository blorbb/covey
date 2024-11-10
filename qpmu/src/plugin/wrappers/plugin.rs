use core::fmt;

use color_eyre::eyre::{eyre, Result};
use tokio::sync::Mutex;
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
    plugin: &'static Mutex<PluginInner>,
    name: &'static str,
    prefix: &'static str,
}

impl Plugin {
    /// Initialises a plugin from it's configuration.
    ///
    /// Note that this will leak the plugin and configuration, as they should
    /// be active for the entire program.
    pub async fn new(config: PluginConfig, binary: Vec<u8>) -> Result<Self> {
        let inner = PluginInner::new(&binary, &config.options.to_string())
            .await
            .map_err(|e| eyre!("failed to initialise {}: {e}", config.name))?;

        let boxed = Box::new(Mutex::new(inner));

        Ok(Self {
            plugin: Box::leak(boxed),
            name: config.name.leak(),
            prefix: config.prefix.leak(),
        })
    }

    pub fn prefix(&self) -> &'static str {
        &self.prefix
    }

    /// Runs the plugin until the query is fully completed.
    ///
    /// Returns `Ok(None)` if any of the results are `QueryResult::Skip`.
    pub async fn complete_query(&self, query: &str) -> Result<Option<Vec<ListItem>>> {
        let mut result = self.plugin.lock().await.call_query(query).await?;
        loop {
            match result {
                QueryResult::SetList(vec) => {
                    return Ok(Some(ListItem::from_many_and_plugin(vec, *self)))
                }
                QueryResult::Defer(deferred_action) => {
                    let deferred_result = deferred_action.run().await;
                    result = self
                        .plugin
                        .lock()
                        .await
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
            .lock()
            .await
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
        self.plugin.lock().await.call_complete(query, item).await
    }
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Plugin")
            .field("name", &self.name)
            .field("prefix", &self.prefix)
            .finish()
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
