use std::sync::OnceLock;

use comette_tauri_types::{Icon, ListItemId};

use crate::ipc::frontend::{EventChannel, ListItem};

type Model = comette::Model<EventChannel>;

/// Must be initialised exactly once with [`AppState::init`].
pub struct AppState {
    inner: OnceLock<comette::lock::SharedMutex<Model>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    pub fn init(&self, model: comette::lock::SharedMutex<Model>) {
        if let Some(existing) = self.inner.get() {
            // this should never run in the real application, but it does
            // in hot module reload.
            tracing::warn!("setting model again");
            *existing.lock() = model.lock().clone();
        } else {
            self.inner
                .set(model)
                .unwrap_or_else(|_| tracing::warn!("already set up"));
        }
    }

    /// # Panics
    /// Panics if this has not been initialised yet.
    pub fn lock(&self) -> comette::lock::SharedMutexGuard<'_, Model> {
        self.inner
            .get()
            .expect("app state has not been set up")
            .lock()
    }

    pub fn register_list_items(
        &self,
        lis: impl ExactSizeIterator<Item = comette::ListItem>,
    ) -> Vec<ListItem> {
        lis.map(|li| {
            let title = li.title().to_owned();
            let description = li.description().to_owned();
            let icon: Option<Icon> = match li.icon() {
                Some(comette::Icon::Name(name)) => freedesktop_icons::lookup(&name)
                    .with_cache()
                    .with_size(48)
                    .find()
                    .map(|path| Icon::File { path }),
                Some(comette::Icon::Text(text)) => Some(Icon::Text { text }),
                None => None,
            };
            let id = ListItemId {
                local_id: li.id().local_id,
                plugin_name: li.id().plugin.name().to_owned(),
            };

            ListItem {
                title,
                description,
                icon,
                id: id.clone(),
            }
        })
        .collect()
    }

    pub fn find_list_item(&self, id: &ListItemId) -> Option<comette::ListItemId> {
        Some(comette::ListItemId {
            plugin: self
                .lock()
                .plugins()
                .iter()
                .find(|plugin| plugin.name() == id.plugin_name)?
                .clone(),
            local_id: id.local_id,
        })
    }
}
