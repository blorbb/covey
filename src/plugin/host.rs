//! WIT host implementation.

use std::{fs, process::Stdio, sync::LazyLock};

use wasmtime_wasi::{async_trait, ResourceTable, WasiCtx, WasiView};

use super::bindings::{self, IoError};
use crate::plugin::bindings::{Capture, ProcessOutput};

pub(super) struct State {
    pub(super) ctx: WasiCtx,
    pub(super) table: ResourceTable,
}

impl WasiView for State {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

#[async_trait]
impl bindings::Host for State {
    async fn spawn(
        &mut self,
        cmd: String,
        args: Vec<String>,
        capture: Capture,
    ) -> Result<ProcessOutput, IoError> {
        eprintln!("calling command {cmd} {args:?}, capturing {capture:?}");
        let mut command = std::process::Command::new(cmd);
        command.args(args);
        // disallow stdin
        command.stdin(Stdio::null());
        if capture.contains(Capture::STDOUT) {
            command.stdout(Stdio::piped());
        }
        if capture.contains(Capture::STDERR) {
            command.stderr(Stdio::piped());
        }

        Ok(ProcessOutput::from(command.spawn()?.wait_with_output()?))
    }

    async fn config_dir(&mut self) -> String {
        static CONFIG_DIR: LazyLock<String> =
            LazyLock::new(|| dirs::config_dir().unwrap().to_str().unwrap().to_string());
        CONFIG_DIR.to_string()
    }

    async fn data_dir(&mut self) -> String {
        static DATA_DIR: LazyLock<String> =
            LazyLock::new(|| dirs::data_dir().unwrap().to_str().unwrap().to_string());
        DATA_DIR.to_string()
    }

    async fn read_dir(&mut self, path: String) -> Result<Vec<String>, IoError> {
        let results: Vec<_> = fs::read_dir(fs::canonicalize(&path)?)?
            .filter_map(|path| {
                path.ok()
                    .and_then(|dir| Some(dir.path().to_str()?.to_string()))
            })
            .collect();
        Ok(results)
    }

    async fn read_file(&mut self, path: String) -> Result<Vec<u8>, IoError> {
        Ok(fs::read(fs::canonicalize(&path)?)?)
    }

    async fn rank(
        &mut self,
        query: String,
        items: Vec<bindings::ListItem>,
        weights: bindings::Weights,
    ) -> Vec<bindings::ListItem> {
        // TODO: frequency weighting
        if query.is_empty() {
            return items;
        }

        let mut scored: Vec<_> = items
            .into_iter()
            .filter_map(|item| {
                macro_rules! score {
                    ($field:ident) => {
                        (weights.$field != 0.0)
                            .then(|| sublime_fuzzy::best_match(&query, &item.$field))
                            .flatten()
                            .map(|m| m.score() as f32 * weights.$field)
                            .unwrap_or(0.0)
                    };
                }

                let title_score = score!(title);
                let desc_score = score!(description);
                let meta_score = score!(metadata);
                let total_score = title_score + desc_score + meta_score;

                (total_score != 0.0).then_some((total_score, item))
            })
            .collect();

        // sort reversed
        scored.sort_by(|(s1, _), (s2, _)| s2.total_cmp(s1));

        scored.into_iter().map(|(_, item)| item).collect()
    }
}
