use crate::Input;

/// An action for Covey to perform.
///
/// Note that actions don't close Covey automatically. Add [`Action::Close`] to
/// close the application. This should usually be first to feel more responsive
/// and avoid waiting for other commands to run before closing.
#[derive(Debug, Clone)]
pub struct Action(covey_proto::PluginAction);

impl Action {
    pub(crate) fn into_proto(self) -> covey_proto::PluginAction {
        self.0
    }
}

/// Helper constructors for action variants
impl Action {
    pub fn close() -> Self {
        Self(covey_proto::PluginAction::Close)
    }

    pub fn copy(str: impl Into<String>) -> Self {
        Self(covey_proto::PluginAction::Copy(str.into()))
    }

    pub fn set_input(input: impl Into<Input>) -> Self {
        Self(covey_proto::PluginAction::SetInput(
            input.into().into_proto(),
        ))
    }

    pub fn display_error(err: impl std::fmt::Display) -> Self {
        Self(covey_proto::PluginAction::DisplayError(err.to_string()))
    }
}

/// Wrapper for a [`Vec<Action>`] with convenient conversion trait
/// implementations.
///
/// [`From`] Implementations:
/// - [`IntoIterator<Item = Action>`] -> `Vec<Action>`
/// - [`Action`] -> `vec![Action]`
/// - [`Input`] -> `vec![Action::SetInput(Input)]`
///
/// While each action will be run in sequence, they will not wait for previous
/// actions to complete. If you need to run something more complex,
/// you can write your desired code in the command callback.
#[non_exhaustive]
pub struct Actions {
    pub list: Vec<Action>,
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
        Self::from(std::iter::once(value))
    }
}

impl From<Input> for Actions {
    fn from(value: Input) -> Self {
        Self::from(Action::set_input(value))
    }
}
