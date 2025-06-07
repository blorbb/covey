mod ipc;
mod state;
mod window;

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
            window::manage_menu_window(app);
            window::init_tray_icon(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::setup,
            ipc::query,
            ipc::activate,
            ipc::show_settings_window,
            ipc::show_menu_window,
            ipc::hide_menu_window,
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
