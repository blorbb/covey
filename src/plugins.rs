use std::{path::Path, process::Stdio};

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

pub mod bindings {
    use std::io;

    use host::SpawnError;

    wasmtime::component::bindgen!({world: "qpmu", path: "./qpmu-api/wit/world.wit"});

    impl From<io::Error> for SpawnError {
        fn from(value: io::Error) -> Self {
            use io::ErrorKind as E;
            match value.kind() {
                E::NotFound => SpawnError::NotFound,
                E::PermissionDenied => SpawnError::PermissionDenied,
                E::BrokenPipe => SpawnError::BrokenPipe,
                E::WouldBlock => SpawnError::WouldBlock,
                E::InvalidInput => SpawnError::InvalidInput,
                E::TimedOut => SpawnError::TimedOut,
                E::Interrupted => SpawnError::Interrupted,
                E::Unsupported => SpawnError::Unsupported,
                E::UnexpectedEof => SpawnError::UnexpectedEof,
                E::OutOfMemory => SpawnError::OutOfMemory,
                _ => SpawnError::Other(value.to_string()),
            }
        }
    }

    impl From<std::process::Output> for host::Output {
        fn from(value: std::process::Output) -> Self {
            Self {
                exit_code: value.status.code(),
                stdout: value.stdout,
                stderr: value.stderr,
            }
        }
    }
}
use bindings::{
    host::{Capture, Output, SpawnError},
    Qpmu,
};

pub use bindings::{ListItem, PluginAction};

pub struct State {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for State {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl bindings::host::Host for State {
    fn spawn(
        &mut self,
        cmd: String,
        args: Vec<String>,
        capture: Capture,
    ) -> Result<Output, SpawnError> {
        let mut command = std::process::Command::new(cmd);
        command.args(args);
        if capture.contains(Capture::STDOUT) {
            command.stdout(Stdio::piped());
        }
        if capture.contains(Capture::STDERR) {
            command.stderr(Stdio::piped());
        }

        Ok(Output::from(command.spawn()?.wait_with_output()?))
    }
}

pub fn initialise_plugin(file: impl AsRef<Path>) -> Result<Plugin, wasmtime::Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).debug_info(true);
    let engine = Engine::new(&config)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    Qpmu::add_to_linker(&mut linker, |s| s)?;

    let builder = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env()
        .inherit_network()
        .preopened_dir(Path::new("/"), "/", DirPerms::READ, FilePerms::READ)?
        .build();
    let mut store = Store::new(
        &engine,
        State {
            ctx: builder,
            table: ResourceTable::new(),
        },
    );

    let component = Component::from_file(&engine, file)?;
    let instance = Qpmu::instantiate(&mut store, &component, &linker)?;
    Ok(Plugin::new(instance, store))
}

pub struct Plugin {
    plugin: Qpmu,
    store: Store<State>,
}

impl Plugin {
    fn new(plugin: Qpmu, store: Store<State>) -> Self {
        Self { plugin, store }
    }

    pub fn call_input(&mut self, input: &str) -> Result<Vec<ListItem>, wasmtime::Error> {
        self.plugin.call_input(&mut self.store, input)
    }

    pub fn call_activate(&mut self, item: &ListItem) -> Result<Vec<PluginAction>, wasmtime::Error> {
        self.plugin.call_activate(&mut self.store, item)
    }
}
