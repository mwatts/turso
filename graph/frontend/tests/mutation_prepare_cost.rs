//! Steady-state mutations must not re-prepare stable helper SQL (R16).
//!
//! Value-carrying property writes still recompile when literals differ (Cell
//! INSERT … VALUES (…, 'name') or legacy column INSERT). Label-junction
//! membership SQL is parameterized on the identity only, so after the first
//! CREATE the session must reuse the prepared program.
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
};

fn constrained_person_graph() -> Arc<Database> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:mutation-prepare-cost",
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
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register constraints");
    database
}

#[test]
fn steady_create_reuses_label_junction_and_constraint_helper_sql() {
    let recorded = PreparedSql::install();
    let database = constrained_person_graph();
    let session = GraphConnection::open(database.connect().expect("connect"), "ontology")
        .expect("open graph session");

    session
        .execute("CREATE (:Person {name: 'warm'})", &Parameters::new())
        .expect("warm create");
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

    assert!(
        statements
            .iter()
            .any(|sql| { sql.contains("INSERT INTO \"people\"") || sql.contains("node_props") }),
        "value-carrying CREATE write must still recompile (entity or Cell insert): {statements:#?}"
    );

    // Label junction and required-property probes are byte-stable across
    // labeled CREATEs of the same type.
    let repeated_helpers: Vec<&String> = statements
        .iter()
        .filter(|sql| {
            sql.contains("__turso_graph_node_labels_")
                || (sql.contains("AS entity") && sql.contains("EXISTS"))
                || sql.contains("schema_generation")
        })
        .collect();
    assert!(
        repeated_helpers.is_empty(),
        "steady CREATE recompiled {} stable helper statements that StatementCache must retain: \
         {repeated_helpers:#?}",
        repeated_helpers.len()
    );
}
