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
    SemanticNodeType, SemanticProperty, SemanticRequiredProperty, SemanticSchemaRegistration,
    SemanticUniqueProperty,
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
