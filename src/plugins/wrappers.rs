use std::fmt;

use color_eyre::eyre::{bail, eyre, Result};
use eframe::egui::mutex::Mutex;
use wasmtime::Store;

use crate::{config::PluginConfig, PLUGINS_DIR};

use super::{
    bindings::{self, Qpmu},
    PluginActivationAction,
};

#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub description: String,
    pub metadata: String,
    pub plugin: Plugin,
}

impl fmt::Debug for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ListItem")
            .field("title", &self.title)
            .field("description", &self.description)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl ListItem {
    fn from_item_and_plugin(item: bindings::ListItem, plugin: Plugin) -> Self {
        Self {
            title: item.title,
            description: item.description,
            metadata: item.metadata,
            plugin,
        }
    }

    fn from_many_and_plugin(items: Vec<bindings::ListItem>, plugin: Plugin) -> Vec<Self> {
        items
            .into_iter()
            .map(|item| Self::from_item_and_plugin(item, plugin))
            .collect()
    }

    pub fn activate(self) -> Result<Vec<PluginActivationAction>> {
        self.plugin.clone().activate(self)
    }
}

impl From<ListItem> for bindings::ListItem {
    fn from(value: ListItem) -> Self {
        Self {
            title: value.title,
            description: value.description,
            metadata: value.metadata,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Plugin(&'static Mutex<PluginInner>);

impl Plugin {
    pub fn from_config(config: PluginConfig) -> color_eyre::Result<Self> {
        let boxed = Box::new(Mutex::new(PluginInner::from_config(config)?));
        Ok(Self(Box::leak(boxed)))
    }

    pub fn try_call_input(&self, query: &str) -> Option<color_eyre::Result<Vec<ListItem>>> {
        Some(
            self.0
                .lock()
                .try_call_input(query)?
                .map(|item| ListItem::from_many_and_plugin(item, *self)),
        )
    }

    pub fn activate(&self, item: ListItem) -> color_eyre::Result<Vec<PluginActivationAction>> {
        self.0.lock().call_activate(&bindings::ListItem::from(item))
    }
}

pub struct PluginInner {
    plugin: Qpmu,
    store: Store<wasm::State>,
    config: PluginConfig,
}

impl fmt::Debug for PluginInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginInner")
            .field("config", &self.config)
            .finish()
    }
}

impl PluginInner {
    fn from_config(config: PluginConfig) -> color_eyre::Result<Self> {
        // wasmtime error is weird, need to do this match
        let (plugin, store) = match wasm::initialise_plugin(
            PLUGINS_DIR.join(format!("{}.wasm", config.name.replace('-', "_"))),
        ) {
            Ok((p, s)) => (p, s),
            Err(e) => bail!("failed to load plugin: {e}"),
        };

        Ok(Self {
            plugin,
            store,
            config,
        })
    }

    /// Calls the input function if the query matches the prefix.
    fn try_call_input(&mut self, query: &str) -> Option<Result<Vec<bindings::ListItem>>> {
        query
            .strip_prefix(&self.config.prefix)
            .map(|q| self.call_input(q))
    }

    fn call_input(&mut self, input: &str) -> color_eyre::Result<Vec<bindings::ListItem>> {
        self.plugin
            .call_input(&mut self.store, input)
            .map_err(|e| eyre!(Box::new(e)))
    }

    fn call_activate(
        &mut self,
        item: &bindings::ListItem,
    ) -> color_eyre::Result<Vec<PluginActivationAction>> {
        self.plugin
            .call_activate(&mut self.store, item)
            .map_err(|e| eyre!(Box::new(e)))
    }
}

pub mod wasm {
    use std::{path::Path, process::Stdio};

    use wasmtime::{
        component::{Component, Linker},
        Config, Engine, Store,
    };
    use wasmtime_wasi::{DirPerms, FilePerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

    use crate::plugins::bindings::{
        self,
        host::{Capture, Output, SpawnError},
        Qpmu,
    };

    pub(super) fn initialise_plugin(
        file: impl AsRef<Path>,
    ) -> Result<(Qpmu, Store<State>), wasmtime::Error> {
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
        Ok((instance, store))
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
            eprintln!("calling command {cmd} {args:?}, capturing {capture:?}");
            let mut command = std::process::Command::new(cmd);
            command.args(args);
            // disallow stdin
            command.stdin(Stdio::null());
            if capture.contains(Capture::STDOUT) {
                command.stdout(Stdio::piped());
            }
            if capture.contains(Capture::STDERR) {
                command.stderr(Stdio::piped());
            }

            Ok(Output::from(command.spawn()?.wait_with_output()?))
        }
    }
}
