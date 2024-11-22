use std::ops::Deref;

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::Plugin;

pub(crate) struct PluginLock<T>(RwLock<Option<T>>);

impl<T: Plugin> PluginLock<T> {
    pub(crate) fn new_empty() -> Self {
        Self(RwLock::new(None))
    }

    pub(crate) async fn write(&self) -> RwLockWriteGuard<'_, Option<T>> {
        self.0.write().await
    }

    /// Reads the plugin, panicking if it is not initialised when
    /// the result is dereferenced.
    pub(crate) async fn force_read(&self) -> PluginReadGuard<'_, T> {
        PluginReadGuard(self.0.read().await)
    }
}

pub(crate) struct PluginReadGuard<'a, T>(RwLockReadGuard<'a, Option<T>>);
impl<T: Plugin> Deref for PluginReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("should have initialised")
    }
}
