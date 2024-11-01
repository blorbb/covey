use std::{cell::RefCell, rc::Rc};

use gtk::{
    prelude::{ListBoxRowExt as _, WidgetExt as _},
    Label, ListBox, ListBoxRow,
};

use crate::plugins::ListItem;

#[derive(Clone)]
pub struct ResultList {
    results: Rc<RefCell<Vec<ListItem>>>,
    list: ListBox,
}

impl ResultList {
    pub fn new() -> Self {
        let list = ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Browse);

        Self {
            results: Rc::new(RefCell::new(vec![])),
            list,
        }
    }

    pub fn connect_row_activated(
        &self,
        f: impl Fn(&ListBox, &ListBoxRow, &Vec<ListItem>, &ListItem) + 'static,
    ) -> glib::SignalHandlerId {
        let results = Rc::clone(&self.results);

        self.list.connect_row_activated(move |list, row| {
            f(
                list,
                row,
                &results.borrow(),
                &results.borrow()[row.index() as usize],
            )
        })
    }

    pub fn get_item(&self, index: usize) -> Option<ListItem> {
        self.results.borrow().get(index).cloned()
    }

    pub fn list_box(&self) -> &ListBox {
        &self.list
    }

    pub fn clear(&self) {
        self.results.borrow_mut().clear();
        self.list.remove_all();
    }

    pub fn push(&self, item: ListItem) {
        let row = ListBoxRow::builder()
            .halign(gtk::Align::Fill)
            .hexpand(true)
            .build();
        row.add_css_class("list-item");
        let label = Label::builder()
            .halign(gtk::Align::Start)
            .wrap(true)
            .build();

        label.set_text(&item.title);
        row.set_child(Some(&label));
        self.list_box().append(&row);

        self.results.borrow_mut().push(item);
    }
}
