pub mod bindings {
    use std::io;

    use host::SpawnError;

    wasmtime::component::bindgen!({world: "qpmu", path: "./qpmu-api/wit/world.wit"});

    impl From<io::Error> for SpawnError {
        fn from(value: io::Error) -> Self {
            use io::ErrorKind as E;
            match value.kind() {
                E::NotFound => SpawnError::NotFound,
                E::PermissionDenied => SpawnError::PermissionDenied,
                E::BrokenPipe => SpawnError::BrokenPipe,
                E::WouldBlock => SpawnError::WouldBlock,
                E::InvalidInput => SpawnError::InvalidInput,
                E::TimedOut => SpawnError::TimedOut,
                E::Interrupted => SpawnError::Interrupted,
                E::Unsupported => SpawnError::Unsupported,
                E::UnexpectedEof => SpawnError::UnexpectedEof,
                E::OutOfMemory => SpawnError::OutOfMemory,
                _ => SpawnError::Other(value.to_string()),
            }
        }
    }

    impl From<std::process::Output> for host::Output {
        fn from(value: std::process::Output) -> Self {
            Self {
                exit_code: value.status.code(),
                stdout: value.stdout,
                stderr: value.stderr,
            }
        }
    }
}

use crate::PLUGINS;
pub use bindings::PluginAction as PluginActivationAction;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub enum PluginEvent {
    SetList(Vec<ListItem>),
    Activate(Vec<PluginActivationAction>),
}

#[derive(Debug)]
pub enum UiEvent {
    InputChanged { query: String },
    Activate { item: ListItem },
}

pub fn process_ui_event(ev: UiEvent) -> Result<PluginEvent> {
    Ok(match ev {
        UiEvent::InputChanged { query } => PluginEvent::SetList(
            PLUGINS
                .iter()
                .find_map(|plugin| plugin.try_call_input(&query))
                .transpose()?
                .unwrap_or_default(),
        ),

        UiEvent::Activate { item } => PluginEvent::Activate(item.activate()?),
    })
}

mod wrappers;
pub use wrappers::{ListItem, Plugin};
