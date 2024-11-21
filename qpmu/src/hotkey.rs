use crate::plugin::proto;
pub use crate::plugin::proto::{Key, Modifiers};

pub struct Hotkey {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl From<Hotkey> for proto::Hotkey {
    fn from(value: Hotkey) -> Self {
        Self {
            key: value.key as i32,
            modifiers: value.modifiers,
        }
    }
}
