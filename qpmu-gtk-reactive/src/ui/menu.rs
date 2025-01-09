use gtk::prelude::Cast as _;

use super::HEIGHT_MAX;
use crate::utils::{
    stores::WidgetRef,
    widget_ext::{WidgetAddChild as _, WidgetSetRef as _},
};

#[tracing::instrument]
#[bon::builder]
pub fn menu(
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
