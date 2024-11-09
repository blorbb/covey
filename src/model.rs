use std::cell::RefCell;

use crate::{
    plugin::{self, event::PluginEvent, InputLine, ListItem, Plugin},
    utils::ReadOnce,
};

#[derive(Debug)]
#[tracker::track]
pub struct Launcher {
    #[do_not_track]
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
    /// Whether the query has been set by a plugin.
    ///
    /// If so, contains a selection range to set the input line to.
    #[do_not_track]
    plugin_line_set: RefCell<ReadOnce<(Plugin, InputLine)>>,
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
            plugin_line_set: RefCell::new(ReadOnce::empty()),
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

    pub fn set_line_by_plugin(&mut self, plugin: Plugin, line: InputLine) {
        self.query = line.query.clone();
        self.plugin_line_set.get_mut().replace((plugin, line));
    }

    /// Returns the current query if it was set by a plugin.
    ///
    /// This will only return [`Some`] once each time
    /// [`Self::set_line_by_plugin`] is run. This will return [`None`]
    /// on subsequent calls until [`Self::set_line_by_plugin`] is called again.
    pub fn line_set_by_plugin(&self) -> Option<(Plugin, plugin::InputLine)> {
        self.plugin_line_set.borrow_mut().take()
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
    /// Perform (tab) completion on the current selected item
    Complete,
    /// Close (hide) the window
    Close,
}

#[derive(Debug)]
pub enum LauncherCmd {
    /// Contains the index of the plugin event, should be strictly increasing
    /// over time.
    ///
    /// Index is used to avoid applying old events.
    PluginEvent(u64, PluginEvent),
    /// Focus the window and select the existing query
    Focus,
}
