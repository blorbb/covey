use relm4::{
    gtk::{
        self,
        gdk::Key,
        prelude::{
            EditableExt as _, EntryExt as _, GtkWindowExt as _, ListBoxRowExt, WidgetExt as _,
        },
        EventControllerKey, ListBox, ListBoxRow,
    },
    Component, ComponentParts, ComponentSender, RelmContainerExt as _, RelmRemoveAllExt,
};

use crate::{
    model::{Launcher, LauncherCmd, LauncherMsg},
    plugins::{self, PluginActivationAction, PluginEvent},
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
impl Component for Launcher {
    type Input = LauncherMsg;
    type Output = ();
    type Init = ();
    type Widgets = LauncherWidgets;
    type Root = gtk::Window;
    type CommandOutput = LauncherCmd;

    fn init_root() -> Self::Root {
        gtk::Window::default()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Launcher::new();
        let window = root.clone();
        window.set_title(Some("qpmu"));
        window.set_default_width(WIDTH);
        window.set_default_height(-1);
        window.set_hide_on_close(true);
        window.set_decorated(false);
        window.set_vexpand(true);
        {
            let sender = sender.clone();
            window.connect_visible_notify(move |window| {
                if window.is_visible() {
                    sender.spawn_oneshot_command(|| {
                        // needs a short delay before focusing, otherwise
                        // it doesn't focus properly
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        LauncherCmd::Focus
                    })
                }
            });
        }

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

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        self.reset();
        // should be here to ensure it is always false when update_view is run.
        self.grab_full_focus = false;
        root.set_default_height(-1);

        match message {
            LauncherMsg::Query(query) => {
                self.set_query(query.clone());

                sender.spawn_oneshot_command(|| {
                    LauncherCmd::PluginEvent(
                        plugins::process_ui_event(plugins::UiEvent::InputChanged { query })
                            .unwrap(),
                    )
                });
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
            LauncherMsg::Activate => {
                if let Some(plugin) = self.results.get(self.selection).cloned() {
                    sender.spawn_oneshot_command(|| {
                        LauncherCmd::PluginEvent(
                            plugins::process_ui_event(plugins::UiEvent::Activate { item: plugin })
                                .unwrap(),
                        )
                    });
                }
            }
            LauncherMsg::Close => {
                root.close();
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let Self::Widgets {
            entry,
            scroller,
            results_list,
        } = widgets;

        if self.grab_full_focus {
            entry.grab_focus();
        } else {
            entry.grab_focus_without_selecting();
        }

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

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            LauncherCmd::PluginEvent(e) => match e {
                PluginEvent::SetList(vec) => sender.input(LauncherMsg::SetList(vec)),
                PluginEvent::Activate(vec) => {
                    use std::process::{Command, Stdio};
                    // TODO: remove unwraps
                    for ev in vec {
                        match ev {
                            PluginActivationAction::Close => sender.input(LauncherMsg::Close),
                            PluginActivationAction::RunCommand((cmd, args)) => {
                                Command::new(cmd)
                                    .args(args)
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .unwrap();
                            }
                            PluginActivationAction::RunCommandString(str) => {
                                Command::new("sh")
                                    .arg("-c")
                                    .arg(str)
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .spawn()
                                    .unwrap();
                            }
                            PluginActivationAction::Copy(string) => {
                                arboard::Clipboard::new().unwrap().set_text(string).unwrap();
                            }
                        }
                    }
                }
            },
            LauncherCmd::Focus => {
                self.grab_full_focus = true;
            }
        }
    }
}

impl Launcher {
    fn key_controller(&self, sender: ComponentSender<Self>) -> EventControllerKey {
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
