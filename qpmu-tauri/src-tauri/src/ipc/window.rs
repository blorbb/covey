use tauri::{Emitter, Manager};


/// Event emitted when the window becomes focused.
pub const FOCUS_WINDOW_EVENT: &str = "focus-window";

// use AppHandle instead of Window so that these commands only affect
// the main window (don't want these to affect the settings)
#[tauri::command]
pub fn set_window_size(app: tauri::AppHandle, width: f64, height: f64) {
    eprintln!("setting window size to {width}x{height}");
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
            .unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

#[tauri::command]
pub fn hide_window(app: tauri::AppHandle) {
    eprintln!("hiding window");
    if let Some(window) = app.get_webview_window("main") {
        window.hide().unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

#[tauri::command]
pub fn show_window(app: tauri::AppHandle) {
    eprintln!("hiding window");
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.set_focus().unwrap();
        app.emit(FOCUS_WINDOW_EVENT, ()).unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}