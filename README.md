
## Window Manager Support

### KDE Plasma 6

- Go to settings > Window Management > Window Rules
- Click `Add New...`
- Description: "qpmu"
- Window class: "qpmu"
- Click `Add Property...` and add all of the following:
  - Keep above other windows: Apply initially
  - Focus stealing prevention: Force, None
  - Focus protection: Force, None
- Click `Apply`.

## Plugins

```sh
cargo install cargo-component --locked
rustup target add wasm32-wasi
```

To build the plugin:
```sh
cargo component build
```

To create a new plugin:
```sh
cargo component new --lib PLUGIN_NAME
```
- Remove the `wit` directory.
- Change the `Cargo.toml` key `package.metadata.component.package = "blorbb:qpmu"`.

Replace `lib.rs` with the following template:
```rs
use qpmu_api::{export, Plugin};

struct PLUGIN_NAME;

impl Plugin for PLUGIN_NAME {
    fn test(name: String) -> Vec<String> {
        vec![name]
    }
}

export!(PLUGIN_NAME with_types_in qpmu_api::bindings);
```