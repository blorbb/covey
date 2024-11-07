pub mod bindings {
    use qpmu::plugin::host;
    use std::{io, process::Stdio};

    wasmtime::component::bindgen!({
        world: "plugin",
        path: "./qpmu-api/wit",
        with: {
            "wasi": wasmtime_wasi::bindings
        },
        async: true,
    });

    impl From<io::Error> for host::SpawnError {
        fn from(value: io::Error) -> Self {
            use host::SpawnError;
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

    impl DeferredAction {
        /// Completes this deferred action.
        pub(super) async fn run(&self) -> DeferredResult {
            match self {
                DeferredAction::Spawn((cmd, args)) => {
                    DeferredResult::ProcessOutput(Self::spawn(cmd, args).await)
                }
            }
        }

        async fn spawn(cmd: &str, args: &[String]) -> Result<host::Output, host::SpawnError> {
            Ok(tokio::process::Command::new(cmd)
                .args(args)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
                .wait_with_output()
                .await?
                .into())
        }
    }
}

pub use bindings::PluginAction as PluginActivationAction;
use color_eyre::eyre::Result;
use futures::{stream::FuturesOrdered, StreamExt};
use tokio::{fs, sync::OnceCell};

#[derive(Debug)]
pub enum PluginEvent {
    SetList(Vec<ListItem>),
    Activate(Vec<PluginActivationAction>),
}

#[derive(Debug)]
pub enum UiEvent {
    InputChanged { query: String },
    Activate { item: ListItem },
}

pub async fn process_ui_event(ev: UiEvent) -> Result<PluginEvent> {
    static CELL: OnceCell<Vec<Plugin>> = OnceCell::const_new();
    async fn cell_init() -> Vec<Plugin> {
        let plugins = &*PLUGINS_DIR;
        if !plugins.is_dir() {
            fs::create_dir_all(plugins)
                .await
                .expect("could not create qpmu/plugins directory");
        }

        let config = Config::read().await.unwrap();

        config
            .plugins
            .into_iter()
            .inspect(|p| eprintln!("loading plugin {}", p.name))
            .map(|p| async move {
                Plugin::from_config(p.clone())
                    .await
                    .inspect_err(|e| eprintln!("{e}"))
                    .ok()
            })
            .collect::<FuturesOrdered<_>>()
            .filter_map(|x| async move { x })
            .collect::<Vec<_>>()
            .await
    }

    match ev {
        UiEvent::InputChanged { query } => {
            for plugin in CELL.get_or_init(cell_init).await {
                if let Some(stripped) = query.strip_prefix(&plugin.prefix().await) {
                    if let Some(list) = plugin.complete_query(stripped).await? {
                        return Ok(PluginEvent::SetList(list));
                    }
                }
            }
            Ok(PluginEvent::SetList(vec![]))
        }

        UiEvent::Activate { item } => Ok(PluginEvent::Activate(item.activate().await?)),
    }
}

mod wrappers;
pub use wrappers::{ListItem, Plugin};

use crate::{config::Config, PLUGINS_DIR};
