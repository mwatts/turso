//! Narrowing validation must not turn validation off.
//!
//! `validate_state` only re-checks constraints whose source table the statement
//! wrote. That the cost stops growing with the size of the ontology is measured
//! in `constraint_scope_cost`; what is pinned here is that every constraint
//! still in scope fires — including the ones a naive narrowing would miss.

use std::sync::Arc;

use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    register_graph, register_semantic_constraints, register_semantic_schema, GraphConnection,
    GraphRegistration, NodeSourceRegistration, Parameters, SemanticConstraintRegistration,
    SemanticNodeType, SemanticProperty, SemanticPropertyValueConstraint, SemanticRangeBound,
    SemanticRequiredProperty, SemanticScalar, SemanticSchemaRegistration, SemanticUniqueProperty,
    SemanticValuePredicate,
};

/// A graph of `types` unrelated node types, each on its own table, each with a
/// required and a unique property. This is the shape of an ontology install:
/// many types that share nothing, written one at a time.
fn graph_with_unrelated_types(types: usize) -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        &format!(":memory:validation-fires-{types}"),
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    for index in 0..types {
        connection
            .execute(format!(
                "CREATE TABLE t{index}(id INTEGER PRIMARY KEY, name TEXT)"
            ))
            .expect("create source table");
    }
    register_graph(
        &connection,
        &GraphRegistration {
            name: "ontology".to_owned(),
            node_sources: (0..types)
                .map(|index| NodeSourceRegistration {
                    name: format!("src{index}"),
                    table: format!("t{index}"),
                    identity_column: "id".to_owned(),
                })
                .collect(),
            relationship_sources: Vec::new(),
        },
    )
    .expect("register graph");
    register_semantic_schema(
        &connection,
        "ontology",
        &SemanticSchemaRegistration {
            node_types: (0..types)
                .map(|index| SemanticNodeType {
                    name: format!("Type{index}"),
                    source: format!("src{index}"),
                    properties: vec![SemanticProperty {
                        name: "name".to_owned(),
                        column: "name".to_owned(),
                    }],
                })
                .collect(),
            relationship_types: Vec::new(),
        },
    )
    .expect("register semantic schema");
    register_semantic_constraints(
        &connection,
        "ontology",
        &SemanticConstraintRegistration {
            required: (0..types)
                .map(|index| SemanticRequiredProperty {
                    owner: format!("Type{index}"),
                    property: "name".to_owned(),
                })
                .collect(),
            unique: (0..types)
                .map(|index| SemanticUniqueProperty {
                    owner: format!("Type{index}"),
                    property: "name".to_owned(),
                })
                .collect(),
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register semantic constraints");
    database
}

#[test]
fn a_write_nested_in_foreach_is_still_in_scope() {
    // Writes are not all in `request.operations`. FOREACH nests them inside
    // stage items, and a scope walk that reads only the top level declares the
    // statement touched nothing and skips validation entirely.
    let database = graph_with_unrelated_types(4);
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Type3 {name: 'taken'})", &Parameters::new())
        .expect("seed the duplicate");
    let error = session
        .execute(
            "FOREACH (n IN [1] | CREATE (:Type3 {name: 'taken'}))",
            &Parameters::new(),
        )
        .expect_err("a duplicate created inside FOREACH must still be rejected");
    let message = error.to_string();
    assert!(
        message.contains("duplicate"),
        "expected a duplicate-value violation from inside FOREACH, got: {message}"
    );
}

#[test]
fn a_required_property_still_fails_on_the_type_being_written() {
    // Narrowing must not turn validation off for the source in scope.
    let database = graph_with_unrelated_types(5);
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    let error = session
        .execute("CREATE (:Type2 {name: null})", &Parameters::new())
        .expect_err("a NULL required property must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("name"),
        "expected the required-property violation to name the property, got: {message}"
    );
}

#[test]
fn a_unique_property_still_fails_against_a_row_written_earlier() {
    // The duplicate is only visible by looking at rows an earlier statement
    // wrote, so this is the case a naive "only check the new row" narrowing
    // would miss.
    let database = graph_with_unrelated_types(6);
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Type1 {name: 'taken'})", &Parameters::new())
        .expect("first create");
    let error = session
        .execute("CREATE (:Type1 {name: 'taken'})", &Parameters::new())
        .expect_err("a duplicate unique property must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("duplicate"),
        "expected a duplicate-value violation, got: {message}"
    );
}

