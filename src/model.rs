use crate::plugins::{ListItem, PluginEvent};

#[derive(Debug)]
#[tracker::track]
pub struct Launcher {
    pub query: String,
    #[no_eq]
    pub results: Vec<ListItem>,
    pub selection: usize,
    #[do_not_track]
    pub grab_full_focus: bool,
    /// How many actions (queries, activations) have been sent before this one.
    #[do_not_track]
    action_index: u64,
    /// Index of the last action that was completed.
    #[do_not_track]
    completed_action_index: u64,
}

impl Launcher {
    pub fn new() -> Self {
        Self {
            query: Default::default(),
            results: Default::default(),
            selection: Default::default(),
            grab_full_focus: false,
            tracker: 0,
            completed_action_index: 0,
            action_index: 0,
        }
    }

    // maybe handle wrap arounds later, but for now u64 is so huge there's no need.
    // if it does wrap around, panic, as something has probably gone wrong
    pub fn next_action(&mut self) -> u64 {
        self.action_index = self
            .action_index
            .checked_add(1)
            .expect("action index overflowed");
        self.action_index
    }

    /// Whether this action should be performed.
    ///
    /// Sets `self` to track this index as being performed if so.
    pub fn should_perform(&mut self, action_index: u64) -> bool {
        let res = self.completed_action_index < action_index;
        self.completed_action_index = self.completed_action_index.max(action_index);
        res
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
    PluginEvent(u64, PluginEvent),
    /// Focus the window and select the existing query
    Focus,
}
