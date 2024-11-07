pub mod bindings {
    wit_bindgen::generate!({
        // path: "./wit",
        world: "plugin",
        pub_export_macro: true,
        export_macro_name: "export",
        generate_all,
    });
}

pub mod host {
    pub use super::bindings::qpmu::plugin::host::{self, Capture, Output, SpawnError};

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

pub use bindings::ListItem;
impl bindings::ListItem {
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

pub use bindings::wasi;
pub use bindings::{export, Guest as Plugin, PluginAction};
