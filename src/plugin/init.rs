//! Initialises plugins using the wasmtime runtime.

use std::{path::Path, sync::LazyLock};

use tracing::instrument;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtxBuilder};

use crate::plugin::{bindings, host::State};

static ENGINE: LazyLock<Engine> = LazyLock::new(|| {
    let mut config = Config::new();
    config
        .wasm_component_model(true)
        .async_support(true)
        .debug_info(true);
    Engine::new(&config).unwrap()
});

static LINKER: LazyLock<Linker<State>> = LazyLock::new(|| {
    let mut linker = Linker::<State>::new(&ENGINE);
    bindings::add_host_to_linker(&mut linker, |s| s).unwrap();
    wasmtime_wasi::add_to_linker_async(&mut linker).unwrap();
    linker
});

#[instrument(fields(file=file.as_ref().to_str()), level="info")]
pub async fn initialise_plugin(
    file: impl AsRef<Path>,
) -> Result<(bindings::Plugin, Store<State>), wasmtime::Error> {
    let component = Component::from_file(&ENGINE, file)?;

    let ctx = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env()
        .inherit_network()
        .preopened_dir(Path::new("/"), "/", DirPerms::READ, FilePerms::READ)?
        .build();

    let mut store = Store::new(
        &ENGINE,
        State {
            ctx,
            table: ResourceTable::new(),
        },
    );

    let instance = bindings::Plugin::instantiate_async(&mut store, &component, &LINKER).await?;
    Ok((instance, store))
}
