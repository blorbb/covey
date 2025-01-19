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

pub(super) fn map_actions(plugin: &Plugin, actions: Vec<proto::Action>) -> Vec<Action> {
    use proto::action::Action as PAction;

    actions
        .into_iter()
        .filter_map(|action| {
            let Some(action) = action.action else {
                tracing::error!("plugin {plugin:?} did not provide an action: ignoring");
                return None;
            };

            Some(match action {
                PAction::Close(()) => Action::Close,
                PAction::RunCommand(proto::Command { cmd, args }) => Action::RunCommand(cmd, args),
                PAction::RunShell(str) => Action::RunShell(str),
                PAction::Copy(str) => Action::Copy(str),
                PAction::SetInput(input) => Action::SetInput(Input::from_proto(plugin, input)),
            })
        })
        .collect()
}
