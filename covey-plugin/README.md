# covey-plugin

Rust bindings for a covey plugin.

See a bunch of examples in [`covey-plugins`](https://github.com/blorbb/covey-plugins). [Protobuf definitions](https://protobuf.dev/) are in [`covey-proto/proto/plugin.proto`](../covey-proto/proto/plugin.proto), if you want to make bindings for another language.

## Usage

Example implementation below. For more details, see this crate's documentation.

Add `covey-plugin` as a dependency to `Cargo.toml`.

```rs
use covey_plugin::{clone_async, Input, List, ListItem, Plugin, Result};

// store any configuration / cache here.
// if none are needed, a unit struct `struct Open;` will work.
struct Open {
    urls: Vec<(String, urls::UrlsValue)>,
}

impl Plugin for Open {
    // `Config` is a struct generated by `include_manifest!()` below.
    type Config = Config;

    async fn new(config: Self::Config) -> Result<Self> {
        Ok(Self {
            urls: config.urls.into_iter().collect(),
        })
    }

    async fn query(&self, query: String) -> Result<List> {
        // do some logic here...
        Ok(List::new(vec![
            ListItem::new("list item title")
                .with_description("a description")
                // define callbacks that will be triggered when
                // the user activates a command.
                // this method is defined an extension trait
                // generated by `include_manifest!()`.
                .on_activate(async move |menu| {
                    menu.close();
                    menu.copy("wahoo");
                    Ok(())
                })
                // you will often need to clone values into
                // the callback. use the helper macro to make
                // this easier.
                .on_complete(clone_async!(query, |menu| {
                    menu.set_input(Input::new(query));
                    Ok(())
                }))
        ]))
    }
}

// generates types from reading `../manifest.toml`.
// also defines an extension trait for methods like `.on_activate`.
covey_plugin::include_manifest!();

fn main() {
    // this will run the server.
    // it requires the name of the binary to be passed in.
    // use `env!` to get it from the cargo project.
    covey_plugin::run_server::<Open>(env!("CARGO_BIN_NAME"))
}
```

## Bindings for other languages

Currently, only Rust bindings exist. Bindings for other languages may be made in the future.

The program needs to run a server with RPC services that follow the protobuf definition.

-   When initialising, it needs to connect to a port in loopback (`[::1]`) and print the port to stdout (e.g. `12345`).
    -   The covey backend will then connect to `http://[::1]:<port>`.
-   If an error occurs during initialisation, you should exit with a non-zero exit code.
-   The backend is guaranteed to call and complete the initialise function before any other functions are called.
