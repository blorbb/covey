use az::{CheckedAs, SaturatingAs};
use gtk::prelude::{ButtonExt, Cast, ListBoxRowExt, WidgetExt};
use qpmu::plugin::Plugin;
use reactive_graph::{
    effect::Effect,
    graph::untrack,
    signal::WriteSignal,
    traits::{Get, GetUntracked, Read, Set as _},
    wrappers::read::Signal,
};
use tap::Tap;
use tracing::warn;

use crate::utils::{
    stores::WidgetRef,
    widget_ext::{WidgetAddChild, WidgetRemoveAll, WidgetSetRef},
};

#[bon::builder]
pub fn plugin_list(
    #[builder(into)] plugins: Signal<Vec<Plugin>>,
    #[builder(into)] set_plugins: WriteSignal<Vec<Plugin>>,
    #[builder(into)] selection: Signal<Option<usize>>,
    #[builder(into)] set_selection: WriteSignal<Option<usize>>,
    #[builder(default)] list_ref: WidgetRef<gtk::ListBox>,
) -> gtk::Widget {
    let up_button = WidgetRef::<gtk::Button>::new();
    let down_button = WidgetRef::<gtk::Button>::new();

    let update_selection = move || {
        if let Some(selection) = selection.get() {
            list_ref.widget().select_row(
                list_ref
                    .widget()
                    .row_at_index(selection.saturating_as::<i32>())
                    .as_ref(),
            );
        }
    };

    Effect::new(move || {
        list_ref.widget().remove_all();
        for plugin in plugins.get() {
            list_ref.widget().append(&plugin_list_item(plugin));
        }
        untrack(update_selection);
    });
    Effect::new(update_selection);

    // set the up/down buttons disabled/enabled when the selection is at bounds
    Effect::new(move || {
        up_button
            .widget()
            .set_sensitive(selection.get().is_some_and(|s| s != 0));
        down_button.widget().set_sensitive(
            selection
                .get()
                .is_some_and(|s| s != plugins.read().len() - 1),
        );
    });

    gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .child(
            &gtk::ListBox::builder()
                .css_classes(["plugin-list"])
                .selection_mode(gtk::SelectionMode::Single)
                .widget_ref(list_ref)
                .tap(|lb| {
                    lb.connect_row_selected(move |_, row| {
                        // TODO: selection is being set to None before
                        // update_selection can run, staying as None.
                        set_selection.set(row.map(|row| {
                            row.index()
                                .checked_as::<usize>()
                                .expect("index should never be negative")
                        }));
                    });
                }),
        )
        .child(
            &gtk::Box::builder()
                .css_classes(["plugin-list-control"])
                .child(
                    &gtk::Button::builder()
                        .child(&gtk::Image::from_icon_name("up"))
                        .widget_ref(up_button)
                        .tap(|btn| {
                            btn.connect_clicked(move |_| {
                                let Some(selection) = selection.get_untracked() else {
                                    warn!("tried to move plugin up without any selection");
                                    return;
                                };
                                set_plugins.set(move_plugin(
                                    plugins.get_untracked(),
                                    selection,
                                    move |x| set_selection.set(Some(x)),
                                    Direction::Up,
                                ))
                            });
                        }),
                )
                .child(
                    &gtk::Button::builder()
                        .child(&gtk::Image::from_icon_name("down"))
                        .widget_ref(down_button)
                        .tap(|btn| {
                            btn.connect_clicked(move |_| {
                                eprintln!("????");
                                let Some(selection) = selection.get_untracked() else {
                                    warn!("tried to move plugin down without any selection");
                                    return;
                                };
                                set_plugins.set(move_plugin(
                                    plugins.get_untracked(),
                                    selection,
                                    move |x| set_selection.set(Some(x)),
                                    Direction::Down,
                                ))
                            });
                        }),
                ),
        )
        .upcast()
}

fn plugin_list_item(plugin: Plugin) -> gtk::ListBoxRow {
    gtk::ListBoxRow::builder()
        .css_classes(["plugin-list-item"])
        .child(&gtk::Label::builder().label(plugin.name()).build())
        .build()
}

enum Direction {
    Up,
    Down,
}

/// # Panics
/// Panics if `selection` is out of bounds.
fn move_plugin(
    mut plugins: Vec<Plugin>,
    selection: usize,
    set_selection: impl FnOnce(usize),
    direction: Direction,
) -> Vec<Plugin> {
    let delta = match direction {
        Direction::Up => -1,
        Direction::Down => 1,
    };

    if selection >= plugins.len() {
        panic!("selection out of bounds");
    }

    let new_index = usize::min(
        selection.saturating_add_signed(delta),
        plugins.len().saturating_sub(1),
    );

    // shift the plugin to the new location
    if selection < new_index {
        // e.g. existing is 'a'
        // [a, b, c] -> [b, c, a]
        // rotate left
        plugins[selection..=new_index].rotate_left(1);
    } else {
        // e.g. existing is 'c'
        // [a, b, c] -> [c, a, b]
        // rotate right
        plugins[new_index..=selection].rotate_right(1);
    }

    warn!("selection is now {new_index}");

    set_selection(new_index);
    plugins
}
