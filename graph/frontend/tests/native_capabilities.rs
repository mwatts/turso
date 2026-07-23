use std::sync::Arc;

use turso_core::{Database, MemoryIO, SqliteDialect, Value};
use turso_graph_frontend::{
    register_graph, GraphConnection, GraphRegistration, NodeSourceRegistration, Parameters,
    RelationshipSourceRegistration, SnapshotPersistenceMode, SnapshotStatus,
};
#[cfg(feature = "fts")]
use turso_graph_frontend::{
    GraphFtsEntityKind, GraphFtsError, GraphFtsIndexSpec, GraphFtsPropertyWeight,
    GraphFtsTokenizer, ParameterTypes, MAX_GRAPH_FTS_INDEX_NAME_BYTES, MAX_GRAPH_FTS_PROPERTIES,
};
#[cfg(feature = "fts")]
use turso_graph_ir::{Nullability, ValueType};

mod fixture;

#[cfg(feature = "fts")]
#[test]
fn graph_fts_scalars_use_a_core_index() {
    let (_database, session) = fixture::social_graph_connection_with_fts();
    session
        .create_fts_index(&GraphFtsIndexSpec {
            name: "people_search".to_owned(),
            entity: GraphFtsEntityKind::Node,
            source: "Person".to_owned(),
            properties: vec!["name".to_owned()],
            tokenizer: GraphFtsTokenizer::Default,
            weights: Vec::new(),
        })
        .expect("create graph FTS index");

    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, 'Ada') \
                 RETURN n.name, fts_score(n.name, 'Ada') AS score",
                &Parameters::new(),
            )
            .expect("query through core FTS"),
        vec![vec![
            Value::build_text("Ada"),
            Value::from_f64(0.6931471824645996),
        ]]
    );
    let plan = session
        .query(
            "EXPLAIN MATCH (n:Person) WHERE fts_match(n.name, 'Ada') RETURN n.name",
            &Parameters::new(),
        )
        .expect("explain graph FTS");
    assert!(
        plan.iter().flatten().any(|value| {
            matches!(value, Value::Text(detail) if detail.as_str().contains("INDEX METHOD") || detail.as_str().contains("__turso_graph_fts_"))
        }),
        "expected core FTS planner evidence, got {plan:?}"
    );
    session
        .execute(
            "MATCH (n:Person {id: 2}) SET n.name = 'Systems engineer'",
            &Parameters::new(),
        )
        .expect("update indexed property");
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, 'systems') RETURN n.name",
                &Parameters::new(),
            )
            .expect("query updated index"),
        vec![vec![Value::build_text("Systems engineer")]]
    );
}

