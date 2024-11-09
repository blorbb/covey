use std::process::Stdio;

use super::{Action, ListItem, Plugin};
use crate::plugin::bindings::{DeferredResult, IoError, ProcessOutput};

/// Event returned by a plugin.
#[derive(Debug)]
pub enum PluginEvent {
    /// Set the displayed list.
    SetList(Vec<ListItem>),
    /// Run a sequence of actions.
    Run {
        plugin: Plugin,
        actions: Vec<Action>,
    },
}

/// Events emitted by the UI to a plugin.
#[derive(Debug)]
pub enum UiEvent {
    InputChanged { query: String },
    Activate { item: ListItem },
    Complete { query: String, item: ListItem },
}

pub use super::bindings::DeferredAction;

impl DeferredAction {
    /// Completes this deferred action.
    pub(super) async fn run(&self) -> DeferredResult {
        match self {
            DeferredAction::Spawn((cmd, args)) => {
                DeferredResult::ProcessOutput(Self::spawn(cmd, args).await)
            }
        }
    }

    async fn spawn(cmd: &str, args: &[String]) -> Result<ProcessOutput, IoError> {
        Ok(tokio::process::Command::new(cmd)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?
            .into())
    }
}
