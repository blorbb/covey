use super::Plugin;
use crate::{plugin::proto, Input};

#[derive(Debug)]
pub enum Action {
    Close,
    RunCommand(String, Vec<String>),
    RunShell(String),
    Copy(String),
    SetInput(Input),
}

pub(super) fn map_actions(plugin: Plugin, actions: Vec<proto::Action>) -> Vec<Action> {
    use proto::action::Action as BA;

    actions
        .into_iter()
        .filter_map(|action| {
            let Some(action) = action.action else {
                tracing::error!("plugin {plugin:?} did not provide an action: ignoring");
                return None;
            };

            Some(match action {
                BA::Close(()) => Action::Close,
                BA::RunCommand(proto::Command { cmd, args }) => Action::RunCommand(cmd, args),
                BA::RunShell(str) => Action::RunShell(str),
                BA::Copy(str) => Action::Copy(str),
                BA::SetInput(input) => Action::SetInput(Input::from_proto(plugin, input)),
            })
        })
        .collect()
}
