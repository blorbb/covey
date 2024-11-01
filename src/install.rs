use std::process::ExitStatus;

use color_eyre::eyre::{ContextCompat, Result};

pub fn install_plugin(args: &[String]) -> Result<ExitStatus> {
    // TODO: don't use unstable feature
    Ok(std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-wasip2")
        .arg("--artifact-dir")
        .arg(
            dirs::config_dir()
                .context("config dir missing")?
                .join("qpmu")
                .join("plugins"),
        )
        .arg("-Z")
        .arg("unstable-options")
        .args(args)
        .spawn()?
        .wait()?)
}
