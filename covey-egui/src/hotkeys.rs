use covey::covey_schema::hotkey::Hotkey;
use egui::{Key, KeyboardShortcut, Modifiers, Ui};

use crate::conv;

/// Returns if a key was pressed and consumes it.
pub(crate) fn key_pressed_consume(ui: &mut Ui, key: Key) -> bool {
    ui.input_mut(|state| state.consume_key(Modifiers::NONE, key))
}

/// Returns if a hotkey (with modifiers) was pressed and consumes it.
pub(crate) fn hotkey_pressed_consume(ui: &mut Ui, key: Hotkey) -> bool {
    let is_mac = ui.ctx().os().is_mac();
    ui.input_mut(|state| {
        state.consume_shortcut(&KeyboardShortcut::new(
            Modifiers {
                alt: key.alt,
                ctrl: key.ctrl,
                shift: key.shift,
                mac_cmd: is_mac && key.ctrl,
                command: key.ctrl,
            },
            conv::covey_key_code_to_egui_key(key.key),
        ))
    })
}

/// Gets the hotkey that was pressed this frame.
///
/// Does not consume the hotkey.
pub(crate) fn hotkey_pressed_now(ui: &mut Ui) -> Option<Hotkey> {
    ui.input(|i| {
        let keys_pressed: Vec<_> = i
            .events
            .iter()
            .filter_map(|ev| match ev {
                egui::Event::Key {
                    key,
                    physical_key: _,
                    pressed: true,
                    repeat: false,
                    modifiers: _,
                } => Some(key),
                _ => None,
            })
            .collect();

        if keys_pressed.len() > 1 {
            return None;
        }

        let key_code = conv::egui_key_to_covey_key_code(**keys_pressed.first()?)?;

        let m = i.modifiers;
        Some(Hotkey {
            key: key_code,
            ctrl: m.command,
            alt: m.alt,
            shift: m.shift,
            meta: false,
        })
    })
}
