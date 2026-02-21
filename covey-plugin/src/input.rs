use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Input {
    pub query: String,
    pub selection: SelectionRange,
}

impl Input {
    /// Sets the input to the provided query and with the cursor placed
    /// at the end.
    pub fn new(query: impl Into<String>) -> Self {
        let selection = SelectionRange::end();
        Self {
            query: query.into(),
            selection,
        }
    }

    #[must_use = "builder method consumes self"]
    pub fn select(mut self, sel: SelectionRange) -> Self {
        self.selection = sel;
        self
    }

    #[must_use = "builder method consumes self"]
    pub(crate) fn into_proto(self) -> covey_proto::Input {
        covey_proto::Input {
            query: self.query,
            selection: self.selection.to_range(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SelectionRange {
    lower_bound: usize,
    upper_bound: usize,
}

impl SelectionRange {
    /// Sets both the start and end bound to the provided index.
    pub fn at(index: usize) -> Self {
        Self {
            lower_bound: index,
            upper_bound: index,
        }
    }

    /// Selects the entire query.
    pub fn all() -> Self {
        Self {
            lower_bound: 0,
            upper_bound: usize::MAX,
        }
    }

    /// Sets the start and end to `0`.
    pub fn start() -> Self {
        Self::at(0)
    }

    pub fn end() -> Self {
        Self::at(usize::MAX)
    }

    pub fn to_range(&self) -> Range<usize> {
        self.lower_bound..self.upper_bound
    }
}
