//! An ordered map (de)serialized as a list with keys.

use core::{fmt, slice};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::id::StringId;

/// A list of items with unique keys.
///
/// The value must implement the [`Identify`] trait, which should be a unique
/// value across all items in this map.
///
/// Implements Serialize/Deserialize as a [`Vec<T>`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[cfg_attr(
    feature = "ts-rs",
    derive(ts_rs::TS),
    // transparent isn't supported, manually override
    ts(type = "Array<T>", bound = "T: ts_rs::TS")
)]
#[serde(default, transparent)]
pub struct KeyedList<T> {
    items: Vec<T>,
}

impl<T: Identify> KeyedList<T> {
    /// Constructs a keyed list with unique ids.
    ///
    /// # Errors
    /// Errors if multiple values have the same id.
    pub fn new(items: impl IntoIterator<Item = T>) -> Result<Self, UniqueIdError<T::Id>> {
        // Check that all ids are unique.
        let mut used = HashSet::new();

        let mut checked_items = Vec::new();
        for item in items {
            let is_new = used.insert(item.id().clone());
            if is_new {
                checked_items.push(item);
            } else {
                return Err(UniqueIdError {
                    duplicate: item.id().clone(),
                });
            }
        }

        Ok(Self {
            items: checked_items,
        })
    }

    /// Constructs an ordered map with unique ids, dropping
    /// subsequent items if they have the same id as a previous item.
    pub fn new_lossy(items: impl IntoIterator<Item = T>) -> Self {
        let mut used = HashSet::new();

        let filtered_items = items
            .into_iter()
            // true if id is new, so can be included
            .filter(|item| used.insert(item.id().clone()))
            .collect();

        Self {
            items: filtered_items,
        }
    }

    /// Get an item by its id.
    pub fn get(&self, id: &T::Id) -> Option<&T> {
        self.items.iter().find(|item| item.id() == id)
    }

    pub fn contains(&self, id: &T::Id) -> bool {
        self.get(id).is_some()
    }

    /// Extends this keyed list with extra items, ignoring items
    /// with ids that already exist in this list or in the iterator itself.
    pub fn extend_lossy(&mut self, iter: impl IntoIterator<Item = T>) {
        // this is O(n^2) but most uses of keyed list have very few items
        // so it doesn't really matter.
        // maybe optimise this in the future and let keyed list contain
        // an indexmap for more efficient getting?
        for item in iter {
            if !self.contains(item.id()) {
                self.items.push(item);
            }
        }
    }
}

impl<T> KeyedList<T> {
    pub fn iter(&self) -> slice::Iter<'_, T> {
        self.items.iter()
    }
}

impl<T> Default for KeyedList<T> {
    fn default() -> Self {
        Self {
            items: Vec::default(),
        }
    }
}

impl<T: Identify> TryFrom<Vec<T>> for KeyedList<T> {
    type Error = UniqueIdError<T::Id>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'de, T: Identify + Deserialize<'de>> Deserialize<'de> for KeyedList<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let items = Vec::deserialize(deserializer)?;
        Self::new(items).map_err(serde::de::Error::custom)
    }
}

impl<T> IntoIterator for KeyedList<T> {
    type Item = T;

    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a KeyedList<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A type that has a unique key.
pub trait Identify {
    type Id: StringId;
    fn id(&self) -> &Self::Id;
}

#[derive(Debug, Clone)]
pub struct UniqueIdError<Id> {
    duplicate: Id,
}

impl<Id: StringId> fmt::Display for UniqueIdError<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id {:?} is duplicated", self.duplicate.to_arc())
    }
}

impl<Id: fmt::Debug + StringId> core::error::Error for UniqueIdError<Id> {}
