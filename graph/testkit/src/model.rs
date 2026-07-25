use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::identity::TestId;

/// 2 added `semantics_version`. Version 1 rows predate the semantic profile
/// and read back with `semantics_version: 0`.
pub const HISTORY_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TestKind {
    Smoke,
    Conformance,
    Regression,
    BugRegression,
    Performance,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Expectation {
    Rows,
    Error,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Passed,
    Failed,
    Unsupported,
    UnexpectedlySupported,
    ResourceExhausted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceIdentity {
    pub name: String,
    pub repository: String,
    pub revision: String,
    pub path: String,
    pub case: String,
    pub license: String,
    pub adaptation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_commit: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunEnvironment {
    pub git_commit: String,
    pub git_dirty: bool,
    pub package_version: String,
    pub profile: String,
    pub os: String,
    pub architecture: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResultRecord {
    pub schema_version: u32,
    /// `turso_graph_ir::SEMANTIC_PROFILE_VERSION` in force when the verdict was
    /// produced. 0 means the row predates the profile and its rules are unknown.
    #[serde(default)]
    pub semantics_version: u32,
    pub run_id: String,
    pub recorded_at: String,
    pub environment: RunEnvironment,
    pub suite: String,
    pub test_id: TestId,
    pub kind: TestKind,
    pub area: String,
    pub fixture: String,
    pub expectation: Expectation,
    pub outcome: Outcome,
    pub duration_ns: u64,
    pub source: SourceIdentity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_shape: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterations: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub throughput_per_second: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dimensions: BTreeMap<String, String>,
}

impl ResultRecord {
    pub fn history_key(&self) -> (&str, &TestId, Option<&str>) {
        (&self.run_id, &self.test_id, self.operation.as_deref())
    }
}
