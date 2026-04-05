use std::{
    collections::HashMap,
    iter,
    ops::Range,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use az::CheckedAs;

use crate::{List, Menu, rank::VisitId};

/// Map list (item) IDs to their command callbacks.
pub(crate) struct CommandMap {
    /// TODO: clear old items
    lists: Vec<ListCallbacks>,
    target_ids: AutoIncrementer,
}

impl CommandMap {
    pub(crate) fn new() -> Self {
        Self {
            lists: vec![],
            target_ids: AutoIncrementer(AtomicU64::new(0)),
        }
    }

    /// Stores the result of a query, returning the response that should be
    /// sent to covey.
    pub(crate) fn store_query_result(&mut self, list: List) -> covey_proto::List {
        let list_target_id = covey_proto::ActivationTarget(self.target_ids.fetch_next());

        let mut items = vec![];
        let mut item_callbacks = vec![];

        let new_item_ids = self.target_ids.fetch_many(list.items.len() as u64);
        for (id, item) in iter::zip(new_item_ids, list.items) {
            let crate::ListItem {
                title,
                description,
                icon,
                visit_id,
                callbacks,
            } = item;

            items.push(covey_proto::ListItem {
                id: covey_proto::ActivationTarget(id),
                title,
                description,
                icon: icon.map(crate::into_proto::icon),
                commands: callbacks.ids().cloned().collect(),
            });
            item_callbacks.push((visit_id, callbacks));
        }

        let list_command_ids = list.callbacks.ids().cloned().collect();

        self.lists.push(ListCallbacks {
            list_target_id,
            item_callbacks,
            list_callbacks: list.callbacks,
        });

        covey_proto::List {
            id: list_target_id,
            commands: list_command_ids,
            items,
            style: list.style.map(crate::into_proto::list_style),
        }
    }

    /// Finds the associated callbacks of an ID.
    ///
    /// This should never return [`None`] if the ID comes from an RPC call.
    /// However, implementation may change in the future which disposes of
    /// callbacks more frequently, and may have extremely rare edge cases where
    /// a callback is disposed but then activated.
    pub(crate) fn target_callbacks(
        &self,
        id: covey_proto::ActivationTarget,
    ) -> Option<(Option<&VisitId>, &TargetCallbacks)> {
        self.lists
            .iter()
            .find_map(|commands| commands.target_callbacks(id))
    }
}

/// INVARIANTS:
/// - IDs of the list items are increasing and contiguous.
/// - `list_target_id` is the target id of the list itself.
/// - The IDs of the list items start at `list_target_id + 1`.
struct ListCallbacks {
    list_target_id: covey_proto::ActivationTarget,
    list_callbacks: TargetCallbacks,
    item_callbacks: Vec<(VisitId, TargetCallbacks)>,
}

impl ListCallbacks {
    fn target_callbacks(
        &self,
        target: covey_proto::ActivationTarget,
    ) -> Option<(Option<&VisitId>, &TargetCallbacks)> {
        if target.0 == self.list_target_id.0 {
            Some((None, &self.list_callbacks))
        } else {
            let offset = target.0.checked_sub(self.list_target_id.0 + 1)?;
            let (id, callbacks) = self.item_callbacks.get(
                offset
                    .checked_as::<usize>()
                    .expect("there should not be way too many callbacks stored (over u32::MAX)"),
            )?;

            Some((Some(id), callbacks))
        }
    }
}

// ActivationFunction needs Send + Sync for blocking plugins to work.
// The callback exposed by `add_callback` takes an input of `&Menu` instead of
// `Menu` to force the user to complete everything they want before returning
// from the callback. Might want to add something that happens after the
// callback returns.
type DynFuture<T> = Pin<Box<dyn Future<Output = T>>>;
type ActivationFunction = Arc<dyn Fn(Menu) -> DynFuture<()> + Send + Sync>;

#[derive(Clone)]
pub(crate) struct TargetCallbacks {
    commands: HashMap<covey_proto::CommandId, ActivationFunction>,
}

impl TargetCallbacks {
    pub(crate) fn new() -> Self {
        Self {
            commands: HashMap::default(),
        }
    }

    pub(crate) fn add_callback(
        &mut self,
        command_id: covey_proto::CommandId,
        callback: impl AsyncFn(&Menu) -> crate::Result<()> + Send + Sync + 'static,
    ) {
        let callback = Arc::new(callback);
        self.commands.insert(
            command_id,
            Arc::new(move |menu| {
                let callback = Arc::clone(&callback);
                Box::pin(async move {
                    if let Err(e) = callback(&menu).await {
                        menu.display_error(format!("{e:#}"));
                    }
                })
            }),
        );
    }

    pub(crate) fn get_callback(
        &self,
        command_id: &covey_proto::CommandId,
    ) -> Option<&ActivationFunction> {
        self.commands.get(command_id)
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = &covey_proto::CommandId> {
        self.commands.keys()
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
