//! Utilities for managing the window

use tauri::Manager;

pub fn hide_menu(app: &tauri::AppHandle) {
    tracing::debug!("hiding window");
    if let Some(window) = app.get_webview_window("main") {
        tracing::info!("hiding main window");
        window.hide().unwrap();
    } else {
        tracing::warn!("main window was not found");
    }
}

pub fn show_menu(app: &tauri::AppHandle) {
    tracing::debug!("showing window");
    if let Some(window) = app.get_webview_window("main") {
        tracing::info!("showing main window");
        window.show().unwrap();
        window.set_focus().unwrap();
        // maximise in case the target monitor changes.
        window.set_resizable(true).unwrap();
        window.maximize().unwrap();
        window.set_resizable(false).unwrap();
        tracing::info!("finished showing main window");
    } else {
        tracing::warn!("main window was not found");
    }
}
