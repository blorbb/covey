use color_eyre::eyre::Result;
use covey_tauri_types::{Event, Hotkey, Key, ListItemId};
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
pub fn activate(state: State<'_, AppState>, list_item_id: ListItemId) {
    find_or_warn(&state, list_item_id).map(|item| tokio::spawn(state.host().activate(item)));
}

#[tauri::command]
pub fn alt_activate(state: State<'_, AppState>, list_item_id: ListItemId) {
    find_or_warn(&state, list_item_id).map(|item| tokio::spawn(state.host().alt_activate(item)));
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
        tokio::spawn(state.host().hotkey_activate(
            item,
            covey::hotkey::Hotkey {
                key: key_to_covey(key),
                ctrl,
                alt,
                shift,
                meta,
            },
        ))
    });
}

#[tauri::command]
pub fn complete(state: State<'_, AppState>, list_item_id: ListItemId) {
    find_or_warn(&state, list_item_id).map(|item| tokio::spawn(state.host().complete(item)));
}

fn find_or_warn(state: &AppState, id: ListItemId) -> Option<covey::ListItemId> {
    let item = state.find_list_item(&id);
    if item.is_none() {
        tracing::warn!("list item with id {id:?} not found")
    }
    item
}

fn key_to_covey(key: Key) -> covey::hotkey::Key {
    use covey::hotkey::Key as coveyKey;

    match key {
        Key::Digit0 => coveyKey::Digit0,
        Key::Digit1 => coveyKey::Digit1,
        Key::Digit2 => coveyKey::Digit2,
        Key::Digit3 => coveyKey::Digit3,
        Key::Digit4 => coveyKey::Digit4,
        Key::Digit5 => coveyKey::Digit5,
        Key::Digit6 => coveyKey::Digit6,
        Key::Digit7 => coveyKey::Digit7,
        Key::Digit8 => coveyKey::Digit8,
        Key::Digit9 => coveyKey::Digit9,
        Key::A => coveyKey::A,
        Key::B => coveyKey::B,
        Key::C => coveyKey::C,
        Key::D => coveyKey::D,
        Key::E => coveyKey::E,
        Key::F => coveyKey::F,
        Key::G => coveyKey::G,
        Key::H => coveyKey::H,
        Key::I => coveyKey::I,
        Key::J => coveyKey::J,
        Key::K => coveyKey::K,
        Key::L => coveyKey::L,
        Key::M => coveyKey::M,
        Key::N => coveyKey::N,
        Key::O => coveyKey::O,
        Key::P => coveyKey::P,
        Key::Q => coveyKey::Q,
        Key::R => coveyKey::R,
        Key::S => coveyKey::S,
        Key::T => coveyKey::T,
        Key::U => coveyKey::U,
        Key::V => coveyKey::V,
        Key::W => coveyKey::W,
        Key::X => coveyKey::X,
        Key::Y => coveyKey::Y,
        Key::Z => coveyKey::Z,
        Key::F1 => coveyKey::F1,
        Key::F2 => coveyKey::F2,
        Key::F3 => coveyKey::F3,
        Key::F4 => coveyKey::F4,
        Key::F5 => coveyKey::F5,
        Key::F6 => coveyKey::F6,
        Key::F7 => coveyKey::F7,
        Key::F8 => coveyKey::F8,
        Key::F9 => coveyKey::F9,
        Key::F10 => coveyKey::F10,
        Key::F11 => coveyKey::F11,
        Key::F12 => coveyKey::F12,
        Key::F13 => coveyKey::F13,
        Key::F14 => coveyKey::F14,
        Key::F15 => coveyKey::F15,
        Key::F16 => coveyKey::F16,
        Key::F17 => coveyKey::F17,
        Key::F18 => coveyKey::F18,
        Key::F19 => coveyKey::F19,
        Key::F20 => coveyKey::F20,
        Key::F21 => coveyKey::F21,
        Key::F22 => coveyKey::F22,
        Key::F23 => coveyKey::F23,
        Key::F24 => coveyKey::F24,
        Key::Backtick => coveyKey::Backtick,
        Key::Hyphen => coveyKey::Hyphen,
        Key::Equal => coveyKey::Equal,
        Key::Tab => coveyKey::Tab,
        Key::LeftBracket => coveyKey::LeftBracket,
        Key::RightBracket => coveyKey::RightBracket,
        Key::Backslash => coveyKey::Backslash,
        Key::Semicolon => coveyKey::Semicolon,
        Key::Apostrophe => coveyKey::Apostrophe,
        Key::Enter => coveyKey::Enter,
        Key::Comma => coveyKey::Comma,
        Key::Period => coveyKey::Period,
        Key::Slash => coveyKey::Slash,
    }
}

