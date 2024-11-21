# qpmu-api

Rust bindings for a qpmu plugin.

See a bunch of examples in [`qpmu-plugins`](https://github.com/blorbb/qpmu-plugins). [Protobuf definitions](https://protobuf.dev/) are in [`proto/plugin.proto`](./proto/plugin.proto), if you want to make bindings for another language.

## Usage

Example implementation below.

Add `qpmu-api` to `Cargo.toml`. You may also need `tokio` and `serde`.

-   `new` will be called on initialisation. You can perform some expensive / config dependent stuff here.
-   `query` is called on every keypress, when the input changes. Return a list of items to be displayed.
    -   The list of items may not actually be set on the GUI. Future queries may finish earlier, in which case those queries will set the list and this one will be ignored.
    -   Avoid causing noticable side-effects here.
-   `activate` is called when a list item is activated (press <kbd>Enter</kbd> or click). It is fine to cause noticable side-effects here.
-   `complete` is called to perform some kind of completion on the list item (e.g. finishing part of the query).
    -   It should return a new `Input` (contents and selection) or `None` if no change should occur.
    -   Does not need to be implemented, returns `Ok(None)` by default.

```rs
use anyhow::Result;
use qpmu_api::*;
use serde::Deserialize;

fn main() {
    // This will run the server
    qpmu_api::main::<MyPlugin>()
}

// If you need any data from the configuration, add
// `serde` and `toml` to dependencies.
// Otherwise, a unit struct `struct MyPlugin;` will work.
#[derive(Deserialize)]
struct MyPlugin {
    some_data: Vec<String>,
}

impl Plugin for MyPlugin {
    async fn new(config: String) -> Result<Self> {
        let some_data = toml::from_str(&config)?;
        Ok(Self { some_data })
    }

    async fn query(&self, query: String) -> Result<Vec<ListItem>> {
        let list_items = /* ... */;
        // ...
        Ok(list_items)
    }

    async fn activate(
        &self,
        ActivationContext { item, .. }: ActivationContext
    ) -> Result<Vec<Action>> {
        // ...
        // Return a vec of actions to perform. These will be run in order.
        Ok(vec![
            Action::Close, // You will usually want to close the app.
            Action::RunShell(item.metadata),
        ])
    }

    async fn complete(
        &self,
        ActivationContext { item, query, .. }: ActivationContext
    ) -> Result<Option<Input>> {
        // ...
        Ok(Some(Input::new("new input")))
    }
}
```

`qpmu-api` also gives access to an sqlite pool at `qpmu_api::sql::pool()`. This is a connection to the plugin's sqlite database. By default, it stores a `activations` table that updates on every activation.

The `activations` table has the following schema:

```sql
CREATE TABLE activations (
    id          INTEGER PRIMARY KEY,
    title       TEXT NOT NULL UNIQUE,
    frequency   INTEGER NOT NULL,
    last_use    DATETIME NOT NULL
);
```

The plugin can store any other tables it wants here.

## Bindings for other languages

Currently, only Rust bindings exist. Bindings for other languages may be made in the future.

The program needs to run a server with RPC services that follow the protobuf definition.

-   When initialising, it needs to connect to a port in loopback (`[::1]`) and print `PORT:<port>` to stdout (e.g. `PORT:12345`).
    -   The qpmu backend will then connect to `http://[::1]:<port>`.
-   If an error occurs during initialisation, you should print `ERROR:<message>` to stdout and exit. The message can be over multiple lines.
-   qpmu will give two arguments to the binary: the first is an sqlite URL that the plugin can connect to for storing any kind of data, and the second is a string of TOML that is the plugin's extra options.
