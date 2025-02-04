use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use anyhow::Result;

use crate::{proto, Action, Actions};

pub struct List {
    pub(crate) items: Vec<ListItem>,
    /// The kind of list to show.
    ///
    /// If this is [`None`], the list style will be the default set by
    /// the user. Plugins should only set one if the content makes the most
    /// sense with one of these styles.
    pub(crate) style: Option<ListStyle>,
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
    pub(crate) fn into_proto(self) -> proto::query_response::ListStyle {
        use proto::query_response::ListStyle as Proto;
        match self {
            ListStyle::Rows => Proto::Rows(()),
            ListStyle::Grid => Proto::Grid(()),
            ListStyle::GridWithColumns(columns) => Proto::GridWithColumns(columns),
        }
    }
}

// This should only be converted into a proto::ListItem via the ListItemStore.
#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub description: String,
    pub icon: Option<Icon>,
    /// Key is the command's ID.
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
    pub fn add_command(mut self, name: &'static str, callback: ActivationFunction) -> Self {
        self.commands.add_command(name, callback);
        self
    }
}

#[derive(Debug, Clone)]
pub enum Icon {
    Name(String),
    Text(String),
}

impl Icon {
    pub(crate) fn into_proto(self) -> proto::list_item::Icon {
        use proto::list_item::Icon as Proto;
        match self {
            Self::Name(name) => Proto::Name(name),
            Self::Text(text) => Proto::Text(text),
        }
    }
}

type DynFuture<T> = Pin<Box<dyn Future<Output = T> + Send + Sync>>;
type ActivationFunction = Arc<dyn Fn() -> DynFuture<Result<Actions>> + Send + Sync>;

#[derive(Clone)]
pub(crate) struct ListItemCallbacks {
    /// Key is the command's ID.
    commands: HashMap<&'static str, ActivationFunction>,
    item_title: String,
}

impl ListItemCallbacks {
    pub(crate) fn new(title: String) -> Self {
        Self {
            commands: HashMap::default(),
            item_title: title,
        }
    }

    pub(crate) fn add_command(&mut self, name: &'static str, callback: ActivationFunction) {
        self.commands.insert(name, callback);
    }

    /// Calls a command by name, returning an empty vec if the command is not found.
    pub(crate) async fn call_command(&self, name: &str) -> Result<Vec<Action>> {
        if let Some(cmd) = self.commands.get(name) {
            crate::rank::register_usage(&self.item_title);
            cmd().await.map(|actions| actions.list)
        } else {
            Ok(vec![])
        }
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = &'static str> + use<'_> {
        self.commands.keys().copied()
    }
}
