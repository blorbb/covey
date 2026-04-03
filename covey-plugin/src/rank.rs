//! Rank items based on query and visit history.
//!
//! Visit stats are stored in `<plugin dir>/activations.json`.
//!
//! Frecency calculation formula is based on
//! <https://github.com/homerours/jumper/blob/master/algorithm.md>.
//!
//! The implementation details are based on
//! <https://github.com/homerours/jumper/blob/master/src/record.c>.

use std::{
    cmp::Reverse,
    collections::HashMap,
    hash::BuildHasher,
    io::Read,
    path::PathBuf,
    sync::LazyLock,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use skim::fuzzy_matcher::{FuzzyMatcher, arinae::ArinaeMatcher};

use crate::ListItem;

fn time_diff_secs(now: SystemTime, earlier: SystemTime) -> f32 {
    now.duration_since(earlier).unwrap_or_default().as_secs() as f32
}

const SHORT_DECAY: f32 = 2e-5;
const LONG_DECAY: f32 = 3e-7;

/// An ID to identify this list item when keeping track of how many times it
/// has been previously visited/activated.
///
/// If not explicitly set, the visit id will be the list item title.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct VisitId(String);

impl<T: Into<String>> From<T> for VisitId {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(transparent, default)]
pub struct Visits {
    map: HashMap<VisitId, VisitScore>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VisitScore {
    /// Time in seconds since [`SystemTime::UNIX_EPOCH`].
    #[serde(with = "timestamp")]
    last_visit: SystemTime,
    decayed_score: f32,
}

mod timestamp {
    use std::time::{Duration, SystemTime};

    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(
            t.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_secs(),
        )
    }
    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<SystemTime, D::Error> {
        u64::deserialize(d).map(|timestamp| SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp))
    }
}

impl Visits {
    fn file_path() -> PathBuf {
        crate::plugin_data_dir().join("activations.json")
    }

    pub fn from_file() -> Self {
        (|| {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .read(true)
                .open(Self::file_path())
                .inspect_err(|e| eprintln!("error: {e}"))
                .ok()?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf).ok()?;

            serde_json::from_slice(&buf).ok()
        })()
        .unwrap_or_default()
    }

    pub fn mark_visit(&mut self, visit_id: VisitId, now: SystemTime) {
        self.mark_visit_weighted(visit_id, now, 1.0);
    }

    /// Activating a list item automatically marks the item's visit ID with a
    /// weight of 1.
    pub fn mark_visit_weighted(&mut self, visit_id: VisitId, now: SystemTime, weight: f32) {
        let entry = self.map.entry(visit_id).or_insert_with(|| VisitScore {
            last_visit: now,
            decayed_score: 0.0,
        });

        entry.decayed_score = weight
            + f32::exp(-LONG_DECAY * time_diff_secs(now, entry.last_visit)) * entry.decayed_score;
        entry.last_visit = now;
    }

    /// Atomically writes visit data to this plugin's `activations.json` without
    /// blocking and cleans up very old entries.
    pub fn write_to_file(mut self) {
        tokio::task::spawn_blocking(move || {
            let now = SystemTime::now();

            // take at least 100 entries or less than 50 days old
            let mut map = Vec::from_iter(self.map);
            map.sort_unstable_by_key(|(_, v)| Reverse(v.last_visit));
            self.map = map
                .into_iter()
                .enumerate()
                .take_while(|(i, (_, v))| {
                    *i < 100
                        || now.duration_since(v.last_visit).unwrap_or_default()
                            <= Duration::from_hours(24 * 50)
                })
                .map(|(_, kv)| kv)
                .collect();

            let json_string = serde_json::to_string(&self).expect("Visits should be serializable");
            let visits_path = Self::file_path();
            let random = std::hash::RandomState::new().hash_one(&json_string);
            let tmp_visits_path = visits_path.with_added_extension(format!("tmp.{random:x}"));
            match std::fs::write(&tmp_visits_path, json_string) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!(
                        "WARNING: failed to write visits to {}: {e:#}",
                        tmp_visits_path.display()
                    );
                    _ = std::fs::remove_file(&tmp_visits_path);
                    return;
                }
            }
            match std::fs::rename(&tmp_visits_path, &visits_path) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!(
                        "WARNING: failed to commit visits to {}: {e:#}",
                        visits_path.display()
                    );
                    _ = std::fs::remove_file(&tmp_visits_path);
                    return;
                }
            }
        });
    }

    pub(crate) fn update_file_with_visit(visit_id: VisitId) {
        let mut current = Visits::from_file();
        current.mark_visit(visit_id, SystemTime::now());
        current.write_to_file();
    }
}

