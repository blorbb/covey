
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

## Styling

[GTK4 CSS properties](https://docs.gtk.org/gtk4/css-properties.html) can be used to style the launcher. Styles are read from `style.css` in the config directory (`~/.config/qpmu` on Linux).

Class structure:
```
┌─.window────────────────────────────────────────────────────────────┐
│┌─.main-entry──────────────────────────────────────────────────────┐│
││                                                                  ││
││                                                                  ││
││                                                                  ││
│└──────────────────────────────────────────────────────────────────┘│
│┌─.main-scroller───────────────────────────────────────────────────┐│
││┌─.main-list─────────────────────────────────────────────────────┐││
│││                                                                │││
│││   ┌─.list-item─────────────────────────────────────────────┐   │││
│││   │ ┌─.list-item-hbox┬───────────────────────────────────┐ │   │││
│││   │ │                │                                   │ │   │││
│││   │ │   ┌─.list──┐   │   ┌─.list-item-vbox───────────┐   │ │   │││
│││   │ │   │ -item  │   │   │ .list-item-title          │   │ │   │││
│││   │ │   │ -icon  │   │   ├───────────────────────────┤   │ │   │││
│││   │ │   │        │   │   │ .list-item-description    │   │ │   │││
│││   │ │   └────────┘   │   └───────────────────────────┘   │ │   │││
│││   │ │                │                                   │ │   │││
│││   │ └────────────────┴───────────────────────────────────┘ │   │││
│││   └────────────────────────────────────────────────────────┘   │││
│││                                                                │││
│││                                                                │││
│││                                                                │││
│││                                                                │││
│││                                                                │││
│││                                                                │││
││└────────────────────────────────────────────────────────────────┘││
│└──────────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────────┘
```

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
