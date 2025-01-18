//! Utilities for managing the window

use tauri::Manager;

pub fn hide_menu(app: tauri::AppHandle) {
    eprintln!("hiding window");
    if let Some(window) = app.get_webview_window("main") {
        window.hide().unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

pub fn show_menu(app: tauri::AppHandle) {
    eprintln!("showing window");
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.set_focus().unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}
