use gtk::{
    glib::SignalHandlerId,
    prelude::{IsA, ObjectExt as _, ObjectType},
};
use reactive_graph::{
    computed::Memo,
    owner::{LocalStorage, StoredValue},
    signal::{signal, ArcReadSignal, WriteSignal},
    traits::{GetValue, WithValue, WriteValue as _},
};

pub fn signal_diffed<T: Send + Sync + PartialEq + Clone + 'static>(
    value: T,
) -> (Memo<T>, WriteSignal<T>) {
    let (read, write) = signal(value);
    let read_memoized = Memo::from(ArcReadSignal::from(read));
    (read_memoized, write)
}

#[derive(Debug)]
pub struct WidgetRef<T>(StoredValue<Option<T>, LocalStorage>);

impl<T> Clone for WidgetRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WidgetRef<T> {}

impl<T: 'static> Default for WidgetRef<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> WidgetRef<T> {
    pub fn new() -> Self {
        Self(StoredValue::new_local(None))
    }

    pub fn set(&self, value: T) {
        self.0.write_value().get_or_insert(value);
    }
}

impl<T: Clone + 'static> WidgetRef<T> {
    /// Runs the function if the widget is currently filled.
    ///
    /// Ignores the output.
    pub fn with<U>(&self, f: impl FnOnce(T) -> U) {
        self.0.get_value().map(f);
    }

    /// Gets the widget, panicking if it hasn't been set.
    pub fn widget(&self) -> T {
        self.0
            .get_value()
            .expect("widget reference should have been set")
    }
}

#[derive(Debug)]
pub struct EventHandler<T>(StoredValue<Option<(T, SignalHandlerId)>, LocalStorage>);

impl<T> Clone for EventHandler<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for EventHandler<T> {}

impl<T: IsA<gtk::Widget>> Default for EventHandler<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ObjectType> EventHandler<T> {
    pub fn new() -> Self {
        Self(StoredValue::new_local(None))
    }

    pub fn set(&self, widget: &T, handler: SignalHandlerId) {
        self.0.write_value().get_or_insert((widget.clone(), handler));
    }

    pub fn suppress(&self, f: impl Fn(&T)) {
        self.0.with_value(|opt| {
            if let Some((widget, handler)) = opt {
                widget.block_signal(handler);
                f(widget);
                widget.unblock_signal(handler);
            };
        })
    }
}
