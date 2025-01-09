use az::{CheckedAs as _, SaturatingAs as _};
use gtk::prelude::{BoxExt, FlowBoxChildExt as _, WidgetExt as _};
use qpmu::{Icon, ListItem, ListStyle};
use reactive_graph::{
    effect::Effect, graph::untrack, prelude::*, signal::WriteSignal, wrappers::read::Signal,
};
use tap::Tap;
use tracing::{debug, warn};

use crate::{
    clone_scoped,
    gtk_utils::SetWidgetRef as _,
    reactive::{EventHandler, WidgetRef},
    styles,
};

#[tracing::instrument(skip_all)]
#[bon::builder]
pub fn results_list(
    items: Signal<Vec<ListItem>>,
    style: impl Fn() -> ListStyle + 'static,
    /// Called after the UI is updated.
    after_list_update: Option<impl Fn() + Clone + 'static>,
    selection: Signal<usize>,
    set_selection: WriteSignal<usize>,
    on_activate: impl Fn() + 'static,
) -> gtk::FlowBox {
    // widgets //
    let list = WidgetRef::<gtk::FlowBox>::new();
    let selection_handler = EventHandler::<gtk::FlowBox>::new();

    // effects //
    let update_list_selection = move || {
        let target_row = list
            .widget()
            .child_at_index(selection.get().saturating_as::<i32>());
        match target_row {
            Some(target) => {
                selection_handler.suppress(|list| list.select_child(&target));
                // scroll to the target, but don't lose focus on the entry
                target.grab_focus();
            }
            None => {
                warn!("missing child at index {}", selection.get());
            }
        }

        if let Some(cb) = after_list_update.clone() {
            cb()
        }
    };

    // set items from list
    Effect::new(clone_scoped!(update_list_selection, move || {
        debug!("updating list widget");
        while let Some(child) = list.widget().last_child() {
            list.widget().remove(&child);
        }
        list.widget().set_css_classes(&["main-list"]);
        match style() {
            ListStyle::Rows => set_list_rows(&list.widget(), &items.get()),
            ListStyle::Grid => set_list_grid(&list.widget(), &items.get(), 5),
            ListStyle::GridWithColumns(columns) => {
                set_list_grid(&list.widget(), &items.get(), columns)
            }
        }
        // always needs to run as the selection gets cleared by resetting it
        untrack(|| update_list_selection());
    }));

    Effect::new(update_list_selection);

    gtk::FlowBox::builder()
        .css_classes(["main-list"])
        .selection_mode(gtk::SelectionMode::Browse)
        .row_spacing(0)
        .column_spacing(0)
        .widget_ref(list)
        .tap(|list| {
            selection_handler.set(
                list,
                list.connect_selected_children_changed({
                    move |flow_box| {
                        debug!("selected child changed at ui");
                        if let Some(child) = flow_box.selected_children().first() {
                            set_selection.set(
                                child
                                    .index()
                                    .checked_as::<usize>()
                                    .expect("index should never be negative"),
                            );
                        }
                    }
                }),
            );
        })
        .tap(|list| {
            // selection will happen first, so activating the current selection works
            // even if clicking on a row that isn't currently selected.
            list.connect_child_activated(move |_, _| on_activate());
        })
}

fn add_icon_to(icon: &Icon, to: &gtk::Box) {
    match icon {
        Icon::Name(name) => {
            let image = gtk::Image::from_icon_name(name);
            image.set_css_classes(&["list-item-icon", "list-item-icon-name"]);
            image.set_size_request(32, 32);
            image.set_icon_size(gtk::IconSize::Large);
            to.append(&image);
        }
        Icon::Text(text) => {
            let label = gtk::Label::builder()
                .label(text)
                .css_classes(["list-item-icon", "list-item-icon-text"])
                .build();
            to.append(&label);
        }
    };
}

fn set_list_rows(list: &gtk::FlowBox, children: &[ListItem]) {
    list.set_min_children_per_line(1);
    list.set_max_children_per_line(1);
    list.add_css_class("main-list-rows");
    styles::add_inline_css(list, "--qpmu-gtk-main-list-num-columns: 1;");
    for item in children {
        // item format:
        // icon | title
        //      | description

        list.append(&make_list_item(
            item.title(),
            item.description(),
            item.icon(),
            true,
        ));
    }
}

fn set_list_grid(list: &gtk::FlowBox, children: &[ListItem], columns: u32) {
    list.set_min_children_per_line(columns);
    list.set_max_children_per_line(columns);
    list.add_css_class("main-list-grid");
    styles::add_inline_css(
        list,
        &format!("--qpmu-gtk-main-list-num-columns: {columns};"),
    );

    for item in children {
        // item format:
        //    icon
        //   -------
        //    title
        // description

        list.append(&make_list_item(
            item.title(),
            item.description(),
            item.icon(),
            false,
        ));
    }

    if let Some(missing_space) = columns.checked_sub(children.len().saturating_as::<u32>()) {
        for _ in 0..missing_space {
            // make a bunch of stub items to fill in the space
            let child = make_list_item("", "", None, false);
            child.set_can_focus(false);
            child.set_can_target(false);
            child.set_focusable(false);
            child.set_opacity(0.0);
            child.set_sensitive(false);

            list.append(&child);
        }
    }
}

fn make_list_item(
    title: &str,
    description: &str,
    icon: Option<Icon>,
    is_rows: bool,
) -> gtk::FlowBoxChild {
    let hbox = gtk::Box::builder()
        .orientation(if is_rows {
            gtk::Orientation::Horizontal
        } else {
            gtk::Orientation::Vertical
        })
        .spacing(16)
        .css_classes(["list-item-hbox"])
        .build();

    if let Some(icon) = icon {
        add_icon_to(&icon, &hbox);
    }

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .css_classes(["list-item-vbox"])
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();

    let text_alignment = if is_rows {
        gtk::Align::Start
    } else {
        gtk::Align::Center
    };

    let title = gtk::Label::builder()
        .label(title)
        .halign(text_alignment)
        .css_classes(["list-item-title"])
        .wrap(true)
        .wrap_mode(gtk::pango::WrapMode::WordChar)
        .build();
    vbox.append(&title);

    if !description.is_empty() {
        let description = gtk::Label::builder()
            .label(description)
            .halign(text_alignment)
            .css_classes(["list-item-description"])
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .build();
        vbox.append(&description);
    }
    hbox.append(&vbox);

    gtk::FlowBoxChild::builder()
        .css_classes(["list-item"])
        .child(&hbox)
        .build()
}
