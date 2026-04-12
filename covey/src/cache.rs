use std::{
    collections::HashMap,
    fmt,
    hash::Hash,
    sync::{Arc, Mutex, mpsc},
};

type ValueComputation<K, V> = Box<dyn FnMut(&K) -> V + Send>;

enum Task<K, V> {
    ChangeComputation(ValueComputation<K, V>),
    ComputeKey(K),
}

pub(crate) struct Cache<K, V> {
    /// A [`None`] value means it's loading. No value means nothing is cached.
    map: Arc<Mutex<HashMap<K, Option<V>>>>,
    tasks: mpsc::Sender<Task<K, V>>,
}

impl<K: Hash + Eq + Send + 'static, V: Send + 'static> Cache<K, V> {
    pub(crate) fn new(f: impl FnMut(&K) -> V + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        let map = Arc::new(Mutex::new(HashMap::new()));
        let thread_map = Arc::clone(&map);

        let this = Self { map, tasks: tx };

        // For now, what we cache is blocking but very cheap, so just one task
        // is enough. Consider a thread pool if we choose to do more
        // expensive things.
        std::thread::spawn(move || {
            let mut f: Box<dyn FnMut(&K) -> V + Send + 'static> = Box::new(f);

            while let Ok(x) = rx.recv() {
                match x {
                    Task::ChangeComputation(g) => f = g,
                    Task::ComputeKey(k) => {
                        let v = f(&k);
                        match thread_map.lock().unwrap().get_mut(&k) {
                            Some(old_v) => {
                                assert!(old_v.is_none(), "cache raced between two initialisers");
                                *old_v = Some(v);
                            }
                            // The None that's meant to be there if it's loading got cleared.
                            // Not inserting this new value so that the cache doesn't suddenly
                            // contain things after being cleared.
                            None => {}
                        }
                    }
                }
            }
        });

        this
    }

    pub(crate) fn get_or_insert_with(&self, k: K) -> Option<V>
    where
        K: fmt::Debug + Clone,
        V: Clone,
    {
        self.map
            .lock()
            .unwrap()
            .entry(k)
            .or_insert_with_key(|key| {
                tracing::debug!(?key, "recomputing uncached key");
                self.tasks
                    .send(Task::ComputeKey(key.clone()))
                    .expect("cache receiver should always be alive");
                None
            })
            .clone()
    }

    pub(crate) fn clear(&self, new_f: impl FnMut(&K) -> V + Send + 'static) {
        *self.map.lock().unwrap() = HashMap::new();
        self.tasks
            .send(Task::ChangeComputation(Box::new(new_f)))
            .expect("cache receiver should always be alive");
    }
}
