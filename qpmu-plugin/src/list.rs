use std::{future::Future, pin::Pin, sync::Arc};

use anyhow::Result;

use crate::{proto, Action, Hotkey, Input};

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

    pub fn as_grid_with_columns(mut self, columns: u32) -> Self {
        self.style = Some(ListStyle::GridWithColumns(columns));
        self
    }

    pub fn as_grid(mut self) -> Self {
        self.style = Some(ListStyle::Grid);
        self
    }

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
    pub(crate) callbacks: ListItemCallbacks,
}

impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            description: String::new(),
            callbacks: ListItemCallbacks::default(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_icon(mut self, icon: Option<Icon>) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_icon_name(mut self, name: impl Into<String>) -> Self {
        self.icon = Some(Icon::Name(name.into()));
        self
    }

    pub fn with_icon_text(mut self, text: impl Into<String>) -> Self {
        self.icon = Some(Icon::Text(text.into()));
        self
    }

    /// Add a callback to run on activation.
    ///
    /// This should be called after everything else is initialised.
    pub fn on_activate<Fut>(mut self, callback: impl Fn() -> Fut + Send + Sync + 'static) -> Self
    where
        Fut: Future<Output = Result<Vec<Action>>> + Send + Sync + 'static,
    {
        self.callbacks.activate = Some(Arc::new(move || Box::pin(callback())));
        self.callbacks
            .item_title
            .get_or_insert_with(|| self.title.clone());
        self
    }

    /// Add a callback to run on alt-activate.
    ///
    /// This should be called after everything else is initialised.
    pub fn on_alt_activate<Fut>(
        mut self,
        callback: impl Fn() -> Fut + Send + Sync + 'static,
    ) -> Self
    where
        Fut: Future<Output = Result<Vec<Action>>> + Send + Sync + 'static,
    {
        self.callbacks.alt_activate = Some(Arc::new(move || Box::pin(callback())));
        self.callbacks
            .item_title
            .get_or_insert_with(|| self.title.clone());
        self
    }

    /// Add a callback to run when a hotkey fires.
    ///
    /// This should be called after everything else is initialised.
    pub fn on_hotkey_activate<Fut>(
        mut self,
        callback: impl Fn(Hotkey) -> Fut + Send + Sync + 'static,
    ) -> Self
    where
        Fut: Future<Output = Result<Vec<Action>>> + Send + Sync + 'static,
    {
        self.callbacks.hotkey_activate = Some(Arc::new(move |hotkey| Box::pin(callback(hotkey))));
        self.callbacks
            .item_title
            .get_or_insert_with(|| self.title.clone());
        self
    }

    /// Add a callback to run on tab completion.
    ///
    /// This should be called after everything else is initialised.
    pub fn on_complete<Fut>(mut self, callback: impl Fn() -> Fut + Send + Sync + 'static) -> Self
    where
        Fut: Future<Output = Result<Option<Input>>> + Send + Sync + 'static,
    {
        self.callbacks.complete = Some(Arc::new(move || Box::pin(callback())));
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

type ReturnFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send + Sync>>;
type ActionsFuture = ReturnFuture<Vec<Action>>;
type ActivationFunction = Arc<dyn Fn() -> ActionsFuture + Send + Sync>;

#[derive(Clone, Default)]
pub(crate) struct ListItemCallbacks {
    activate: Option<ActivationFunction>,
    alt_activate: Option<ActivationFunction>,
    hotkey_activate: Option<Arc<dyn Fn(Hotkey) -> ActionsFuture + Send + Sync>>,
    complete: Option<Arc<dyn Fn() -> ReturnFuture<Option<Input>> + Send + Sync>>,
    item_title: Option<String>,
}

macro_rules! generate_call_for {
    ($( $name:ident($($arg:ident : $ty:ty),*) ),*) => {
        $(
            pub(crate) async fn $name(self, $($arg: $ty),*) -> Result<Vec<Action>> {
                if let Some(callback) = &self.$name {
                    if let Some(title) = &self.item_title {
                        crate::sql::increment_frequency_table(title).await?;
                    }
                    callback($($arg),*).await
                } else {
                    Ok(vec![])
                }
            }
        )*
    };
}

impl ListItemCallbacks {
    generate_call_for!(activate(), alt_activate(), hotkey_activate(hotkey: Hotkey));

    pub(crate) async fn complete(self) -> Result<Option<Input>> {
        if let Some(callback) = self.complete {
            callback().await
        } else {
            Ok(None)
        }
    }
}
