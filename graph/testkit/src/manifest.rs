use std::{collections::HashSet, fs, path::Path};

use serde::Deserialize;
use thiserror::Error;

use crate::{
    identity::TestId,
    model::{Expectation, SourceIdentity, TestKind},
};

#[derive(Debug, Deserialize)]
pub struct ScenarioManifest {
    pub version: u32,
    pub purpose: String,
    pub scenario: Vec<Scenario>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Scenario {
    pub id: TestId,
    pub tiers: Vec<String>,
    pub kind: TestKind,
    pub area: String,
    pub fixture: String,
    pub expectation: Expectation,
    pub action: String,
    pub ordering: String,
    pub query: String,
    #[serde(default)]
    pub setup_sql: Vec<String>,
    #[serde(default)]
    pub parameters: std::collections::BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub expected_rows: Vec<Vec<String>>,
    pub expected_error_contains: Option<String>,
    pub verification_query: Option<String>,
    pub source: SourceIdentity,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read manifest {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse manifest {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
    #[error("unsupported manifest schema version {0}")]
    Version(u32),
    #[error("manifest purpose must not be empty")]
    EmptyPurpose,
    #[error("manifest discovered zero scenarios")]
    Empty,
    #[error("duplicate scenario identity `{0}`")]
    Duplicate(TestId),
    #[error("scenario `{id}` is invalid: {reason}")]
    Invalid { id: TestId, reason: String },
}

impl ScenarioManifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| ManifestError::Read {
            path: path.display().to_string(),
            source,
        })?;
        let manifest = toml::from_str::<Self>(&content).map_err(|source| ManifestError::Parse {
            path: path.display().to_string(),
            source,
        })?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.version != 1 {
            return Err(ManifestError::Version(self.version));
        }
        if self.purpose.trim().is_empty() {
            return Err(ManifestError::EmptyPurpose);
        }
        if self.scenario.is_empty() {
            return Err(ManifestError::Empty);
        }
        let mut ids = HashSet::new();
        for scenario in &self.scenario {
            if !ids.insert(scenario.id.clone()) {
                return Err(ManifestError::Duplicate(scenario.id.clone()));
            }
            let invalid = scenario.tiers.is_empty()
                || !scenario
                    .tiers
                    .iter()
                    .all(|tier| matches!(tier.as_str(), "smoke" | "deep"))
                || !matches!(scenario.action.as_str(), "query" | "mutation")
                || !matches!(scenario.ordering.as_str(), "ordered" | "unordered")
                || scenario.source.revision.len() != 40
                || !scenario
                    .source
                    .repository
                    .starts_with("https://github.com/")
                || !matches!(scenario.source.license.as_str(), "MIT" | "Apache-2.0");
            if invalid {
                return Err(ManifestError::Invalid {
                    id: scenario.id.clone(),
                    reason: "tier, action, ordering, or provenance contract failed".to_owned(),
                });
            }
            // SEMANTIC_PROFILE.row_order is OrderedOnlyUnderExplicitOrderBy, so
            // a scenario that claims an order without asking for one is pinned
            // to SQLite's B-tree layout and fails on an index change while
            // nothing is broken. A mutation's compared rows come from its
            // verification query, so that is where the ORDER BY has to be.
            let ordering_source = scenario
                .verification_query
                .as_deref()
                .unwrap_or(&scenario.query);
            if scenario.ordering == "ordered"
                && !ordering_source.to_ascii_uppercase().contains("ORDER BY")
            {
                return Err(ManifestError::Invalid {
                    id: scenario.id.clone(),
                    reason: "ordering = \"ordered\" requires ORDER BY in the query".to_owned(),
                });
            }
            if scenario.expectation == Expectation::Rows
                && scenario.action == "mutation"
                && scenario.verification_query.is_none()
            {
                return Err(ManifestError::Invalid {
                    id: scenario.id.clone(),
                    reason: "mutation row expectation requires verification_query".to_owned(),
                });
            }
            if matches!(
                scenario.expectation,
                Expectation::Error | Expectation::Unsupported
            ) && scenario.expected_error_contains.is_none()
            {
                return Err(ManifestError::Invalid {
                    id: scenario.id.clone(),
                    reason: "error expectation requires expected_error_contains".to_owned(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> ScenarioManifest {
        ScenarioManifest {
            version: 1,
            purpose: "exercise manifest validation".to_owned(),
            scenario: vec![Scenario {
                id: TestId::parse("turso.manifest.ordering.sample").unwrap(),
                tiers: vec!["smoke".to_owned()],
                kind: TestKind::Smoke,
                area: "expression".to_owned(),
                fixture: "social".to_owned(),
                expectation: Expectation::Rows,
                action: "query".to_owned(),
                ordering: "unordered".to_owned(),
                query: "MATCH (n) RETURN n.name".to_owned(),
                setup_sql: Vec::new(),
                parameters: std::collections::BTreeMap::new(),
                expected_rows: Vec::new(),
                expected_error_contains: None,
                verification_query: None,
                source: SourceIdentity {
                    name: "Turso".to_owned(),
                    repository: "https://github.com/tursodatabase/turso".to_owned(),
                    revision: "a".repeat(40),
                    path: "graph/testdata/sample.toml".to_owned(),
                    case: "sample".to_owned(),
                    license: "MIT".to_owned(),
                    adaptation: "turso-authored".to_owned(),
                    issue: None,
                    fixed_commit: None,
                },
            }],
        }
    }

    #[test]
    fn the_sample_manifest_is_otherwise_valid() {
        // Guards the tests below: a failure here would make them pass for the
        // wrong reason.
        sample_manifest().validate().expect("sample is valid");
    }

    #[test]
    fn a_scenario_may_only_claim_ordered_when_its_query_says_order_by() {
        // Declaring a result ordered when the query never asked for an order
        // pins the test to SQLite's B-tree layout. That test fails on an index
        // change while nothing is broken.
        let mut manifest = sample_manifest();
        manifest.scenario[0].ordering = "ordered".to_owned();
        manifest.scenario[0].query = "MATCH (n) RETURN n.name".to_owned();

        let error = manifest
            .validate()
            .expect_err("ordered claim must be rejected");

        assert!(
            matches!(error, ManifestError::Invalid { ref reason, .. } if reason.contains("ORDER BY")),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn an_ordered_scenario_with_order_by_validates() {
        let mut manifest = sample_manifest();
        manifest.scenario[0].ordering = "ordered".to_owned();
        manifest.scenario[0].query = "MATCH (n) RETURN n.name ORDER BY n.name".to_owned();

        manifest
            .validate()
            .expect("ORDER BY justifies the ordered claim");
    }

    #[test]
    fn a_mutation_reads_its_order_from_the_verification_query() {
        // A mutation's compared rows come from verification_query, not from the
        // statement itself, so that is where ORDER BY has to appear.
        let mut manifest = sample_manifest();
        manifest.scenario[0].action = "mutation".to_owned();
        manifest.scenario[0].ordering = "ordered".to_owned();
        manifest.scenario[0].query = "CREATE (:Person {name: 'Ada'})".to_owned();
        manifest.scenario[0].verification_query =
            Some("MATCH (n:Person) RETURN n.name ORDER BY n.name".to_owned());

        manifest
            .validate()
            .expect("the verification query defines the order");
    }
}
