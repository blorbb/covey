use std::sync::Arc;

use crate::{List, Result, manifest::ManifestDeserialization};

pub trait Plugin: Sized + 'static {
    /// The user's configuration for this plugin.
    ///
    /// Use `()` if this plugin has no configuration.
    type Config: ManifestDeserialization;

    #[expect(async_fn_in_trait, reason = "plugin is single threaded")]
    async fn new(config: Self::Config) -> Result<Self>;

    #[expect(async_fn_in_trait, reason = "plugin is single threaded")]
    async fn query(&self, query: String) -> Result<List>;
}

pub trait PluginBlocking: Sized + Send + Sync + 'static {
    type Config: ManifestDeserialization;

    #[expect(async_fn_in_trait, reason = "plugin is single threaded")]
    async fn new(config: Self::Config) -> Result<Self>;

    fn query(&self, query: String) -> Result<List>;
}

impl<T: PluginBlocking> Plugin for Arc<T> {
    type Config = T::Config;

    async fn new(config: Self::Config) -> Result<Self> {
        Ok(Arc::new(T::new(config).await?))
    }

    async fn query(&self, query: String) -> Result<List> {
        let this = Arc::clone(self);
        tokio::task::spawn_blocking(move || T::query(&this, query))
            .await
            .unwrap()
    }
}
