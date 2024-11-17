use super::{Action, ListItem};

/// Event returned by a plugin.
#[derive(Debug)]
pub enum PluginEvent {
    /// Set the displayed list.
    SetList { list: Vec<ListItem>, index: u64 },
    /// Run a sequence of actions.
    Run(Vec<Action>),
}
