use relm4::gtk::gdk;

/// Converts a gdk key and modifier to a qpmu hotkey.
///
/// Returns [`None`] if the key is not recognised, or the modifiers don't contain
/// at least one of ctrl, alt or super.
pub fn to_qpmu_hotkey(key: gdk::Key, modifier: gdk::ModifierType) -> Option<qpmu::hotkey::Hotkey> {
    let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);
    let alt = modifier.contains(gdk::ModifierType::ALT_MASK);
    let shift = modifier.contains(gdk::ModifierType::SHIFT_MASK);
    let super_ = modifier.contains(gdk::ModifierType::SUPER_MASK);

    // Require one of these keys to be pressed to be considered a hotkey
    if !(ctrl || alt || super_) {
        return None;
    }

    Some(qpmu::hotkey::Hotkey {
        key: to_qpmu_key(key)?,
        modifiers: qpmu::hotkey::Modifiers {
            ctrl,
            alt,
            shift,
            super_,
        },
    })
}

fn to_qpmu_key(key: gdk::Key) -> Option<qpmu::hotkey::Key> {
    use gdk::Key as GK;
    use qpmu::hotkey::Key as QK;

    Some(match key {
        GK::_0 => QK::Digit0,
        GK::_1 => QK::Digit1,
        GK::_2 => QK::Digit2,
        GK::_3 => QK::Digit3,
        GK::_4 => QK::Digit4,
        GK::_5 => QK::Digit5,
        GK::_6 => QK::Digit6,
        GK::_7 => QK::Digit7,
        GK::_8 => QK::Digit8,
        GK::_9 => QK::Digit9,
        GK::A => QK::A,
        GK::B => QK::B,
        GK::C => QK::C,
        GK::D => QK::D,
        GK::E => QK::E,
        GK::F => QK::F,
        GK::G => QK::G,
        GK::H => QK::H,
        GK::I => QK::I,
        GK::J => QK::J,
        GK::K => QK::K,
        GK::L => QK::L,
        GK::M => QK::M,
        GK::N => QK::N,
        GK::O => QK::O,
        GK::P => QK::P,
        GK::Q => QK::Q,
        GK::R => QK::R,
        GK::S => QK::S,
        GK::T => QK::T,
        GK::U => QK::U,
        GK::V => QK::V,
        GK::W => QK::W,
        GK::X => QK::X,
        GK::Y => QK::Y,
        GK::Z => QK::Z,
        GK::F1 => QK::F1,
        GK::F2 => QK::F2,
        GK::F3 => QK::F3,
        GK::F4 => QK::F4,
        GK::F5 => QK::F5,
        GK::F6 => QK::F6,
        GK::F7 => QK::F7,
        GK::F8 => QK::F8,
        GK::F9 => QK::F9,
        GK::F10 => QK::F10,
        GK::F11 => QK::F11,
        GK::F12 => QK::F12,
        GK::F13 => QK::F13,
        GK::F14 => QK::F14,
        GK::F15 => QK::F15,
        GK::F16 => QK::F16,
        GK::F17 => QK::F17,
        GK::F18 => QK::F18,
        GK::F19 => QK::F19,
        GK::F20 => QK::F20,
        GK::F21 => QK::F21,
        GK::F22 => QK::F22,
        GK::F23 => QK::F23,
        GK::F24 => QK::F24,
        GK::grave => QK::Backtick,
        GK::hyphen => QK::Hyphen,
        GK::equal => QK::Equal,
        GK::Tab => QK::Tab,
        GK::bracketleft => QK::LeftBracket,
        GK::bracketright => QK::RightBracket,
        GK::backslash => QK::Backslash,
        GK::semicolon => QK::Semicolon,
        GK::apostrophe => QK::Apostrophe,
        GK::Return => QK::Enter,
        GK::comma => QK::Comma,
        GK::period => QK::Period,
        GK::slash => QK::Slash,
        _ => return None,
    })
}
