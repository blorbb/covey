use std::fmt::Display;

use tokio::sync::mpsc::UnboundedSender;

use crate::{Action, Input};

/// Provides methods to interact with the app menu.
///
/// This type is cheap to clone.
#[derive(Clone)]
pub struct Menu {
    pub(crate) sender: UnboundedSender<covey_proto::plugin_response::Action>,
}

impl Menu {
    fn send_action(&self, action: Action) {
        match self.sender.send(action.into_proto()) {
            Ok(()) => {}
            Err(e) => eprintln!("failed to send action: {e}"),
        }
    }

    pub fn close(&self) {
        self.send_action(Action::close())
    }

    pub fn copy(&self, str: impl Into<String>) {
        self.send_action(Action::copy(str))
    }

    pub fn set_input(&self, input: impl Into<Input>) {
        self.send_action(Action::set_input(input))
    }

    pub fn display_error(&self, err: impl Display) {
        self.send_action(Action::display_error(err))
    }
}
