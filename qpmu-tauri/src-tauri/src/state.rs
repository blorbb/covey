use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, OnceLock},
};

use parking_lot::Mutex;

use crate::ipc::frontend::{EventChannel, ListItem};

type Model = qpmu::Model<EventChannel>;

/// Must be initialised exactly once with [`AppState::init`].
pub struct AppState {
    inner: OnceLock<qpmu::lock::SharedMutex<Model>>,
    id_counter: AtomicU64,
    // TODO: make this actually efficient
    list_items: Mutex<HashMap<u64, qpmu::ListItem>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: OnceLock::new(),
            id_counter: AtomicU64::new(0),
            list_items: Mutex::new(HashMap::new()),
        }
    }

    pub fn init(&self, model: qpmu::lock::SharedMutex<Model>) {
        self.inner
            .set(model)
            .unwrap_or_else(|_| tracing::warn!("already set up"));
    }

    /// # Panics
    /// Panics if this has not been initialised yet.
    pub fn lock(&self) -> qpmu::lock::SharedMutexGuard<'_, Model> {
        self.inner
            .get()
            .expect("app state has not been set up")
            .lock()
    }

    pub fn register_list_items(
        &self,
        lis: impl ExactSizeIterator<Item = qpmu::ListItem>,
    ) -> Vec<ListItem> {
        let mut list_items = self.list_items.lock();
        list_items.clear();
        lis.map(|li| {
            let id = self
                .id_counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let new = ListItem {
                title: li.title().to_owned(),
                description: li.description().to_owned(),
                id,
            };
            list_items.insert(id, li);
            new
        })
        .collect()
    }

    pub fn find_list_item(&self, id: u64) -> Option<qpmu::ListItem> {
        self.list_items.lock().get(&id).cloned()
    }
}
