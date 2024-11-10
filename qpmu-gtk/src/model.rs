use color_eyre::eyre::Result;
use qpmu::{
    plugin::{event::PluginEvent, Plugin},
    Input, Model,
};
use relm4::{
    gtk::{
        self,
        prelude::{EditableExt, GtkWindowExt, ObjectExt, WidgetExt as _},
    },
    RelmContainerExt as _, RelmRemoveAllExt as _,
};
use tracing::info;

use crate::ui::LauncherWidgets;

#[derive(Debug)]
pub struct Launcher {
    pub model: Model,
}

impl Launcher {
    pub fn new(plugins: &'static [Plugin]) -> Self {
        Self {
            model: Model::new(plugins),
        }
    }
}

#[derive(Debug)]
pub enum LauncherMsg {
    /// Set the query to a string
    SetInput(Input),
    /// Set the results list
    PluginEvent(Result<PluginEvent>),
    /// Selects a specific index of the results list
    Select(usize),
    /// Change the selection index by a certain amount
    SelectDelta(isize),
    /// Activate the current selected item
    Activate,
    /// Perform (tab) completion on the current selected item
    Complete,
    /// Close (hide) the window
    Close,
    /// Open and focus the entry
    Focus,
}

pub struct Frontend<'a> {
    pub widgets: &'a mut LauncherWidgets,
    pub root: &'a gtk::Window,
}

impl<'a> qpmu::Frontend for Frontend<'a> {
    fn close(&mut self) {
        self.root.close();
    }

    fn copy(&mut self, str: String) {
        arboard::Clipboard::new().unwrap().set_text(str).unwrap();
    }

    fn set_input(&mut self, input: Input) {
        self.widgets
            .entry
            .block_signal(&self.widgets.entry_change_handler);
        self.widgets.entry.set_text(&input.contents);
        self.widgets
            .entry
            .select_region(i32::from(input.selection.0), i32::from(input.selection.1));
        self.widgets
            .entry
            .unblock_signal(&self.widgets.entry_change_handler);
    }

    fn set_list(&mut self, list: &qpmu::ResultList) {
        info!("setting list");

        let results_list = &self.widgets.results_list;

        self.widgets.scroller.set_visible(!list.is_empty());
        results_list.remove_all();
        // recreate list of results
        for item in list.list() {
            // item format:
            // icon | title
            //      | description

            let hbox = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .css_classes(["list-item-hbox"])
                .spacing(16)
                .build();
            if let Some(icon_name) = item.icon() {
                let icon = gtk::Image::from_icon_name(&icon_name);
                icon.set_css_classes(&["list-item-icon"]);
                icon.set_size_request(32, 32);
                icon.set_icon_size(gtk::IconSize::Large);
                hbox.container_add(&icon);
            }

            let vbox = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .css_classes(["list-item-vbox"])
                .spacing(4)
                .build();
            let title = gtk::Label::builder()
                .label(item.title())
                .halign(gtk::Align::Start)
                .css_classes(["list-item-title"])
                .wrap(true)
                .build();
            vbox.container_add(&title);

            if !item.description().is_empty() {
                let description = gtk::Label::builder()
                    .label(item.description())
                    .halign(gtk::Align::Start)
                    .css_classes(["list-item-description"])
                    .wrap(true)
                    .build();
                vbox.container_add(&description);
            }
            hbox.container_add(&vbox);

            results_list.container_add(
                &gtk::ListBoxRow::builder()
                    .css_classes(["list-item"])
                    .child(&hbox)
                    .build(),
            );
        }

        self.set_list_selection(list.selection());
        self.root.set_default_height(-1);
    }

    fn set_list_selection(&mut self, index: usize) {
        self.widgets.results_list.select_row(
            self.widgets
                .results_list
                .row_at_index(index as i32)
                .as_ref(),
        );
    }
}