/// One node type with a required name and a range on `score`.
fn graph_with_range_on_score() -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:validation-range",
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
fn a_value_range_still_fails_after_many_valid_rows_exist() {
    // Identity-scoped required/value checks must still catch a bad write after
    // bulk load. Unique is deliberately not installed here so the path under
    // test is the value predicate (SQL LIMIT 1 + written-id filter).
    let database = graph_with_range_on_score();
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    for index in 0..20 {
        session
            .execute(
                &format!("CREATE (:Person {{name: 'p{index}', score: {index}}})"),
                &Parameters::new(),
            )
            .expect("valid create");
    }
    let error = session
        .execute(
            "CREATE (:Person {name: 'bad', score: 101})",
            &Parameters::new(),
        )
        .expect_err("score above the configured range must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("score") || message.contains("range") || message.contains("above"),
        "expected a range violation naming the property, got: {message}"
    );
}

#[test]
fn a_value_range_rejects_a_set_on_an_existing_row() {
    let database = graph_with_range_on_score();
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute(
            "CREATE (:Person {name: 'ok', score: 10})",
            &Parameters::new(),
        )
        .expect("seed");
    let error = session
        .execute(
            "MATCH (p:Person {name: 'ok'}) SET p.score = -1",
            &Parameters::new(),
        )
        .expect_err("SET below the configured range must be rejected");
    let message = error.to_string();
    assert!(
        message.contains("score") || message.contains("range") || message.contains("below"),
        "expected a range violation on SET, got: {message}"
    );
}

/// Untyped `score` column (Cypher `Any`) with an integer range: storing TEXT
/// must fail under the SQL LIMIT-1 path the same way the old Rust path did.
fn graph_with_integer_range_on_any_column() -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:validation-any-range",
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    // No declared type → Any; registration accepts an integer range, and
    // SQLite keeps inserted TEXT as text so typeof() can see the mismatch.
    connection
        .execute("CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, score)")
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
fn a_text_value_still_fails_an_integer_range_predicate() {
    // Pins the typeof guard on the **validate_state SQL LIMIT-1** path.
    // CREATE of a TEXT score on an Any property goes through insert_entity's
    // Any branch (`check_runtime_value` → `validate_runtime` in Rust) and can
    // fail before validate_state runs, so that path does not prove the SQL
    // probe. Seed the bad TEXT with raw SQL into the semantic property column
    // + label membership (bypasses Cypher write-time checks), then SET another
    // property so validate_state re-checks the written identity via SQL.
    let database = graph_with_integer_range_on_any_column();
    let connection = database.connect().expect("connect");
    let session =
        GraphConnection::open(connection.clone(), "ontology").expect("open graph session");
    let labels = turso_graph_frontend::labels_table_name(session.graph_id());

    connection
        .execute("INSERT INTO people(id, name, score) VALUES (1, 't1', 'abc')")
        .expect("seed non-numeric TEXT score without Cypher validation");
    connection
        .prepare_internal(format!(
            "INSERT INTO \"{labels}\"(source_id, node_id, label) VALUES (1, 1, 'Person')"
        ))
        .expect("prepare seeded node label")
        .run_ignore_rows()
        .expect("record seeded node label");

    let error_abc = session
        .execute(
            "MATCH (p:Person {name: 't1'}) SET p.name = 't1-touched'",
            &Parameters::new(),
        )
        .expect_err(
            "validate_state must reject TEXT 'abc' on an integer range when the mutation \
             only SETs name (score was never re-checked in Rust on this statement)",
        );
    let message_abc = error_abc.to_string();
    assert!(
        message_abc.contains("score")
            || message_abc.contains("comparable")
            || message_abc.contains("range")
            || message_abc.contains("incompatible"),
        "expected a type/range violation for seeded TEXT 'abc', got: {message_abc}"
    );

    connection
        .execute("INSERT INTO people(id, name, score) VALUES (2, 't2', '50')")
        .expect("seed numeric-looking TEXT score");
    connection
        .prepare_internal(format!(
            "INSERT INTO \"{labels}\"(source_id, node_id, label) VALUES (1, 2, 'Person')"
        ))
        .expect("prepare second label")
        .run_ignore_rows()
        .expect("record second label");

    let error_numeric_text = session
        .execute(
            "MATCH (p:Person {name: 't2'}) SET p.name = 't2-touched'",
            &Parameters::new(),
        )
        .expect_err("validate_state must reject TEXT '50' against an integer range");
    let message_numeric_text = error_numeric_text.to_string();
    assert!(
        message_numeric_text.contains("score")
            || message_numeric_text.contains("comparable")
            || message_numeric_text.contains("range")
            || message_numeric_text.contains("incompatible"),
        "expected a type/range violation for seeded TEXT '50', got: {message_numeric_text}"
    );

    // Same-type integers still pass through CREATE (happy path).
    session
        .execute(
            "CREATE (:Person {name: 'ok', score: 50})",
            &Parameters::new(),
        )
        .expect("integer inside the range must still be accepted");
}
