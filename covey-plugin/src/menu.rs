use std::fmt::Display;

use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::{Action, Input};

/// Provides methods to interact with the app menu.
///
/// This type is cheap to clone.
#[derive(Clone)]
pub struct Menu {
    pub(crate) sender: Sender<Result<covey_proto::ActivationResponse, tonic::Status>>,
}

impl Menu {
    fn send_action(&self, action: Action) -> JoinHandle<()> {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            _ = sender
                .send(Ok(covey_proto::ActivationResponse {
                    action: action.into_proto(),
                }))
                .await
        })
    }

    pub fn close(&self) -> JoinHandle<()> {
        self.send_action(Action::close())
    }

    pub fn copy(&self, str: impl Into<String>) -> JoinHandle<()> {
        self.send_action(Action::copy(str))
    }

    pub fn set_input(&self, input: impl Into<Input>) -> JoinHandle<()> {
        self.send_action(Action::set_input(input))
    }

    pub fn display_error(&self, err: impl Display) -> JoinHandle<()> {
        self.send_action(Action::display_error(err))
    }
}
