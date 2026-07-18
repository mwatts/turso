use std::{collections::HashSet, path::PathBuf};

use turso_graph_testkit::{
    age::AgeCorpus,
    grafeo::GrafeoCorpus,
    ladybug::LadybugCorpus,
    manifest::ScenarioManifest,
    model::{Outcome, RunEnvironment},
    performance::PerformanceManifest,
    runner::ScenarioRunner,
    rust_donor::{RustDonorCorpus, CQLITE, SPARROWDB},
    tck::TckCorpus,
};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn vendored_corpus_has_all_source_identities() {
    let root = repository_root();
    let tck = TckCorpus::load(root.join("graph/testdata/tck/opencypher/features")).unwrap();
    let grafeo = GrafeoCorpus::load(root.join("graph/testdata/donors/grafeo/tests")).unwrap();
    let ladybug =
        LadybugCorpus::load(root.join("graph/testdata/donors/ladybug/test_files")).unwrap();
    let age = AgeCorpus::load(root.join("graph/testdata/donors/age/sql")).unwrap();
    let sparrowdb = RustDonorCorpus::load(
        root.join("graph/testdata/donors/sparrowdb/tests"),
        SPARROWDB,
    )
    .unwrap();
    let cqlite =
        RustDonorCorpus::load(root.join("graph/testdata/donors/cqlite/tests"), CQLITE).unwrap();

    assert_eq!(
        tck.stats().expanded
            + grafeo.stats().cypher_cases
            + ladybug.stats().statements
            + age.stats().queries
            + sparrowdb.stats().queries
            + cqlite.stats().queries,
        26_332
    );
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
