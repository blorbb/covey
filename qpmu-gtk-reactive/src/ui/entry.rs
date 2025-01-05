use az::SaturatingAs;
use gtk::prelude::EditableExt;
use qpmu::Input;
use reactive_graph::{signal::WriteSignal, traits::Set, wrappers::read::Signal};
use tap::Tap;

use crate::{gtk_utils::SetWidgetRef, reactive::WidgetRef};

#[tracing::instrument]
#[bon::builder]
pub fn entry(
    input: Signal<Input>,
    set_input: WriteSignal<Input>,
    #[builder(default)] entry_ref: WidgetRef<gtk::Entry>,
) -> gtk::Entry {
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
            entry.connect_changed(move |entry| {
                set_input.set(Input {
                    contents: entry.text().replace('\n', ""),
                    selection: {
                        let (a, b) = entry.selection_bounds().unwrap_or_else(|| {
                            let pos = entry.position();
                            (pos, pos)
                        });
                        (a.saturating_as(), b.saturating_as())
                    },
                });
            });
        })
}
