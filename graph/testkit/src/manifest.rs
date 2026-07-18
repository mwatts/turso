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
