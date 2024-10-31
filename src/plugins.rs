use std::process::Stdio;

use bindings::{
    host::{Capture, Output, SpawnError},
    Qpmu,
};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

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

pub fn initialise_plugin(file: &str) -> Result<(Store<State>, Qpmu), wasmtime::Error> {
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
    Ok((store, instance))
}
