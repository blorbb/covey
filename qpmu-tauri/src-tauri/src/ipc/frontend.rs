use serde::Serialize;
use tauri::{ipc::Channel, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;

use crate::{ipc, state::AppState};

#[derive(Clone)]
pub struct EventChannel {
    pub channel: Channel<Event>,
    pub app: tauri::AppHandle,
}

impl qpmu::Frontend for EventChannel {
    fn close(&mut self) {
        ipc::window::hide_window(self.app.clone());
    }

    fn copy(&mut self, str: String) {
        self.app.clipboard().write_text(str).unwrap();
    }

    fn set_input(&mut self, input: qpmu::Input) {
        self.channel
            .send(Event::SetInput {
                contents: input.contents,
                selection: input.selection,
            })
            .unwrap();
    }

    fn set_list(&mut self, list: qpmu::ResultList) {
        let state = self.app.state::<AppState>();
        self.channel
            .send(Event::SetList {
                items: state.register_list_items(list.items.into_iter()),
                style: list.style.map(ListStyle::from),
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

/// This must have an equivalent type on the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Event {
    SetInput {
        contents: String,
        selection: (u16, u16),
    },
    SetList {
        items: Vec<ListItem>,
        style: Option<ListStyle>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) id: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ListStyle {
    Rows,
    Grid,
    GridWithColumns { columns: u32 },
}

impl From<qpmu::ListStyle> for ListStyle {
    fn from(value: qpmu::ListStyle) -> Self {
        match value {
            qpmu::ListStyle::Rows => Self::Rows,
            qpmu::ListStyle::Grid => Self::Grid,
            qpmu::ListStyle::GridWithColumns(columns) => Self::GridWithColumns { columns },
        }
    }
}
