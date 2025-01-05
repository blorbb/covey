use gtk::prelude::{BoxExt as _, Cast as _, WidgetExt as _};

use super::HEIGHT_MAX;
use crate::reactive::WidgetRef;

#[tracing::instrument]
#[bon::builder]
pub fn menu(
    entry: &gtk::Entry,
    list: &gtk::FlowBox,
    #[builder(default)] scroller_ref: WidgetRef<gtk::ScrolledWindow>,
) -> gtk::Widget {
    // main box layout
    let vbox = gtk::Box::builder()
        .css_classes(["main-box"])
        .orientation(gtk::Orientation::Vertical)
        .overflow(gtk::Overflow::Hidden)
        .build();

    // results list
    let list_scroller = gtk::ScrolledWindow::builder()
        .css_classes(["main-scroller"])
        .min_content_height(0)
        .max_content_height(HEIGHT_MAX)
        .propagate_natural_height(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    list_scroller.set_visible(false);
    list_scroller.set_child(Some(list));

    scroller_ref.set(list_scroller.clone());

    vbox.append(entry);
    vbox.append(&list_scroller);
    vbox.upcast()
}
