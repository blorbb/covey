use color_eyre::eyre::Result;
use tauri::{ipc::Channel, State};

use super::frontend::{Event, EventChannel};
use crate::state::AppState;

#[tauri::command]
pub fn setup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    events: Channel<Event>,
) -> Result<(), String> {
    setup_impl(app, state, events).map_err(|e| format!("{e:#}"))
}

fn setup_impl(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    channel: Channel<Event>,
) -> Result<()> {
    let plugins = qpmu::config::Config::from_file()?.load();
    let frontend = EventChannel { channel, app };
    let model = qpmu::Model::new(plugins, frontend);
    state.init(model);
    Ok(())
}

#[tauri::command]
pub fn query(state: State<'_, AppState>, text: String) {
    state.lock().query(text);
}

#[tauri::command]
pub fn activate(state: State<'_, AppState>, list_item_id: u64) {
    find_or_warn(state.clone(), list_item_id).map(|item| state.lock().activate(item));
}

#[tauri::command]
pub fn alt_activate(state: State<'_, AppState>, list_item_id: u64) {
    find_or_warn(state.clone(), list_item_id).map(|item| state.lock().alt_activate(item));
}

// TODO: how to pass this in
// #[tauri::command]
// pub fn hotkey_activate(state: State<'_, AppState>, list_item_id: u64) {
//     find_or_warn(state.clone(), list_item_id).map(|item| state.lock().hotkey_activate(item, Hotkey {}));
// }


#[tauri::command]
pub fn complete(state: State<'_, AppState>, list_item_id: u64) {
    find_or_warn(state.clone(), list_item_id).map(|item| state.lock().complete(item));
}

fn find_or_warn(state: State<'_, AppState>, id: u64) -> Option<qpmu::ListItem> {
    let item = state.find_list_item(id);
    if item.is_none() {
        tracing::warn!("list item with id {id} not found")
    }
    item
}