#[cfg(feature = "fts")]
#[test]
fn graph_fts_administration_is_transactional_persistent_and_bounded() {
    let (database, _seed_session) = fixture::social_graph_connection_with_fts();
    let connection = fixture::second_connection(&database);
    let session = GraphConnection::open_with_parameters(
        connection.clone(),
        "social",
        ParameterTypes::from([("query".to_owned(), (ValueType::Text, Nullability::NonNull))]),
    )
    .expect("open FTS graph session");
    let spec = GraphFtsIndexSpec {
        name: "people_search".to_owned(),
        entity: GraphFtsEntityKind::Node,
        source: "Person".to_owned(),
        properties: vec!["name".to_owned()],
        tokenizer: GraphFtsTokenizer::Simple,
        weights: vec![GraphFtsPropertyWeight {
            property: "name".to_owned(),
            weight: 2.0,
        }],
    };

    assert!(session.list_fts_indexes().unwrap().is_empty());
    for invalid in [
        GraphFtsIndexSpec {
            properties: Vec::new(),
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            properties: vec!["name".to_owned(), "NAME".to_owned()],
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            properties: vec!["id".to_owned()],
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            properties: vec!["age".to_owned()],
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            source: "Missing".to_owned(),
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            name: "x".repeat(MAX_GRAPH_FTS_INDEX_NAME_BYTES + 1),
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            properties: (0..=MAX_GRAPH_FTS_PROPERTIES)
                .map(|index| format!("property_{index}"))
                .collect(),
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            weights: vec![GraphFtsPropertyWeight {
                property: "missing".to_owned(),
                weight: 1.0,
            }],
            ..spec.clone()
        },
        GraphFtsIndexSpec {
            weights: vec![GraphFtsPropertyWeight {
                property: "name".to_owned(),
                weight: f64::NAN,
            }],
            ..spec.clone()
        },
    ] {
        assert!(
            session.create_fts_index(&invalid).is_err(),
            "invalid definition must fail: {invalid:?}"
        );
        assert!(session.list_fts_indexes().unwrap().is_empty());
    }
    assert!(connection
        .prepare(
            "SELECT name FROM sqlite_schema \
             WHERE name GLOB '__turso_graph_fts_*'",
        )
        .unwrap()
        .run_collect_rows()
        .unwrap()
        .is_empty());

    connection.execute("BEGIN").unwrap();
    connection
        .prepare("SELECT name FROM people LIMIT 1")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert!(matches!(
        session.create_fts_index(&spec),
        Err(turso_graph_frontend::Error::Fts(
            GraphFtsError::RequiresWriteTransaction
        ))
    ));
    connection.execute("ROLLBACK").unwrap();

    connection.execute("BEGIN IMMEDIATE").unwrap();
    let rolled_back_spec = GraphFtsIndexSpec {
        name: "rolled_back".to_owned(),
        ..spec.clone()
    };
    let rolled_back = session
        .create_fts_index(&rolled_back_spec)
        .expect("create in transaction");
    assert_eq!(session.list_fts_indexes().unwrap(), vec![rolled_back]);
    connection.execute("ROLLBACK").unwrap();
    assert!(session.list_fts_indexes().unwrap().is_empty());
    assert!(connection
        .prepare(
            "SELECT name FROM sqlite_schema \
             WHERE name GLOB '__turso_graph_fts_*'",
        )
        .unwrap()
        .run_collect_rows()
        .unwrap()
        .is_empty());

    let created = session.create_fts_index(&spec).expect("create index");
    assert!(created.physical_name.starts_with("__turso_graph_fts_"));
    assert_eq!(
        session.create_fts_index(&spec).expect("idempotent create"),
        created
    );
    let conflict = GraphFtsIndexSpec {
        tokenizer: GraphFtsTokenizer::Raw,
        ..spec
    };
    assert!(matches!(
        session.create_fts_index(&conflict),
        Err(turso_graph_frontend::Error::Fts(
            GraphFtsError::ConflictingDefinition(name)
        )) if name == "people_search"
    ));

    let reopened =
        GraphConnection::open(fixture::second_connection(&database), "social").expect("reopen");
    assert_eq!(reopened.list_fts_indexes().unwrap(), vec![created.clone()]);
    assert_eq!(reopened.list_fts_indexes().unwrap(), vec![created.clone()]);

    let query = Parameters::from([("query".to_owned(), Value::build_text("Ada"))]);
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, $query) RETURN n.name",
                &query,
            )
            .expect("parameterized FTS query"),
        vec![vec![Value::build_text("Ada")]]
    );
    let update_program = connection
        .prepare("EXPLAIN UPDATE people SET name = 'Systems engineer' WHERE id = 2")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert!(
        format!("{update_program:?}").contains(&created.physical_name),
        "indexed-column update must maintain the custom index: {update_program:?}"
    );
    connection
        .execute("UPDATE people SET name = 'Systems engineer' WHERE id = 2")
        .expect("raw SQL update indexed value");
    let old_matches = connection
        .prepare("SELECT name FROM people WHERE fts_match(name, 'grace')")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(
        connection
            .prepare("SELECT name FROM people WHERE fts_match(name, 'Systems')")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::build_text("Systems engineer")]],
        "core must maintain FTS for ordinary updates; old matches: {old_matches:?}"
    );
    session
        .execute(
            "MATCH (n:Person {id: 1}) SET n.name = 'Database pioneer'",
            &Parameters::new(),
        )
        .expect("update indexed value");
    assert_eq!(
        connection
            .prepare("SELECT name FROM people WHERE id = 1")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::build_text("Database pioneer")]],
        "the graph mutation must update the canonical row"
    );
    assert_eq!(
        connection
            .prepare("SELECT name FROM people WHERE fts_match(name, 'Database')")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::build_text("Database pioneer")]],
        "core must maintain the FTS index for the graph mutation"
    );
    let query = Parameters::from([("query".to_owned(), Value::build_text("Database"))]);
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, $query) RETURN n.name",
                &query,
            )
            .expect("updated FTS query"),
        vec![vec![Value::build_text("Database pioneer")]]
    );
    session
        .execute("MATCH (n:Person {id: 2}) DELETE n", &Parameters::new())
        .expect("delete indexed row");
    let query = Parameters::from([("query".to_owned(), Value::build_text("Grace"))]);
    assert!(session
        .query(
            "MATCH (n:Person) WHERE fts_match(n.name, $query) RETURN n.name",
            &query,
        )
        .expect("deleted FTS query")
        .is_empty());

    assert!(session.drop_fts_index("people_search").unwrap());
    assert!(!session.drop_fts_index("people_search").unwrap());
    assert!(session.list_fts_indexes().unwrap().is_empty());
    assert!(session
        .query(
            "MATCH (n:Person) WHERE fts_match(n.name, $query) RETURN n.name",
            &query,
        )
        .expect("a missing FTS index has no matches")
        .is_empty());
}

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
