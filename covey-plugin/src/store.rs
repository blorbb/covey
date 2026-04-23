use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use az::CheckedAs;

use crate::{List, Menu, rank::VisitId};

/// Map list (item) IDs to their command callbacks.
///
/// Cloning this creates another reference to the same map.
#[derive(Clone)]
pub(crate) struct CommandMap {
    lists: Arc<Mutex<VecDeque<ListCallbacks>>>,
    target_ids: Arc<AutoIncrementer>,
}

impl CommandMap {
    pub(crate) fn new() -> Self {
        Self {
            lists: Arc::new(Mutex::new(VecDeque::new())),
            target_ids: Arc::new(AutoIncrementer(AtomicU64::new(0))),
        }
    }

    /// Stores the result of a query, returning the response that should be
    /// sent to covey.
    pub(crate) fn store_query_result(&self, list: List) -> covey_proto::List {
        let new_ids = self.target_ids.fetch_many(list.total_len() as u64 + 1);
        let list_target_id = covey_proto::ActivationTarget(new_ids.start);
        let mut next_item_id = new_ids.start + 1;

        let mut item_callbacks = vec![];

        let sections: Vec<_> = list
            .sections
            .into_iter()
            .map(
                |crate::ListSection { title, items }| covey_proto::ListSection {
                    title,
                    items: items
                        .into_iter()
                        .map(
                            |crate::ListItem {
                                 title,
                                 description,
                                 icon,
                                 visit_id,
                                 callbacks,
                             }| {
                                let id = covey_proto::ActivationTarget(next_item_id);
                                next_item_id += 1;
                                let commands = callbacks.ids().cloned().collect();
                                item_callbacks.push((visit_id, callbacks));
                                covey_proto::ListItem {
                                    id,
                                    title,
                                    description,
                                    icon: icon.map(crate::into_proto::icon),
                                    commands,
                                }
                            },
                        )
                        .collect(),
                },
            )
            .collect();
        assert_eq!(next_item_id, new_ids.end);

        let list_command_ids = list.callbacks.ids().cloned().collect();

        let num_lists = {
            let mut lists = self.lists.lock().unwrap();
            lists.push_back(ListCallbacks {
                list_target_id,
                item_callbacks,
                list_callbacks: list.callbacks,
            });
            lists.len()
        };

        // Remove the older item after 1s. This should be more than enough time for the
        // frontend to have caught up to this list and made it impossible for the user
        // to try and activate an old item.
        if num_lists >= 2 {
            let lists = Arc::clone(&self.lists);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let mut lists = lists.lock().unwrap();
                lists.pop_front();
            });
        }

        covey_proto::List {
            id: list_target_id,
            commands: list_command_ids,
            sections,
        }
    }

    /// Finds the associated callbacks of an ID.
    ///
    /// This should never return [`None`] if the ID comes from an RPC call.
    /// However, implementation may change in the future which disposes of
    /// callbacks more frequently, and may have extremely rare edge cases where
    /// a callback is disposed but then activated.
    pub(crate) fn find_callback(
        &self,
        target_id: covey_proto::ActivationTarget,
        command_id: &covey_proto::CommandId,
    ) -> Option<(Option<VisitId>, ActivationFunction)> {
        let lists = self.lists.lock().unwrap();
        let (visit_id, callbacks) = lists
            .iter()
            .find_map(|commands| commands.target_callbacks(target_id))?;
        Some((
            visit_id.cloned(),
            callbacks.get_callback(command_id)?.clone(),
        ))
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
}
