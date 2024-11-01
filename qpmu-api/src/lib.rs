pub mod bindings {
    wit_bindgen::generate!({
        path: "./wit/world.wit",
        world: "qpmu",
        pub_export_macro: true,
        export_macro_name: "export",
    });
}

pub mod host {
    use crate::bindings::host;

    pub use host::{Capture, Output, SpawnError};

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

pub use bindings::{export, Guest as Plugin, ListItem, PluginAction};
