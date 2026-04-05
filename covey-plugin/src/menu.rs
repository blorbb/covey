use std::fmt::Display;

use crate::{Action, Input};

/// Provides methods to interact with the app menu.
pub struct Menu {
    /// The request ID that all responses are replying to.
    pub(crate) request_id: covey_proto::RequestId,
}

impl Menu {
    fn send_action(&self, action: Action) {
        let response = covey_proto::Response::perform_action(
            self.request_id,
            crate::into_proto::action(action),
        );
        println!("{}", response.serialize());
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
