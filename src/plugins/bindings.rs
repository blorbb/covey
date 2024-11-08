//! Generated WIT bindings and conversion trait implementations.

use qpmu::plugin::host;
pub use qpmu::plugin::types::*;
use std::{io, process::Stdio};

wasmtime::component::bindgen!({
    world: "plugin",
    path: "./qpmu-api/wit",
    async: true,
});

impl From<io::Error> for host::IoError {
    fn from(value: io::Error) -> Self {
        use host::IoError as E2;
        use io::ErrorKind as E;
        match value.kind() {
            E::NotFound => E2::NotFound,
            E::PermissionDenied => E2::PermissionDenied,
            E::ConnectionRefused => E2::ConnectionRefused,
            E::ConnectionReset => E2::ConnectionReset,
            E::ConnectionAborted => E2::ConnectionAborted,
            E::NotConnected => E2::NotConnected,
            E::AddrInUse => E2::AddrInUse,
            E::AddrNotAvailable => E2::AddrNotAvailable,
            E::BrokenPipe => E2::BrokenPipe,
            E::AlreadyExists => E2::AlreadyExists,
            E::WouldBlock => E2::WouldBlock,
            E::InvalidInput => E2::InvalidInput,
            E::TimedOut => E2::TimedOut,
            E::WriteZero => E2::WriteZero,
            E::Interrupted => E2::Interrupted,
            E::Unsupported => E2::Unsupported,
            E::UnexpectedEof => E2::UnexpectedEof,
            E::OutOfMemory => E2::OutOfMemory,
            _ => E2::Other(value.to_string()),
        }
    }
}

impl From<std::process::Output> for host::ProcessOutput {
    fn from(value: std::process::Output) -> Self {
        Self {
            exit_code: value.status.code(),
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}

impl DeferredAction {
    /// Completes this deferred action.
    pub(super) async fn run(&self) -> DeferredResult {
        match self {
            DeferredAction::Spawn((cmd, args)) => {
                DeferredResult::ProcessOutput(Self::spawn(cmd, args).await)
            }
        }
    }

    async fn spawn(cmd: &str, args: &[String]) -> Result<host::ProcessOutput, host::IoError> {
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
