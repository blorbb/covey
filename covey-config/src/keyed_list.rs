//! An ordered map (de)serialized as a list with keys.

use core::{fmt, slice};
use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};

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
    pub fn new(items: impl IntoIterator<Item = T>) -> Result<Self, UniqueIdError> {
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
    pub fn get(&self, id: &str) -> Option<&T> {
        self.items.iter().find(|item| item.id().as_str() == id)
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
    type Error = UniqueIdError;

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

/// A string ID that is cheap to clone.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
// ts-rs doesn't support transparent but this is still typed as a string
#[serde(transparent)]
pub struct Id(Arc<str>);

impl Id {
    pub fn new(s: &str) -> Self {
        Self(Arc::from(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A type that has a key that should be unique.
pub trait Identify {
    fn id(&self) -> &Id;
}

#[derive(Debug, Clone)]
pub struct UniqueIdError {
    duplicate: Id,
}

impl fmt::Display for UniqueIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id {:?} is duplicated", self.duplicate.as_str())
    }
}

impl core::error::Error for UniqueIdError {}
