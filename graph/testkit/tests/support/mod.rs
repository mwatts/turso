use std::path::PathBuf;

use turso_graph_testkit::identity::TestId;
use turso_graph_testkit::model::{
    Expectation, Outcome, ResultRecord, RunEnvironment, SourceIdentity, TestKind,
    HISTORY_SCHEMA_VERSION,
};

pub fn registry_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("graph/testkit has a parent")
        .join("registries")
}

pub fn record_with(test_id: &str, outcome: Outcome) -> ResultRecord {
    let mut record = ResultRecord {
        schema_version: HISTORY_SCHEMA_VERSION,
        semantics_version: turso_graph_ir::SEMANTIC_PROFILE_VERSION,
        run_id: "20260101T000000.000000Z-testtesttest-corpus-deep".to_owned(),
        recorded_at: "2026-07-25T00:00:00.000000Z".to_owned(),
        environment: RunEnvironment {
            git_commit: "0".repeat(40),
            git_dirty: false,
            package_version: "0.0.0".to_owned(),
            profile: "dev".to_owned(),
            os: "macos".to_owned(),
            architecture: "aarch64".to_owned(),
        },
        suite: "age-deep".to_owned(),
        test_id: TestId::parse(test_id).expect("valid test id"),
        kind: TestKind::Conformance,
        area: "age_global_graph".to_owned(),
        fixture: "empty".to_owned(),
        expectation: Expectation::Unsupported,
        outcome,
        duration_ns: 0,
        source: SourceIdentity {
            name: "Apache AGE".to_owned(),
            repository: "https://github.com/apache/age".to_owned(),
            revision: "0".repeat(40),
            path: "regress/sql/x.sql".to_owned(),
            case: "x".to_owned(),
            license: "Apache-2.0".to_owned(),
            adaptation: "fixture-adaptation".to_owned(),
            issue: None,
            fixed_commit: None,
        },
        operation: None,
        graph_shape: None,
        scale: None,
        iterations: None,
        throughput_per_second: None,
        row_count: None,
        node_count: None,
        relationship_count: None,
        result_digest: None,
        message: None,
        dimensions: Default::default(),
    };
    record.dimensions.insert(
        "vendor_unsupported_function".to_owned(),
        "vertex_stats".to_owned(),
    );
    record
}
