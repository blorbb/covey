use color_eyre::eyre::Result;
use covey::{Action, host};
pub use covey_tauri_types::{Event, ListItem, ListStyle};
use covey_tauri_types::{Icon, ListItemId};
use parking_lot::Mutex;
use tauri::ipc::Channel;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

use crate::window;

/// Must be initialised exactly once with [`AppState::init`].
pub struct AppState {
    pub(crate) sender: Mutex<host::RequestSender>,
    handle: tokio::task::JoinHandle<()>,
}

impl AppState {
    pub fn new(app: tauri::AppHandle, channel: Channel<Event>) -> Result<Self> {
        let (tx, mut rx) = host::channel()?;

        // forward actions to the channel
        let handle = tokio::spawn(async move {
            loop {
                let action = rx.recv_action().await;
                let event = match action {
                    Action::Close => {
                        window::hide_menu(&app);
                        continue;
                    }
                    Action::SetList(list) => Event::SetList {
                        items: convert_list_items(&list.items),
                        style: list.style.map(list_style_from_covey),
                        plugin_id: list.plugin.id().clone(),
                    },
                    Action::SetInput(input) => Event::SetInput {
                        contents: input.contents,
                        selection: input.selection,
                    },
                    Action::Copy(str) => {
                        app.clipboard().write_text(str).unwrap();
                        continue;
                    }
                    Action::DisplayError(title, body) => {
                        app.notification()
                            .builder()
                            .title(title)
                            .body(body)
                            .show()
                            .unwrap();
                        continue;
                    }
                };

                channel.send(event).expect("event channel must remain open");
            }
        });

        Ok(Self {
            sender: Mutex::new(tx),
            handle,
        })
    }

    pub fn find_list_item(&self, id: &ListItemId) -> Option<covey::ListItemId> {
        Some(covey::ListItemId {
            plugin: self
                .sender
                .lock()
                .plugins()
                .get(id.plugin_id.as_str())?
                .clone(),
            local_id: id.local_id.parse().ok()?,
        })
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn convert_list_items(lis: &[covey::ListItem]) -> Vec<ListItem> {
    lis.iter()
        .map(|li| {
            let icon: Option<Icon> = match li.icon().cloned() {
                Some(covey::ResolvedIcon::File(path)) => Some(Icon::File { path }),
                Some(covey::ResolvedIcon::Text(text)) => Some(Icon::Text { text }),
                None => None,
            };
            let id = ListItemId {
                local_id: li.id().local_id.to_string(),
                plugin_id: li.id().plugin.id().to_owned(),
            };

            ListItem {
                title: li.title().to_owned(),
                description: li.description().to_owned(),
                icon,
                id,
                available_commands: li.available_commands().to_vec(),
            }
        })
        .collect()
}

fn list_style_from_covey(value: covey::ListStyle) -> ListStyle {
    match value {
        covey::ListStyle::Rows => ListStyle::Rows,
        covey::ListStyle::Grid => ListStyle::Grid,
        covey::ListStyle::GridWithColumns(columns) => ListStyle::GridWithColumns { columns },
    }
}
