use relm4::{
    gtk::{
        self,
        gdk::Key,
        prelude::{
            EditableExt as _, EntryExt as _, GtkWindowExt as _, ListBoxRowExt, ObjectExt as _,
            WidgetExt as _,
        },
        EventControllerKey, ListBox,
    },
    Component, ComponentParts, ComponentSender, RelmContainerExt as _, RelmRemoveAllExt,
};

use crate::{
    model::{Launcher, LauncherCmd, LauncherMsg},
    plugins::{self, ListItem, PluginActivationAction, PluginEvent, UiEvent, WithPlugin},
    styles::load_css,
};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

#[derive(Debug)]
pub struct LauncherWidgets {
    entry: gtk::Entry,
    scroller: gtk::ScrolledWindow,
    results_list: gtk::ListBox,
    #[allow(dead_code)]
    root: gtk::Window,
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
        load_css();
        let model = Launcher::new();
        let window = root.clone();
        window.set_title(Some("qpmu"));
        window.set_default_width(WIDTH);
        window.set_default_height(-1);
        window.set_hide_on_close(true);
        window.set_decorated(false);
        window.set_vexpand(true);
        window.set_css_classes(&["window"]);

        window.connect_visible_notify({
            let sender = sender.clone();
            move |window| {
                if window.is_visible() {
                    sender.spawn_oneshot_command(|| {
                        // needs a short delay before focusing, otherwise
                        // it doesn't focus properly
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        LauncherCmd::Focus
                    })
                }
            }
        });

        {
            // close on focus leave
            let leave_controller = gtk::EventControllerFocus::new();
            leave_controller.connect_leave({
                let window = window.clone();
                move |_| window.close()
            });
            window.add_controller(leave_controller);
        }

        // main box layout
        let vbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();

        // main input line
        let entry = gtk::Entry::builder()
            .placeholder_text("Search...")
            .css_classes(["main-entry"])
            // must guarantee that there are no new lines
            .truncate_multiline(true)
            .build();

        entry.connect_changed({
            let sender = sender.clone();
            move |entry| {
                sender.input(LauncherMsg::Query(entry.text().replace('\n', "")));
            }
        });

        // results list
        let list_scroller = gtk::ScrolledWindow::builder()
            .min_content_height(0)
            .max_content_height(HEIGHT_MAX)
            .propagate_natural_height(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .css_classes(["main-scroller"])
            .build();
        list_scroller.set_visible(!model.results.is_empty());

        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Browse)
            .css_classes(["main-list"])
            .build();
        list.select_row(list.row_at_index(model.selection as i32).as_ref());

        list.connect_row_selected({
            let sender = sender.clone();
            move |_, row| {
                if let Some(row) = row {
                    sender.input(LauncherMsg::Select(row.index() as usize));
                }
            }
        });
        // selection will happen first, so activating the current selection works
        // even if clicking on a row that isn't currently selected.
        list.connect_row_activated({
            let sender = sender.clone();
            move |_, _| sender.input(LauncherMsg::Activate)
        });

        window.container_add(&vbox);
        window.add_controller(model.key_controller(sender));
        vbox.container_add(&entry);
        vbox.container_add(&list_scroller);
        list_scroller.container_add(&list);

        let widgets = Self::Widgets {
            entry,
            scroller: list_scroller,
            results_list: list,
            root: window,
        };
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        self.reset();
        // should be here to ensure it is always false when update_view is run.
        self.grab_full_focus = false;
        // FIXME: after tab completion the window becomes full height for some reason.
        root.set_default_height(-1);

        match message {
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
            LauncherMsg::Query(query) => {
                self.perform(sender, UiEvent::InputChanged { query });
            }
            LauncherMsg::Activate => {
                self.perform_with_selection(sender, |_query, item| UiEvent::Activate { item });
            }
            LauncherMsg::Complete => {
                self.perform_with_selection(sender, |query, item| UiEvent::Complete {
                    query,
                    item,
                });
            }
            LauncherMsg::Close => {
                root.close();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        root.set_default_height(-1);

        match message {
            LauncherCmd::PluginEvent(index, e) => match e {
                PluginEvent::SetList(vec) => {
                    if self.should_perform(index) {
                        sender.input(LauncherMsg::SetList(vec))
                    }
                }
                PluginEvent::Activate(plugin, vec) => {
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
                            PluginActivationAction::RunShell(str) => {
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
                            PluginActivationAction::SetInputLine(input_line) => {
                                self.set_line_by_plugin(WithPlugin::new(plugin, input_line));
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

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        let Self::Widgets {
            entry,
            scroller,
            results_list,
            root,
        } = widgets;

        if self.grab_full_focus {
            entry.grab_focus();
        } else {
            entry.grab_focus_without_selecting();
        }

        // set the entry field
        if let Some(WithPlugin { plugin, mut value }) = self.line_set_by_plugin() {
            let prefix_len = i32::try_from(plugin.prefix().chars().count())
                .expect("plugin prefix is way too long");
            value.query.insert_str(0, plugin.prefix());
            entry.set_text(&value.query);

            entry.select_region(
                prefix_len + i32::from(value.range.lower_bound),
                prefix_len + i32::from(value.range.upper_bound),
            );
        }

        scroller.set_visible(!self.results.is_empty());

        if self.changed_results() {
            results_list.remove_all();
            // recreate list of results
            for item in &self.results {
                // item format:
                // icon | title
                //      | description

                let hbox = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .css_classes(["list-item-hbox"])
                    .spacing(16)
                    .build();
                if let Some(icon_name) = &item.icon {
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
                    .label(&item.title)
                    .halign(gtk::Align::Start)
                    .css_classes(["list-item-title"])
                    .wrap(true)
                    .build();
                vbox.container_add(&title);

                if !item.description.is_empty() {
                    let description = gtk::Label::builder()
                        .label(&item.description)
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
        }

        if self.changed_selection() {
            results_list.select_row(
                results_list
                    .row_at_index(*self.get_selection() as i32)
                    .as_ref(),
            );
        }

        root.set_default_height(-1);
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
                Key::Tab => sender.input(LauncherMsg::Complete),
                _ => return gtk::glib::Propagation::Proceed,
            }
            gtk::glib::Propagation::Stop
        });

        key_events
    }

    fn perform(&mut self, sender: ComponentSender<Self>, e: UiEvent) {
        let i = self.next_action();
        sender.oneshot_command(async move {
            LauncherCmd::PluginEvent(i, plugins::process_ui_event(e).await.unwrap())
        });
    }

    fn perform_with_selection(
        &mut self,
        sender: ComponentSender<Self>,
        f: impl FnOnce(String, ListItem) -> UiEvent + Send + 'static,
    ) {
        if let Some(item) = self.results.get(self.selection).cloned() {
            let i = self.next_action();
            let query = self.query.clone();
            sender.oneshot_command(async move {
                LauncherCmd::PluginEvent(
                    i,
                    plugins::process_ui_event(f(query, item)).await.unwrap(),
                )
            });
        }
    }
}
