use gtk::prelude::{Cast as _, EntryExt as _, GtkWindowExt as _, WidgetExt as _};
use reactive_graph::{
    traits::{Get, ReadUntracked as _, ReadValue as _},
    wrappers::read::Signal,
};

use super::{entry, results_list, state::State, HEIGHT_MAX};
use crate::utils::{
    stores::WidgetRef,
    widget_ext::{WidgetAddChild as _, WidgetSetRef as _},
};

#[tracing::instrument(skip_all)]
pub fn menu(state: State, entry_ref: WidgetRef<gtk::Entry>) -> gtk::Widget {
    let scroller_ref = WidgetRef::<gtk::ScrolledWindow>::new();

    menu_layout()
        .entry(
            &entry::entry()
                .input(state.input)
                .set_input(state.set_input)
                .entry_ref(entry_ref)
                .call(),
        )
        .list(
            &results_list::results_list()
                .items(state.items)
                .selection(state.selection)
                .set_selection(state.set_selection)
                .style(Signal::derive(move || {
                    state.style.get().unwrap_or(qpmu::ListStyle::Rows)
                }))
                .after_list_update(move || {
                    scroller_ref.with(|s| s.set_visible(!state.items.read_untracked().is_empty()));
                    state.window.with(|w| w.set_default_height(-1));
                    entry_ref.with(|e| e.grab_focus_without_selecting());
                })
                .on_activate(move || {
                    state.model.read_value().lock().activate();
                })
                .call(),
        )
        .scroller_ref(scroller_ref)
        .call()
}

#[bon::builder]
#[builder(on(_, into))]
pub fn menu_layout(
    entry: &gtk::Entry,
    list: &gtk::FlowBox,
    #[builder(default)] scroller_ref: WidgetRef<gtk::ScrolledWindow>,
) -> gtk::Widget {
    gtk::Box::builder()
        .css_classes(["main-box"])
        .orientation(gtk::Orientation::Vertical)
        .overflow(gtk::Overflow::Hidden)
        .child(entry)
        .child(
            &gtk::ScrolledWindow::builder()
                .css_classes(["main-scroller"])
                .min_content_height(0)
                .max_content_height(HEIGHT_MAX)
                .propagate_natural_height(true)
                .hscrollbar_policy(gtk::PolicyType::Never)
                .visible(false)
                .child(list)
                .widget_ref(scroller_ref),
        )
        .upcast()
}
