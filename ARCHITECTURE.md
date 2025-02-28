# Architecture

WIP

## Terminology

-   **Frontend** - the UI for Covey. Handles windowing and user events.
-   **Backend** / **Host** - a simple API that allows communication between the frontend and plugins.
-   **Plugin** - a mini program which provides list items given some text **input** taken from the frontend.
-   **Command** - a callback that is run on a list item provided by the plugin. When a command is **activated** by a hotkey, the selected list item runs it's callback.
-   **Config file** - settings of the entire Covey app. Stored in Covey's config directory, e.g. `~/.config/covey/config.toml` on Linux. This consists of two main parts:
    -   **App settings** - global settings that apply for the entire app, like styling and layouts.
    -   **Plugins** - a list of plugins with further settings, split into two sections:
        -   **Plugin commands** - change the hotkey(s) used to activate a command.
        -   **Plugin settings** - change any other settings specific to the plugin, defined by the plugin's **manifest** file.
-   **Plugin manifest** - the **manifest.toml** required for each plugin. The manifest defines metadata about the plugin, as well as a **schema** for the plugin's configuration.

## Crates

-   `covey` - backend/host implementation.

    _public, intended for use by alternative frontends_

-   `covey-schema` - type definitions with `serde::{Serialize, Deserialize}` implementations. Also includes the proc macro implementation of `covey-manifest-macros`.

    _public, intended for use by alternative frontends, alongside `covey`_

-   `covey-plugin` - library for plugin authors to use.

    _public, intended for use by plugin authors_

-   `covey-manifest-macros` - defines the `include_manifest!` proc macro.

    _private, use the macros exposed by `covey-plugin` instead_

-   `covey-tauri` - frontend implementation. Contains 3 sub-packages:

    -   `covey-tauri-types` - types similar to those defined in `covey-types` and `covey`, but made to be serializable and work with TypeScript.
    -   `src` - SvelteKit frontend, interfaces with `src-tauri`.
    -   `src-tauri` - Tauri backend, interfaces with `covey`, handling type conversions and interactions to the frontend.

    _private and unpublished, intended to be used as a binary only_

## Why web/Tauri frontend?

It's not perfect, but it's good enough.

**Pros:**

-   **Supports system tray** - This is a must-have to ensure fast startup and to maintain state.
-   **Fast enough** - Since the app is just shown/hidden to the system tray, there is basically zero latency.
-   **Powerful layout and styling options** - Super easy for users to style. Not just the colours, but the menu layout as well.
-   **Easy to build** - I don't want to spend super long writing the UI, managing complex state, fixing performance bugs, etc. The web platform just works, with great frameworks to make writing UIs easy.
-   **Cross platform** - Cause it's the web. Tauri has some issues with [performance on Linux](https://github.com/tauri-apps/tauri/issues/3988#issuecomment-1447098957), but I haven't encountered any noticable issues, and this will hopefully be fixed in the future.
-   **Strong native API support** with plugins - like notifications, clipboard, window settings, etc.

**Cons:**

-   The frontend being separated from the system is annoying. It requires the generation of TypeScript types and a bunch of almost-duplicated types to support serialization. It would be much simpler to build a truly native frontend that can depend on `covey` directly without this layer of separation.
-   High memory usage.

### Attempted Alternatives

#### [egui/eframe](https://github.com/blorbb/covey/pull/1)

**Pros:**

-   Managing state is super easy due to being immediate mode. The menu doesn't really maintain state anyways, given that the list is completely replaced on every keypress.
-   Can depend on `covey` directly, no separation of the frontend from the system.

**Cons:**

-   Styling is not as powerful as web CSS.
-   This would be a mostly ideal alternative, if not for one major caveat: the primary framework (eframe) uses winit for windowing. Winit does not support hiding the window on Wayland. eframe also does not support opening/closing the window multiple times without quitting the entire process, the only viable alternative.

#### [GTK / relm4](https://github.com/blorbb/covey/tree/reactive) ([older](https://github.com/blorbb/covey/pull/2))

**Pros:**

-   Can depend on `covey` directly, no separation of the frontend from the system.
-   Stable.

**Cons:**

-   [Poor performance](https://github.com/blorbb/covey/tree/grid-view-rendering) (see latest commit message) - even when trying a virtualised list, rendering the menu with ~50 items takes around 250ms. Probably a skill issue on my end but I can't be bothered debugging this even more.
-   Difficult to manage state - synchronising state between the host and the frontend is annoying, led to many infinite loops.
-   High memory usage.
-   Cannot configure layouts easily with CSS.
