use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::SystemTime};

use crate::{
    Menu,
    rank::{self, VisitId},
};

#[non_exhaustive]
pub struct List {
    pub items: Vec<ListItem>,
    /// The kind of list to show.
    ///
    /// If this is [`None`], the list style will be the default set by
    /// the user. Plugins should only set one if the content makes the most
    /// sense with one of these styles.
    pub style: Option<ListStyle>,
    pub(crate) callbacks: CommandCallbacks,
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        Self {
            items,
            style: None,
            callbacks: CommandCallbacks::new(),
        }
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

    /// Adds a command that can be called.
    ///
    /// This should not be used directly, use the extension trait generated
    /// by [`crate::include_manifest!`] instead.
    #[doc(hidden)]
    #[must_use]
    pub fn add_command(
        mut self,
        name: &'static str,
        callback: impl AsyncFn(Menu) -> crate::Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.callbacks
            .add_callback(covey_proto::CommandId::new(name), callback);
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

#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub description: String,
    pub icon: Option<Icon>,
    pub(crate) visit_id: VisitId,
    pub(crate) callbacks: CommandCallbacks,
}

impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            title: title.clone(),
            icon: None,
            description: String::new(),
            visit_id: VisitId::from(title),
            callbacks: CommandCallbacks::new(),
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

    #[must_use = "builder method consumes self"]
    pub fn with_visit_id(mut self, id: impl Into<VisitId>) -> Self {
        self.visit_id = id.into();
        self
    }

    /// An ID to identify this list item when keeping track of how many times it
    /// has been previously used/activated.
    ///
    /// If not explicitly set, the usage id will be the list item title.
    pub fn visit_id(&self) -> &VisitId {
        &self.visit_id
    }

    pub fn accuracy(&self, query: &str, weights: rank::Weights) -> f32 {
        rank::accuracy(query, self, weights)
    }

    pub fn frecency(
        &self,
        visits: &rank::Visits,
        now: SystemTime,
        weights: rank::Weights,
    ) -> rank::Frecency {
        rank::frecency(self, visits, now, weights)
    }

    /// Score including both accuracy and frecency.
    pub fn score(
        &self,
        query: &str,
        visits: &rank::Visits,
        now: SystemTime,
        weights: rank::Weights,
    ) -> f32 {
        if weights.frecency == 0.0 {
            self.accuracy(query, weights)
        } else {
            self.frecency(visits, now, weights)
                .combine_with_accuracy(self.accuracy(query, weights))
        }
    }

    /// Adds a command that can be called.
    ///
    /// This should not be used directly, use the extension trait generated
    /// by [`crate::include_manifest!`] instead.
    #[doc(hidden)]
    #[must_use]
    pub fn add_command(
        mut self,
        name: &'static str,
        callback: impl AsyncFn(Menu) -> crate::Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.callbacks
            .add_callback(covey_proto::CommandId::new(name), callback);
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
pub(crate) struct CommandCallbacks {
    commands: HashMap<covey_proto::CommandId, ActivationFunction>,
}

impl CommandCallbacks {
    pub(crate) fn new() -> Self {
        Self {
            commands: HashMap::default(),
        }
    }

    pub(crate) fn add_callback(
        &mut self,
        command_id: covey_proto::CommandId,
        callback: impl AsyncFn(Menu) -> crate::Result<()> + Send + Sync + 'static,
    ) {
        let callback = Arc::new(callback);
        self.commands.insert(
            command_id,
            Arc::new(move |menu| {
                let callback = Arc::clone(&callback);
                Box::pin(async move {
                    if let Err(e) = callback(menu.clone()).await {
                        menu.display_error(format!("{e:#}"));
                    }
                })
            }),
        );
    }

    pub(crate) fn get_callback(
        &self,
        command_id: &covey_proto::CommandId,
    ) -> Option<&ActivationFunction> {
        self.commands.get(command_id)
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = &covey_proto::CommandId> {
        self.commands.keys()
    }
}
