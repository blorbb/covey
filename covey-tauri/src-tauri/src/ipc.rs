use color_eyre::eyre::Result;
use covey_tauri_types::{Event, ListItemId};
use tauri::{ipc::Channel, State};

use crate::state::{AppState, EventChannel};

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
    let frontend = EventChannel { channel, app };
    state.init(frontend)?;
    Ok(())
}

#[tauri::command]
pub fn query(state: State<'_, AppState>, text: String) {
    tokio::spawn(state.host().query(text));
}

#[tauri::command]
pub fn activate(state: State<'_, AppState>, list_item_id: ListItemId, command_name: String) {
    let state = &state;
    let id = list_item_id;
    let item = state.find_list_item(&id);

    if let Some(item) = item {
        tokio::spawn(state.host().activate(item, command_name));
    } else {
        tracing::warn!("list item with id {id:?} not found")
    }
}
