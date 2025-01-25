//! Quick utilities for spawning processes

use std::{ffi::OsStr, process::Stdio};

use color_eyre::eyre::Result;
use tokio::process::Child;

/// Spawn a process with `Stdio::null()` for stdin/out/err.
pub(crate) fn free_null(
    cmd: impl AsRef<OsStr>,
    args: impl IntoIterator<Item: AsRef<OsStr>>,
) -> Result<Child> {
    Ok(tokio::process::Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?)
}
