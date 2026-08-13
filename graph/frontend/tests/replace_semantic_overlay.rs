//! Host API: replace the semantic overlay without dropping graph sources.

use std::sync::Arc;

use turso_graph_frontend::{
    CatalogError, GraphConnection, GraphRegistration, NodeSourceRegistration, Parameters,
    SemanticCatalogError, SemanticConstraintRegistration, SemanticNodeType, SemanticProperty,
    SemanticReplaceOutcome, SemanticRequiredProperty, SemanticSchemaRegistration,
    core::{Connection, Database, MemoryIO, SqliteDialect, Value},
    load_registered_graph, load_semantic_snapshot, register_graph, register_semantic_schema,
    register_semantic_schema_with_fragments, replace_semantic_overlay,
};

fn database() -> Arc<Database> {
    Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:replace-overlay",
        Arc::new(SqliteDialect),
    )
    .expect("open database")
}

fn person_schema() -> SemanticSchemaRegistration {
    SemanticSchemaRegistration {
        node_types: vec![SemanticNodeType {
            name: "Person".to_owned(),
            source: "Person".to_owned(),
            properties: vec![SemanticProperty {
                name: "name".to_owned(),
                column: "name".to_owned(),
            }],
        }],
        relationship_types: Vec::new(),
    }
}

fn person_and_team_schema() -> SemanticSchemaRegistration {
    SemanticSchemaRegistration {
        node_types: vec![
            SemanticNodeType {
                name: "Person".to_owned(),
                source: "Person".to_owned(),
                properties: vec![SemanticProperty {
                    name: "name".to_owned(),
                    column: "name".to_owned(),
                }],
            },
            SemanticNodeType {
                name: "Team".to_owned(),
                source: "Team".to_owned(),
                properties: vec![SemanticProperty {
                    name: "name".to_owned(),
                    column: "name".to_owned(),
                }],
            },
        ],
        relationship_types: Vec::new(),
    }
}

fn setup() -> (Arc<Database>, Arc<Connection>) {
    let database = database();
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
             CREATE TABLE teams(id INTEGER PRIMARY KEY, name TEXT);",
        )
        .expect("create tables");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "Team".to_owned(),
                    table: "teams".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: Vec::new(),
        },
    )
    .expect("register graph");
    (database, connection)
}

#[test]
fn replace_on_a_missing_graph_is_not_found() {
    let database = database();
    let connection = database.connect().expect("connect");
    let error = replace_semantic_overlay(
        &connection,
        "missing",
        &person_schema(),
        &Default::default(),
        &SemanticConstraintRegistration::default(),
    )
    .expect_err("missing graph");
    assert!(matches!(error, SemanticCatalogError::GraphNotFound(name) if name == "missing"));
}

#[test]
fn replace_adds_a_type_and_keeps_existing_rows() {
    let (_database, connection) = setup();
    register_semantic_schema(&connection, "social", &person_schema()).expect("first schema");
    connection
        .execute("INSERT INTO people(id, name) VALUES (1, 'Ada')")
        .expect("insert person");
    let before = load_registered_graph(&connection, "social").expect("load");
    let person_source = before
        .node_sources
        .iter()
        .find(|source| source.name == "Person")
        .expect("Person source")
        .id;

    let outcome = replace_semantic_overlay(
        &connection,
        "social",
        &person_and_team_schema(),
        &Default::default(),
        &SemanticConstraintRegistration::default(),
    )
    .expect("replace");
    assert!(matches!(outcome, SemanticReplaceOutcome::Replaced { .. }));

    let after = load_registered_graph(&connection, "social").expect("reload");
    let person_after = after
        .node_sources
        .iter()
        .find(|source| source.name == "Person")
        .expect("Person source");
    assert_eq!(person_after.id, person_source);
    let snapshot = load_semantic_snapshot(&connection, &after)
        .expect("load overlay")
        .expect("overlay present");
    assert!(snapshot.node_type("Person").is_some());
    assert!(snapshot.node_type("Team").is_some());
    let names = connection
        .prepare("SELECT name FROM people WHERE id = 1")
        .expect("prepare")
        .run_collect_rows()
        .expect("query");
    assert!(
        matches!(&names[0][0], Value::Text(text) if text.as_str() == "Ada"),
        "person row must survive replace: {:?}",
        names[0][0]
    );
}

#[test]
fn an_exact_replay_is_unchanged() {
    let (_database, connection) = setup();
    register_semantic_schema(&connection, "social", &person_schema()).expect("first schema");
    let before = load_registered_graph(&connection, "social").expect("load");
    let outcome = replace_semantic_overlay(
        &connection,
        "social",
        &person_schema(),
        &Default::default(),
        &SemanticConstraintRegistration::default(),
    )
    .expect("replace");
    assert_eq!(outcome, SemanticReplaceOutcome::Unchanged);
    let after = load_registered_graph(&connection, "social").expect("reload");
    assert_eq!(after.schema_generation, before.schema_generation);
}

#[test]
fn a_failing_required_constraint_rolls_back() {
    let (_database, connection) = setup();
    register_semantic_schema(&connection, "social", &person_schema()).expect("first schema");
    GraphConnection::open(connection.clone(), "social")
        .expect("open session")
        .execute("CREATE (:Person)", &Parameters::new())
        .expect("create person without name");
    let before = load_registered_graph(&connection, "social").expect("load");
    let error = replace_semantic_overlay(
        &connection,
        "social",
        &person_schema(),
        &Default::default(),
        &SemanticConstraintRegistration {
            required: vec![SemanticRequiredProperty {
                owner: "Person".to_owned(),
                property: "name".to_owned(),
            }],
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect_err("required name fails");
    assert!(matches!(
        error,
        SemanticCatalogError::ConstraintViolation { .. }
            | SemanticCatalogError::InvalidConstraint { .. }
    ));
    let after = load_registered_graph(&connection, "social").expect("reload");
    assert_eq!(after.schema_generation, before.schema_generation);
    let snapshot = load_semantic_snapshot(&connection, &after)
        .expect("load overlay")
        .expect("overlay present");
    assert!(snapshot.node_type("Person").is_some());
    assert!(snapshot.node_type("Team").is_none());
}

#[test]
fn exact_register_still_rejects_a_changed_overlay() {
    let (_database, connection) = setup();
    register_semantic_schema(&connection, "social", &person_schema()).expect("first schema");
    let error = register_semantic_schema_with_fragments(
        &connection,
        "social",
        &person_and_team_schema(),
        &Default::default(),
    )
    .expect_err("exact register stays fail-closed");
    assert!(matches!(error, SemanticCatalogError::ConflictingSchema(_)));
}

#[test]
fn register_graph_still_rejects_a_second_create() {
    let (_database, connection) = setup();
    let error = register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        },
    )
    .expect_err("second create");
    assert!(matches!(error, CatalogError::GraphAlreadyExists(_)));
}
