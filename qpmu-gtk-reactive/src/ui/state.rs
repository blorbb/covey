use qpmu::{config::Config, lock::SharedMutex, plugin::Plugin, Input, ListItem, ListStyle};
use reactive_graph::{
    computed::Memo,
    owner::{LocalStorage, StoredValue},
    signal::{signal, ReadSignal, WriteSignal},
    traits::Get,
};

use crate::utils::{reactive::signal_diffed, stores::WidgetRef};

/// This can implement `Copy` but doesn't as it is very big. Clone is cheap.
#[derive(Debug, Clone)]
pub struct State {
    pub input: Memo<Input>,
    pub set_input: WriteSignal<Input>,
    pub items: ReadSignal<Vec<ListItem>>,
    pub set_items: WriteSignal<Vec<ListItem>>,
    pub selection: Memo<usize>,
    pub set_selection: WriteSignal<usize>,
    pub style: Memo<Option<ListStyle>>,
    pub set_style: WriteSignal<Option<ListStyle>>,
    pub plugins: ReadSignal<Vec<Plugin>>,
    pub set_plugins: WriteSignal<Vec<Plugin>>,
    pub model: StoredValue<SharedMutex<qpmu::Model<super::Frontend>>, LocalStorage>,
    pub window: WidgetRef<gtk::ApplicationWindow>,
}

impl State {
    pub fn new(window: WidgetRef<gtk::ApplicationWindow>) -> State {
        let (input, set_input) = signal_diffed(Input::default());
        let (items, set_items) = signal(vec![]);
        let (style, set_style) = signal_diffed(None);
        let (selection, set_selection) = signal_diffed(0usize);

        let (plugins, set_plugins) =
            signal(Config::from_file().expect("failed to read config").load());
        let model = StoredValue::new_local(qpmu::Model::new(
            plugins.get(),
            super::Frontend {
                window,
                set_input,
                set_items,
                set_style,
                set_selection,
            },
        ));

        Self {
            input,
            set_input,
            items,
            set_items,
            selection,
            set_selection,
            style,
            set_style,
            plugins,
            set_plugins,
            model,
            window,
        }
    }
}
