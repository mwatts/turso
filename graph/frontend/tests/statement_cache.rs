//! Reusing a prepared statement must not change what a mutation sees.
//!
//! The session keeps the SQL it runs around every write — the catalog freshness
//! probe and the constraint checks — compiled instead of recompiling it per
//! mutation. That the recompiles stop is measured in `statement_cache_cost`;
//! what is pinned here is that nothing about the answers changes: a reused
//! statement still sees rows written after it was prepared, a failed check does
//! not poison the next mutation, and DDL underneath a held statement is safe.

use std::sync::Arc;

use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    register_graph, register_semantic_constraints, register_semantic_schema, GraphConnection,
    GraphRegistration, NodeSourceRegistration, Parameters, SemanticConstraintRegistration,
    SemanticNodeType, SemanticProperty, SemanticRequiredProperty, SemanticSchemaRegistration,
    SemanticUniqueProperty,
};

/// One node type on one table, with the two constraint kinds a bootstrap loop
/// actually installs: a required property and a unique property.
fn constrained_graph(name: &str) -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        &format!(":memory:statement-cache-{name}"),
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute("CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT)")
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
                properties: vec![SemanticProperty {
                    name: "name".to_owned(),
                    column: "name".to_owned(),
                }],
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
            unique: vec![SemanticUniqueProperty {
                owner: "Person".to_owned(),
                property: "name".to_owned(),
            }],
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register semantic constraints");
    database
}

#[test]
fn a_reused_statement_still_sees_rows_written_after_it_was_prepared() {
    // A cached statement steps an already-compiled program. If reuse skipped
    // the reset, or served the rows the previous run collected, the duplicate
    // written by the second CREATE would be invisible to the uniqueness check
    // that the first CREATE prepared.
    let database = constrained_graph("reuse");
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Person {name: 'taken'})", &Parameters::new())
        .expect("first create");
    let error = session
        .execute("CREATE (:Person {name: 'taken'})", &Parameters::new())
        .expect_err("a duplicate unique property must be rejected");
    assert!(
        error.to_string().contains("duplicate"),
        "expected a duplicate-value violation, got: {error}"
    );
}

#[test]
fn a_failed_check_does_not_poison_the_next_mutation() {
    // A statement that errors is dropped rather than kept. If a half-run
    // statement went back into the cache, the mutation after a rejection would
    // step it from the wrong state.
    let database = constrained_graph("poison");
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Person {name: null})", &Parameters::new())
        .expect_err("a NULL required property must be rejected");
    session
        .execute("CREATE (:Person {name: 'valid'})", &Parameters::new())
        .expect("a valid create after a rejected one must still work");
    let error = session
        .execute("CREATE (:Person {name: 'valid'})", &Parameters::new())
        .expect_err("the constraint must still fire after an earlier failure");
    assert!(
        error.to_string().contains("duplicate"),
        "expected a duplicate-value violation, got: {error}"
    );
}

#[test]
fn a_held_statement_survives_a_schema_change_under_it() {
    // Holding a statement is only safe because core re-prepares one whose
    // schema moved: `step` re-prepares when the connection's prepare context
    // has changed, and a schema-cookie bump surfaces as `SchemaUpdated`, which
    // re-prepares and retries. Adding a column to the source table moves the
    // schema under statements the session already has prepared.
    let database = constrained_graph("ddl");
    let connection = database.connect().expect("connect");
    let session =
        GraphConnection::open(connection.clone(), "ontology").expect("open graph session");

    session
        .execute("CREATE (:Person {name: 'before'})", &Parameters::new())
        .expect("create before the schema change");
    connection
        .execute("ALTER TABLE people ADD COLUMN nickname TEXT")
        .expect("add a column under the prepared statements");

    session
        .execute("CREATE (:Person {name: 'after'})", &Parameters::new())
        .expect("create after the schema change");
    let error = session
        .execute("CREATE (:Person {name: 'after'})", &Parameters::new())
        .expect_err("the uniqueness check must still fire after the schema change");
    assert!(
        error.to_string().contains("duplicate"),
        "expected a duplicate-value violation, got: {error}"
    );
}
