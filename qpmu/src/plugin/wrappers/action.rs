use super::Plugin;
use crate::{plugin::bindings, Input};

#[derive(Debug)]
pub enum Action {
    Close,
    RunCommand(String, Vec<String>),
    RunShell(String),
    Copy(String),
    SetInputLine(Input),
}

pub fn map_actions(plugin: Plugin, actions: Vec<bindings::Action>) -> Vec<Action> {
    use bindings::action::Action as BA;

    actions
        .into_iter()
        .filter_map(|action| {
            let Some(action) = action.action else {
                tracing::error!("plugin {plugin:?} did not provide an action: ignoring");
                return None;
            };

            Some(match action {
                BA::Close(()) => Action::Close,
                BA::RunCommand(bindings::Command { cmd, args }) => Action::RunCommand(cmd, args),
                BA::RunShell(str) => Action::RunShell(str),
                BA::Copy(str) => Action::Copy(str),
                BA::SetInputLine(input_line) => {
                    Action::SetInputLine(Input::from_wit_input(plugin, input_line))
                }
            })
        })
        .collect()
}
