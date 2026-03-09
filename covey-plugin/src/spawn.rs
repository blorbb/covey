//! Utilities for spawning processes.
//!
//! These are intended for use in command callbacks. They will spawn processes
//! without catching any stdio.

use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

/// Spawns a command in the background, ignoring stdin/out/err and the exit
/// code.
pub fn command(
    program: impl AsRef<OsStr>,
    args: impl IntoIterator<Item: AsRef<OsStr>>,
) -> std::io::Result<()> {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
