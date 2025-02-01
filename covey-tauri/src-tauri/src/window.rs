//! Utilities for managing the window

use tauri::Manager;

pub fn hide_menu(app: &tauri::AppHandle) {
    eprintln!("hiding window");
    if let Some(window) = app.get_webview_window("main") {
        window.hide().unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

pub fn show_menu(app: &tauri::AppHandle) {
    eprintln!("showing window");
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.set_focus().unwrap();
        // maximise in case the target monitor changes.
        window.set_resizable(true).unwrap();
        window.maximize().unwrap();
        window.set_resizable(false).unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}
