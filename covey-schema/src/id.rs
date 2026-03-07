use std::{fmt, hash::Hash, sync::Arc};

use serde::{Deserialize, Serialize};

/// A unique ID represented by a string.
///
/// Implementors should have a cheap clone.
pub trait StringId: Clone + Eq + Hash + Ord {
    fn to_arc(&self) -> Arc<str>;
    fn as_str(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommandId(Arc<str>);

impl CommandId {
    pub fn new(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl StringId for CommandId {
    fn to_arc(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId(Arc<str>);

impl PluginId {
    pub fn new(s: &str) -> Self {
        Self(Arc::from(s))
    }
}

impl StringId for PluginId {
    fn to_arc(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
