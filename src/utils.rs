/// Wrapper around an [`Option<T>`] that can only be read by [`take`]ing
/// the value out of the option.
///
/// [`take`]: Option::take
#[derive(Debug, Default, Clone, Copy)]
pub struct ReadOnce<T>(Option<T>);

impl<T> ReadOnce<T> {
    /// Initialises with nothing stored.
    pub fn empty() -> Self {
        Self(None)
    }

    /// Replaces the value with the value given in the parameter,
    /// returning the old value if present.
    pub fn replace(&mut self, to: T) -> Option<T> {
        self.0.replace(to)
    }

    pub fn take(&mut self) -> Option<T> {
        self.0.take()
    }
}
