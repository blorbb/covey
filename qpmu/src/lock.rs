use std::{ops, sync::Arc};

use parking_lot::Mutex;

/// Simple wrapper around an `Arc<Mutex<T>>` hiding implementation details.
#[derive(Debug)]
pub struct SharedMutex<T>(Arc<Mutex<T>>);

impl<T> SharedMutex<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub fn lock(&self) -> SharedMutexGuard<'_, T> {
        SharedMutexGuard(self.0.lock())
    }
}

impl<T> Clone for SharedMutex<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Debug)]
pub struct SharedMutexGuard<'a, T>(parking_lot::MutexGuard<'a, T>);

impl<T> ops::Deref for SharedMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> ops::DerefMut for SharedMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
