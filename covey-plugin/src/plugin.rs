use std::future::Future;

use crate::{List, Result, manifest::ManifestDeserialization};

pub trait Plugin: Sized + Send + Sync + 'static {
    /// The user's configuration for this plugin.
    ///
    /// Use `()` if this plugin has no configuration.
    type Config: ManifestDeserialization;

    fn new(config: Self::Config) -> impl Future<Output = Result<Self>> + Send;

    fn query(&self, query: String) -> impl Future<Output = Result<List>> + Send;
}
