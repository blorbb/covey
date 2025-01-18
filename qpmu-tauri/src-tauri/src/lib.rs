mod ipc;
mod state;

use state::AppState;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            app.manage(AppState::new());

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&Menu::with_items(
                    app,
                    &[&MenuItem::with_id(app, "show", "Show", true, None::<&str>)?],
                )?)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        ipc::window::show_window(app.clone());
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
                        api.prevent_close();
                        main_window.hide().unwrap();
                    }
                    tauri::WindowEvent::Focused(focused) => {
                        if !*focused {
                            // main_window.hide().unwrap();
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::window::set_window_size,
            ipc::window::hide_window,
            ipc::window::show_window,
            ipc::model::setup,
            ipc::model::query,
            ipc::model::activate,
            ipc::model::alt_activate,
            ipc::model::complete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
