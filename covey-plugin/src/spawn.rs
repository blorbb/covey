//! Utilities for spawning processes.
//!
//! These are intended for use in command callbacks. They will spawn processes without catching
//! any stdio.

use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

pub fn program(
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

/// Runs a string as a shell script.
pub fn script(script: impl AsRef<OsStr>) -> std::io::Result<()> {
    self::program("sh", ["-c".as_ref(), script.as_ref()])
}
