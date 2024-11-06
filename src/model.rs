use crate::plugins::{ListItem, PluginEvent};

#[derive(Debug)]
#[tracker::track]
pub struct Launcher {
    pub query: String,
    #[no_eq]
    pub results: Vec<ListItem>,
    pub selection: usize,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            query: Default::default(),
            results: Default::default(),
            selection: Default::default(),
            tracker: 0,
        }
    }
}

#[derive(Debug)]
pub enum LauncherMsg {
    /// Set the query to a string
    Query(String),
    /// Set the results list
    SetList(Vec<ListItem>),
    /// Selects a specific index of the results list
    Select(usize),
    /// Change the selection index by a certain amount
    SelectDelta(isize),
    /// Activate the current selected item
    Activate,
    /// Close (hide) the window
    Close,
}

#[derive(Debug)]
pub enum LauncherCmd {
    PluginEvent(PluginEvent),
}
