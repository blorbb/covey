use gtk::prelude::{Cast, GridExt};
use tap::Tap;

use super::state::State;
use crate::utils::{reactive::signal_diffed, stores::WidgetRef};

mod plugin_list;

pub fn settings(state: State) -> gtk::Window {
    let (plugin_selection, set_plugin_selection) = signal_diffed(None);
    let list_ref = WidgetRef::new();

    gtk::Window::builder()
        .css_classes(["settings-window"])
        .default_width(600)
        .default_height(400)
        .child(
            &settings_layout()
                .plugin_list(
                    &plugin_list::plugin_list()
                        .plugins(state.plugins)
                        .set_plugins(state.set_plugins)
                        .selection(plugin_selection)
                        .set_selection(set_plugin_selection)
                        .list_ref(list_ref)
                        .call(),
                )
                .call(),
        )
        .build()
}

#[bon::builder]
fn settings_layout(plugin_list: &gtk::Widget) -> gtk::Widget {
    gtk::Grid::builder()
        .css_classes(["settings-grid"])
        .build()
        .tap(|g| g.attach(plugin_list, 0, 0, 1, 1))
        .upcast()
}
