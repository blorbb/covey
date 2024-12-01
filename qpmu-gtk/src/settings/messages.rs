use qpmu::plugin::Plugin;

use crate::model::LauncherMsg;

#[derive(Debug)]
pub enum SettingsMsg {
    SetPluginList(Vec<Plugin>),
    SetSelection(Option<usize>),
    Show,
}

#[derive(Debug)]
pub enum SettingsOutput {
    ReloadPlugins,
}

pub fn output_transform(input: SettingsOutput) -> LauncherMsg {
    match input {
        SettingsOutput::ReloadPlugins => LauncherMsg::ReloadPlugins,
    }
}