#[cfg(test)]
mod tests {
    use covey_tauri_types::Key;

    // ensure every covey key has a ts key.
    // no assertions, this should just compile.
    #[test]
    fn one_to_one_keys() {
        use covey::hotkey::Key as coveyKey;
        match coveyKey::A {
            coveyKey::Digit0 => Key::Digit0,
            coveyKey::Digit1 => Key::Digit1,
            coveyKey::Digit2 => Key::Digit2,
            coveyKey::Digit3 => Key::Digit3,
            coveyKey::Digit4 => Key::Digit4,
            coveyKey::Digit5 => Key::Digit5,
            coveyKey::Digit6 => Key::Digit6,
            coveyKey::Digit7 => Key::Digit7,
            coveyKey::Digit8 => Key::Digit8,
            coveyKey::Digit9 => Key::Digit9,
            coveyKey::A => Key::A,
            coveyKey::B => Key::B,
            coveyKey::C => Key::C,
            coveyKey::D => Key::D,
            coveyKey::E => Key::E,
            coveyKey::F => Key::F,
            coveyKey::G => Key::G,
            coveyKey::H => Key::H,
            coveyKey::I => Key::I,
            coveyKey::J => Key::J,
            coveyKey::K => Key::K,
            coveyKey::L => Key::L,
            coveyKey::M => Key::M,
            coveyKey::N => Key::N,
            coveyKey::O => Key::O,
            coveyKey::P => Key::P,
            coveyKey::Q => Key::Q,
            coveyKey::R => Key::R,
            coveyKey::S => Key::S,
            coveyKey::T => Key::T,
            coveyKey::U => Key::U,
            coveyKey::V => Key::V,
            coveyKey::W => Key::W,
            coveyKey::X => Key::X,
            coveyKey::Y => Key::Y,
            coveyKey::Z => Key::Z,
            coveyKey::F1 => Key::F1,
            coveyKey::F2 => Key::F2,
            coveyKey::F3 => Key::F3,
            coveyKey::F4 => Key::F4,
            coveyKey::F5 => Key::F5,
            coveyKey::F6 => Key::F6,
            coveyKey::F7 => Key::F7,
            coveyKey::F8 => Key::F8,
            coveyKey::F9 => Key::F9,
            coveyKey::F10 => Key::F10,
            coveyKey::F11 => Key::F11,
            coveyKey::F12 => Key::F12,
            coveyKey::F13 => Key::F13,
            coveyKey::F14 => Key::F14,
            coveyKey::F15 => Key::F15,
            coveyKey::F16 => Key::F16,
            coveyKey::F17 => Key::F17,
            coveyKey::F18 => Key::F18,
            coveyKey::F19 => Key::F19,
            coveyKey::F20 => Key::F20,
            coveyKey::F21 => Key::F21,
            coveyKey::F22 => Key::F22,
            coveyKey::F23 => Key::F23,
            coveyKey::F24 => Key::F24,
            coveyKey::Backtick => Key::Backtick,
            coveyKey::Hyphen => Key::Hyphen,
            coveyKey::Equal => Key::Equal,
            coveyKey::Tab => Key::Tab,
            coveyKey::LeftBracket => Key::LeftBracket,
            coveyKey::RightBracket => Key::RightBracket,
            coveyKey::Backslash => Key::Backslash,
            coveyKey::Semicolon => Key::Semicolon,
            coveyKey::Apostrophe => Key::Apostrophe,
            coveyKey::Enter => Key::Enter,
            coveyKey::Comma => Key::Comma,
            coveyKey::Period => Key::Period,
            coveyKey::Slash => Key::Slash,
        };
    }
}
