
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
rustup target add wasm32-wasip2
```

To build the plugin:
```sh
cargo build --release --target wasm32-wasip2
```

To create a new plugin:
```sh
cargo new --lib PLUGIN_NAME
```

Then replace `Cargo.toml` with:
```toml
[package]
...

[dependencies]
qpmu-api = "0.1"

[lib]
crate-type = ["cdylib"]

[profile.release]
codegen-units = 1
opt-level = "s"
debug = false
strip = true
lto = true
```

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