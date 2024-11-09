//! Wrapper structs for the raw WIT bindings.

use std::fmt;

use color_eyre::eyre::{bail, eyre, Result};
use tokio::sync::Mutex;
use wasmtime::Store;

use crate::{config::PluginConfig, PLUGINS_DIR};

use super::{
    bindings::{self, DeferredResult, InputLine, QueryResult},
    init, PluginActivationAction,
};

/// A row in the results list.
///
/// Contains an associated plugin.
#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub icon: Option<String>,
    pub description: String,
    pub metadata: String,
    plugin: Plugin,
}

impl fmt::Debug for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListItem")
            .field("title", &self.title)
            .field("description", &self.description)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl ListItem {
    fn from_item_and_plugin(item: bindings::ListItem, plugin: Plugin) -> Self {
        Self {
            title: item.title,
            icon: item.icon,
            description: item.description,
            metadata: item.metadata,
            plugin,
        }
    }

    fn from_many_and_plugin(items: Vec<bindings::ListItem>, plugin: Plugin) -> Vec<Self> {
        items
            .into_iter()
            .map(|item| Self::from_item_and_plugin(item, plugin))
            .collect()
    }

    pub fn plugin(&self) -> Plugin {
        self.plugin
    }

    pub async fn activate(self) -> Result<Vec<PluginActivationAction>> {
        self.plugin.clone().activate(self).await
    }

    pub async fn complete(self, query: &str) -> Result<Option<InputLine>> {
        self.plugin.clone().complete(query, self).await
    }
}

impl From<ListItem> for bindings::ListItem {
    fn from(value: ListItem) -> Self {
        Self {
            title: value.title,
            icon: value.icon,
            description: value.description,
            metadata: value.metadata,
        }
    }
}

/// A static reference to a plugin wasm instance.
#[derive(Clone, Copy)]
pub struct Plugin {
    plugin: &'static Mutex<PluginInner>,
    config: &'static PluginConfig,
}

impl Plugin {
    /// Initialises a plugin from it's configuration.
    ///
    /// Note that this will leak the plugin and configuration, as they should
    /// be active for the entire program.
    pub async fn from_config(config: PluginConfig) -> Result<Self> {
        let boxed = Box::new(Mutex::new(PluginInner::from_config(&config).await?));
        Ok(Self {
            plugin: Box::leak(boxed),
            config: Box::leak(Box::new(config)),
        })
    }

    pub fn prefix(&self) -> &'static str {
        &self.config.prefix
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

    async fn activate(&self, item: ListItem) -> Result<Vec<PluginActivationAction>> {
        self.plugin
            .lock()
            .await
            .call_activate(&bindings::ListItem::from(item))
            .await
    }

    async fn complete(&self, query: &str, item: ListItem) -> Result<Option<InputLine>> {
        self.plugin
            .lock()
            .await
            .call_complete(query, &bindings::ListItem::from(item))
            .await
    }
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Plugin")
            .field("config", &self.config)
            .finish()
    }
}

/// Internals of a plugin.
///
/// Simple wrapper around `bindings::Plugin` so that calling functions doesn't
/// need to pass in a `Store`.
struct PluginInner {
    plugin: bindings::Plugin,
    store: Store<init::State>,
}

impl PluginInner {
    async fn from_config(config: &PluginConfig) -> Result<Self> {
        // wasmtime error is weird, need to do this match
        let (plugin, store) = match init::initialise_plugin(
            PLUGINS_DIR.join(format!("{}.wasm", config.name.replace('-', "_"))),
        )
        .await
        {
            Ok((p, s)) => (p, s),
            Err(e) => bail!("failed to load plugin {name}: {e}", name = config.name),
        };

        Ok(Self { plugin, store })
    }

    async fn call_query(&mut self, input: &str) -> Result<QueryResult> {
        self.plugin
            .call_query(&mut self.store, input)
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
            .call_handle_deferred(&mut self.store, query, result)
            .await
            .unwrap()
            .map_err(|e| eyre!(e))
    }

    async fn call_activate(
        &mut self,
        item: &bindings::ListItem,
    ) -> Result<Vec<PluginActivationAction>> {
        self.plugin
            .call_activate(&mut self.store, item)
            .await
            .unwrap()
            .map_err(|e| eyre!(e))
    }

    async fn call_complete(
        &mut self,
        query: &str,
        item: &bindings::ListItem,
    ) -> Result<Option<InputLine>> {
        self.plugin
            .call_complete(&mut self.store, query, item)
            .await
            .unwrap()
            .map_err(|e| eyre!(e))
    }
}
