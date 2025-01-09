mod entry;
mod menu;
mod results_list;

use color_eyre::eyre::eyre;
use gtk::prelude::{ApplicationExt, EntryExt, GtkWindowExt, WidgetExt};
use qpmu::{config::Config, Input, ListItem, ListStyle};
use reactive_graph::{
    effect::Effect,
    owner::StoredValue,
    prelude::*,
    signal::{signal, WriteSignal},
};
use tap::Tap;
use tracing::info;

use crate::utils::{
    clone::clone_scoped, reactive::signal_diffed, stores::WidgetRef, widget_ext::WidgetSetRef as _,
};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

pub fn root() -> gtk::ApplicationWindow {
    // widgets //
    let window = WidgetRef::<gtk::ApplicationWindow>::new();
    let entry = WidgetRef::<gtk::Entry>::new();
    let scroller = WidgetRef::<gtk::ScrolledWindow>::new();

    // signals //
    let (input, set_input) = signal_diffed(Input::default());
    let (items, set_items) = signal(vec![]);
    let (style, set_style) = signal_diffed(None);
    let (selection, set_selection) = signal_diffed(0usize);

    let plugins = Config::from_file().expect("failed to read config").load();
    let model = StoredValue::new_local(qpmu::Model::new(
        plugins,
        Frontend {
            window,
            set_input,
            set_items,
            set_style,
            set_selection,
        },
    ));
    crate::styles::load_css();

    // effects //
    Effect::new(move || {
        model.read_value().lock().set_input(input.get());
    });
    Effect::new(move || {
        model
            .read_value()
            .lock()
            .set_list_selection(selection.get());
    });

    gtk::ApplicationWindow::builder()
        .title("qpmu")
        .hide_on_close(true)
        .decorated(false)
        .vexpand(true)
        .css_classes(["window"])
        .width_request(WIDTH)
        .height_request(-1)
        .child(
            &menu::menu()
                .entry(
                    &entry::entry()
                        .input(input.into())
                        .set_input(set_input)
                        .entry_ref(entry)
                        .call(),
                )
                .list(
                    &results_list::results_list()
                        .items(items.into())
                        .selection(selection.into())
                        .set_selection(set_selection)
                        .style(move || style.get().unwrap_or(qpmu::ListStyle::Rows))
                        .after_list_update(move || {
                            scroller.with(|s| s.set_visible(!items.read_untracked().is_empty()));
                            window.with(|w| w.set_default_height(-1));
                            entry.with(|e| e.grab_focus_without_selecting());
                        })
                        .on_activate(move || {
                            model.read_value().lock().activate();
                        })
                        .call(),
                )
                .scroller_ref(scroller)
                .call(),
        )
        .widget_ref(window)
        .tap(|w| {
            w.connect_visible_notify(move |w| {
                if w.is_visible() {
                    w.present();
                    entry.with(|entry| entry.grab_focus());
                }
            });
        })
        .tap(|w| {
            w.add_controller(clone_scoped!(
                w,
                controller::leave().on_leave(move || w.close()).call()
            ));
        })
        .tap(|w| {
            w.add_controller(
                controller::key()
                    .on_activate(move || model.read_value().lock().activate())
                    .on_alt_activate(move || model.read_value().lock().alt_activate())
                    .on_complete(move || model.read_value().lock().complete())
                    .on_hotkey_activate(move |hotkey| {
                        model.read_value().lock().hotkey_activate(hotkey)
                    })
                    .on_selection_move(move |delta| {
                        model.read_value().lock().move_list_selection(delta)
                    })
                    .on_close(clone_scoped!(w, move || w.close()))
                    .call(),
            )
        })
}

struct Frontend {
    window: WidgetRef<gtk::ApplicationWindow>,
    set_input: WriteSignal<Input>,
    set_items: WriteSignal<Vec<ListItem>>,
    set_style: WriteSignal<Option<ListStyle>>,
    set_selection: WriteSignal<usize>,
}

impl qpmu::Frontend for Frontend {
    fn close(&mut self) {
        self.window.widget().close();
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

        self.set_list_selection(0);
        self.set_style.set(list.style());
        self.set_items.set(list.items().to_vec());
    }

    fn set_list_selection(&mut self, index: usize) {
        self.set_selection.set(index);
    }

    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report) {
        let notif = gtk::gio::Notification::new(title);
        let error_body = error
            .chain()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        tracing::error!("displaying error:\n{title}\n{error_body}");

        notif.set_body(Some(&error_body));
        self.window
            .widget()
            .application()
            .expect("application should be alive")
            .send_notification(None, &notif);
    }
}

mod controller {
    use gtk::{
        gdk::{Key, ModifierType},
        EventControllerFocus, EventControllerKey,
    };

    #[bon::builder]
    pub(super) fn leave(on_leave: impl Fn() + 'static) -> EventControllerFocus {
        let leave_controller = gtk::EventControllerFocus::new();
        leave_controller.connect_leave(move |_| on_leave());
        leave_controller
    }

    #[bon::builder]
    pub(super) fn key(
        on_close: impl Fn() + 'static,
        on_selection_move: impl Fn(isize) + 'static,
        on_activate: impl Fn() + 'static,
        on_alt_activate: impl Fn() + 'static,
        on_complete: impl Fn() + 'static,
        on_hotkey_activate: impl Fn(qpmu::hotkey::Hotkey) + 'static,
    ) -> EventControllerKey {
        let key_events = EventControllerKey::builder()
            .propagation_phase(gtk::PropagationPhase::Capture)
            .build();

        key_events.connect_key_pressed(move |_self, key, _keycode, modifiers| {
            match key {
                Key::Escape => on_close(),
                Key::Down => on_selection_move(1),
                Key::Up => on_selection_move(-1),
                Key::Return if modifiers.contains(ModifierType::ALT_MASK) => on_alt_activate(),
                Key::Return if modifiers.is_empty() => on_activate(),
                Key::Tab if modifiers.is_empty() => on_complete(),
                // try run a hotkey
                other => {
                    if let Some(hotkey) = crate::hotkey::to_qpmu_hotkey(other, modifiers) {
                        on_hotkey_activate(hotkey)
                    }
                    return gtk::glib::Propagation::Proceed;
                }
            }
            gtk::glib::Propagation::Stop
        });

        key_events
    }
}
