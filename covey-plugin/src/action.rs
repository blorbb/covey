use crate::Input;

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
    Copy(String),
    SetInput(Input),
    DisplayError(String),
}

impl Action {
    pub(crate) fn into_proto(self) -> covey_proto::Action {
        use covey_proto::action::Action as PrAction;

        let inner_action = match self {
            Self::Close => PrAction::Close(()),
            Self::Copy(str) => PrAction::Copy(str),
            Self::SetInput(input) => PrAction::SetInput(input.into_proto()),
            Self::DisplayError(err) => PrAction::DisplayError(err),
        };

        covey_proto::Action {
            action: Some(inner_action),
        }
    }
}

/// Helper constructors for action variants
impl Action {
    pub fn close() -> Self {
        Self::Close
    }

    pub fn copy(str: impl Into<String>) -> Self {
        Self::Copy(str.into())
    }

    pub fn set_input(input: impl Into<Input>) -> Self {
        Self::SetInput(input.into())
    }

    pub fn display_error(err: impl std::fmt::Display) -> Self {
        Self::DisplayError(err.to_string())
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
