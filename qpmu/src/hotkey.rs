use crate::plugin::proto;
pub use crate::plugin::proto::Key;

#[derive(Debug, Clone, Copy)]
pub struct Hotkey {
    pub key: Key,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// The ⌘ Command key on Mac, ⊞ Windows key on Windows,
    /// or Super key on Linux.
    pub meta: bool,
}

impl From<Hotkey> for proto::Hotkey {
    fn from(value: Hotkey) -> Self {
        Self {
            key: value.key as i32,
            ctrl: value.ctrl,
            alt: value.alt,
            shift: value.shift,
            meta: value.meta,
        }
    }
}
