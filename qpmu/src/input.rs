use az::SaturatingAs as _;

use crate::plugin::{proto, Plugin};

/// The main text input contents and selection.
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub contents: String,
    /// Range in terms of chars, not bytes
    pub selection: (u16, u16),
}

impl Input {
    pub(crate) fn prefix_with(&mut self, prefix: &str) {
        self.contents.insert_str(0, prefix);
        let prefix_len =
            u16::try_from(prefix.chars().count()).expect("prefix should not be insanely long");

        let (a, b) = self.selection;
        self.selection = (a.saturating_add(prefix_len), b.saturating_add(prefix_len));
    }

    pub(crate) fn from_proto(plugin: Plugin, il: proto::Input) -> Self {
        let mut input = Self {
            contents: il.query,
            selection: (il.range_lb.saturating_as(), il.range_ub.saturating_as()),
        };
        input.prefix_with(plugin.prefix());
        input
    }
}
