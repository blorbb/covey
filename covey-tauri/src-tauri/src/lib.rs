mod ipc;
mod state;
mod window;

use state::AppState;
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show_menu(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
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
                        window::show_menu(tray.app_handle());
                    }
                    _ => {}
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        window::show_menu(app);
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

            let Some(main_window) = app.get_webview_window("main") else {
                panic!("missing main window")
            };

            main_window.on_window_event({
                let main_window = main_window.clone();
                move |ev| match ev {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        tracing::debug!("close requested");
                        api.prevent_close();
                        main_window.hide().unwrap();
                    }
                    tauri::WindowEvent::Focused(focused) => {
                        tracing::debug!("changed focus to {focused}");
                        if !*focused {
                            main_window.hide().unwrap();
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::setup,
            ipc::query,
            ipc::activate,
            ipc::show_settings_window,
            ipc::get_global_config,
            ipc::set_global_config,
            ipc::get_manifest,
            ipc::reload_plugin,
            ipc::read_any_file,
            ipc::log_error,
            ipc::log_warn,
            ipc::log_info,
            ipc::log_debug,
            ipc::log_trace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
