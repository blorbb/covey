use std::sync::OnceLock;

use color_eyre::eyre::Result;
use covey::{Frontend, Host};
pub use covey_tauri_types::{Event, ListItem, ListStyle};
use covey_tauri_types::{Icon, ListItemId};
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
                .set(covey::Host::new(fe)?)
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
        lis: impl ExactSizeIterator<Item = covey::ListItem>,
    ) -> Vec<ListItem> {
        lis.map(|li| {
            let title = li.title().to_owned();
            let description = li.description().to_owned();
            let icon: Option<Icon> = match li.icon() {
                Some(covey::Icon::Name(name)) => freedesktop_icons::lookup(&name)
                    .with_cache()
                    .with_size(48)
                    .find()
                    .map(|path| Icon::File { path }),
                Some(covey::Icon::Text(text)) => Some(Icon::Text { text }),
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

    pub fn find_list_item(&self, id: &ListItemId) -> Option<covey::ListItemId> {
        Some(covey::ListItemId {
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

impl covey::Frontend for EventChannel {
    fn close(&mut self) {
        window::hide_menu(&self.app);
    }

    fn copy(&mut self, str: String) {
        self.app.clipboard().write_text(str).unwrap();
    }

    fn set_input(&mut self, input: covey::Input) {
        self.channel
            .send(Event::SetInput {
                contents: input.contents,
                selection: input.selection,
            })
            .unwrap();
    }

    fn set_list(&mut self, list: covey::List) {
        let state = self.app.state::<AppState>();
        self.channel
            .send(Event::SetList {
                items: state.register_list_items(list.items.into_iter()),
                style: list.style.map(list_style_from_covey),
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

fn list_style_from_covey(value: covey::ListStyle) -> ListStyle {
    match value {
        covey::ListStyle::Rows => ListStyle::Rows,
        covey::ListStyle::Grid => ListStyle::Grid,
        covey::ListStyle::GridWithColumns(columns) => ListStyle::GridWithColumns { columns },
    }
}
