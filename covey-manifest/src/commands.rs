use core::fmt;
use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
pub struct CommandId(Arc<str>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
pub struct Command {
    pub title: String,
    pub description: Option<String>,
    pub default_hotkey: Option<Hotkey>,
}

impl CommandId {
    pub fn new(name: &str) -> Self {
        Self(Arc::from(name))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<Arc<str>> for CommandId {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
pub struct Hotkey {
    pub key: Key,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub meta: bool,
}

/// A single key on a standard US QWERTY keyboard without shift being held.
///
/// Does **NOT** include:
/// - Modifiers
/// - Text editing keys like backspace / delete / insert.
/// - Movement keys like page up / home / down arrow.
/// - Escape or lock keys.
/// - Media keys.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(rename_all = "camelCase"))]
pub enum Key {
    Digit0, Digit1, Digit2,
    Digit3, Digit4, Digit5,
    Digit6, Digit7, Digit8,
    Digit9,
    A, B, C, D, E, F, G, H,
    I, J, K, L, M, N, O, P,
    Q, R, S, T, U, V, W, X,
    Y, Z,
    F1, F2, F3, F4,
    F5, F6, F7, F8,
    F9, F10, F11, F12,
    F13, F14, F15, F16,
    F17, F18, F19, F20,
    F21, F22, F23, F24,
    Backtick,
    Hyphen, Equal,
    Tab,
    LeftBracket, RightBracket, Backslash,
    Semicolon, Apostrophe, Enter,
    Comma, Period, Slash,
}

// FromStr and Display implementations //

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAcceleratorError {
    /// A duplicate or other incompatible modifier.
    ///
    /// Modifiers are incompatible if they are the same or
    /// aliases of each other (e.g. Ctrl and Control, or
    /// just having Alt twice).
    IncompatibleModifier(String, String),
    /// An unknown modifier.
    UnknownModifier(String),
    /// Unknown key.
    UnknownKey(String),
    /// Input is empty.
    Empty,
}

impl fmt::Display for ParseAcceleratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleModifier(m1, m2) => write!(
                f,
                "incompatible modifiers {m1:?} and {m2:?}: specify only one of these"
            ),
            Self::UnknownModifier(m) => write!(f, "unknown modifier {m:?}"),
            Self::UnknownKey(k) => write!(f, "unknown key {k:?}"),
            Self::Empty => write!(f, "no accelerator provided"),
        }
    }
}

impl std::error::Error for ParseAcceleratorError {}

impl FromStr for Hotkey {
    type Err = ParseAcceleratorError;

    /// A set of modifiers then a key code, separated by `+` characters.
    ///
    /// Parsing is case insensitive.
    ///
    /// Modifiers are one of "ctrl", "control", "alt", "shift" or "meta".
    ///
    /// Keys are the character produced when the key is pressed, or for
    /// enter and tab, the strings "enter" and "tab". See [`Key`] for the
    /// supported keys.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseAcceleratorError as E;

        // key code should be extracted from the back
        let mut modifiers = s.split('+');
        let key = modifiers.next_back().ok_or(E::Empty)?;
        let key = key
            .parse::<Key>()
            .map_err(|ParseKeyError(s)| E::UnknownKey(s))?;

        let mut ctrl = None;
        let mut alt = None;
        let mut shift = None;
        let mut meta = None;

        for modifier in modifiers {
            match &*modifier.to_lowercase() {
                "ctrl" | "control" => {
                    if let Some(prev) = ctrl.replace(modifier) {
                        return Err(E::IncompatibleModifier(
                            prev.to_string(),
                            modifier.to_string(),
                        ));
                    }
                }
                "alt" => {
                    if let Some(prev) = alt.replace(modifier) {
                        return Err(E::IncompatibleModifier(
                            prev.to_string(),
                            modifier.to_string(),
                        ));
                    }
                }
                "shift" => {
                    if let Some(prev) = shift.replace(modifier) {
                        return Err(E::IncompatibleModifier(
                            prev.to_string(),
                            modifier.to_string(),
                        ));
                    }
                }
                "meta" => {
                    if let Some(prev) = meta.replace(modifier) {
                        return Err(E::IncompatibleModifier(
                            prev.to_string(),
                            modifier.to_string(),
                        ));
                    }
                }
                _ => return Err(E::UnknownModifier(modifier.to_string())),
            };
        }

        Ok(Self {
            key,
            ctrl: ctrl.is_some(),
            alt: alt.is_some(),
            shift: shift.is_some(),
            meta: meta.is_some(),
        })
    }
}

impl fmt::Display for Hotkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.ctrl {
            write!(f, "Ctrl+")?;
        }
        if self.alt {
            write!(f, "Alt+")?;
        }
        if self.shift {
            write!(f, "Shift+")?;
        }
        if self.meta {
            write!(f, "Meta+")?;
        }
        write!(f, "{}", self.key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseKeyError(String);

impl fmt::Display for ParseKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown key {:?}", self.0)
    }
}

