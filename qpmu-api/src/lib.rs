#[doc(hidden)]
pub mod __raw_bindings {
    wit_bindgen::generate!({
        // path: "./wit",
        world: "plugin",
        pub_export_macro: true,
        export_macro_name: "export",
        generate_all,
    });
}

pub mod host {
    pub use super::__raw_bindings::qpmu::plugin::host::{self, Capture, Output, SpawnError};

    pub fn spawn(
        cmd: &str,
        args: impl IntoIterator<Item: AsRef<str>>,
        capture: Capture,
    ) -> Result<Output, SpawnError> {
        host::spawn(
            cmd,
            &args
                .into_iter()
                .map(|a| a.as_ref().to_owned())
                .collect::<Vec<_>>(),
            capture,
        )
    }
}

pub use __raw_bindings::ListItem;
impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
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
}

pub use __raw_bindings::{DeferredAction, DeferredResult, PluginAction, QueryResult};
pub use anyhow;
use anyhow::Result;

pub trait Plugin {
    fn query(query: String) -> Result<QueryResult>;

    #[allow(unused_variables)]
    fn handle_deferred(query: String, result: DeferredResult) -> Result<QueryResult> {
        Ok(QueryResult::Nothing)
    }

    fn activate(selected: ListItem) -> Result<impl IntoIterator<Item = PluginAction>>;
}

impl<T> __raw_bindings::Guest for T
where
    T: Plugin,
{
    fn query(query: String) -> Result<QueryResult, String> {
        Self::query(query).map_err(|e| e.to_string())
    }

    fn handle_deferred(query: String, result: DeferredResult) -> Result<QueryResult, String> {
        Self::handle_deferred(query, result).map_err(|e| e.to_string())
    }

    fn activate(selected: ListItem) -> Result<Vec<PluginAction>, String> {
        match Self::activate(selected) {
            Ok(list) => Ok(list.into_iter().collect()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub use __raw_bindings::wasi;

#[macro_export]
macro_rules! register {
    ($plugin:ident) => {
        $crate::__raw_bindings::export!($plugin with_types_in $crate::__raw_bindings);
    };
}
