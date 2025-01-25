use std::{
    collections::VecDeque,
    iter,
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

use az::CheckedAs;
use parking_lot::Mutex;

use crate::{list::ListItemCallbacks, proto, Icon, List, ListItem, ListStyle};

static STORE: Mutex<ListItemStore> = Mutex::new(ListItemStore::new());

/// Stores the result of a query, returning the response that should be
/// sent to covey.
pub(crate) fn store_query_result(list: List) -> proto::QueryResponse {
    STORE.lock().store_query_result(list)
}

/// Finds the associated callbacks of an ID.
///
/// This should never return [`None`] if the ID comes from an RPC call.
/// However, implementation may change in the future which disposes of
/// callbacks more frequently, and may have extremely rare edge cases where
/// a callback is disposed but then activated.
pub(crate) fn fetch_callbacks_of(list_item_id: u64) -> Option<ListItemCallbacks> {
    STORE.lock().fetch_callbacks_of(list_item_id)
}

/// Store to map list item IDs to their callbacks.
///
/// Only one should be constructed.
struct ListItemStore {
    /// A deque of all queries which have been made.
    ///
    /// Old queries are only dropped when an activation is performed, as
    /// this is the only way to guarantee that old list items will not be
    /// activated. Long latency from sending the query results to the front
    /// end could result in old queries being activated.
    ///
    /// This disposal implementation may change in the future.
    ///
    /// The IDs stored in the deque should be increasing and contiguous.
    queries: VecDeque<QueryListItemStore>,
    ids: AutoIncrementer,
}

impl ListItemStore {
    const fn new() -> Self {
        Self {
            queries: VecDeque::new(),
            ids: AutoIncrementer(AtomicU64::new(0)),
        }
    }

    fn store_query_result(&mut self, list: List) -> proto::QueryResponse {
        // Don't store an empty result
        if list.items.is_empty() {
            return proto::QueryResponse {
                items: vec![],
                list_style: list.style.map(ListStyle::into_proto),
            };
        }

        let (items, callbacks) = split_item_vec(&self.ids, list.items);

        self.queries.push_back(QueryListItemStore {
            callbacks,
            first_id: items.first().expect("list should be non empty").id,
        });

        return proto::QueryResponse {
            items,
            list_style: list.style.map(ListStyle::into_proto),
        };

        fn split_item_vec(
            ids: &AutoIncrementer,
            vec: Vec<ListItem>,
        ) -> (Vec<proto::ListItem>, Vec<ListItemCallbacks>) {
            let new_ids = ids.fetch_many(vec.len() as u64);

            let mut items = vec![];
            let mut callbacks = vec![];

            for (id, item) in iter::zip(new_ids, vec) {
                items.push(proto::ListItem {
                    id,
                    title: item.title,
                    description: item.description,
                    icon: item.icon.map(Icon::into_proto),
                    available_commands: item.commands.ids().cloned().collect(),
                });
                callbacks.push(item.commands);
            }

            (items, callbacks)
        }
    }

    fn fetch_callbacks_of(&mut self, id: u64) -> Option<ListItemCallbacks> {
        // linear search is good enough
        let (found_index, found_callback) =
            self.queries.iter().enumerate().find_map(|(i, query)| {
                query
                    .callback_of_id(id)
                    .map(|callbacks| (i, callbacks.clone()))
            })?;

        // Remove old queries.
        // Don't include the current query, since the action could be
        // nothing and the same query could have another list item activated.
        self.queries.drain(..found_index);

        Some(found_callback)
    }
}

/// INVARIANTS:
/// - IDs of the list items are increasing and contiguous.
/// - Number of items stored is non-zero.
struct QueryListItemStore {
    callbacks: Vec<ListItemCallbacks>,
    first_id: u64,
}

impl QueryListItemStore {
    pub fn callback_of_id(&self, id: u64) -> Option<&ListItemCallbacks> {
        let offset = id - self.first_id;
        self.callbacks.get(
            offset
                .checked_as::<usize>()
                .expect("there should not be way too many callbacks stored (over u32::MAX)"),
        )
    }
}

/// Unique ID generator by incrementing numbers.
struct AutoIncrementer(AtomicU64);

impl AutoIncrementer {
    /// Retrieves several auto-incremented IDs at once.
    pub fn fetch_many(&self, count: u64) -> Range<u64> {
        let lower_bound = self.0.fetch_add(count, Ordering::Relaxed);
        let upper_bound = lower_bound + count;

        lower_bound..upper_bound
    }
}
