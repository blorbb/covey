use crate::{Input, proto};

/// An action for Covey to perform.
///
/// There are associated constructor functions which are easier to use
/// than constructing from the enum variants.
///
/// Note that actions don't close Covey automatically. Add [`Action::Close`] to
/// close the application. This should usually be first to feel more responsive
/// and avoid waiting for other commands to run before closing.
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

/// Helper constructors for action variants
impl Action {
    pub fn close() -> Self {
        Self::Close
    }

    pub fn run_command<Arg: Into<String>>(
        cmd: impl Into<String>,
        args: impl IntoIterator<Item = Arg>,
    ) -> Self {
        Self::RunCommand(cmd.into(), args.into_iter().map(Into::into).collect())
    }

    pub fn run_shell(script: impl Into<String>) -> Self {
        Self::RunShell(script.into())
    }

    pub fn copy(str: impl Into<String>) -> Self {
        Self::Copy(str.into())
    }

    pub fn set_input(input: impl Into<Input>) -> Self {
        Self::SetInput(input.into())
    }
}

/// Wrapper for a [`Vec<Action>`] with convenient conversion trait implementations.
///
/// [`From`] Implementations:
/// - [`IntoIterator<Item = Action>`] -> `Vec<Action>`
/// - [`Action`] -> `vec![Action]`
/// - [`Input`] -> `vec![Action::SetInput(Input)]`
///
/// While each action will be run in sequence, they will not wait for previous
/// actions to complete. If you need to run something more complex,
/// you can write your desired code in the command callback.
pub struct Actions {
    pub list: Vec<Action>,
    _priv: (),
}

impl<T: IntoIterator<Item = Action>> From<T> for Actions {
    fn from(value: T) -> Self {
        Self {
            list: value.into_iter().collect(),
            _priv: (),
        }
    }
}

impl From<Action> for Actions {
    fn from(value: Action) -> Self {
        Self::from(std::iter::once(value))
    }
}

impl From<Input> for Actions {
    fn from(value: Input) -> Self {
        Self::from(Action::SetInput(value))
    }
}
