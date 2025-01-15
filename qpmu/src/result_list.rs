use crate::{
    plugin::{proto, Plugin},
    ListItem,
};

/// A list of results to show.
#[derive(Debug, Default)]
pub struct ResultList {
    pub items: Vec<ListItem>,
    pub style: Option<ListStyle>,
}

impl ResultList {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub(crate) fn from_proto(plugin: &Plugin, proto: proto::QueryResponse) -> Self {
        let style = proto.list_style.map(ListStyle::from_proto);
        let list: Vec<_> = proto
            .items
            .into_iter()
            .map(|li| ListItem::new(Plugin::clone(plugin), li))
            .collect();
        Self {
            style,
            items: list,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ListStyle {
    Rows,
    Grid,
    GridWithColumns(u32),
}

impl ListStyle {
    pub(crate) fn from_proto(proto: proto::query_response::ListStyle) -> Self {
        use proto::query_response::ListStyle as Ls;
        match proto {
            Ls::Rows(()) => Self::Rows,
            Ls::Grid(()) => Self::Grid,
            Ls::GridWithColumns(columns) => Self::GridWithColumns(columns),
        }
    }
}

/// A [`usize`] with a bounded upper limit.
///
/// The `bound` is always inclusive.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedUsize {
    /// The current actual value. Must always be `<= bound`.
    value: usize,
    /// The maximum possible value for `value`. This is **inclusive**.
    bound: usize,
}

impl BoundedUsize {
    /// Makes a new [`BoundedUsize`] with `bound` as the maximum value.
    ///
    /// The actual value initialises to zero.
    pub fn new_with_bound(bound: usize) -> Self {
        Self { value: 0, bound }
    }

    /// Sets the value, saturating if the value is greater than the bound.
    pub fn saturating_set(&mut self, value: usize) {
        self.value = usize::min(value, self.bound);
    }

    pub fn saturating_add_signed(&mut self, delta: isize) {
        self.saturating_set(self.value.saturating_add_signed(delta));
    }

    pub fn wrapping_add_signed(&mut self, delta: isize) {
        // working with i128 where overflow can never happen is simpler
        let value = self.value as i128;
        let bound = self.bound as i128;
        let delta = delta as i128;

        // this `as usize` case is fine since rhs of modulo is within usize
        self.value = (value + delta).rem_euclid(bound + 1) as usize;
    }

    pub fn value(&self) -> usize {
        self.value
    }

    /// Whether the value is 0.
    pub fn is_min(&self) -> bool {
        self.value == 0
    }

    /// Whether the value equals the upper bound.
    pub fn is_max(&self) -> bool {
        self.value == self.bound
    }

    /// Whether the value is at the ends of the bounds (0 or equal to upper bound).
    pub fn is_at_bounds(&self) -> bool {
        self.is_min() || self.is_max()
    }
}