impl std::error::Error for ParseKeyError {}

impl FromStr for Key {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[rustfmt::skip]
        let v = match &*s.to_lowercase() {
            // Digits
            "0" => Self::Digit0,
            "1" => Self::Digit1,
            "2" => Self::Digit2,
            "3" => Self::Digit3,
            "4" => Self::Digit4,
            "5" => Self::Digit5,
            "6" => Self::Digit6,
            "7" => Self::Digit7,
            "8" => Self::Digit8,
            "9" => Self::Digit9,

            // Letters
            "a" => Self::A, "b" => Self::B, "c" => Self::C,
            "d" => Self::D, "e" => Self::E, "f" => Self::F,
            "g" => Self::G, "h" => Self::H, "i" => Self::I,
            "j" => Self::J, "k" => Self::K, "l" => Self::L,
            "m" => Self::M, "n" => Self::N, "o" => Self::O,
            "p" => Self::P, "q" => Self::Q, "r" => Self::R,
            "s" => Self::S, "t" => Self::T, "u" => Self::U,
            "v" => Self::V, "w" => Self::W, "x" => Self::X,
            "y" => Self::Y, "z" => Self::Z,

            // Function keys
            "f1" => Self::F1, "f2" => Self::F2, "f3" => Self::F3,
            "f4" => Self::F4, "f5" => Self::F5, "f6" => Self::F6,
            "f7" => Self::F7, "f8" => Self::F8, "f9" => Self::F9,
            "f10" => Self::F10, "f11" => Self::F11, "f12" => Self::F12,
            "f13" => Self::F13, "f14" => Self::F14, "f15" => Self::F15,
            "f16" => Self::F16, "f17" => Self::F17, "f18" => Self::F18,
            "f19" => Self::F19, "f20" => Self::F20, "f21" => Self::F21,
            "f22" => Self::F22, "f23" => Self::F23, "f24" => Self::F24,

            // Special characters
            "`" => Self::Backtick,
            "-" => Self::Hyphen,
            "=" => Self::Equal,
            "tab" => Self::Tab,
            "[" => Self::LeftBracket,
            "]" => Self::RightBracket,
            "\\" => Self::Backslash,
            ";" => Self::Semicolon,
            "'" => Self::Apostrophe,
            "enter" => Self::Enter,
            "," => Self::Comma,
            "." => Self::Period,
            "/" => Self::Slash,

            _ => return Err(ParseKeyError(s.to_string())),
        };

        Ok(v)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[rustfmt::skip]
        let s = match self {
            // Digits
            Key::Digit0 => "0",
            Key::Digit1 => "1",
            Key::Digit2 => "2",
            Key::Digit3 => "3",
            Key::Digit4 => "4",
            Key::Digit5 => "5",
            Key::Digit6 => "6",
            Key::Digit7 => "7",
            Key::Digit8 => "8",
            Key::Digit9 => "9",

            // Letters
            Key::A => "A", Key::B => "B", Key::C => "C",
            Key::D => "D", Key::E => "E", Key::F => "F",
            Key::G => "G", Key::H => "H", Key::I => "I",
            Key::J => "J", Key::K => "K", Key::L => "L",
            Key::M => "M", Key::N => "N", Key::O => "O",
            Key::P => "P", Key::Q => "Q", Key::R => "R",
            Key::S => "S", Key::T => "T", Key::U => "U",
            Key::V => "V", Key::W => "W", Key::X => "X",
            Key::Y => "Y", Key::Z => "Z",

            // Function keys
            Key::F1 => "F1", Key::F2 => "F2", Key::F3 => "F3",
            Key::F4 => "F4", Key::F5 => "F5", Key::F6 => "F6",
            Key::F7 => "F7", Key::F8 => "F8", Key::F9 => "F9",
            Key::F10 => "F10", Key::F11 => "F11", Key::F12 => "F12",
            Key::F13 => "F13", Key::F14 => "F14", Key::F15 => "F15",
            Key::F16 => "F16", Key::F17 => "F17", Key::F18 => "F18",
            Key::F19 => "F19", Key::F20 => "F20", Key::F21 => "F21",
            Key::F22 => "F22", Key::F23 => "F23", Key::F24 => "F24",

            // Special characters
            Key::Backtick => "`",
            Key::Hyphen => "-",
            Key::Equal => "=",
            Key::Tab => "Tab",
            Key::LeftBracket => "[",
            Key::RightBracket => "]",
            Key::Backslash => "\\",
            Key::Semicolon => ";",
            Key::Apostrophe => "'",
            Key::Enter => "Enter",
            Key::Comma => ",",
            Key::Period => ".",
            Key::Slash => "/",
        };

        f.write_str(s)
    }
}
