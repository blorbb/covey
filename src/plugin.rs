//! API for interacting with plugins.

mod bindings;
pub mod event;
mod host;
mod init;
mod wrappers;

pub use bindings::{Action, InputLine};
use color_eyre::eyre::Result;
use event::{PluginEvent, UiEvent};
use futures::{stream::FuturesOrdered, StreamExt};
use tokio::{fs, sync::OnceCell};
pub use wrappers::{ListItem, Plugin};

use crate::{config::Config, PLUGINS_DIR};

/// Asynchronously processes a UI event and returns the result.
pub async fn process_ui_event(ev: UiEvent) -> Result<PluginEvent> {
    match ev {
        UiEvent::InputChanged { query } => {
            // run plugins in order, skipping if their prefix doesn't match or
            // the plugin returns a `skip` action.
            for plugin in PLUGINS.get_or_init(cell_init).await {
                if let Some(stripped) = query.strip_prefix(&plugin.prefix()) {
                    if let Some(list) = plugin.complete_query(stripped).await? {
                        return Ok(PluginEvent::SetList(list));
                    }
                }
            }

            // No plugin activated, empty list.
            Ok(PluginEvent::SetList(vec![]))
        }

        UiEvent::Activate { item } => Ok(PluginEvent::Run {
            plugin: item.plugin(),
            actions: item.activate().await?,
        }),

        UiEvent::Complete { query, item } => Ok(PluginEvent::Run {
            plugin: item.plugin(),
            actions: item
                .complete(&query)
                .await?
                .map(Action::SetInputLine)
                .as_slice()
                .to_vec(),
        }),
    }
}

static PLUGINS: OnceCell<Vec<Plugin>> = OnceCell::const_new();

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
