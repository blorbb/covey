# covey-config

Type definitions for covey's configuration and plugin manifests.

See the configuration schema at [src/config.rs](./src/config.rs), and the manifest schema at [src/manifest.rs](./src/manifest.rs).

## Configuration format

```toml
# global application configuration is under `app`
[[app.icon-themes]]
kind = "system"
name = "hicolor"

[[app.icon-themes]]
kind = "iconify-icon"
name = "ph"

# plugin configuration:
# order matters!
# plugins defined at the top will try match their
# prefix first, before plugins defined below.

[[plugins]]
id = "open" # must be the same as the name of the binary
prefix = "@"  # prefix to use to activate this plugin

# additional plugin-specific configuration can be
# defined too, under the `config` table within the plugin.
[plugins.config.urls]
std = { name = "Rust stdlib", url = "https://doc.rust-lang.org/std/?search=%s" }
g = { name = "Google", url = "https://www.google.com/search?q=%s" }

# next plugin definition
[[plugins]]
id = "qalc"
prefix = "="

[[plugins]]
id = "app-switcher"
prefix = ""
```
