use crate::ListItem;

/// A list of results to show.
#[derive(Debug, Default)]
pub struct ResultList {
    pub(crate) list: Vec<ListItem>,
    pub(crate) selection: usize,
}

impl ResultList {
    pub fn reset(&mut self, list: Vec<ListItem>) {
        self.list = list;
        self.selection = 0;
    }

    pub fn list(&self) -> &[ListItem] {
        &self.list
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn selection(&self) -> usize {
        self.selection
    }

    pub fn selected_item(&self) -> Option<&ListItem> {
        self.list.get(self.selection)
    }
}
