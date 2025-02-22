use parking_lot::Mutex;
use tokio::sync::RwLock;

use crate::{Plugin, store::ListItemStore};

pub(crate) struct ServerState<P> {
    pub(crate) plugin: RwLock<Option<P>>,
    pub(crate) list_item_store: Mutex<ListItemStore>,
}

impl<T: Plugin> ServerState<T> {
    pub(crate) fn new_empty() -> Self {
        Self {
            plugin: RwLock::new(None),
            list_item_store: Mutex::new(ListItemStore::new()),
        }
    }
}
