use color_eyre::eyre::Result;
use qpmu::{
    plugin::{Plugin, PluginEvent},
    Input, Model,
};

#[derive(Debug)]
pub struct Launcher {
    pub model: Model,
}

impl Launcher {
    pub fn new(plugins: Vec<Plugin>) -> Self {
        Self {
            model: Model::new(plugins),
        }
    }
}

#[derive(Debug)]
pub enum LauncherMsg {
    /// Set the query to a string
    SetInput(Input),
    /// Set the results list
    PluginEvent(Result<PluginEvent>),
    /// Selects a specific index of the results list
    Select(usize),
    /// Change the selection index by a certain amount
    SelectDelta(isize),
    /// Activate the current selected item
    Activate,
    /// Run the alternative activation function on the current selected item
    AltActivate,
    /// Perform (tab) completion on the current selected item
    Complete,
    /// Close (hide) the window
    Close,
    /// Open and focus the entry
    Focus,
}
