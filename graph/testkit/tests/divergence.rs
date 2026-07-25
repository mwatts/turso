//! The registry turns "53 unsupported vendor behaviors" from a sentence in
//! CONFORMANCE.md into a checked fact. Every unsupported outcome in a recorded
//! corpus run must be named by exactly one registry entry, and every entry must
//! name at least one test that the run actually contains.

use turso_graph_testkit::divergence::{DivergenceError, DivergenceRegistry};
use turso_graph_testkit::model::Outcome;

mod support;
use support::{record_with, registry_root};

#[test]
fn the_checked_in_registry_loads() {
    DivergenceRegistry::load(registry_root().join("divergence.toml"))
        .expect("graph/registries/divergence.toml must parse");
}

#[test]
fn every_registry_entry_names_at_least_one_test() {
    let registry =
        DivergenceRegistry::load(registry_root().join("divergence.toml")).expect("registry loads");
    for entry in &registry.entry {
        assert!(
            !entry.tests.is_empty(),
            "divergence `{}` claims a behavior with no test to prove it",
            entry.id
        );
        assert!(
            !entry.reason.trim().is_empty(),
            "divergence `{}` has no reason",
            entry.id
        );
        assert!(
            !entry.reason.contains("TODO"),
            "divergence `{}` still carries a generated placeholder reason",
            entry.id
        );
    }
}

#[test]
fn an_unsupported_outcome_with_no_registry_entry_fails_verification() {
    let registry = DivergenceRegistry {
        version: 1,
        entry: Vec::new(),
    };
    let records = vec![record_with(
        "age.age.global.graph.query-4",
        Outcome::Unsupported,
    )];

    let error = registry
        .verify(&records)
        .expect_err("an unregistered divergence must fail CI");
    assert!(
        matches!(error, DivergenceError::Unregistered { ref test_id } if test_id == "age.age.global.graph.query-4"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_registry_entry_whose_test_vanished_fails_verification() {
    let registry = DivergenceRegistry::sync(&[record_with(
        "age.age.global.graph.query-4",
        Outcome::Unsupported,
    )]);
    // The run no longer contains the test the entry names, so the claim is
    // stale. That is exactly the drift the registry exists to catch.
    let error = registry
        .verify(&[record_with("age.age.global.graph.query-9", Outcome::Passed)])
        .expect_err("a missing test must fail CI");
    assert!(
        matches!(error, DivergenceError::MissingTest { ref test_id, .. } if test_id == "age.age.global.graph.query-4"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_divergence_that_started_passing_fails_verification() {
    // Newly gained support is good news that still has to be recorded: the
    // entry must be removed, not left claiming an unsupported behavior.
    let registry = DivergenceRegistry::sync(&[record_with(
        "age.age.global.graph.query-4",
        Outcome::Unsupported,
    )]);
    let error = registry
        .verify(&[record_with("age.age.global.graph.query-4", Outcome::Passed)])
        .expect_err("a now-supported divergence must fail CI");
    assert!(
        matches!(error, DivergenceError::NoLongerDivergent { ref test_id, .. } if test_id == "age.age.global.graph.query-4"),
        "unexpected error: {error}"
    );
}

#[test]
fn a_matching_run_verifies_and_counts() {
    let records = vec![
        record_with("age.age.global.graph.query-4", Outcome::Unsupported),
        record_with("age.age.global.graph.query-5", Outcome::Unsupported),
        record_with("age.age.global.graph.query-6", Outcome::Passed),
    ];
    let registry = DivergenceRegistry::sync(&records);
    let report = registry.verify(&records).expect("registry matches the run");
    assert_eq!(
        report.matched, 2,
        "both unsupported outcomes are accounted for"
    );
}

#[test]
fn the_registry_accounts_for_every_divergent_test_in_the_corpus() {
    // CONFORMANCE.md quotes this number. The registry is what makes it true.
    let registry =
        DivergenceRegistry::load(registry_root().join("divergence.toml")).expect("registry loads");
    let total: usize = registry.entry.iter().map(|entry| entry.tests.len()).sum();
    assert_eq!(
        total, 53,
        "the divergence count moved; update CONFORMANCE.md in the same commit"
    );
}
