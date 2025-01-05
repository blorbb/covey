use az::{CheckedAs as _, SaturatingAs as _};
use gtk::prelude::{BoxExt, Cast, FlowBoxChildExt as _, WidgetExt as _};
use qpmu::{Icon, ListItem, ListStyle};
use reactive_graph::{
    effect::Effect,
    traits::{Get, Read},
    wrappers::read::Signal,
};
use tracing::{debug, warn};

use crate::styles;

#[tracing::instrument(skip_all)]
#[bon::builder]
pub fn results_list(
    items: Signal<Vec<ListItem>>,
    style: Signal<ListStyle>,
    /// Called after the list layout is updated.
    after_list_update: Option<impl Fn() + 'static>,
    selection: Signal<usize>,
    set_selection: impl Fn(usize) + 'static,
    on_activate: impl Fn() + 'static,
) -> gtk::FlowBox {
    // widgets //
    let list = gtk::FlowBox::builder()
        .css_classes(["main-list"])
        .selection_mode(gtk::SelectionMode::Browse)
        .row_spacing(0)
        .column_spacing(0)
        .build();

    // effects //
    // set items
    Effect::new({
        let list = list.clone();

        move || {
            // let span = tracing::span!(tracing::Level::WARN, "in effect");
            // let _a = span.enter();
            // warn!("effect 0");
            while let Some(child) = list.last_child() {
                list.remove(&child);
            }
            list.set_css_classes(&["main-list"]);
            // warn!("effect 1");
            // warn!("effect 2");
            match style.get() {
                ListStyle::Rows => set_list_rows(&list, &items.read()),
                ListStyle::Grid => set_list_grid(&list, &items.read(), 5),
                ListStyle::GridWithColumns(columns) => set_list_grid(&list, &items.read(), columns),
            }
            // warn!("effect 3");
            if let Some(cb) = &after_list_update {
                cb()
            }
        }
    });

    // set selection
    Effect::new({
        let list = list.clone();
        move || {
            let target_row = list.child_at_index(selection.get().saturating_as::<i32>());
            match target_row {
                Some(target) => {
                    list.select_child(&target);
                    // scroll to the target, but don't lose focus on the entry
                    target.grab_focus();
                }
                None => {
                    warn!("missing child at index {}", selection.get());
                }
            }
        }
    });

    // handlers //
    list.connect_selected_children_changed({
        move |flow_box| {
            debug!("selected child changed at ui");
            if let Some(child) = flow_box.selected_children().first() {
                set_selection(
                    child
                        .index()
                        .checked_as::<usize>()
                        .expect("index should never be negative"),
                );
            }
        }
    });
    // selection will happen first, so activating the current selection works
    // even if clicking on a row that isn't currently selected.
    list.connect_child_activated(move |_, _| on_activate());

    list
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
    styles::add_inline_css(list.upcast_ref(), "--qpmu-gtk-main-list-num-columns: 1;");
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
        list.upcast_ref(),
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
