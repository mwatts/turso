//! Statement classification is syntactic and never result-dependent.
//!
//! Callers previously routed by trying `query()` and falling back to
//! `execute()` when it errored, which turns a genuine read failure into a
//! mutation attempt. `SEMANTIC_PROFILE.write_classification` is
//! `SyntacticNeverResultDependent`: whether a statement writes is decided by
//! what it says, not by what it changed.

use turso_graph_frontend::{classify_statement, StatementKind};

fn classify(source: &str) -> StatementKind {
    let query = turso_graph_cypher::parse(source).expect("source parses");
    classify_statement(&query)
}

#[test]
fn a_match_return_is_read_only() {
    assert_eq!(classify("MATCH (n) RETURN n"), StatementKind::ReadOnly);
}

#[test]
fn a_with_pipeline_without_mutation_is_read_only() {
    assert_eq!(
        classify("MATCH (n) WITH n WHERE n.name > 'a' RETURN n.name ORDER BY n.name"),
        StatementKind::ReadOnly
    );
}

#[test]
fn a_create_without_return_writes_without_rows() {
    assert_eq!(
        classify("CREATE (n:Person {name: 'a'})"),
        StatementKind::WriteWithoutRows
    );
}

#[test]
fn a_create_with_return_writes_and_returns_rows() {
    assert_eq!(
        classify("CREATE (n:Person {name: 'a'}) RETURN n"),
        StatementKind::WriteReturningRows
    );
}

#[test]
fn a_delete_that_can_match_nothing_is_still_a_write() {
    // The rule that makes classification useful: emptiness is a runtime fact,
    // and a runtime fact must never change a compile-time classification.
    // Otherwise a read-only connection would accept a DELETE and reject it only
    // when it happened to match a row.
    assert_eq!(
        classify("MATCH (n:NoSuchLabelAnywhere) DELETE n"),
        StatementKind::WriteWithoutRows
    );
}

#[test]
fn set_remove_merge_and_detach_delete_all_write() {
    for source in [
        "MATCH (n) SET n.name = 'a'",
        "MATCH (n) REMOVE n.name",
        "MERGE (n:Person {name: 'a'})",
        "MATCH (n) DETACH DELETE n",
        "MATCH (n) SET n:Archived",
    ] {
        assert!(
            classify(source).writes(),
            "`{source}` must classify as a write"
        );
    }
}

#[test]
fn a_write_nested_in_foreach_still_writes() {
    // FOREACH carries its own clause list. Reading only the top level would
    // classify a statement that mutates every element of a list as a read.
    assert!(
        classify("MATCH (n) FOREACH (x IN [1, 2] | SET n.name = 'a')").writes(),
        "a FOREACH body that sets a property is a write"
    );
}

#[test]
fn a_write_nested_in_a_call_subquery_still_writes() {
    assert!(
        classify("MATCH (n) CALL { CREATE (:Audit) } RETURN n").writes(),
        "a CALL subquery that creates a node is a write"
    );
}

#[test]
fn a_write_in_a_union_branch_still_writes() {
    assert!(
        classify("MATCH (n) RETURN n UNION CREATE (m:Person) RETURN m").writes(),
        "a union branch that creates a node is a write"
    );
}

#[test]
fn read_only_never_writes() {
    assert!(!StatementKind::ReadOnly.writes());
    assert!(StatementKind::WriteReturningRows.writes());
    assert!(StatementKind::WriteWithoutRows.writes());
}
