use relm4::{
    gtk::{
        self,
        gdk::Key,
        prelude::{
            EditableExt as _, EntryExt as _, GtkWindowExt as _, ListBoxRowExt, WidgetExt as _,
        },
        EventControllerKey, ListBox, ListBoxRow,
    },
    ComponentParts, RelmContainerExt as _, RelmRemoveAllExt, SimpleComponent,
};

use crate::{
    model::{Launcher, LauncherMsg},
    plugins::Plugin,
};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

#[derive(Debug)]
pub struct LauncherWidgets {
    entry: gtk::Entry,
    scroller: gtk::ScrolledWindow,
    results_list: gtk::ListBox,
}

// not using the macro because the app has a lot of custom behaviour
// and the list of items is not static.
impl SimpleComponent for Launcher {
    type Input = LauncherMsg;
    type Output = ();
    type Init = Vec<Plugin>;
    type Widgets = LauncherWidgets;
    type Root = gtk::Window;

    fn init_root() -> Self::Root {
        gtk::Window::default()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Launcher::new(init);
        let window = root.clone();
        window.set_title(Some("qpmu"));
        window.set_default_width(WIDTH);
        window.set_default_height(HEIGHT_MAX);
        window.set_hide_on_close(true);
        window.set_decorated(false);
        window.set_vexpand(true);

        // main box layout
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        // main input line
        let entry = gtk::Entry::builder().placeholder_text("Search...").build();
        {
            let sender = sender.clone();
            entry.connect_changed(move |entry| {
                sender.input(LauncherMsg::Query(entry.text().to_string()));
            });
        }

        // results list
        let list_scroller = gtk::ScrolledWindow::builder()
            .min_content_height(0)
            .max_content_height(HEIGHT_MAX)
            .propagate_natural_height(true)
            .build();
        list_scroller.set_visible(!model.results.is_empty());

        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Browse)
            .build();
        list.select_row(list.row_at_index(model.selection as i32).as_ref());
        {
            let sender = sender.clone();
            list.connect_row_selected(move |_, row| {
                if let Some(row) = row {
                    sender.input(LauncherMsg::Select(row.index() as usize));
                }
            });
        }

        window.container_add(&vbox);
        window.add_controller(model.key_controller(sender));
        vbox.container_add(&entry);
        vbox.container_add(&list_scroller);
        list_scroller.container_add(&list);

        let widgets = Self::Widgets {
            entry,
            scroller: list_scroller,
            results_list: list,
        };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        self.reset();

        match message {
            LauncherMsg::Query(query) => {
                self.set_query(query.clone());
                _ = self
                    .ui_events()
                    .send(crate::plugins::UiEvent::InputChanged { query });
                let result = self.plugin_events().recv().unwrap();
                match result {
                    crate::plugins::PluginEvent::Activate(_) => todo!(),
                    crate::plugins::PluginEvent::SetList(list) => {
                        sender.input(LauncherMsg::SetList(list));
                    }
                }
            }
            LauncherMsg::SetList(list) => {
                self.set_results(list);
                // always mark as changed
                self.update_selection(|x| *x = 0);
            }
            LauncherMsg::Select(index) => {
                self.set_selection(index);
            }
            LauncherMsg::SelectDelta(delta) => {
                self.update_selection(|sel| *sel = sel.saturating_add_signed(delta));
            }
            LauncherMsg::Activate => todo!(),
            LauncherMsg::Close => todo!(),
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: relm4::ComponentSender<Self>) {
        let Self::Widgets {
            entry,
            scroller,
            results_list,
        } = widgets;

        entry.grab_focus_without_selecting();

        if *self.get_query() != entry.text() {
            entry.set_text(&self.get_query());
        }

        scroller.set_visible(!self.results.is_empty());

        if self.changed_results() {
            results_list.remove_all();
            for item in &self.results {
                results_list.append(
                    &ListBoxRow::builder()
                        .child(&gtk::Label::new(Some(&item.title)))
                        .build(),
                );
            }
        }

        if self.changed_selection() {
            results_list.select_row(
                results_list
                    .row_at_index(*self.get_selection() as i32)
                    .as_ref(),
            );
        }
    }
}

impl Launcher {
    fn key_controller(&self, sender: relm4::ComponentSender<Self>) -> EventControllerKey {
        let key_events = EventControllerKey::builder()
            .propagation_phase(gtk::PropagationPhase::Capture)
            .build();

        key_events.connect_key_pressed(move |_self, key, _keycode, _modifiers| {
            match key {
                Key::Escape => sender.input(LauncherMsg::Close),
                Key::Down => sender.input(LauncherMsg::SelectDelta(1)),
                Key::Up => sender.input(LauncherMsg::SelectDelta(-1)),
                Key::Return => sender.input(LauncherMsg::Activate),
                _ => return gtk::glib::Propagation::Proceed,
            }
            gtk::glib::Propagation::Stop
        });

        key_events
    }
}
