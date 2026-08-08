//! A session must not recompile the same SQL once per mutation.
//!
//! The SQL a mutation runs around its writes does not depend on the row being
//! written: the catalog freshness probe is a primary-key lookup on a fixed
//! table, and each constraint check is built from the resolved constraint. So
//! the text repeats exactly, mutation after mutation, and every repeat used to
//! be a fresh parse, plan and codegen. In a bootstrap loop that is most of the
//! cost — a 51-entity ontology install that had not finished after 16 minutes
//! against a 60 s timeout is what prompted this.
//!
//! One test in this binary, by design: see `prepared_sql`.

mod prepared_sql;

use std::sync::Arc;

use prepared_sql::PreparedSql;
use turso_graph_frontend::{
    core::{Database, MemoryIO, SqliteDialect},
    register_graph, register_semantic_constraints, register_semantic_schema, GraphConnection,
    GraphRegistration, NodeSourceRegistration, Parameters, SemanticConstraintRegistration,
    SemanticNodeType, SemanticProperty, SemanticRequiredProperty, SemanticSchemaRegistration,
    SemanticUniqueProperty,
};

/// One node type on one table, with the two constraint kinds a bootstrap loop
/// actually installs: a required property and a unique property.
fn constrained_graph() -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:statement-cache-cost",
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
fn a_steady_mutation_recompiles_nothing_but_its_own_write() {
    let recorded = PreparedSql::install();
    let database = constrained_graph();
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    // Two mutations warm the session and two more are measured, so a cache that
    // only ever held the most recent statement would still be caught.
    for index in 0..2 {
        session
            .execute(
                &format!("CREATE (:Person {{name: 'warm{index}'}})"),
                &Parameters::new(),
            )
            .expect("warm create");
    }
    recorded.take();
    for index in 0..2 {
        session
            .execute(
                &format!("CREATE (:Person {{name: 'steady{index}'}})"),
                &Parameters::new(),
            )
            .expect("steady create");
    }
    let statements = recorded.take();

    // Guards the assertion below against passing because nothing was recorded.
    assert!(
        statements
            .iter()
            .any(|sql| sql.contains("INSERT INTO \"people\"")),
        "expected each CREATE to still compile its own INSERT, whose inlined value changes every \
         time, but recorded {statements:#?}"
    );

    // The freshness probe and both constraint checks are byte-identical to the
    // previous mutation's, so the session is meant to still hold them prepared.
    let repeated: Vec<&String> = statements
        .iter()
        .filter(|sql| {
            sql.contains("schema_generation")
                || (sql.contains("AS entity") && sql.contains("EXISTS"))
        })
        .collect();
    assert!(
        repeated.is_empty(),
        "two steady-state CREATEs recompiled {} statements whose text does not change between \
         mutations: {repeated:#?}",
        repeated.len()
    );
}
