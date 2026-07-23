use std::sync::Arc;

use turso_core::{Database, MemoryIO, SqliteDialect, Value};
use turso_graph_frontend::{
    register_graph, GraphConnection, GraphRegistration, NodeSourceRegistration, Parameters,
    RelationshipSourceRegistration, SnapshotPersistenceMode, SnapshotStatus,
};

mod fixture;

#[test]
fn diagnostics_report_missing_current_and_stale_without_refreshing() {
    let (database, session) = fixture::social_graph_connection();
    let reopened = GraphConnection::open(fixture::second_connection(&database), "social")
        .expect("open independent graph session");
    assert_eq!(
        reopened.diagnostics().unwrap().status,
        SnapshotStatus::Missing
    );

    session
        .query(
            "MATCH (:Person)-[:KNOWS*1..1]->(n) RETURN n.name",
            &Parameters::new(),
        )
        .expect("build the calling session snapshot");
    let current = session.diagnostics().expect("current diagnostics");
    assert_eq!(current.graph_id, session.graph_id());
    assert_eq!(current.graph_name, "social");
    assert_eq!(
        current.persistence_mode,
        SnapshotPersistenceMode::InMemoryRebuildOnDemand
    );
    let SnapshotStatus::Current(metadata) = current.status else {
        panic!("snapshot must be current")
    };
    assert_eq!(metadata.node_count, 2);
    assert_eq!(metadata.relationship_count, 0);
    assert!(metadata.estimated_heap_bytes > 0);
    assert!(metadata.estimated_peak_build_bytes >= metadata.estimated_heap_bytes);

    fixture::second_connection(&database)
        .execute("INSERT INTO people VALUES (3, 'Katherine', 101)")
        .expect("mutate a registered source");
    let stale = session.diagnostics().expect("stale diagnostics");
    let SnapshotStatus::Stale {
        snapshot,
        current_generation,
        ..
    } = stale.status
    else {
        panic!("diagnostics must observe stale state without refreshing")
    };
    assert_eq!(snapshot.node_count, 2);
    assert!(current_generation > snapshot.source_generation);
    assert_eq!(
        session.diagnostics().expect("repeat diagnostics"),
        stale,
        "diagnostics must not refresh or publish state"
    );
}

#[test]
fn endpoint_functions_resolve_relationship_layout_and_preserve_nulls() {
    let (_database, session) = fixture::social_graph_connection();
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("seed relationship");

    assert_eq!(
        session
            .query(
                "MATCH ()-[r:KNOWS]->() RETURN startNode(r), endNode(r)",
                &Parameters::new(),
            )
            .expect("direct endpoints"),
        vec![vec![Value::from_i64(1), Value::from_i64(2)]]
    );
    assert_eq!(
        session
            .query(
                "MATCH ()-[r:KNOWS]->() WITH r RETURN startNode(r), endNode(r)",
                &Parameters::new(),
            )
            .expect("endpoints carried through WITH"),
        vec![vec![Value::from_i64(1), Value::from_i64(2)]]
    );
    assert_eq!(
        session
            .query("RETURN startNode(null), endNode(null)", &Parameters::new(),)
            .expect("null endpoints"),
        vec![vec![Value::Null, Value::Null]]
    );
    let error = session
        .query("RETURN startNode(1)", &Parameters::new())
        .expect_err("non-relationship argument must be rejected");
    assert!(
        error
            .to_string()
            .contains("require a relationship argument"),
        "unexpected error: {error}"
    );
}

#[test]
fn existing_catalog_procedures_use_the_explicit_procedure_pipeline() {
    let (database, session) = fixture::social_graph_connection();
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("seed relationship type");

    assert_eq!(
        session
            .query(
                "CALL db.labels() YIELD label RETURN label ORDER BY label",
                &Parameters::new(),
            )
            .expect("labels procedure"),
        vec![vec![Value::Text("Person".into())]]
    );
    let reopened = GraphConnection::open(fixture::second_connection(&database), "social")
        .expect("reopen graph");
    assert_eq!(
        reopened
            .query(
                "CALL db.relationshipTypes() YIELD relationshipType \
                 RETURN relationshipType ORDER BY relationshipType",
                &Parameters::new(),
            )
            .expect("relationship types procedure"),
        vec![vec![Value::Text("KNOWS".into())]]
    );
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) CALL db.labels() YIELD label \
                 RETURN n.name, label ORDER BY n.name",
                &Parameters::new(),
            )
            .expect("procedure composed with graph input"),
        vec![
            vec![Value::Text("Ada".into()), Value::Text("Person".into())],
            vec![Value::Text("Grace".into()), Value::Text("Person".into())],
        ]
    );
}

#[test]
fn property_keys_enumerates_declared_logical_payloads_across_sources() {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open_file(
        io,
        ":memory:native-capabilities-property-keys",
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(\
                 id INTEGER PRIMARY KEY,\
                 name TEXT,\
                 empty_declared TEXT,\
                 cyprop_id TEXT\
             );\
             CREATE TABLE places(\
                 id INTEGER PRIMARY KEY,\
                 name TEXT,\
                 score REAL\
             );\
             CREATE TABLE relationships(\
                 id INTEGER PRIMARY KEY,\
                 src INTEGER,\
                 dst INTEGER,\
                 since INTEGER,\
                 cyprop_src TEXT\
             );",
        )
        .expect("create empty sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "catalog".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "Place".to_owned(),
                    table: "places".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "KNOWS".to_owned(),
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
                start_node_source: "Person".to_owned(),
                end_node_source: "Person".to_owned(),
            }],
        },
    )
    .expect("register graph");
    let session = GraphConnection::open(connection, "catalog").expect("open graph session");

    let rows = session
        .query(
            "CALL db.propertyKeys() YIELD propertyKey \
             RETURN propertyKey ORDER BY propertyKey",
            &Parameters::new(),
        )
        .expect("catalog procedure");

    assert_eq!(
        rows,
        ["empty_declared", "id", "name", "score", "since", "src"]
            .into_iter()
            .map(|name| vec![Value::Text(name.into())])
            .collect::<Vec<_>>()
    );
}
