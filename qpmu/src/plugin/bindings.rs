//! Generated WIT bindings and conversion trait implementations.

use std::io;

pub use bindgen::{
    qpmu::plugin::{
        host::{add_to_linker as add_host_to_linker, Host},
        types::*,
    },
    Plugin,
};

mod bindgen {
    wasmtime::component::bindgen!({
        world: "plugin",
        path: "../qpmu-api/wit",
        async: true,
    });
}

impl From<io::Error> for IoError {
    fn from(value: io::Error) -> Self {
        use io::ErrorKind as E;
        use IoError as E2;
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

impl From<std::process::Output> for ProcessOutput {
    fn from(value: std::process::Output) -> Self {
        Self {
            exit_code: value.status.code(),
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}
