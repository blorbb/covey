#![expect(clippy::needless_pass_by_value, reason = "commands need owned types")]

use std::path::PathBuf;

use color_eyre::eyre::Result;
use covey::Plugin;
use covey_schema::{config::GlobalConfig, keyed_list::Id, manifest::PluginManifest};
use covey_tauri_types::{Event, ListItemId};
use tauri::{Manager, State, WebviewWindowBuilder, ipc::Channel};

use crate::{state::AppState, window::MenuWindow};

#[tauri::command]
pub fn setup(app: tauri::AppHandle, channel: Channel<Event>) -> Result<(), String> {
    let moved_app = app.clone();
    let old_state = app.unmanage::<AppState>();
    if old_state.is_some() {
        tracing::warn!("removed previous app state")
    }
    app.manage(AppState::new(moved_app, channel).map_err(|e| format!("{e:#}"))?);
    Ok(())
}

#[tauri::command]
pub fn query(state: State<'_, AppState>, text: String) {
    tokio::spawn(state.sender.lock().send_query(text));
}

#[tauri::command]
pub fn activate(state: State<'_, AppState>, list_item_id: ListItemId, command_name: String) {
    let state = &state;
    let id = list_item_id;
    let item = state.find_list_item(&id);

    if let Some(item) = item {
        tokio::spawn(state.sender.lock().activate(item, command_name));
    } else {
        tracing::warn!("list item with id {id:?} not found")
    }
}

#[tauri::command]
pub fn reload_plugin(state: State<'_, AppState>, plugin_id: Id) {
    state.sender.lock().reload_plugin(&plugin_id);
}

#[tauri::command]
pub fn show_settings_window(app: tauri::AppHandle) {
    let window = app.get_webview_window("settings").unwrap_or_else(|| {
        let settings_window = app
            .config()
            .app
            .windows
            .iter()
            .find(|window| window.label == "settings")
            .expect("app config should have settings window");

        WebviewWindowBuilder::from_config(&app, settings_window)
            .unwrap()
            .build()
            .unwrap()
    });

    window.show().unwrap();
    window.set_focus().unwrap();
}

#[tauri::command]
pub fn show_menu_window(window: State<'_, MenuWindow>) {
    window.show();
}

#[tauri::command]
pub fn hide_menu_window(window: State<'_, MenuWindow>) {
    window.hide();
}

/// Must be called after the app is initialised.
#[tauri::command]
pub fn get_global_config(state: State<'_, AppState>) -> GlobalConfig {
    state.sender.lock().config().clone()
}

/// Must be called after the app is initialised.
#[tauri::command]
pub fn set_global_config(state: State<'_, AppState>, config: GlobalConfig) {
    tracing::debug!("received global config {config:#?}");
    state.sender.lock().reload(config)
}

#[tauri::command]
pub fn get_manifest(state: State<'_, AppState>, plugin_name: String) -> Option<PluginManifest> {
    state
        .sender
        .lock()
        .plugins()
        .get(&plugin_name)
        .map(Plugin::manifest)
        .cloned()
}

/// Reads any file on the device without restrictions.
#[tauri::command]
pub fn read_any_file(path: PathBuf) -> Result<tauri::ipc::Response, String> {
    Ok(tauri::ipc::Response::new(
        std::fs::read(path).map_err(|e| e.to_string())?,
    ))
}

#[tauri::command]
pub fn log_error(log: String) {
    tracing::error!("ui: {log}")
}

#[tauri::command]
pub fn log_warn(log: String) {
    tracing::warn!("ui: {log}")
}

#[tauri::command]
pub fn log_info(log: String) {
    tracing::info!("ui: {log}")
}

#[tauri::command]
pub fn log_debug(log: String) {
    tracing::debug!("ui: {log}")
}

#[tauri::command]
pub fn log_trace(log: String) {
    tracing::trace!("ui: {log}")
}
