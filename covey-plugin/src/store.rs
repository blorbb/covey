use std::{
    collections::HashMap,
    iter,
    ops::Range,
    sync::atomic::{AtomicU64, Ordering},
};

use az::CheckedAs;

use crate::{Icon, List, ListStyle, list::CommandCallbacks, rank::VisitId};

/// Map list (item) IDs to their command callbacks.
pub(crate) struct CommandMap {
    /// TODO: clear old items
    lists: HashMap<covey_proto::ListId, ListCommands>,
    item_ids: AutoIncrementer,
    list_ids: AutoIncrementer,
}

impl CommandMap {
    pub(crate) fn new() -> Self {
        Self {
            lists: HashMap::new(),
            item_ids: AutoIncrementer(AtomicU64::new(0)),
            list_ids: AutoIncrementer(AtomicU64::new(0)),
        }
    }

    /// Stores the result of a query, returning the response that should be
    /// sent to covey.
    pub(crate) fn store_query_result(&mut self, list: List) -> covey_proto::List {
        let mut items = vec![];
        let mut item_callbacks = vec![];

        let new_item_ids = self.item_ids.fetch_many(list.items.len() as u64);
        for (id, item) in iter::zip(new_item_ids, list.items) {
            let crate::ListItem {
                title,
                description,
                icon,
                visit_id,
                callbacks,
            } = item;

            items.push(covey_proto::ListItem {
                id: covey_proto::ListItemId(id),
                title,
                description,
                icon: icon.map(Icon::into_proto),
                commands: callbacks.ids().cloned().collect(),
            });
            item_callbacks.push((visit_id, callbacks));
        }

        let list_id = covey_proto::ListId(self.list_ids.fetch_next());
        let list_command_ids = list.callbacks.ids().cloned().collect();

        self.lists.insert(
            list_id,
            ListCommands {
                first_item_id: items.first().map(|item| item.id),
                list_item_callbacks: item_callbacks,
                list_callbacks: list.callbacks,
            },
        );

        covey_proto::List {
            id: list_id,
            commands: list_command_ids,
            items,
            style: list.style.map(ListStyle::into_proto),
        }
    }

    /// Finds the associated callbacks of an ID.
    ///
    /// This should never return [`None`] if the ID comes from an RPC call.
    /// However, implementation may change in the future which disposes of
    /// callbacks more frequently, and may have extremely rare edge cases where
    /// a callback is disposed but then activated.
    pub(crate) fn item_callbacks(
        &self,
        id: covey_proto::ListItemId,
    ) -> Option<&(VisitId, CommandCallbacks)> {
        self.lists
            .values()
            .find_map(|commands| commands.item_callbacks(id))
    }

    pub(crate) fn list_callbacks(&self, id: covey_proto::ListId) -> Option<&CommandCallbacks> {
        self.lists.get(&id).map(|commands| &commands.list_callbacks)
    }
}

/// INVARIANTS:
/// - IDs of the list items are increasing and contiguous.
/// - `first_id` is None iff `list_item_callbacks` is empty.
struct ListCommands {
    first_item_id: Option<covey_proto::ListItemId>,
    list_item_callbacks: Vec<(VisitId, CommandCallbacks)>,
    list_callbacks: CommandCallbacks,
}

impl ListCommands {
    fn item_callbacks(&self, id: covey_proto::ListItemId) -> Option<&(VisitId, CommandCallbacks)> {
        let offset = id.0 - self.first_item_id?.0;
        self.list_item_callbacks.get(
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
    fn fetch_many(&self, count: u64) -> Range<u64> {
        let lower_bound = self.0.fetch_add(count, Ordering::Relaxed);
        let upper_bound = lower_bound + count;

        lower_bound..upper_bound
    }

    fn fetch_next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}
