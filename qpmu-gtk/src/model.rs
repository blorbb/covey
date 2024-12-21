use qpmu::{lock::SharedMutex, plugin::Plugin, Input, Model};
use relm4::Controller;

use crate::settings::ui::Settings;

pub struct Launcher {
    pub model: SharedMutex<Model<crate::ui::Frontend>>,
    pub settings: Controller<Settings>,
}

impl Launcher {
    pub fn new(
        plugins: Vec<Plugin>,
        settings: Controller<Settings>,
        fe: crate::ui::Frontend,
    ) -> Self {
        Self {
            model: Model::new(plugins, fe),
            settings,
        }
    }
}

#[derive(Debug)]
pub enum LauncherMsg {
    /// Set the query to a string
    SetInput(Input),
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
    /// Open and focus the entry
    Focus,
    /// Run an arbitrary hotkey on the current list item
    /// that is not one of the existing keybinds.
    Hotkey(qpmu::hotkey::Hotkey),
    /// Close (hide) the window
    Close,
    /// Shutdown the entire application, killing all child processes.
    Shutdown,
    OpenSettings,
    /// Reloads the plugins by calling [`Model::reload`].
    ReloadPlugins,
    /// First one is title, second one is body.
    DisplayError(String, color_eyre::eyre::Report),
    /// Update the input without changing the model.
    UpdateInputUi,
    /// Update the result list without changing the model.
    UpdateResultListUi,
    /// Update the selected list item without changing the model.
    UpdateSelected,
}
