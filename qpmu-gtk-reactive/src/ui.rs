mod entry;
mod menu;
mod results_list;

use color_eyre::eyre::eyre;
use gtk::prelude::{EntryExt, GtkWindowExt, WidgetExt};
use qpmu::{config::Config, Input, ListItem, ListStyle};
use reactive_graph::{
    effect::Effect,
    owner::StoredValue,
    signal::{signal, Trigger, WriteSignal},
    traits::{Get, GetUntracked, GetValue, Notify, ReadUntracked, ReadValue, Set, Track},
    wrappers::read::Signal,
};
use tracing::info;

use crate::reactive::{signal_diffed, WidgetRef};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

pub fn root() -> gtk::ApplicationWindow {
    let close_trigger = Trigger::new();

    // signals //
    let (input, set_input) = signal_diffed(Input::default());
    let (items, set_items) = signal(vec![]);
    let (style, set_style) = signal_diffed(None);
    let (selection, set_selection) = signal_diffed(0usize);

    let plugins = Config::from_file().expect("failed to read config").load();
    let model = StoredValue::new_local(qpmu::Model::new(
        plugins,
        Frontend {
            close_trigger,
            set_input,
            set_items,
            set_style,
            set_selection,
        },
    ));
    crate::styles::load_css();

    // effects //
    Effect::new(move || {
        model.get_value().lock().set_input(input.get());
    });
    Effect::new(move || {
        model.get_value().lock().set_list_selection(selection.get());
    });

    // widgets //
    let window = StoredValue::new_local(
        gtk::ApplicationWindow::builder()
            .title("qpmu")
            .hide_on_close(true)
            .decorated(false)
            .vexpand(true)
            .css_classes(["window"])
            .width_request(WIDTH)
            .height_request(-1)
            .build(),
    );

    let entry = StoredValue::new_local(
        entry::entry()
            .input(input.into())
            .set_input(set_input)
            .call(),
    );

    let scroller_ref = WidgetRef::<gtk::ScrolledWindow>::new();

    let list = results_list::results_list()
        .items(items.into())
        .selection(selection.into())
        .style(Signal::derive(move || {
            style.get().unwrap_or(qpmu::ListStyle::Rows)
        }))
        .after_list_update(move || {
            scroller_ref
                .get()
                .map(|r| r.set_visible(!items.read_untracked().is_empty()));
            window.read_value().set_default_height(-1)
        })
        .set_selection(move |new| {
            if selection.get_untracked() != new {
                set_selection.set(new);
            };
        })
        .on_activate(move || {
            model.get_value().lock().activate();
        })
        .call();

    let menu = menu::menu()
        .entry(&entry.get_value())
        .list(&list)
        .scroller_ref(scroller_ref)
        .call();

    window.read_value().set_child(Some(&menu));
    window.read_value().connect_visible_notify(move |window| {
        if window.is_visible() {
            window.present();
            entry.get_value().grab_focus();
        }
    });
    {
        let leave_controller = gtk::EventControllerFocus::new();
        leave_controller.connect_leave(move |_| window.read_value().close());
        window.read_value().add_controller(leave_controller);
    }
    Effect::new(move || {
        selection.track();
        entry.get_value().grab_focus_without_selecting();
    });

    window.get_value()
}

struct Frontend {
    close_trigger: Trigger,
    set_input: WriteSignal<Input>,
    set_items: WriteSignal<Vec<ListItem>>,
    set_style: WriteSignal<Option<ListStyle>>,
    set_selection: WriteSignal<usize>,
}

impl qpmu::Frontend for Frontend {
    fn close(&mut self) {
        self.close_trigger.notify();
    }

    fn copy(&mut self, str: String) {
        info!("copying string {str:?}");

        let result = arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(str));
        if let Err(e) = result {
            self.display_error("Failed to set clipboard", eyre!(e));
        }
    }

    fn set_input(&mut self, input: &Input) {
        info!("setting input to {input:?}");
        self.set_input.set(input.clone())
    }

    fn set_list(&mut self, list: &qpmu::ResultList) {
        info!("setting list to {} elements", list.len());

        // for some reason style MUST be set before items
        // otherwise a deadlock can occur
        self.set_style.set(list.style());
        self.set_items.set(list.items().to_vec());
        self.set_list_selection(0);
    }

    fn set_list_selection(&mut self, index: usize) {
        self.set_selection.set(index);
    }

    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report) {
        todo!("{}: {}", title, error)
    }
}
