use crate::{proto, Input};

#[derive(Debug, Clone)]
pub enum Action {
    Close,
    RunCommand(String, Vec<String>),
    RunShell(String),
    Copy(String),
    SetInput(Input),
}

impl Action {
    pub(crate) fn into_proto(self) -> proto::Action {
        use proto::action::Action as PrAction;

        let inner_action = match self {
            Self::Close => PrAction::Close(()),
            Self::RunCommand(cmd, args) => PrAction::RunCommand(proto::Command { cmd, args }),
            Self::RunShell(str) => PrAction::RunShell(str),
            Self::Copy(str) => PrAction::Copy(str),
            Self::SetInput(input) => PrAction::SetInput(input.into_proto()),
        };

        proto::Action {
            action: Some(inner_action),
        }
    }
}
