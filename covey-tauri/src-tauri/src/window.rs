//! Utilities for managing the window

#[cfg(all(
    feature = "layer-shell",
    not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )),
))]
compile_error!("layer-shell feature can only be enabled on linux/bsd systems");

#[cfg(feature = "layer-shell")]
pub mod layer_shell {
    use gtk::{
        gdk::traits::MonitorExt,
        traits::{ContainerExt, GtkWindowExt, WidgetExt},
    };
    use gtk_layer_shell::LayerShell;
    use tauri::{Emitter, Manager};
    use tokio::sync::mpsc;

    use crate::window::MenuWindow;

    /// Replaces the main tauri window with a new gtk window with layer shell enabled.
    ///
    /// Any APIs that manage windows need to use gtk-specific APIs.
    pub fn setup(app: &tauri::App) -> MenuWindow {
        tracing::info!("setting up layer shell");
        // https://github.com/tauri-apps/tauri/issues/2083#issuecomment-2424150009

        // replace the main tauri window with a gtk window
        let main_window = app.get_webview_window("main").unwrap();
        main_window.hide().unwrap();
        let gtk_window =
            gtk::ApplicationWindow::new(&main_window.gtk_window().unwrap().application().unwrap());

        // to prevent the window from being black initially
        gtk_window.set_app_paintable(true);

        gtk_window.set_decorated(false);

        // extract the contents of the tauri window
        // and put it in the gtk window
        let tauri_contents = main_window.default_vbox().unwrap();
        main_window.gtk_window().unwrap().remove(&tauri_contents);
        gtk_window.add(&tauri_contents);

        // do layer shell stuff
        gtk_window.init_layer_shell();
        gtk_window.set_keyboard_mode(gtk_layer_shell::KeyboardMode::Exclusive);

        gtk_window.set_position(gtk::WindowPosition::CenterAlways);
        gtk_window.set_resizable(false);
        gtk_window.set_layer(gtk_layer_shell::Layer::Overlay);
        gtk_window.show_all();

        // maximise doesn't work so must do manually
        maximise(&gtk_window);

        gtk_window.connect_focus_out_event(move |gtk_window, _| {
            tracing::debug!("lost focus");
            gtk_window.hide();
            gtk::glib::Propagation::Proceed
        });
        // for some reason this is making it not show up at all
        // is this even necessary?
        // gtk_window.hide_on_delete();

        let (tx, mut rx) = mpsc::unbounded_channel();
        let app_handle = app.handle().clone();
        gtk::glib::MainContext::default().spawn_local(async move {
            tracing::info!("window handler loop started");
            while let Some(action) = rx.recv().await {
                tracing::debug!("window action {action:?} received");
                match action {
                    WindowAction::Hide => {
                        gtk_window.hide();
                    }
                    WindowAction::Show => {
                        gtk_window.show_all();
                        maximise(&gtk_window);
                        app_handle.emit("focus-menu", ()).unwrap();
                    }
                }
            }
            tracing::info!("window handler loop stopped");
        });

        MenuWindow { channel: tx }
    }

    fn maximise(window: &gtk::ApplicationWindow) {
        // maximise doesn't work so must do manually
        let monitor = gtk::gdk::Display::default()
            .and_then(|d| d.monitor_at_window(&window.window().unwrap()));
        if let Some(monitor) = monitor {
            window.set_width_request(monitor.geometry().width());
            window.set_height_request(monitor.geometry().height());
        } else {
            window.set_width_request(1200);
            window.set_height_request(800);
        }
    }

    #[derive(Debug)]
    pub(super) enum WindowAction {
        Hide,
        Show,
    }

    impl MenuWindow {
        pub fn hide(&self) {
            self.channel.send(WindowAction::Hide).unwrap();
        }

        pub fn show(&self) {
            self.channel.send(WindowAction::Show).unwrap();
        }
    }
}

pub struct MenuWindow {
    #[cfg(feature = "layer-shell")]
    channel: tokio::sync::mpsc::UnboundedSender<layer_shell::WindowAction>,
    #[cfg(not(feature = "layer-shell"))]
    window: tauri::WebviewWindow,
}

#[cfg(not(feature = "layer-shell"))]
impl MenuWindow {
    pub fn new(window: tauri::WebviewWindow) -> Self {
        Self { window }
    }

    pub fn hide(&self) {
        tracing::info!("hiding main window");
        self.window.hide().unwrap();
    }

    pub fn show(&self) {
        tracing::info!("showing main window");
        self.window.show().unwrap();
        self.window.set_focus().unwrap();
        // maximise in case the target monitor changes.
        self.window.set_resizable(true).unwrap();
        self.window.maximize().unwrap();
        self.window.set_resizable(false).unwrap();
        tracing::info!("finished showing main window");
    }

    pub fn hook_window_requests(&self) {
        self.window.on_window_event({
            let window = self.window.clone();
            move |ev| match ev {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    tracing::debug!("close requested");
                    api.prevent_close();
                    window.hide().unwrap();
                }
                tauri::WindowEvent::Focused(focused) => {
                    tracing::debug!("changed focus to {focused}");
                    if !*focused {
                        window.hide().unwrap();
                    }
                }
                _ => {}
            }
        });
    }
}

use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::state::AppState;

pub fn manage_menu_window(app: &tauri::App) {
    #[cfg(feature = "layer-shell")]
    let menu_window = layer_shell::setup(app);
    #[cfg(not(feature = "layer-shell"))]
    let menu_window = MenuWindow::new(app.get_webview_window("main").unwrap());

    app.manage(menu_window);
}

pub fn init_tray_icon(app: &tauri::App) -> tauri::Result<()> {
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .menu(&Menu::with_items(
            app,
            &[
                &MenuItem::with_id(app, "show", "Show", true, None::<&str>)?,
                &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
            ],
        )?)
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                tracing::info!("left clicked on tray icon");
                show_menu(tray.app_handle());
            }
            _ => {}
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                show_menu(app);
            }
            "quit" => {
                // state must be manually unmanaged to drop them
                // this must be dropped to kill child processes.
                app.unmanage::<AppState>();
                app.exit(0);
            }
            other => panic!("unknown tray menu event {other}"),
        })
        .build(app)?;

    Ok(())
}

pub fn hide_menu(app: &tauri::AppHandle) {
    tracing::debug!("hiding window");
    app.state::<MenuWindow>().hide();
}

pub fn show_menu(app: &tauri::AppHandle) {
    tracing::debug!("showing window");
    app.state::<MenuWindow>().show();
}
