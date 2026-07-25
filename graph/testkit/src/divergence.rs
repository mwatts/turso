//! Registry of behaviors Turso deliberately does not implement because they
//! are specific to one other database.
//!
//! CONFORMANCE.md used to state a count. A count in prose drifts silently: a
//! divergence can be gained, lost, or renamed with nothing to notice. Each
//! entry here names the tests that prove it, and `verify` fails when the
//! registry and a recorded corpus run disagree in either direction.

use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    identity::TestId,
    model::{Outcome, ResultRecord},
};

pub const DIVERGENCE_REGISTRY_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DivergenceRegistry {
    pub version: u32,
    #[serde(default)]
    pub entry: Vec<DivergenceEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DivergenceEntry {
    /// Stable slug, e.g. `apache-age.vertex_stats`.
    pub id: String,
    /// The database whose behavior this is.
    pub vendor: String,
    /// Language area: `vendor-function`, `vendor-operator`, `vendor-ddl`, …
    pub area: String,
    /// Why Turso does not implement it.
    pub reason: String,
    /// Corpus identities that exercise it. Never empty.
    pub tests: Vec<TestId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DivergenceReport {
    pub registered: usize,
    pub matched: usize,
}

#[derive(Debug, Error)]
pub enum DivergenceError {
    #[error("failed to read divergence registry {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse divergence registry {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error("unsupported divergence registry version {0}")]
    Version(u32),
    #[error("divergence `{id}` names no test")]
    EmptyEntry { id: String },
    #[error("test `{test_id}` is claimed by divergences `{first}` and `{second}`")]
    DuplicateTest {
        test_id: String,
        first: String,
        second: String,
    },
    #[error("test `{test_id}` reported an unsupported outcome but no divergence entry claims it")]
    Unregistered { test_id: String },
    #[error("divergence `{id}` names test `{test_id}`, which the run does not contain")]
    MissingTest { id: String, test_id: String },
    #[error(
        "divergence `{id}` names test `{test_id}`, which now reports `{outcome:?}`: remove the entry"
    )]
    NoLongerDivergent {
        id: String,
        test_id: String,
        outcome: Outcome,
    },
}

impl DivergenceRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, DivergenceError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| DivergenceError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let registry: Self = toml::from_str(&content).map_err(|source| DivergenceError::Parse {
            path: path.display().to_string(),
            source,
        })?;
        if registry.version != DIVERGENCE_REGISTRY_VERSION {
            return Err(DivergenceError::Version(registry.version));
        }
        registry.claims()?;
        Ok(registry)
    }

    /// test id -> owning entry id, rejecting empty entries and double claims.
    fn claims(&self) -> Result<BTreeMap<String, String>, DivergenceError> {
        let mut claims = BTreeMap::new();
        for entry in &self.entry {
            if entry.tests.is_empty() {
                return Err(DivergenceError::EmptyEntry {
                    id: entry.id.clone(),
                });
            }
            for test in &entry.tests {
                if let Some(first) = claims.insert(test.to_string(), entry.id.clone()) {
                    return Err(DivergenceError::DuplicateTest {
                        test_id: test.to_string(),
                        first,
                        second: entry.id.clone(),
                    });
                }
            }
        }
        Ok(claims)
    }

    pub fn verify(&self, records: &[ResultRecord]) -> Result<DivergenceReport, DivergenceError> {
        let claims = self.claims()?;
        let observed: BTreeMap<String, Outcome> = records
            .iter()
            .map(|record| (record.test_id.to_string(), record.outcome))
            .collect();

        // Direction 1: nothing diverges without being named.
        for (test_id, outcome) in &observed {
            if *outcome == Outcome::Unsupported && !claims.contains_key(test_id) {
                return Err(DivergenceError::Unregistered {
                    test_id: test_id.clone(),
                });
            }
        }

        // Direction 2: nothing is named without still diverging.
        let mut matched = 0;
        for (test_id, id) in &claims {
            match observed.get(test_id) {
                None => {
                    return Err(DivergenceError::MissingTest {
                        id: id.clone(),
                        test_id: test_id.clone(),
                    })
                }
                Some(Outcome::Unsupported) => matched += 1,
                Some(outcome) => {
                    return Err(DivergenceError::NoLongerDivergent {
                        id: id.clone(),
                        test_id: test_id.clone(),
                        outcome: *outcome,
                    })
                }
            }
        }

        Ok(DivergenceReport {
            registered: claims.len(),
            matched,
        })
    }

    /// Build a registry from a recorded run. Used once to seed the file and
    /// afterwards only to regenerate it deliberately; `reason` needs a human.
    pub fn sync(records: &[ResultRecord]) -> Self {
        let mut grouped: BTreeMap<String, DivergenceEntry> = BTreeMap::new();
        for record in records
            .iter()
            .filter(|record| record.outcome == Outcome::Unsupported)
        {
            // AGE stamps the offending function name as a dimension; group by
            // it so one entry covers one behavior, not one test.
            let behavior = record
                .dimensions
                .get("vendor_unsupported_function")
                .cloned()
                .unwrap_or_else(|| record.area.clone());
            let id = format!("{}.{behavior}", slug(&record.source.name));
            grouped
                .entry(id.clone())
                .or_insert_with(|| DivergenceEntry {
                    id,
                    vendor: record.source.name.clone(),
                    area: "vendor-function".to_owned(),
                    reason: "TODO: state why Turso does not implement this".to_owned(),
                    tests: Vec::new(),
                })
                .tests
                .push(record.test_id.clone());
        }
        for entry in grouped.values_mut() {
            entry.tests.sort();
        }
        Self {
            version: DIVERGENCE_REGISTRY_VERSION,
            entry: grouped.into_values().collect(),
        }
    }
}

fn slug(name: &str) -> String {
    name.to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}
