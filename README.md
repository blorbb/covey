# Covey

A general-purpose desktop command palette / app launcher.

[covey-demo.webm](https://github.com/user-attachments/assets/20787700-626e-4342-9777-4436cc85dbe4)

Inspired by tools like [rofi](https://github.com/davatorium/rofi), [Ulauncher](https://github.com/Ulauncher/Ulauncher), [Albert](https://github.com/albertlauncher/albert) and the likes.

## Key Features

-   **Instant Feedback**
    Low latency and highly concurrent interactions with plugins via gRPC.

-   **Full Flexibility**
    _Everything_ is a plugin. Pick your favourites to create your ideal launcher.

-   **Simple Plugin System**
    Powered by [protobuf](https://protobuf.dev/) for cross-language plugin support, with easy-to-use bindings for [Rust](./covey-plugin/) (more in the future!).

-   **Customisable UI**
    UI can be customised with web CSS.

-   **Modular Architecture**
    With a simple backend to interact with plugins, you can easily write another frontend if you so wish to!

## Configuration

Covey's settings are stored in the `covey` folder of your OS's [config directory](https://docs.rs/dirs/latest/dirs/fn.config_dir.html). This contains the main `config.toml` file that configures the entire application.

(TODO: make all configuration accessible through the GUI)

Plugins keep data stored in the `covey/plugins` folder of your OS's [data directory](https://docs.rs/dirs/latest/dirs/fn.data_dir.html). Each plugin is in a subfolder with a binary and `manifest.toml`. Plugins will usually store extra data in this folder.

Example folder structure with default Linux paths:

```sh
~/.config
└── covey
    └── config.toml

~/.local/share
└── covey
    └── plugins
        ├── app-switcher          # plugin id
        │   ├── activations.json  # extra data
        │   ├── manifest.toml     # manifest
        │   └── app-switcher      # binary with same name as id
        ├── qalc
        │   ├── manifest.toml
        │   └── qalc
        ...
```

## Plugins

Find a collection of plugins at [`blorbb/covey-plugins`](https://github.com/blorbb/covey-plugins).
To create your own plugin, check out the [`covey-plugin`](./covey-plugin/) documentation.

To install a plugin, place the plugin's binary and `manifest.toml` within the plugin data folder (`<data dir>/covey/plugins/<plugin id>`). See the above folder structure for an example.

(TODO: auto detection of plugins)

## Desktop Environment Support

Covey is built to be cross-platform. If you encounter any problems, please open an issue!

### Wayland

Due to Wayland’s stricter window styling rules, you might need to adjust your compositor settings to ensure Covey displays correctly. The process will likely be similar to the steps outlined for [KDE Plasma](#kde-plasma-6) below. If you discover a solution for your desktop environment, contributions via issues or pull requests are welcome!

### KDE Plasma 6

To ensure the window is correctly placed and focused:

-   Go to Settings > Window Management > Window Rules
-   Click `Add New...`
-   Set description: "Covey"
-   Set window class: "covey"
-   Click `Add Property...` and add all of the following:
    -   Keep above other windows: Apply initially
    -   Focus stealing prevention: Force, None
    -   Focus protection: Force, None
-   Click `Apply` to save your changes.
