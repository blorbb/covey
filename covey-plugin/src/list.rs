use std::time::SystemTime;

use crate::{
    Menu,
    rank::{self, VisitId},
    store::TargetCallbacks,
};

#[non_exhaustive]
pub struct List {
    pub(crate) sections: Vec<ListSection>,
    pub(crate) callbacks: TargetCallbacks,
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        Self {
            sections: vec![ListSection::new(String::new(), items)],
            callbacks: TargetCallbacks::new(),
        }
    }

    pub fn from_sections(sections: Vec<ListSection>) -> Self {
        Self {
            sections,
            callbacks: TargetCallbacks::new(),
        }
    }

    pub fn total_len(&self) -> usize {
        self.sections
            .iter()
            .map(|section| section.items.len())
            .sum()
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
        callback: impl AsyncFn(&Menu) -> crate::Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.callbacks
            .add_callback(covey_proto::CommandId::new(name), callback);
        self
    }
}

pub struct ListSection {
    pub(crate) title: String,
    pub(crate) items: Vec<ListItem>,
}

impl ListSection {
    pub fn new(title: impl Into<String>, items: Vec<ListItem>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }

    pub fn unnamed(items: Vec<ListItem>) -> Self {
        Self::new(String::new(), items)
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn items(&self) -> &[ListItem] {
        &self.items
    }
}

#[derive(Clone)]
pub struct ListItem {
    pub title: String,
    pub description: String,
    pub icon: Option<Icon>,
    pub(crate) visit_id: VisitId,
    pub(crate) callbacks: TargetCallbacks,
}

impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            title: title.clone(),
            icon: None,
            description: String::new(),
            visit_id: VisitId::from(title),
            callbacks: TargetCallbacks::new(),
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
        callback: impl AsyncFn(&Menu) -> crate::Result<()> + Send + Sync + 'static,
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
