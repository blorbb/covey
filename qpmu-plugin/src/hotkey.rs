pub use crate::proto::{Key, Modifiers};

pub struct Hotkey {
    pub modifiers: Modifiers,
    pub key: Key,
}
