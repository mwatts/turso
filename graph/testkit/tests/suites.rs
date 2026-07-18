use std::{collections::HashSet, path::PathBuf};

use turso_graph_testkit::{
    manifest::ScenarioManifest,
    model::{Outcome, RunEnvironment},
    performance::PerformanceManifest,
    runner::ScenarioRunner,
};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn environment() -> RunEnvironment {
    RunEnvironment {
        git_commit: "0".repeat(40),
        git_dirty: false,
        package_version: env!("CARGO_PKG_VERSION").to_owned(),
        profile: "test".to_owned(),
        os: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
    }
}

#[test]
fn deep_suite_has_unique_identities_and_no_unclassified_failures() {
    let root = repository_root();
    let paths = [
        root.join("graph/testdata/suites/conformance.toml"),
        root.join("graph/testdata/suites/portable.toml"),
        root.join("graph/testdata/suites/regressions.toml"),
    ];
    let mut identities = HashSet::new();
    let mut scenarios = Vec::new();
    for path in paths {
        let manifest = ScenarioManifest::load(path).unwrap();
        for scenario in manifest.scenario {
            assert!(identities.insert(scenario.id.clone()));
            if scenario.tiers.iter().any(|tier| tier == "deep") {
                scenarios.push(scenario);
            }
        }
    }
    assert_eq!(scenarios.len(), 38);
    let runner = ScenarioRunner::new(environment(), "integration-deep", "deep");
    for scenario in &scenarios {
        let record = runner.run(scenario).unwrap();
        assert!(
            matches!(record.outcome, Outcome::Passed | Outcome::Unsupported),
            "{}: {:?}",
            record.test_id,
            record.message
        );
    }
}

#[test]
fn performance_smoke_covers_each_lifecycle_operation_at_each_scale() {
    let manifest =
        PerformanceManifest::load(repository_root().join("graph/testdata/suites/performance.toml"))
            .unwrap();
    let records = manifest
        .run("smoke", environment(), "integration-performance")
        .unwrap();
    assert_eq!(records.len(), 10);
    assert!(records
        .iter()
        .all(|record| record.outcome == Outcome::Passed));
    for operation in ["create", "bulk-load", "load", "query", "delete"] {
        assert_eq!(
            records
                .iter()
                .filter(|record| record.operation.as_deref() == Some(operation))
                .count(),
            2
        );
    }
}
