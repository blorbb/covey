use az::SaturatingAs;
use gtk::prelude::EditableExt;
use qpmu::Input;
use reactive_graph::{
    effect::Effect,
    signal::WriteSignal,
    traits::{Get, Set},
    wrappers::read::Signal,
};
use tap::Tap;

use crate::{
    gtk_utils::SetWidgetRef,
    reactive::{EventHandler, WidgetRef},
};

#[tracing::instrument]
#[bon::builder]
pub fn entry(
    input: Signal<Input>,
    set_input: WriteSignal<Input>,
    #[builder(default)] entry_ref: WidgetRef<gtk::Entry>,
) -> gtk::Entry {
    let change_handler = EventHandler::<gtk::Entry>::new();

    Effect::new(move || {
        let contents = &input.get().contents;
        let selection = input.get().selection;
        let selection = (i32::from(selection.0), i32::from(selection.1));
        eprintln!("{contents} at {selection:?}");
        change_handler.suppress(|e| {
            if &e.text() != contents {
                e.set_text(&input.get().contents);
            }
            if e.selection_bounds() != Some(selection) {
                e.select_region(selection.0, selection.1);
            }
        });
    });

    gtk::Entry::builder()
        .placeholder_text("Search...")
        .css_classes(["main-entry"])
        .primary_icon_name("search")
        .secondary_icon_name("settings")
        .secondary_icon_activatable(true)
        // must guarantee that there are no new lines
        .truncate_multiline(true)
        .widget_ref(entry_ref)
        .tap(|entry| {
            change_handler.set(
                entry,
                entry.connect_changed(move |entry| {
                    set_input.set(Input {
                        contents: entry.text().replace('\n', ""),
                        selection: {
                            eprintln!("HERE");
                            let (a, b) = entry.selection_bounds().unwrap_or_else(|| {
                                let pos = entry.position();
                                (pos, pos)
                            });
                            (a.saturating_as(), b.saturating_as())
                        },
                    });
                }),
            );
        })
}
