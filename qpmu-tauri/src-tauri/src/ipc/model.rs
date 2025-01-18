use color_eyre::eyre::Result;
use qpmu_tauri_types::{Hotkey, Key, ListItemId};
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
pub fn activate(state: State<'_, AppState>, list_item_id: ListItemId) {
    find_or_warn(&state, list_item_id).map(|item| state.lock().activate(item));
}

#[tauri::command]
pub fn alt_activate(state: State<'_, AppState>, list_item_id: ListItemId) {
    find_or_warn(&state, list_item_id).map(|item| state.lock().alt_activate(item));
}

#[tauri::command]
pub fn hotkey_activate(state: State<'_, AppState>, list_item_id: ListItemId, hotkey: Hotkey) {
    let Hotkey {
        key,
        ctrl,
        alt,
        shift,
        meta,
    } = hotkey;

    find_or_warn(&state, list_item_id).map(|item| {
        state.lock().hotkey_activate(
            item,
            qpmu::hotkey::Hotkey {
                key: key_to_qpmu(key),
                ctrl,
                alt,
                shift,
                meta,
            },
        )
    });
}

#[tauri::command]
pub fn complete(state: State<'_, AppState>, list_item_id: ListItemId) {
    find_or_warn(&state, list_item_id).map(|item| state.lock().complete(item));
}

fn find_or_warn(state: &AppState, id: ListItemId) -> Option<qpmu::ListItemId> {
    let item = state.find_list_item(&id);
    if item.is_none() {
        tracing::warn!("list item with id {id:?} not found")
    }
    item
}

fn key_to_qpmu(key: Key) -> qpmu::hotkey::Key {
    use qpmu::hotkey::Key as QpmuKey;

    match key {
        Key::Digit0 => QpmuKey::Digit0,
        Key::Digit1 => QpmuKey::Digit1,
        Key::Digit2 => QpmuKey::Digit2,
        Key::Digit3 => QpmuKey::Digit3,
        Key::Digit4 => QpmuKey::Digit4,
        Key::Digit5 => QpmuKey::Digit5,
        Key::Digit6 => QpmuKey::Digit6,
        Key::Digit7 => QpmuKey::Digit7,
        Key::Digit8 => QpmuKey::Digit8,
        Key::Digit9 => QpmuKey::Digit9,
        Key::A => QpmuKey::A,
        Key::B => QpmuKey::B,
        Key::C => QpmuKey::C,
        Key::D => QpmuKey::D,
        Key::E => QpmuKey::E,
        Key::F => QpmuKey::F,
        Key::G => QpmuKey::G,
        Key::H => QpmuKey::H,
        Key::I => QpmuKey::I,
        Key::J => QpmuKey::J,
        Key::K => QpmuKey::K,
        Key::L => QpmuKey::L,
        Key::M => QpmuKey::M,
        Key::N => QpmuKey::N,
        Key::O => QpmuKey::O,
        Key::P => QpmuKey::P,
        Key::Q => QpmuKey::Q,
        Key::R => QpmuKey::R,
        Key::S => QpmuKey::S,
        Key::T => QpmuKey::T,
        Key::U => QpmuKey::U,
        Key::V => QpmuKey::V,
        Key::W => QpmuKey::W,
        Key::X => QpmuKey::X,
        Key::Y => QpmuKey::Y,
        Key::Z => QpmuKey::Z,
        Key::F1 => QpmuKey::F1,
        Key::F2 => QpmuKey::F2,
        Key::F3 => QpmuKey::F3,
        Key::F4 => QpmuKey::F4,
        Key::F5 => QpmuKey::F5,
        Key::F6 => QpmuKey::F6,
        Key::F7 => QpmuKey::F7,
        Key::F8 => QpmuKey::F8,
        Key::F9 => QpmuKey::F9,
        Key::F10 => QpmuKey::F10,
        Key::F11 => QpmuKey::F11,
        Key::F12 => QpmuKey::F12,
        Key::F13 => QpmuKey::F13,
        Key::F14 => QpmuKey::F14,
        Key::F15 => QpmuKey::F15,
        Key::F16 => QpmuKey::F16,
        Key::F17 => QpmuKey::F17,
        Key::F18 => QpmuKey::F18,
        Key::F19 => QpmuKey::F19,
        Key::F20 => QpmuKey::F20,
        Key::F21 => QpmuKey::F21,
        Key::F22 => QpmuKey::F22,
        Key::F23 => QpmuKey::F23,
        Key::F24 => QpmuKey::F24,
        Key::Backtick => QpmuKey::Backtick,
        Key::Hyphen => QpmuKey::Hyphen,
        Key::Equal => QpmuKey::Equal,
        Key::Tab => QpmuKey::Tab,
        Key::LeftBracket => QpmuKey::LeftBracket,
        Key::RightBracket => QpmuKey::RightBracket,
        Key::Backslash => QpmuKey::Backslash,
        Key::Semicolon => QpmuKey::Semicolon,
        Key::Apostrophe => QpmuKey::Apostrophe,
        Key::Enter => QpmuKey::Enter,
        Key::Comma => QpmuKey::Comma,
        Key::Period => QpmuKey::Period,
        Key::Slash => QpmuKey::Slash,
    }
}

