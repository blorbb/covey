# Covey

Yet another general-purpose desktop command palette / launcher.

Extremely WIP and not ready to be used by anyone.
Perhaps check out [Rofi](https://github.com/davatorium/rofi)/[Ulauncher](https://github.com/Ulauncher/Ulauncher)/[Albert](https://github.com/albertlauncher/albert)/[Vicinae](https://github.com/vicinaehq/vicinae)/[Sherlock](https://github.com/Skxxtz/sherlock)/[Walker](https://github.com/abenz1267/walker)/[Centerpiece](https://github.com/friedow/centerpiece)/[Lucien](https://github.com/Wachamuli/lucien)/... for something a bit more done.

[covey-demo.webm](https://github.com/user-attachments/assets/f8d05b93-eca7-440b-ab09-336eb04e4593)

## Why does this exist?

Because I felt like making it.
The primary goal is for the codebase to have a (somewhat) simple architecture and make it easy to create plugins.

## Configuration

Covey's settings are stored in the `covey` folder of your OS's [config directory](https://docs.rs/dirs/latest/dirs/fn.config_dir.html). This contains the main `config.toml` file that configures the entire application.

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

To install a plugin, place the plugin's binary and `manifest.toml` within the plugin data folder (`<data dir>/covey/plugins/<plugin id>`).
See the above folder structure for an example.
The plugin needs to be enabled within Covey's `config.toml`.

## Desktop environment support

Covey should hopefully be cross-platform.

### Wayland

Due to Wayland’s stricter window styling rules, you might need to adjust your compositor settings to ensure Covey displays correctly. The process will likely be similar to the steps outlined for [KDE Plasma](#kde-plasma-6) below.

### KDE Plasma 6

To ensure the window is correctly placed and focused:

- Go to Settings > Window Management > Window Rules
- Click `Add New...`
- Set description: "Covey"
- Set window class: "covey"
- Click `Add Property...` and add all of the following:
  - Keep above other windows: Apply initially
  - Focus stealing prevention: Force, None
  - Focus protection: Force, None
- Click `Apply` to save your changes.
