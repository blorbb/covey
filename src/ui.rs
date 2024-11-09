use qpmu::{plugin::Plugin, Input};
use relm4::{
    gtk::{
        self,
        gdk::Key,
        prelude::{EditableExt as _, GtkWindowExt as _, ListBoxRowExt, WidgetExt as _},
        EventControllerKey, ListBox,
    },
    prelude::{AsyncComponent, AsyncComponentParts},
    AsyncComponentSender, RelmContainerExt as _,
};
use tracing::{info, instrument};

use crate::{
    model::{Frontend, Launcher, LauncherMsg},
    styles::load_css,
};

const WIDTH: i32 = 800;
const HEIGHT_MAX: i32 = 600;

#[derive(Debug)]
pub struct LauncherWidgets {
    pub entry: gtk::Entry,
    pub scroller: gtk::ScrolledWindow,
    pub results_list: gtk::ListBox,
}

// not using the macro because the app has a lot of custom behaviour
// and the list of items is not static.
impl AsyncComponent for Launcher {
    type Input = LauncherMsg;
    type Output = ();
    type Init = &'static [Plugin];
    type Widgets = LauncherWidgets;
    type Root = gtk::Window;
    type CommandOutput = LauncherMsg;

    fn init_root() -> Self::Root {
        gtk::Window::default()
    }

    #[instrument(skip_all)]
    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        info!("initialising new application instance");
        // FIXME: weird window size ?????

        let model = Launcher::new(init);
        load_css();

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
                sender.input(LauncherMsg::SetInput(Input::new(
                    entry.text().replace('\n', ""),
                    entry
                        .selection_bounds()
                        .map_or((0, 0), |(a, b)| (a as u16, b as u16)),
                )));
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
        vbox.container_add(&entry);
        vbox.container_add(&list_scroller);
        list_scroller.container_add(&list);
        window.add_controller(key_controller(sender));

        let widgets = Self::Widgets {
            entry,
            scroller: list_scroller,
            results_list: list,
        };
        AsyncComponentParts { model, widgets }
    }

    #[instrument(skip_all, fields(message))]
    async fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        // should be here to ensure it is always false when update_view is run.
        // FIXME: after tab completion the window becomes full height for some reason.
        root.set_default_height(-1);

        match message {
            LauncherMsg::SetInput(input) => {
                let fut = self.model.set_input(input);
                sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) })
            }
            LauncherMsg::PluginEvent(plugin_event) => {
                self.model
                    .handle_event(plugin_event, &mut Frontend { widgets, root })
                    .await;
            }
            LauncherMsg::Select(index) => self.model.set_list_selection(index),
            LauncherMsg::SelectDelta(delta) => self.model.move_list_selection(delta),
            LauncherMsg::Activate => {
                let fut = self.model.activate();
                sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) });
            }
            LauncherMsg::Complete => {
                let fut = self.model.complete();
                sender.oneshot_command(async { LauncherMsg::PluginEvent(fut.await) });
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

    #[instrument(skip_all, fields(message))]
    async fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        info!("received command {message:?}");
        self.update_with_view(widgets, message, sender, root).await
    }
}

fn key_controller(sender: AsyncComponentSender<Launcher>) -> EventControllerKey {
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
