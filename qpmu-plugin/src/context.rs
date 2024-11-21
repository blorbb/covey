use crate::{proto, ListItem};

#[non_exhaustive]
pub struct ActivationContext {
    /// Query at the time this list item was created.
    pub query: String,
    /// Item that was activated.
    pub item: ListItem,
}

impl ActivationContext {
    // making this a private function instead of `impl From`
    // so that it's not public
    pub(crate) fn from_request(req: proto::ActivationRequest) -> Self {
        Self {
            query: req.query,
            item: req.selected,
        }
    }
}
