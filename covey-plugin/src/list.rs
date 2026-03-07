use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use crate::Menu;

#[non_exhaustive]
pub struct List {
    pub items: Vec<ListItem>,
    /// The kind of list to show.
    ///
    /// If this is [`None`], the list style will be the default set by
    /// the user. Plugins should only set one if the content makes the most
    /// sense with one of these styles.
    pub style: Option<ListStyle>,
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        Self { items, style: None }
    }

    #[must_use = "builder method consumes self"]
    pub fn as_grid_with_columns(mut self, columns: u32) -> Self {
        self.style = Some(ListStyle::GridWithColumns(columns));
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn as_grid(mut self) -> Self {
        self.style = Some(ListStyle::Grid);
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn as_rows(mut self) -> Self {
        self.style = Some(ListStyle::Rows);
        self
    }
}

#[non_exhaustive]
pub enum ListStyle {
    Rows,
    Grid,
    GridWithColumns(u32),
}

impl ListStyle {
    pub(crate) fn into_proto(self) -> covey_proto::ListStyle {
        match self {
            Self::Rows => covey_proto::ListStyle::Rows,
            Self::Grid => covey_proto::ListStyle::Grid,
            Self::GridWithColumns(columns) => covey_proto::ListStyle::GridWithColumns(columns),
        }
    }
}

// This should only be converted into a covey_proto::ListItem via the
// ListItemStore.
#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub description: String,
    pub icon: Option<Icon>,
    pub(crate) commands: ListItemCallbacks,
}

impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            title: title.clone(),
            icon: None,
            description: String::new(),
            commands: ListItemCallbacks::new(title),
        }
    }

    #[must_use = "builder method consumes self"]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn with_icon(mut self, icon: Option<Icon>) -> Self {
        self.icon = icon;
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn with_icon_name(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(Icon::Name(name.into()));
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn with_icon_text(mut self, text: impl Into<String>) -> Self {
        self.icon = Some(Icon::Text(text.into()));
        self
    }

    /// Adds a command that can be called.
    ///
    /// This should not be used directly, use the extension trait generated
    /// by [`crate::include_manifest!`] instead.
    #[doc(hidden)]
    #[must_use]
    pub fn add_command(mut self, name: &'static str, callback: ActivationFunction) -> Self {
        self.commands
            .add_command(covey_proto::CommandId::new(name), callback);
        self
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Icon {
    Name(String),
    Text(String),
}

impl Icon {
    pub(crate) fn into_proto(self) -> covey_proto::ListItemIcon {
        match self {
            Self::Name(name) => covey_proto::ListItemIcon::Name(name),
            Self::Text(text) => covey_proto::ListItemIcon::Text(text),
        }
    }
}

// ActivationFunction needs Send + Sync for PluginBlocking to work.
type DynFuture<T> = Pin<Box<dyn Future<Output = T>>>;
type ActivationFunction = Arc<dyn Fn(Menu) -> DynFuture<()> + Send + Sync>;

#[derive(Clone)]
pub(crate) struct ListItemCallbacks {
    commands: HashMap<covey_proto::CommandId, ActivationFunction>,
    item_title: String,
}

impl ListItemCallbacks {
    pub(crate) fn new(title: String) -> Self {
        Self {
            commands: HashMap::default(),
            item_title: title,
        }
    }

    pub(crate) fn add_command(
        &mut self,
        command_id: covey_proto::CommandId,
        callback: ActivationFunction,
    ) {
        self.commands.insert(command_id, callback);
    }

    /// Calls a command by name, doing nothing if the command is not found.
    pub(crate) async fn call_command(&self, name: &covey_proto::CommandId, menu: Menu) {
        if let Some(cmd) = self.commands.get(name) {
            crate::rank::register_usage(&self.item_title);
            cmd(menu).await;
        }
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = &covey_proto::CommandId> {
        self.commands.keys()
    }
}
