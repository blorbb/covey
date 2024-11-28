use crate::proto;

#[derive(Debug, Clone)]
pub struct Input {
    pub query: String,
    pub range_lb: u16,
    pub range_ub: u16,
}

impl Input {
    /// Sets the input to the provided query and with the cursor placed
    /// at the end.
    pub fn new(query: impl Into<String>) -> Self {
        let range = SelectionRange::end();
        Self {
            query: query.into(),
            range_lb: range.lower_bound,
            range_ub: range.upper_bound,
        }
    }

    #[must_use = "builder method consumes self"]
    pub fn select(mut self, sel: SelectionRange) -> Self {
        self.range_lb = sel.lower_bound;
        self.range_ub = sel.lower_bound;
        self
    }

    #[must_use = "builder method consumes self"]
    pub(crate) fn into_proto(self) -> proto::Input {
        proto::Input {
            query: self.query,
            range_lb: u32::from(self.range_lb),
            range_ub: u32::from(self.range_ub),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SelectionRange {
    lower_bound: u16,
    upper_bound: u16,
}

impl SelectionRange {
    /// Sets both the start and end bound to the provided index.
    pub fn at(index: u16) -> Self {
        Self {
            lower_bound: index,
            upper_bound: index,
        }
    }

    /// Selects the entire query.
    pub fn all() -> Self {
        Self {
            lower_bound: 0,
            upper_bound: u16::MAX,
        }
    }

    /// Sets the start and end to `0`.
    pub fn start() -> Self {
        Self::at(0)
    }

    pub fn end() -> Self {
        Self::at(u16::MAX)
    }
}