#[cfg(test)]
mod tests {
    use qpmu_tauri_types::Key;

    // ensure every qpmu key has a ts key.
    // no assertions, this should just compile.
    #[test]
    fn one_to_one_keys() {
        use qpmu::hotkey::Key as QpmuKey;
        match QpmuKey::A {
            QpmuKey::Digit0 => Key::Digit0,
            QpmuKey::Digit1 => Key::Digit1,
            QpmuKey::Digit2 => Key::Digit2,
            QpmuKey::Digit3 => Key::Digit3,
            QpmuKey::Digit4 => Key::Digit4,
            QpmuKey::Digit5 => Key::Digit5,
            QpmuKey::Digit6 => Key::Digit6,
            QpmuKey::Digit7 => Key::Digit7,
            QpmuKey::Digit8 => Key::Digit8,
            QpmuKey::Digit9 => Key::Digit9,
            QpmuKey::A => Key::A,
            QpmuKey::B => Key::B,
            QpmuKey::C => Key::C,
            QpmuKey::D => Key::D,
            QpmuKey::E => Key::E,
            QpmuKey::F => Key::F,
            QpmuKey::G => Key::G,
            QpmuKey::H => Key::H,
            QpmuKey::I => Key::I,
            QpmuKey::J => Key::J,
            QpmuKey::K => Key::K,
            QpmuKey::L => Key::L,
            QpmuKey::M => Key::M,
            QpmuKey::N => Key::N,
            QpmuKey::O => Key::O,
            QpmuKey::P => Key::P,
            QpmuKey::Q => Key::Q,
            QpmuKey::R => Key::R,
            QpmuKey::S => Key::S,
            QpmuKey::T => Key::T,
            QpmuKey::U => Key::U,
            QpmuKey::V => Key::V,
            QpmuKey::W => Key::W,
            QpmuKey::X => Key::X,
            QpmuKey::Y => Key::Y,
            QpmuKey::Z => Key::Z,
            QpmuKey::F1 => Key::F1,
            QpmuKey::F2 => Key::F2,
            QpmuKey::F3 => Key::F3,
            QpmuKey::F4 => Key::F4,
            QpmuKey::F5 => Key::F5,
            QpmuKey::F6 => Key::F6,
            QpmuKey::F7 => Key::F7,
            QpmuKey::F8 => Key::F8,
            QpmuKey::F9 => Key::F9,
            QpmuKey::F10 => Key::F10,
            QpmuKey::F11 => Key::F11,
            QpmuKey::F12 => Key::F12,
            QpmuKey::F13 => Key::F13,
            QpmuKey::F14 => Key::F14,
            QpmuKey::F15 => Key::F15,
            QpmuKey::F16 => Key::F16,
            QpmuKey::F17 => Key::F17,
            QpmuKey::F18 => Key::F18,
            QpmuKey::F19 => Key::F19,
            QpmuKey::F20 => Key::F20,
            QpmuKey::F21 => Key::F21,
            QpmuKey::F22 => Key::F22,
            QpmuKey::F23 => Key::F23,
            QpmuKey::F24 => Key::F24,
            QpmuKey::Backtick => Key::Backtick,
            QpmuKey::Hyphen => Key::Hyphen,
            QpmuKey::Equal => Key::Equal,
            QpmuKey::Tab => Key::Tab,
            QpmuKey::LeftBracket => Key::LeftBracket,
            QpmuKey::RightBracket => Key::RightBracket,
            QpmuKey::Backslash => Key::Backslash,
            QpmuKey::Semicolon => Key::Semicolon,
            QpmuKey::Apostrophe => Key::Apostrophe,
            QpmuKey::Enter => Key::Enter,
            QpmuKey::Comma => Key::Comma,
            QpmuKey::Period => Key::Period,
            QpmuKey::Slash => Key::Slash,
        };
    }
}
