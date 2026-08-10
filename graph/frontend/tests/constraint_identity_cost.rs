//! Identity-scoped required/value validation must not re-scan every row of a
//! type on every CREATE, and value predicates must stop at the first violation
//! in SQL (`LIMIT 1`) rather than materialising the whole property column.
//!
//! Correctness of the narrowed checks is pinned in
//! `constraint_validation_scope`. What is measured here is the SQL shape of a
//! steady-state CREATE after bulk load.
//!
//! One test in this binary, by design: see `prepared_sql`.

mod prepared_sql;

use std::sync::Arc;

use prepared_sql::PreparedSql;
use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    register_graph, register_semantic_constraints, register_semantic_schema, GraphConnection,
    GraphRegistration, NodeSourceRegistration, Parameters, SemanticConstraintRegistration,
    SemanticNodeType, SemanticProperty, SemanticPropertyValueConstraint, SemanticRangeBound,
    SemanticRequiredProperty, SemanticScalar, SemanticSchemaRegistration, SemanticValuePredicate,
};

fn graph_with_required_and_range() -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:identity-cost",
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute("CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, score INTEGER)")
        .expect("create source table");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "ontology".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "people".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        },
    )
    .expect("register graph");
    register_semantic_schema(
        &connection,
        "ontology",
        &SemanticSchemaRegistration {
            node_types: vec![SemanticNodeType {
                name: "Person".to_owned(),
                source: "people".to_owned(),
                properties: vec![
                    SemanticProperty {
                        name: "name".to_owned(),
                        column: "name".to_owned(),
                    },
                    SemanticProperty {
                        name: "score".to_owned(),
                        column: "score".to_owned(),
                    },
                ],
            }],
            relationship_types: Vec::new(),
        },
    )
    .expect("register semantic schema");
    register_semantic_constraints(
        &connection,
        "ontology",
        &SemanticConstraintRegistration {
            required: vec![SemanticRequiredProperty {
                owner: "Person".to_owned(),
                property: "name".to_owned(),
            }],
            values: vec![SemanticPropertyValueConstraint {
                owner: "Person".to_owned(),
                property: "score".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(0),
                        inclusive: true,
                    }),
                    maximum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(100),
                        inclusive: true,
                    }),
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register semantic constraints");
    database
}

#[test]
fn steady_create_filters_required_and_value_to_written_ids_with_limit_one() {
    let recorded = PreparedSql::install();
    let database = graph_with_required_and_range();
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    // First CREATE prepares the constraint SQL. Later creates reuse the same
    // text via StatementCache, so the shape under test is only visible on the
    // first prepare — capture that, then prove a bulk load still reuses it.
    recorded.take();
    session
        .execute(
            "CREATE (:Person {name: 'first', score: 1})",
            &Parameters::new(),
        )
        .expect("first create");
    let first = recorded.take();

    let required: Vec<&String> = first
        .iter()
        .filter(|sql| sql.contains("IS NULL") && sql.contains("AS entity"))
        .collect();
    assert!(
        !required.is_empty(),
        "expected a required-property probe on first CREATE, got {first:#?}"
    );
    for sql in &required {
        assert!(
            sql.contains("__turso_graph_mutation_written"),
            "required check must filter through the written-identities temp table, got: {sql}"
        );
        assert!(
            sql.contains("LIMIT 1"),
            "required check must stop at the first violation, got: {sql}"
        );
    }

    let value: Vec<&String> = first
        .iter()
        .filter(|sql| {
            sql.contains("AS entity")
                && sql.contains("score")
                && !sql.contains("IS NULL")
                && sql.contains("EXISTS")
        })
        .collect();
    assert!(
        !value.is_empty(),
        "expected a value-predicate probe against score on first CREATE, got {first:#?}"
    );
    for sql in &value {
        assert!(
            sql.contains("__turso_graph_mutation_written"),
            "value check must filter through the written-identities temp table, got: {sql}"
        );
        assert!(
            sql.contains("LIMIT 1"),
            "value check must use LIMIT 1 instead of materialising every score, got: {sql}"
        );
        assert!(
            sql.contains("typeof("),
            "value check must typeof-guard the column so TEXT cannot coerce past a numeric \
             range (the SQL path, not only Rust validate_runtime), got: {sql}"
        );
    }

    // Bulk load, then one more CREATE: constraint SQL must not recompile
    // (identity filter is the temp table, not inlined ids).
    for index in 0..25 {
        session
            .execute(
                &format!("CREATE (:Person {{name: 'warm{index}', score: {index}}})"),
                &Parameters::new(),
            )
            .expect("bulk create");
    }
    recorded.take();
    session
        .execute(
            "CREATE (:Person {name: 'steady', score: 50})",
            &Parameters::new(),
        )
        .expect("steady create");
    let steady = recorded.take();
    let recompiled_constraints: Vec<&String> = steady
        .iter()
        .filter(|sql| sql.contains("AS entity") && sql.contains("EXISTS"))
        .collect();
    assert!(
        recompiled_constraints.is_empty(),
        "steady CREATE after bulk load recompiled constraint SQL (identity filter must not \
         embed row ids in the prepared text): {recompiled_constraints:#?}"
    );
}
