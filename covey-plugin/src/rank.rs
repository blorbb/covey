//! Rank items based on query and usage.
//!
//! Usage stats are stored in [`DATA_DIR`]/activations.json.

use std::{collections::HashMap, io::Read, path::PathBuf};

use az::SaturatingAs;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ListItem;

fn activations_path() -> PathBuf {
    crate::plugin_data_dir().join("activations.json")
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent, default)]
struct AllActivations {
    map: HashMap<String, ItemActivations>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ItemActivations {
    frequency: u64,
    #[serde(with = "time::serde::timestamp")]
    last_use: time::OffsetDateTime,
}

fn activations() -> AllActivations {
    (|| {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .read(true)
            .open(activations_path())
            .inspect_err(|e| eprintln!("error: {e}"))
            .ok()?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).ok()?;

        serde_json::from_slice(&buf).ok()
    })()
    .unwrap_or_default()
}

pub(crate) fn register_usage(title: &str) {
    let mut current = activations();

    // increment usage
    let entry = current
        .map
        .entry(title.to_string())
        .or_insert_with(|| ItemActivations {
            frequency: 0,
            last_use: OffsetDateTime::now_utc(),
        });

    entry.frequency += 1;
    entry.last_use = OffsetDateTime::now_utc();

    // write to file
    let Ok(json_string) = serde_json::to_string(&current) else {
        eprintln!("failed stringifying {current:?}");
        return;
    };
    _ = std::fs::write(activations_path(), json_string);
}

#[expect(clippy::unused_async, reason = "may require async in the future")]
pub async fn rank<'iter>(
    query: &str,
    items: impl IntoIterator<Item = &'iter ListItem>,
    weights: Weights,
) -> Vec<ListItem> {
    let should_track_history = weights.frequency != 0.0 || weights.recency != 0.0;
    let activations = if should_track_history {
        self::activations()
    } else {
        AllActivations::default()
    };

    let now = OffsetDateTime::now_utc();

    #[expect(clippy::cast_precision_loss, reason = "precision isn't needed")]
    let mut scored: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            macro_rules! score {
                ($field:ident) => {
                    (weights.$field != 0.0)
                        .then(|| sublime_fuzzy::best_match(&query, &item.$field))
                        .flatten()
                        .map(|m| m.score() as f32 * weights.$field)
                        .unwrap_or(0.0)
                };
            }

            let title_score = score!(title);
            let desc_score = score!(description);

            let (freq, elapsed_secs) = activations.map.get(&item.title).map_or(
                (0, u64::MAX),
                |ItemActivations {
                     frequency,
                     last_use,
                 }| {
                    (
                        *frequency,
                        (now - *last_use).whole_seconds().saturating_as::<u64>(),
                    )
                },
            );

            let elapsed_min = elapsed_secs / 1000;
            // between (0, 1]
            let recency = 1.0 / elapsed_min.saturating_add(20) as f32;

            let fuzzy_score = title_score + desc_score;
            // factor in recency and fuzzy matching score for the frequency
            let freq_score =
                freq as f32 * weights.frequency * recency * (fuzzy_score / 500.0 + 0.1);
            let recency_score = recency * weights.recency;

            let total_score = fuzzy_score + freq_score + recency_score;
            let should_show = query.is_empty() || fuzzy_score > 1.0;
            should_show.then_some((total_score, item))
        })
        .collect();
    // sort reversed
    scored.sort_by(|(s1, _), (s2, _)| s2.total_cmp(s1));
    scored.into_iter().map(|(_, item)| item).cloned().collect()
}

pub struct Weights {
    title: f32,
    description: f32,
    frequency: f32,
    recency: f32,
}

impl Weights {
    /// Weights based on the title as well as frequency and recency.
    pub fn with_history() -> Self {
        Self::without_history().frequency(50.0).recency(500.0)
    }

    /// Weights based on the title only, not including usage history.
    pub fn without_history() -> Self {
        Self {
            title: 1.0,
            description: 0.0,
            frequency: 0.0,
            recency: 0.0,
        }
    }

    #[must_use = "builder method consumes self"]
    pub fn title(mut self, title: f32) -> Self {
        self.title = title;
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn description(mut self, description: f32) -> Self {
        self.description = description;
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    #[must_use = "builder method consumes self"]
    pub fn recency(mut self, recency: f32) -> Self {
        self.recency = recency;
        self
    }
}
