use bindings::Qpmu;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

impl WasiView for State {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

pub struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

pub mod bindings {
    wasmtime::component::bindgen!({world: "qpmu", path: "./qpmu-api/wit/world.wit"});
}

pub fn initialise_plugin(file: &str) -> Result<(Store<State>, Qpmu), wasmtime::Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).debug_info(true);
    let engine = Engine::new(&config)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;

    let builder = WasiCtxBuilder::new().inherit_stdio().build();
    let mut store = Store::new(
        &engine,
        State {
            ctx: builder,
            table: ResourceTable::new(),
        },
    );

    let component = Component::from_file(&engine, file)?;
    let instance = Qpmu::instantiate(&mut store, &component, &linker)?;
    Ok((store, instance))
}
