use color_eyre::eyre::Result;
use comette_tauri_types::{Hotkey, Key, ListItemId};
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
    let frontend = EventChannel { channel, app };
    let model = comette::Model::new(frontend)?;
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
            comette::hotkey::Hotkey {
                key: key_to_comette(key),
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

fn find_or_warn(state: &AppState, id: ListItemId) -> Option<comette::ListItemId> {
    let item = state.find_list_item(&id);
    if item.is_none() {
        tracing::warn!("list item with id {id:?} not found")
    }
    item
}

fn key_to_comette(key: Key) -> comette::hotkey::Key {
    use comette::hotkey::Key as cometteKey;

    match key {
        Key::Digit0 => cometteKey::Digit0,
        Key::Digit1 => cometteKey::Digit1,
        Key::Digit2 => cometteKey::Digit2,
        Key::Digit3 => cometteKey::Digit3,
        Key::Digit4 => cometteKey::Digit4,
        Key::Digit5 => cometteKey::Digit5,
        Key::Digit6 => cometteKey::Digit6,
        Key::Digit7 => cometteKey::Digit7,
        Key::Digit8 => cometteKey::Digit8,
        Key::Digit9 => cometteKey::Digit9,
        Key::A => cometteKey::A,
        Key::B => cometteKey::B,
        Key::C => cometteKey::C,
        Key::D => cometteKey::D,
        Key::E => cometteKey::E,
        Key::F => cometteKey::F,
        Key::G => cometteKey::G,
        Key::H => cometteKey::H,
        Key::I => cometteKey::I,
        Key::J => cometteKey::J,
        Key::K => cometteKey::K,
        Key::L => cometteKey::L,
        Key::M => cometteKey::M,
        Key::N => cometteKey::N,
        Key::O => cometteKey::O,
        Key::P => cometteKey::P,
        Key::Q => cometteKey::Q,
        Key::R => cometteKey::R,
        Key::S => cometteKey::S,
        Key::T => cometteKey::T,
        Key::U => cometteKey::U,
        Key::V => cometteKey::V,
        Key::W => cometteKey::W,
        Key::X => cometteKey::X,
        Key::Y => cometteKey::Y,
        Key::Z => cometteKey::Z,
        Key::F1 => cometteKey::F1,
        Key::F2 => cometteKey::F2,
        Key::F3 => cometteKey::F3,
        Key::F4 => cometteKey::F4,
        Key::F5 => cometteKey::F5,
        Key::F6 => cometteKey::F6,
        Key::F7 => cometteKey::F7,
        Key::F8 => cometteKey::F8,
        Key::F9 => cometteKey::F9,
        Key::F10 => cometteKey::F10,
        Key::F11 => cometteKey::F11,
        Key::F12 => cometteKey::F12,
        Key::F13 => cometteKey::F13,
        Key::F14 => cometteKey::F14,
        Key::F15 => cometteKey::F15,
        Key::F16 => cometteKey::F16,
        Key::F17 => cometteKey::F17,
        Key::F18 => cometteKey::F18,
        Key::F19 => cometteKey::F19,
        Key::F20 => cometteKey::F20,
        Key::F21 => cometteKey::F21,
        Key::F22 => cometteKey::F22,
        Key::F23 => cometteKey::F23,
        Key::F24 => cometteKey::F24,
        Key::Backtick => cometteKey::Backtick,
        Key::Hyphen => cometteKey::Hyphen,
        Key::Equal => cometteKey::Equal,
        Key::Tab => cometteKey::Tab,
        Key::LeftBracket => cometteKey::LeftBracket,
        Key::RightBracket => cometteKey::RightBracket,
        Key::Backslash => cometteKey::Backslash,
        Key::Semicolon => cometteKey::Semicolon,
        Key::Apostrophe => cometteKey::Apostrophe,
        Key::Enter => cometteKey::Enter,
        Key::Comma => cometteKey::Comma,
        Key::Period => cometteKey::Period,
        Key::Slash => cometteKey::Slash,
    }
}

#[cfg(test)]
mod tests {
    use comette_tauri_types::Key;

    // ensure every comette key has a ts key.
    // no assertions, this should just compile.
    #[test]
    fn one_to_one_keys() {
        use comette::hotkey::Key as cometteKey;
        match cometteKey::A {
            cometteKey::Digit0 => Key::Digit0,
            cometteKey::Digit1 => Key::Digit1,
            cometteKey::Digit2 => Key::Digit2,
            cometteKey::Digit3 => Key::Digit3,
            cometteKey::Digit4 => Key::Digit4,
            cometteKey::Digit5 => Key::Digit5,
            cometteKey::Digit6 => Key::Digit6,
            cometteKey::Digit7 => Key::Digit7,
            cometteKey::Digit8 => Key::Digit8,
            cometteKey::Digit9 => Key::Digit9,
            cometteKey::A => Key::A,
            cometteKey::B => Key::B,
            cometteKey::C => Key::C,
            cometteKey::D => Key::D,
            cometteKey::E => Key::E,
            cometteKey::F => Key::F,
            cometteKey::G => Key::G,
            cometteKey::H => Key::H,
            cometteKey::I => Key::I,
            cometteKey::J => Key::J,
            cometteKey::K => Key::K,
            cometteKey::L => Key::L,
            cometteKey::M => Key::M,
            cometteKey::N => Key::N,
            cometteKey::O => Key::O,
            cometteKey::P => Key::P,
            cometteKey::Q => Key::Q,
            cometteKey::R => Key::R,
            cometteKey::S => Key::S,
            cometteKey::T => Key::T,
            cometteKey::U => Key::U,
            cometteKey::V => Key::V,
            cometteKey::W => Key::W,
            cometteKey::X => Key::X,
            cometteKey::Y => Key::Y,
            cometteKey::Z => Key::Z,
            cometteKey::F1 => Key::F1,
            cometteKey::F2 => Key::F2,
            cometteKey::F3 => Key::F3,
            cometteKey::F4 => Key::F4,
            cometteKey::F5 => Key::F5,
            cometteKey::F6 => Key::F6,
            cometteKey::F7 => Key::F7,
            cometteKey::F8 => Key::F8,
            cometteKey::F9 => Key::F9,
            cometteKey::F10 => Key::F10,
            cometteKey::F11 => Key::F11,
            cometteKey::F12 => Key::F12,
            cometteKey::F13 => Key::F13,
            cometteKey::F14 => Key::F14,
            cometteKey::F15 => Key::F15,
            cometteKey::F16 => Key::F16,
            cometteKey::F17 => Key::F17,
            cometteKey::F18 => Key::F18,
            cometteKey::F19 => Key::F19,
            cometteKey::F20 => Key::F20,
            cometteKey::F21 => Key::F21,
            cometteKey::F22 => Key::F22,
            cometteKey::F23 => Key::F23,
            cometteKey::F24 => Key::F24,
            cometteKey::Backtick => Key::Backtick,
            cometteKey::Hyphen => Key::Hyphen,
            cometteKey::Equal => Key::Equal,
            cometteKey::Tab => Key::Tab,
            cometteKey::LeftBracket => Key::LeftBracket,
            cometteKey::RightBracket => Key::RightBracket,
            cometteKey::Backslash => Key::Backslash,
            cometteKey::Semicolon => Key::Semicolon,
            cometteKey::Apostrophe => Key::Apostrophe,
            cometteKey::Enter => Key::Enter,
            cometteKey::Comma => Key::Comma,
            cometteKey::Period => Key::Period,
            cometteKey::Slash => Key::Slash,
        };
    }
}
