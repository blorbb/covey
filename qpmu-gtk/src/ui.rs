use color_eyre::eyre::eyre;
use qpmu::{config::Config, Icon, Input, ListItem, ListStyle};
use relm4::{
    gtk::{
        self,
        gdk::{Key, ModifierType},
        gio::Notification,
        glib::SignalHandlerId,
        prelude::*,
        EventControllerKey, FlowBox, Widget,
    },
    prelude::ComponentParts,
    Component, ComponentController, ComponentSender, RelmContainerExt, RelmRemoveAllExt as _,
    RelmWidgetExt,
};
use tracing::{error, info, instrument, warn};

use crate::{
    model::{Launcher, LauncherMsg},
    settings::{self, Settings, SettingsMsg},
    styles::load_css,
    tray_icon,
};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

#[derive(Debug)]
pub struct LauncherWidgets {
    pub entry: gtk::Entry,
    pub scroller: gtk::ScrolledWindow,
    pub results_list: gtk::FlowBox,
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

        let plugins = Config::load_plugins().expect("failed to load plugins");
        let model = Launcher::new(
            plugins.clone(),
            Settings::builder()
                .launch(plugins)
                .forward(sender.input_sender(), settings::output_transform),
        );
        load_css();

        tray_icon::create_tray_icon(sender.clone()).expect("failed to load tray icon");

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
            .secondary_icon_name("settings")
            .secondary_icon_activatable(true)
            // must guarantee that there are no new lines
            .truncate_multiline(true)
            .build();

        entry.connect_icon_press({
            let sender = sender.clone();
            move |_, icon_position| {
                if icon_position == gtk::EntryIconPosition::Secondary {
                    sender.input(LauncherMsg::OpenSettings)
                }
            }
        });

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

        let list = FlowBox::builder()
            .selection_mode(gtk::SelectionMode::Browse)
            .css_classes(["main-list"])
            .row_spacing(0)
            .column_spacing(0)
            .build();

        list.connect_selected_children_changed({
            let sender = sender.clone();
            move |flow_box| {
                if let Some(child) = flow_box.selected_children().first() {
                    sender.input(LauncherMsg::Select(child.index() as usize));
                }
            }
        });
        // selection will happen first, so activating the current selection works
        // even if clicking on a row that isn't currently selected.
        list.connect_child_activated({
            let sender = sender.clone();
            move |_, _| sender.input(LauncherMsg::Activate)
        });

        window.container_add(&vbox);
        window.add_controller(key_controller(sender.clone()));
        vbox.container_add(&entry);
        vbox.container_add(&list_scroller);
        list_scroller.container_add(&list);

        // set css property for the user CSS to receive
        window.inline_css(&format!("--qpmu-gtk-window-width: {WIDTH}px;"));

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
            LauncherMsg::Shutdown => {
                root.application().unwrap().quit();
            }
            LauncherMsg::OpenSettings => {
                info!("opened settings");
                self.settings.emit(SettingsMsg::Show)
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

    fn shutdown(&mut self, _: &mut Self::Widgets, _: relm4::Sender<Self::Output>) {
        info!("application shutting down");
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
        info!("setting list to {} items", list.items().len());

        let results_list = &self.widgets.results_list;

        self.widgets.scroller.set_visible(!list.is_empty());
        results_list.remove_all();
        results_list.set_css_classes(&["main-list"]);

        // recreate list of results
        match list.style() {
            Some(ListStyle::Rows) => Self::set_list_rows(results_list, list.items()),
            Some(ListStyle::Grid) => Self::set_list_grid(results_list, list.items(), 5),
            Some(ListStyle::GridWithColumns(columns)) => {
                Self::set_list_grid(results_list, list.items(), columns)
            }
            None => {
                // TODO: select based on configuration
                Self::set_list_rows(results_list, list.items());
            }
        }

        self.set_list_selection(list.selection());
        self.root.set_default_height(-1);
    }

    fn set_list_selection(&mut self, index: usize) {
        info!("set list selection to index {index}");
        let target_row = self.widgets.results_list.child_at_index(index as i32);

        if let Some(target_row) = target_row {
            self.widgets.results_list.select_child(&target_row);
            // scroll to the target, but don't lose focus on the entry
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

impl Frontend<'_> {
    fn add_icon_to(icon: &Icon, to: &impl RelmContainerExt<Child = Widget>) {
        match icon {
            Icon::Name(name) => {
                let image = gtk::Image::from_icon_name(&name);
                image.set_css_classes(&["list-item-icon", "list-item-icon-name"]);
                image.set_size_request(32, 32);
                image.set_icon_size(gtk::IconSize::Large);
                to.container_add(&image);
            }
            Icon::Text(text) => {
                let label = gtk::Label::builder()
                    .label(text)
                    .css_classes(["list-item-icon", "list-item-icon-text"])
                    .build();
                to.container_add(&label);
            }
        };
    }

    fn set_list_rows(list: &FlowBox, children: &[ListItem]) {
        list.set_min_children_per_line(1);
        list.set_max_children_per_line(1);
        list.add_css_class("main-list-rows");
        list.inline_css("--qpmu-gtk-main-list-num-columns: 1;");
        for item in children {
            // item format:
            // icon | title
            //      | description

            list.append(&Self::make_list_item(
                item.title(),
                item.description(),
                item.icon(),
                true,
            ));
        }
    }

    fn set_list_grid(list: &FlowBox, children: &[ListItem], columns: u32) {
        list.set_min_children_per_line(columns);
        list.set_max_children_per_line(columns);
        list.add_css_class("main-list-grid");
        list.inline_css(&format!("--qpmu-gtk-main-list-num-columns: {columns};"));

        for item in children {
            // item format:
            //    icon
            //   -------
            //    title
            // description

            list.append(&Self::make_list_item(
                item.title(),
                item.description(),
                item.icon(),
                false,
            ));
        }

        if let Some(missing_space) = columns.checked_sub(children.len() as u32) {
            for _ in 0..missing_space {
                // make a bunch of stub items to fill in the space
                let child = Self::make_list_item("", "", None, false);
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
            Self::add_icon_to(&icon, &hbox);
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
        vbox.container_add(&title);

        if !description.is_empty() {
            let description = gtk::Label::builder()
                .label(description)
                .halign(text_alignment)
                .css_classes(["list-item-description"])
                .wrap(true)
                .wrap_mode(gtk::pango::WrapMode::WordChar)
                .build();
            vbox.container_add(&description);
        }
        hbox.container_add(&vbox);

        gtk::FlowBoxChild::builder()
            .css_classes(["list-item"])
            .child(&hbox)
            .build()
    }
}
