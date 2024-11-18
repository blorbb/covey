//! Wrappers to rank items based on their query.

use std::collections::HashMap;

use az::SaturatingAs;
use time::OffsetDateTime;

use crate::{sql, ListItem};

async fn activations() -> Option<HashMap<String, (u64, OffsetDateTime)>> {
    let a = sqlx::query_as::<_, (String, i64, time::OffsetDateTime)>(
        "
        SELECT title, frequency, last_use FROM activations
        ",
    )
    .fetch_all(sql::pool())
    .await
    .ok()?;

    Some(
        a.into_iter()
            .map(|(title, freq, last)| (title, (freq.saturating_as::<u64>(), last)))
            .collect::<HashMap<_, _>>(),
    )
}

pub async fn rank<'iter>(
    query: &str,
    items: impl IntoIterator<Item = &'iter ListItem>,
    weights: Weights,
) -> Vec<ListItem> {
    let activations = self::activations().await.unwrap_or_else(HashMap::new);

    let now = OffsetDateTime::now_utc();

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
            let meta_score = score!(metadata);

            let (freq, elapsed_secs) =
                activations
                    .get(&item.title)
                    .map_or((0, u64::MAX), |(freq, time)| {
                        (
                            *freq,
                            (now.unix_timestamp() - time.unix_timestamp()).saturating_as::<u64>(),
                        )
                    });

            let elapsed_min = elapsed_secs / 1000;
            // between (0, 1]
            let recency = 1.0 / elapsed_min.saturating_add(20) as f32;

            let fuzzy_score = title_score + desc_score + meta_score;
            // factor in recency and fuzzy matching score for the frequency
            let freq_score = freq as f32 * weights.frequency * recency * (fuzzy_score / 500.0 + 0.1);
            let recency_score = recency * weights.recency;

            let total_score = fuzzy_score + freq_score + recency_score;
            let should_show = query.is_empty() || fuzzy_score > 1.0;
            should_show.then_some((
                total_score,
                item.clone().with_description(format!(
                    "fz {fuzzy_score} fr {freq_score} re {recency_score} tt {total_score} el {elapsed_secs}"
                )),
            ))
        })
        .collect();
    // sort reversed
    scored.sort_by(|(s1, _), (s2, _)| s2.total_cmp(s1));
    scored.into_iter().map(|(_, item)| item).collect()
}

pub struct Weights {
    title: f32,
    description: f32,
    metadata: f32,
    frequency: f32,
    recency: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            title: 1.0,
            description: 0.0,
            metadata: 0.0,
            frequency: 50.0,
            recency: 500.0,
        }
    }
}

impl Weights {
    pub fn title(mut self, title: f32) -> Self {
        self.title = title;
        self
    }

    pub fn description(mut self, description: f32) -> Self {
        self.description = description;
        self
    }

    pub fn metadata(mut self, metadata: f32) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn recency(mut self, recency: f32) -> Self {
        self.recency = recency;
        self
    }
}