static MATCHER: LazyLock<ArinaeMatcher> =
    LazyLock::new(|| ArinaeMatcher::new(skim::CaseMatching::Smart, true, true));

/// Does not add frecency.
pub(crate) fn accuracy(query: &str, item: &ListItem, weights: Weights) -> f32 {
    let title_score = (weights.title != 0.0)
        .then(|| MATCHER.fuzzy_match(&item.title, query))
        .flatten()
        .unwrap_or(0);
    let desc_score = (weights.description != 0.0)
        .then(|| MATCHER.fuzzy_match(&item.description, query))
        .flatten()
        .unwrap_or(0);

    title_score as f32 * weights.title + desc_score as f32 * weights.description
}

pub(crate) fn frecency(
    item: &ListItem,
    visits: &Visits,
    now: SystemTime,
    weights: Weights,
) -> Frecency {
    match visits.map.get(item.visit_id()) {
        Some(VisitScore {
            last_visit,
            decayed_score,
        }) => {
            let secs_since_last_visit = time_diff_secs(now, *last_visit);
            // Scaling up here (3.0) since the accuracy score seems to dominate most of the
            // time. Don't want to make the default frecency weight something other than
            // 1.0 either.
            let frecency = (weights.frecency * 3.0)
                * 2.4
                * f32::ln(
                    0.1 + 10.0 / (1.0 + secs_since_last_visit * SHORT_DECAY)
                        + f32::exp(-LONG_DECAY * secs_since_last_visit) * decayed_score,
                );
            Frecency(frecency)
        }
        None => Frecency::ZERO,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Frecency(f32);

impl Frecency {
    pub const ZERO: Self = Self(0.0);

    pub fn combine_with_accuracy(self, accuracy: f32) -> f32 {
        self.0 + accuracy
    }
}

/// Ranks a slice of list items by the given weights in descending order of
/// score.
///
/// Returns up to 100 items that have a score of at least 1.
#[expect(clippy::unused_async, reason = "may require async in the future")]
pub async fn rank(query: &str, items: &[ListItem], weights: Weights) -> Vec<ListItem> {
    let mut scored: Vec<_> = if weights.frecency == 0.0 {
        items
            .iter()
            .map(|item| (item, item.accuracy(query, weights)))
            .filter(|(_, score)| query.is_empty() || *score > 1.0)
            .collect()
    } else {
        let visits = Visits::from_file();
        let now = SystemTime::now();

        items
            .iter()
            .map(|item| (item, item.score(query, &visits, now, weights)))
            .filter(|(_, score)| query.is_empty() || *score > 1.0)
            .collect()
    };

    // reverse order
    scored.sort_unstable_by(|(_, s1), (_, s2)| s2.total_cmp(s1));
    scored
        .into_iter()
        .map(|(item, _)| item)
        .take(100)
        .cloned()
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub struct Weights {
    title: f32,
    description: f32,
    pub(crate) frecency: f32,
}

impl Weights {
    /// Weights based on the title as well as frecency.
    pub fn with_history() -> Self {
        Self::without_history().frecency(1.0)
    }

    /// Weights based on the title only, not including usage history.
    pub fn without_history() -> Self {
        Self {
            title: 1.0,
            description: 0.5,
            frecency: 0.0,
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
    pub fn frecency(mut self, frecency: f32) -> Self {
        self.frecency = frecency;
        self
    }
}
