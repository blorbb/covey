use color_eyre::eyre::eyre;
use qpmu::{config::Config, Input};
use relm4::{
    gtk::{
        self,
        gdk::{Key, ModifierType},
        gio::Notification,
        glib::SignalHandlerId,
        prelude::*,
        EventControllerKey, ListBox,
    },
    prelude::ComponentParts,
    Component, ComponentSender, RelmContainerExt as _, RelmRemoveAllExt as _,
};
use tracing::{error, info, instrument, warn};

use crate::{
    model::{Launcher, LauncherMsg},
    styles::load_css,
};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

#[derive(Debug)]
pub struct LauncherWidgets {
    pub entry: gtk::Entry,
    pub scroller: gtk::ScrolledWindow,
    pub results_list: gtk::ListBox,
    pub entry_change_handler: SignalHandlerId,
}

// not using the macro because the app has a lot of custom behaviour
// and the list of items is not static.
// DO NOT MAKE IT ASYNC! weird window size stuff happens.
impl Component for Launcher {
    type Input = LauncherMsg;
    type Output = ();
    type Init = ();
    type Widgets = LauncherWidgets;
    type Root = gtk::Window;
    type CommandOutput = LauncherMsg;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("qpmu")
            .hide_on_close(true)
            .decorated(false)
            .vexpand(true)
            .css_classes(["window"])
            .width_request(WIDTH)
            .height_request(-1)
            .build()
    }

    #[instrument(skip_all)]
    fn init(
        _init: Self::Init,
        window: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        info!("initialising new application instance");

        let model = Launcher::new(Config::load_plugins().expect("failed to load plugins"));
        load_css();

        window.connect_visible_notify({
            let sender = sender.clone();
            move |window| {
                if window.is_visible() {
                    info!("is visible");
                    sender.spawn_oneshot_command(|| {
                        // needs a short delay before focusing, otherwise
                        // it doesn't focus properly
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        LauncherMsg::Focus
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
            .css_classes(["main-box"])
            .overflow(gtk::Overflow::Hidden)
            .build();

        // main input line
        let entry = gtk::Entry::builder()
            .placeholder_text("Search...")
            .css_classes(["main-entry"])
            .primary_icon_name("search")
            // must guarantee that there are no new lines
            .truncate_multiline(true)
            .build();

        let entry_change_handler = entry.connect_changed({
            let sender = sender.clone();
            move |entry| {
                sender.input(LauncherMsg::SetInput(Input {
                    contents: entry.text().replace('\n', ""),
                    selection: entry
                        .selection_bounds()
                        .map_or((0, 0), |(a, b)| (a as u16, b as u16)),
                }));
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
        list_scroller.set_visible(false);

        let list = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Browse)
            .css_classes(["main-list"])
            .build();
        list.select_row(list.row_at_index(0).as_ref());

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
        window.add_controller(key_controller(sender.clone()));
        vbox.container_add(&entry);
        vbox.container_add(&list_scroller);
        list_scroller.container_add(&list);

        let widgets = Self::Widgets {
            entry,
            scroller: list_scroller,
            results_list: list,
            entry_change_handler,
        };
        ComponentParts { model, widgets }
    }

    #[instrument(skip_all)]
    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            LauncherMsg::SetInput(input) => {
                let fut = self.model.set_input(input);
                sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) })
            }
            LauncherMsg::PluginEvent(plugin_event) => {
                // TODO: fix this hack
                let reset_input = self
                    .model
                    .handle_event(plugin_event, &mut Frontend { widgets, root });
                if reset_input {
                    warn!("resetting input");
                    sender.input(LauncherMsg::SetInput(self.model.input().clone()))
                }
            }
            LauncherMsg::Select(index) => {
                self.model
                    .set_list_selection(index, &mut Frontend { widgets, root });
            }
            LauncherMsg::SelectDelta(delta) => {
                self.model
                    .move_list_selection(delta, &mut Frontend { widgets, root });
            }
            LauncherMsg::Activate => {
                let fut = self.model.activate();
                if let Some(fut) = fut {
                    sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) });
                }
            }
            LauncherMsg::AltActivate => {
                let fut = self.model.alt_activate();
                if let Some(fut) = fut {
                    sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) });
                }
            }
            LauncherMsg::Hotkey(hotkey) => {
                let fut = self.model.hotkey_activate(hotkey);
                if let Some(fut) = fut {
                    sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) });
                }
            }
            LauncherMsg::Complete => {
                let fut = self.model.complete();
                if let Some(fut) = fut {
                    sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) });
                }
            }
            LauncherMsg::Focus => {
                root.present();
                widgets.entry.grab_focus();
            }
            LauncherMsg::Close => {
                root.close();
            }
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.update_with_view(widgets, message, sender, root)
    }
}

fn key_controller(sender: ComponentSender<Launcher>) -> EventControllerKey {
    let key_events = EventControllerKey::builder()
        .propagation_phase(gtk::PropagationPhase::Capture)
        .build();

    key_events.connect_key_pressed(move |_self, key, _keycode, modifiers| {
        match key {
            Key::Escape => sender.input(LauncherMsg::Close),
            Key::Down => sender.input(LauncherMsg::SelectDelta(1)),
            Key::Up => sender.input(LauncherMsg::SelectDelta(-1)),
            Key::Return if modifiers.contains(ModifierType::ALT_MASK) => {
                sender.input(LauncherMsg::AltActivate)
            }
            Key::Return if modifiers.is_empty() => sender.input(LauncherMsg::Activate),
            Key::Tab if modifiers.is_empty() => sender.input(LauncherMsg::Complete),
            // try run a hotkey
            other => {
                if let Some(hotkey) = crate::hotkey::to_qpmu_hotkey(other, modifiers) {
                    sender.input(LauncherMsg::Hotkey(hotkey));
                }
                return gtk::glib::Propagation::Proceed;
            }
        }
        gtk::glib::Propagation::Stop
    });

    key_events
}

pub struct Frontend<'a> {
    pub widgets: &'a mut LauncherWidgets,
    pub root: &'a gtk::Window,
}

impl<'a> qpmu::Frontend for Frontend<'a> {
    fn close(&mut self) {
        info!("closing window");

        self.root.close();
    }

    fn copy(&mut self, str: String) {
        info!("copying string {str:?}");

        let result = arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(str));
        if let Err(e) = result {
            self.display_error("Failed to set clipboard", eyre!(e));
        }
    }

    fn set_input(&mut self, input: Input) {
        info!(
            "set input to {:?} with selection {}..{}",
            input.contents, input.selection.0, input.selection.1
        );

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
        info!("setting list to {} items", list.list().len());

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
                .halign(gtk::Align::Fill)
                .hexpand(true)
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
        info!("set list selection to index {index}");
        let target_row = self.widgets.results_list.row_at_index(index as i32);

        self.widgets.results_list.select_row(target_row.as_ref());

        // scroll to the target, but don't lose focus on the entry
        if let Some(target_row) = target_row {
            target_row.grab_focus();
            self.widgets.entry.grab_focus_without_selecting();
        }
    }

    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report) {
        error!("displaying error {title}");

        let notif = Notification::new(title);
        let error_body = error
            .chain()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        notif.set_body(Some(&error_body));

        self.root
            .application()
            .expect("missing application instance")
            .send_notification(None, &notif);
    }
}
