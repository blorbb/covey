use reactive_graph::{
    computed::Memo,
    signal::{signal, ArcReadSignal, WriteSignal},
};

pub fn signal_diffed<T: Send + Sync + PartialEq + Clone + 'static>(
    value: T,
) -> (Memo<T>, WriteSignal<T>) {
    let (read, write) = signal(value);
    let read_memoized = Memo::from(ArcReadSignal::from(read));
    (read_memoized, write)
}
