use crate::proto;

pub struct List {
    items: Vec<ListItem>,
    /// The kind of list to show.
    ///
    /// If this is [`None`], the list style will be the default set by
    /// the user. Plugins should only set one if the content makes the most
    /// sense with one of these styles.
    style: Option<ListStyle>,
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        Self { items, style: None }
    }

    pub(crate) fn into_proto(self) -> proto::QueryResponse {
        proto::QueryResponse {
            items: ListItem::into_proto_vec(self.items),
            list_style: self.style.map(ListStyle::into_proto),
        }
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

#[derive(Debug, Clone)]
pub struct ListItem {
    pub title: String,
    pub icon: Option<String>,
    pub description: String,
    pub metadata: String,
}

impl ListItem {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            description: String::new(),
            metadata: String::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_metadata(mut self, meta: impl Into<String>) -> Self {
        self.metadata = meta.into();
        self
    }

    pub fn with_icon(mut self, icon: Option<impl Into<String>>) -> Self {
        self.icon = icon.map(Into::into);
        self
    }

    pub(crate) fn from_proto(proto: proto::ListItem) -> Self {
        Self {
            title: proto.title,
            icon: proto.icon,
            description: proto.description,
            metadata: proto.metadata,
        }
    }

    pub(crate) fn into_proto(self) -> proto::ListItem {
        proto::ListItem {
            title: self.title,
            icon: self.icon,
            description: self.description,
            metadata: self.metadata,
        }
    }

    pub(crate) fn into_proto_vec(vec: Vec<Self>) -> Vec<proto::ListItem> {
        vec.into_iter().map(Self::into_proto).collect()
    }
}
