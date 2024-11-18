use crate::ListItem;

/// A list of results to show.
#[derive(Debug, Default)]
pub struct ResultList {
    list: Vec<ListItem>,
    selection: BoundedUsize,
}

impl ResultList {
    pub fn reset(&mut self, list: Vec<ListItem>) {
        self.selection = BoundedUsize::new_with_bound(list.len().saturating_sub(1));
        self.list = list;
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
        self.selection.value()
    }

    pub fn set_selection(&mut self, value: usize) {
        self.selection.saturating_set(value);
    }

    pub fn move_selection_signed(&mut self, delta: isize) {
        // varying behaviour depending on the position of the selection
        // if the selection is at the start or end of the list, wrap
        // otherwise, saturate.
        // this is so that large deltas (e.g. when pressing PgUp/PgDown)
        // will jump to the top/bottom first before wrapping around.
        if self.selection.is_at_bounds() {
            self.selection.wrapping_add_signed(delta);
        } else {
            self.selection.saturating_add_signed(delta);
        }
    }

    /// Gets the current selection.
    ///
    /// Returns [`None`] if the list is empty.
    pub fn selected_item(&self) -> Option<&ListItem> {
        self.list.get(self.selection())
    }
}

/// A [`usize`] with a bounded upper limit.
///
/// The `bound` is always inclusive.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BoundedUsize {
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
