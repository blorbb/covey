use tauri::Manager;

#[tauri::command]
fn set_window_size(app: tauri::AppHandle, width: f64, height: f64) {
    eprintln!("setting window size to {width}x{height}");
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
            .unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![set_window_size])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
