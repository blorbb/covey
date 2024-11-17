//! Wrappers to rank items based on their query.

use crate::ListItem;

pub fn rank<'iter>(
    query: &str,
    items: impl IntoIterator<Item = &'iter ListItem>,
    weights: Weights,
) -> Vec<ListItem> {
    // TODO: frequency weighting
    if query.is_empty() {
        return items.into_iter().cloned().collect();
    }

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
            let total_score = title_score + desc_score + meta_score;
            (total_score != 0.0).then_some((total_score, item))
        })
        .collect();
    // sort reversed
    scored.sort_by(|(s1, _), (s2, _)| s2.total_cmp(s1));
    scored.into_iter().map(|(_, item)| item).cloned().collect()
}

pub struct Weights {
    title: f32,
    description: f32,
    metadata: f32,
    frequency: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            title: 1.0,
            description: 0.0,
            metadata: 0.0,
            frequency: 3.0,
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
}
