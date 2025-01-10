mod entry;
mod menu;
mod results_list;
mod state;

use color_eyre::eyre::eyre;
use gtk::prelude::{GtkWindowExt, WidgetExt};
use qpmu::{Input, ListItem, ListStyle};
use reactive_graph::{
    effect::Effect,
    signal::WriteSignal,
    traits::{Get as _, ReadValue as _, Set as _},
};
use state::State;
use tap::Tap;
use tracing::info;

use crate::utils::{clone::clone_scoped, stores::WidgetRef, widget_ext::WidgetSetRef as _};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

pub fn root() -> gtk::ApplicationWindow {
    // widgets //
    let window = WidgetRef::<gtk::ApplicationWindow>::new();
    let entry_ref = WidgetRef::<gtk::Entry>::new();

    let state = State::new(window);
    crate::styles::load_css();

    // effects //
    Effect::new(move || {
        state.model.read_value().lock().set_input(state.input.get());
    });
    Effect::new(move || {
        state
            .model
            .read_value()
            .lock()
            .set_list_selection(state.selection.get());
    });

    gtk::ApplicationWindow::builder()
        .title("qpmu")
        .hide_on_close(true)
        .decorated(false)
        .vexpand(true)
        .css_classes(["window"])
        .width_request(WIDTH)
        .height_request(-1)
        .child(&menu::menu(state.clone(), entry_ref))
        .widget_ref(window)
        .tap(|w| {
            w.connect_visible_notify(move |w| {
                if w.is_visible() {
                    w.present();
                    entry_ref.with(|entry| entry.grab_focus());
                }
            });
            w.add_controller(clone_scoped!(
                w,
                controller::leave().on_leave(move || w.close()).call()
            ));
            w.add_controller(
                controller::key()
                    .on_activate(move || state.model.read_value().lock().activate())
                    .on_alt_activate(move || state.model.read_value().lock().alt_activate())
                    .on_complete(move || state.model.read_value().lock().complete())
                    .on_hotkey_activate(move |hotkey| {
                        state.model.read_value().lock().hotkey_activate(hotkey)
                    })
                    .on_selection_move(move |delta| {
                        state.model.read_value().lock().move_list_selection(delta)
                    })
                    .on_close(clone_scoped!(w, move || w.close()))
                    .call(),
            )
        })
}

struct Frontend {
    on_close: Box<dyn FnMut() + Send + 'static>,
    /// First is title, second is body.
    on_notification: Box<dyn FnMut(String, String) + Send + 'static>,
    set_input: WriteSignal<Input>,
    set_items: WriteSignal<Vec<ListItem>>,
    set_style: WriteSignal<Option<ListStyle>>,
    set_selection: WriteSignal<usize>,
}

impl qpmu::Frontend for Frontend {
    fn close(&mut self) {
        (self.on_close)();
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
        let error_body = error
            .chain()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        tracing::error!("displaying error:\n{title}\n{error_body}");

        (self.on_notification)(title.to_owned(), error_body);
    }
}

mod controller {
    use gtk::gdk::{Key, ModifierType};

    #[bon::builder]
    pub(super) fn leave(on_leave: impl Fn() + 'static) -> gtk::EventControllerFocus {
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
    ) -> gtk::EventControllerKey {
        let key_events = gtk::EventControllerKey::builder()
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
