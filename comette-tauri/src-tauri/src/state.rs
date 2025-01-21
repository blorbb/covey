use std::sync::OnceLock;

use color_eyre::eyre::Result;
use comette::{Frontend, Host};
pub use comette_tauri_types::{Event, ListItem, ListStyle};
use comette_tauri_types::{Icon, ListItemId};
use tauri::{ipc::Channel, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

use crate::window;

/// Must be initialised exactly once with [`AppState::init`].
pub struct AppState {
    inner: OnceLock<Host>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: OnceLock::new(),
        }
    }

    pub fn init(&self, fe: impl Frontend) -> Result<()> {
        if let Some(existing) = self.inner.get() {
            // this should never run in the real application, but it does
            // in hot module reload.
            tracing::warn!("setting model again");
            existing.set_frontend(fe);
        } else {
            self.inner
                .set(comette::Host::new(fe)?)
                .unwrap_or_else(|_| tracing::warn!("already set up"));
        }

        Ok(())
    }

    /// # Panics
    /// Panics if this has not been initialised yet.
    pub fn host(&self) -> &Host {
        self.inner.get().expect("app state has not been set up")
    }

    pub fn register_list_items(
        &self,
        lis: impl ExactSizeIterator<Item = comette::ListItem>,
    ) -> Vec<ListItem> {
        lis.map(|li| {
            let title = li.title().to_owned();
            let description = li.description().to_owned();
            let icon: Option<Icon> = match li.icon() {
                Some(comette::Icon::Name(name)) => freedesktop_icons::lookup(&name)
                    .with_cache()
                    .with_size(48)
                    .find()
                    .map(|path| Icon::File { path }),
                Some(comette::Icon::Text(text)) => Some(Icon::Text { text }),
                None => None,
            };
            let id = ListItemId {
                local_id: li.id().local_id,
                plugin_name: li.id().plugin.name().to_owned(),
            };

            ListItem {
                title,
                description,
                icon,
                id: id.clone(),
            }
        })
        .collect()
    }

    pub fn find_list_item(&self, id: &ListItemId) -> Option<comette::ListItemId> {
        Some(comette::ListItemId {
            plugin: self.host().plugins().get(&*id.plugin_name)?.clone(),
            local_id: id.local_id,
        })
    }
}

#[derive(Clone)]
pub struct EventChannel {
    pub channel: Channel<Event>,
    pub app: tauri::AppHandle,
}

impl comette::Frontend for EventChannel {
    fn close(&mut self) {
        window::hide_menu(&self.app);
    }

    fn copy(&mut self, str: String) {
        self.app.clipboard().write_text(str).unwrap();
    }

    fn set_input(&mut self, input: comette::Input) {
        self.channel
            .send(Event::SetInput {
                contents: input.contents,
                selection: input.selection,
            })
            .unwrap();
    }

    fn set_list(&mut self, list: comette::List) {
        let state = self.app.state::<AppState>();
        self.channel
            .send(Event::SetList {
                items: state.register_list_items(list.items.into_iter()),
                style: list.style.map(list_style_from_comette),
            })
            .unwrap();
    }

    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report) {
        self.app
            .notification()
            .builder()
            .title(title)
            .body(format!("{error:#}"))
            .show()
            .unwrap();
    }
}

fn list_style_from_comette(value: comette::ListStyle) -> ListStyle {
    match value {
        comette::ListStyle::Rows => ListStyle::Rows,
        comette::ListStyle::Grid => ListStyle::Grid,
        comette::ListStyle::GridWithColumns(columns) => ListStyle::GridWithColumns { columns },
    }
}
