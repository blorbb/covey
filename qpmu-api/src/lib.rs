#[doc(hidden)]
pub mod __raw_bindings {
    wit_bindgen::generate!({
        // path: "./wit",
        world: "plugin",
        pub_export_macro: true,
        export_macro_name: "export",
        // so that everything `use`d in `plugin` is exported,
        // instead of going through `types`.
        generate_unused_types: true,
    });
}

pub mod host {
    use std::path::{Path, PathBuf};

    use super::__raw_bindings::qpmu::plugin::{host, types::*};

    pub fn spawn(
        cmd: &str,
        args: impl IntoIterator<Item: AsRef<str>>,
        capture: Capture,
    ) -> Result<ProcessOutput, IoError> {
        host::spawn(
            cmd,
            &args
                .into_iter()
                .map(|a| a.as_ref().to_owned())
                .collect::<Vec<_>>(),
            capture,
        )
    }

    pub fn config_dir() -> PathBuf {
        PathBuf::from(host::config_dir())
    }

    pub fn data_dir() -> PathBuf {
        PathBuf::from(host::config_dir())
    }

    pub fn read_dir(dir: impl AsRef<Path>) -> Result<Vec<String>, IoError> {
        host::read_dir(dir.as_ref().to_str().ok_or(IoError::InvalidPath)?)
    }

    pub fn read_file(file: impl AsRef<Path>) -> Result<Vec<u8>, IoError> {
        host::read_file(file.as_ref().to_str().ok_or(IoError::InvalidPath)?)
    }

    pub fn rank(query: &str, items: &[ListItem], weights: Weights) -> Vec<host::ListItem> {
        host::rank(query, items, weights)
    }
}

pub use __raw_bindings::ListItem;
impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            description: String::new(),
            metadata: String::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_metadata(mut self, meta: impl Into<String>) -> Self {
        self.metadata = meta.into();
        self
    }

    pub fn with_icon(mut self, icon: Option<impl Into<String>>) -> Self {
        self.icon = icon.map(Into::into);
        self
    }
}

pub use __raw_bindings::Weights;
impl Default for Weights {
    fn default() -> Self {
        Self {
            title: 1.0,
            description: 0.0,
            metadata: 0.0,
            frequency: 3.0,
        }
    }
}

pub use __raw_bindings::SelectionRange;
impl SelectionRange {
    /// Sets both the start and end bound to the provided index.
    pub fn at(index: u16) -> Self {
        Self {
            lower_bound: index,
            upper_bound: index,
        }
    }

    /// Selects the entire query.
    pub fn all() -> Self {
        Self {
            lower_bound: 0,
            upper_bound: u16::MAX,
        }
    }

    /// Sets the start and end to `0`.
    pub fn start() -> Self {
        Self::at(0)
    }

    pub fn end() -> Self {
        Self::at(u16::MAX)
    }
}

pub use __raw_bindings::InputLine;
impl InputLine {
    /// Sets the input to the provided query and with the cursor placed
    /// at the end.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            range: SelectionRange::end(),
        }
    }

    pub fn select(mut self, sel: SelectionRange) -> Self {
        self.range = sel;
        self
    }
}

pub use __raw_bindings::{
    Capture, DeferredAction, DeferredResult, IoError, PluginAction, ProcessOutput, QueryResult,
};
pub use anyhow;
use anyhow::{bail, Result};

pub trait Plugin {
    fn query(query: String) -> Result<QueryResult>;

    #[allow(unused_variables)]
    fn handle_deferred(query: String, result: DeferredResult) -> Result<QueryResult> {
        bail!("plugin has no deferred action handler")
    }

    fn activate(selected: ListItem) -> Result<impl IntoIterator<Item = PluginAction>>;

    #[allow(unused_variables)]
    fn complete(query: String, selected: ListItem) -> Result<Option<InputLine>> {
        Ok(None)
    }
}

impl<T> __raw_bindings::Guest for T
where
    T: Plugin,
{
    fn query(query: String) -> Result<QueryResult, String> {
        Self::query(query).map_err(stringify_error)
    }

    fn handle_deferred(query: String, result: DeferredResult) -> Result<QueryResult, String> {
        Self::handle_deferred(query, result).map_err(stringify_error)
    }

    fn activate(selected: ListItem) -> Result<Vec<PluginAction>, String> {
        match Self::activate(selected) {
            Ok(list) => Ok(list.into_iter().collect()),
            Err(e) => Err(stringify_error(e)),
        }
    }

    fn complete(query: String, selected: ListItem) -> Result<Option<InputLine>, String> {
        Self::complete(query, selected).map_err(stringify_error)
    }
}

#[macro_export]
macro_rules! register {
    ($plugin:ident) => {
        $crate::__raw_bindings::export!($plugin with_types_in $crate::__raw_bindings);
    };
}

fn stringify_error(err: anyhow::Error) -> String {
    err.chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
