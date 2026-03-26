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

/// Private to not expose that [`Arc<Plugin>`] can implement [`Plugin`].
pub(crate) struct BlockingPluginWrapper<T>(Arc<T>);

impl<T: Plugin + Send + Sync> Plugin for BlockingPluginWrapper<T> {
    type Config = T::Config;

    async fn new(config: Self::Config) -> Result<Self> {
        Ok(BlockingPluginWrapper(Arc::new(T::new(config).await?)))
    }

    async fn query(&self, query: String) -> Result<List> {
        let this = Arc::clone(&self.0);
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(T::query(&this, query))
        })
        .await
        .unwrap()
    }
}
