pub mod bindings {
    wit_bindgen::generate!({
        path: "./wit/world.wit",
        world: "qpmu",
        pub_export_macro: true,
        export_macro_name: "export",
    });
}

pub use bindings::{export, host, Guest as Plugin, ListItem, PluginAction};
