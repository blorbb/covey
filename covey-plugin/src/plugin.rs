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
