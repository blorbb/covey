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

pub use bindings::PluginAction as PluginActivationAction;

pub enum PluginEvent {
    SetList(Vec<ListItem>),
    Activate(PluginActivationAction),
}

pub struct UiEvent {
    pub query: String,
}

mod wrappers;
pub use wrappers::{ListItem, Plugin};
pub mod comm;