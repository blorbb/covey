# covey

A general-purpose desktop command palette.

(todo: demo video)

Similar to [rofi](https://github.com/davatorium/rofi), [Ulauncher](https://github.com/Ulauncher/Ulauncher), [Albert](https://github.com/albertlauncher/albert) and the likes with cross-platform support (including Wayland) powered by an [RPC](https://en.wikipedia.org/wiki/Remote_procedure_call) language-agnostic plugin system.

## Features

-   Fast feedback - low latency, highly concurrent interactions with plugins through RPC. No debouncing by default, results are almost instantaneous. Plugins can perform expensive initialisation first to speed up response times while typing.
    -   Long computations (like web requests) are performed concurrently and updates the UI as soon as a newer request is complete.
-   _Everything_ is a [plugin](https://github.com/blorbb/covey-plugins) - anything can be added, removed or changed as you like.
-   Plugin system based on [protobuf](https://protobuf.dev/) for cross-language plugin support, with easy-to-use bindings for [Rust](./covey-plugin/) (more in the future!).
-   [Separate backend](./covey/) - you can write your own [front end](./covey-tauri/) if you wish!

### Planned Features

-   GUI for editing plugin settings.
    -   Allow plugins to define setting interfaces.
-   Support performing actions with arbitrary keyboard shortcuts.
-   Support changing the menu without user interaction (e.g. for a [htop](https://htop.dev/) viewer that automatically updates).
-   Plugin bindings for other languages.
-   Store frequency/recency data of list items.

## Configuration

All configuration is stored in a `covey` folder of the [config directory](https://docs.rs/dirs/latest/dirs/fn.config_dir.html) for your OS. The `covey` folder contains `config.toml` - see below for details.

Plugins have their data stored in a `covey/plugins` folder of the [data directory](https://docs.rs/dirs/latest/dirs/fn.data_dir.html) for your OS. The `covey/plugins` folder contains a folder for every installed plugin. Each of these folders contains a binary with the same name as the plugin for the executable, a `manifest.toml`, and a `data.db` sqlite database.

### Configuration Format

Most configuration is stored in `covey/config.toml` in the [TOML file format](https://toml.io). An example configuration is shown below.

```toml
# order matters!
# plugins defined at the top will try match their
# prefix first, before plugins defined below.

[[plugins]]
name = "open" # must be the same as the name of the binary
prefix = "@"  # prefix to use to activate this plugin

# additional plugin-specific configuration can be
# defined too, under the `config` table within the plugin.
[plugins.config]
std = { name = "Rust stdlib", url = "https://doc.rust-lang.org/std/?search=%s" }
g = { name = "Google", url = "https://www.google.com/search?q=%s" }

# next plugin definition
[[plugins]]
name = "qalc"
prefix = "="

# last plugin definition: this will only be activated
# if none of the other prefixes match the query.
[[plugins]]
name = "app-switcher"
prefix = ""
```

## Plugins

See more details about how to write your own plugin in [`covey-plugin`](./covey-plugin/). A collection of plugins can be found at [`blorbb/covey-plugins`](https://github.com/blorbb/covey-plugins).

To install a plugin, move the binary file to `plugins/` in the covey config folder (`~/.config/covey/plugins/` on Linux). You then need to register the plugin in `config.toml`, as shown above.

## Desktop Environment Support

If covey doesn't work on your desktop environment, please open an issue with details!

### Wayland

Wayland is much more restrictive on how applications are allowed to style their windows. You will likely have to make manual overrides to your compositor for covey to look correct. The steps will likely be similar to [KDE Plasma below](#kde-plasma-6) (please open an issue/PR if you got a working solution for your desktop environment).

### KDE Plasma 6

To let the launcher run with proper focusing:

-   Go to settings > Window Management > Window Rules
-   Click `Add New...`
-   Description: "covey"
-   Window class: "covey"
-   Click `Add Property...` and add all of the following:
    -   Keep above other windows: Apply initially
    -   Focus stealing prevention: Force, None
    -   Focus protection: Force, None
-   Click `Apply`.
