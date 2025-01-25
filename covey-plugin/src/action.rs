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

/// Wrapper for a [`Vec<Action>`] with convenient conversion trait implementations.
///
/// [`From`] Implementations:
/// - [`IntoIterator<Item = Action>`] -> `Vec<Action>`
/// - [`Action`] -> `vec![Action]`
/// - [`Input`] -> `vec![Action::SetInput(Input)]`
pub struct Actions {
    pub(crate) list: Vec<Action>,
}

impl<T: IntoIterator<Item = Action>> From<T> for Actions {
    fn from(value: T) -> Self {
        Self {
            list: value.into_iter().collect(),
        }
    }
}

impl From<Action> for Actions {
    fn from(value: Action) -> Self {
        Self { list: vec![value] }
    }
}

impl From<Input> for Actions {
    fn from(value: Input) -> Self {
        Self::from(Action::SetInput(value))
    }
}
