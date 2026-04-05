use crate::Input;

/// An action for Covey to perform.
///
/// Note that actions don't close Covey automatically. Add [`Action::close`] to
/// close the application. This should usually be first to feel more responsive
/// and avoid waiting for other commands to run before closing.
#[derive(Debug, Clone)]
pub struct Action(pub(crate) covey_proto::PluginAction);

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
            crate::into_proto::input(input.into()),
        ))
    }

    pub fn display_error(err: impl std::fmt::Display) -> Self {
        Self(covey_proto::PluginAction::DisplayError(err.to_string()))
    }
}
