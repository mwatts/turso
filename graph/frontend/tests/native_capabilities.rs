use std::sync::Arc;

use turso_core::{Database, MemoryIO, SqliteDialect, Value};
#[cfg(feature = "fts")]
use turso_core::{DatabaseOpts, Numeric, OpenOptions};
use turso_graph_frontend::{
    graph_generation, register_graph, GraphConnection, GraphRegistration, NodeSourceRegistration,
    Parameters, RelationshipSourceRegistration, SnapshotPersistenceMode, SnapshotStatus,
    GRAPH_CATALOG_VERSION,
};
#[cfg(feature = "fts")]
use turso_graph_frontend::{
    register_semantic_schema, GraphFtsEntityKind, GraphFtsError, GraphFtsIndexSpec,
    GraphFtsPropertyWeight, GraphFtsTokenizer, ParameterTypes, SemanticNodeType, SemanticProperty,
    SemanticSchemaRegistration, MAX_GRAPH_FTS_INDEX_NAME_BYTES, MAX_GRAPH_FTS_PROPERTIES,
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
    assert!(
        plan.iter().flatten().any(|value| {
            matches!(
                value,
                Value::Text(detail)
                    if detail.as_str().contains("__turso_graph_node_labels_")
                        && detail.as_str().contains("_ix1")
            )
        }),
        "label-filtered graph plans must use the semantic-type-first junction index: {plan:?}"
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
fn graph_fts_administration_requires_index_method_capability() {
    let (_database, session) = fixture::social_graph_connection();
    assert!(matches!(
        session.create_fts_index(&GraphFtsIndexSpec {
            name: "people_search".to_owned(),
            entity: GraphFtsEntityKind::Node,
            source: "Person".to_owned(),
            properties: vec!["name".to_owned()],
            tokenizer: GraphFtsTokenizer::Default,
            weights: Vec::new(),
        }),
        Err(turso_graph_frontend::Error::Fts(
            GraphFtsError::IndexMethodsDisabled
        ))
    ));
    assert!(session.list_fts_indexes().unwrap().is_empty());
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
            name: "nul\0name".to_owned(),
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
            properties: vec!["name); DROP TABLE people; --".to_owned()],
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
    assert_eq!(
        reopened
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, 'Ada') RETURN n.name",
                &Parameters::new(),
            )
            .expect("reopened session must query the persisted FTS index"),
        vec![vec![Value::build_text("Ada")]]
    );

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
    let operator_query =
        Parameters::from([("query".to_owned(), Value::build_text("Ada OR Grace"))]);
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, $query) \
                 RETURN n.name ORDER BY n.name",
                &operator_query,
            )
            .expect("FTS operators remain bound query data"),
        vec![
            vec![Value::build_text("Ada")],
            vec![Value::build_text("Grace")],
        ]
    );
    session
        .execute(
            "CREATE (:Person {id: 3, name: 'Compiler researcher', age: 75})",
            &Parameters::new(),
        )
        .expect("insert indexed node");
    let inserted_query = Parameters::from([("query".to_owned(), Value::build_text("Compiler"))]);
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, $query) RETURN n.name",
                &inserted_query,
            )
            .expect("query inserted indexed node"),
        vec![vec![Value::build_text("Compiler researcher")]]
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

    connection.execute("BEGIN IMMEDIATE").unwrap();
    assert!(session.drop_fts_index("people_search").unwrap());
    assert!(session.list_fts_indexes().unwrap().is_empty());
    connection.execute("ROLLBACK").unwrap();
    assert_eq!(session.list_fts_indexes().unwrap(), vec![created]);
    let restored_query = Parameters::from([("query".to_owned(), Value::build_text("Database"))]);
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WHERE fts_match(n.name, $query) RETURN n.name",
                &restored_query,
            )
            .expect("rolled-back drop must restore the index"),
        vec![vec![Value::build_text("Database pioneer")]]
    );

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

    let unusual_name = "safe'; DROP TABLE people; --/../__turso_graph_fts_";
    let unusual = session
        .create_fts_index(&GraphFtsIndexSpec {
            name: unusual_name.to_owned(),
            entity: GraphFtsEntityKind::Node,
            source: "Person".to_owned(),
            properties: vec!["name".to_owned()],
            tokenizer: GraphFtsTokenizer::Default,
            weights: Vec::new(),
        })
        .expect("logical names are data and cannot alter DDL");
    assert_eq!(unusual.spec.name, unusual_name);
    assert_eq!(
        connection
            .prepare("SELECT count(*) FROM people")
            .unwrap()
            .run_collect_rows()
            .unwrap(),
        vec![vec![Value::from_i64(2)]],
        "the SQL-shaped logical name must not execute"
    );
    assert!(session.drop_fts_index(unusual_name).unwrap());
}

#[cfg(feature = "fts")]
#[test]
fn graph_fts_administration_resolves_semantic_properties_to_physical_columns() {
    let database = Database::open(
        Arc::new(MemoryIO::new()),
        ":memory:graph-fts-semantic-properties",
        OpenOptions::new(Arc::new(SqliteDialect))
            .db_opts(DatabaseOpts::default().with_index_method(true)),
    )
    .expect("open FTS database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE articles(\
                 id INTEGER PRIMARY KEY, \
                 title_text TEXT, \
                 body_text TEXT, \
                 views INTEGER\
             )",
        )
        .expect("create article source");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "knowledge".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "articles_src".to_owned(),
                table: "articles".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        },
    )
    .expect("register graph");
    register_semantic_schema(
        &connection,
        "knowledge",
        &SemanticSchemaRegistration {
            node_types: vec![SemanticNodeType {
                name: "Article".to_owned(),
                source: "articles_src".to_owned(),
                properties: vec![
                    SemanticProperty {
                        name: "title".to_owned(),
                        column: "title_text".to_owned(),
                    },
                    SemanticProperty {
                        name: "body".to_owned(),
                        column: "body_text".to_owned(),
                    },
                    SemanticProperty {
                        name: "views".to_owned(),
                        column: "views".to_owned(),
                    },
                ],
            }],
            relationship_types: Vec::new(),
        },
    )
    .expect("register semantic schema");
    let session = GraphConnection::open(connection, "knowledge").expect("open semantic graph");
    session
        .execute(
            "CREATE (:Article {\
                 title: 'Database internals', \
                 body: 'Database database engine', \
                 views: 10\
             }), (:Article {\
                 title: 'Storage notes', \
                 body: 'Database overview', \
                 views: 5\
             })",
            &Parameters::new(),
        )
        .expect("seed semantic article");

    session
        .create_fts_index(&GraphFtsIndexSpec {
            name: "article_search".to_owned(),
            entity: GraphFtsEntityKind::Node,
            source: "articles_src".to_owned(),
            properties: vec!["title".to_owned(), "body".to_owned()],
            tokenizer: GraphFtsTokenizer::Default,
            weights: vec![GraphFtsPropertyWeight {
                property: "title".to_owned(),
                weight: 3.0,
            }],
        })
        .expect("semantic property must resolve to title_text");
    let ranked = session
        .query(
            "MATCH (n:Article) \
             WHERE fts_match(n.title, n.body, 'database') \
             RETURN n.title, fts_score(n.title, n.body, 'database') AS score \
             ORDER BY score DESC",
            &Parameters::new(),
        )
        .expect("query and rank semantic FTS properties");
    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0][0], Value::build_text("Database internals"));
    assert_eq!(ranked[1][0], Value::build_text("Storage notes"));
    assert!(
        matches!(
            (&ranked[0][1], &ranked[1][1]),
            (
                Value::Numeric(Numeric::Float(first)),
                Value::Numeric(Numeric::Float(second))
            ) if first > second
        ),
        "weighted multi-property score must determine ranking: {ranked:?}"
    );
    assert!(matches!(
        session.create_fts_index(&GraphFtsIndexSpec {
            name: "views_search".to_owned(),
            entity: GraphFtsEntityKind::Node,
            source: "articles_src".to_owned(),
            properties: vec!["views".to_owned()],
            tokenizer: GraphFtsTokenizer::Default,
            weights: Vec::new(),
        }),
        Err(turso_graph_frontend::Error::Fts(
            GraphFtsError::NonTextProperty { property, .. }
        )) if property == "views"
    ));
}

#[cfg(feature = "fts")]
#[test]
fn graph_fts_query_dispatches_across_node_sources_with_colliding_identities() {
    let database = Database::open(
        Arc::new(MemoryIO::new()),
        ":memory:graph-fts-multi-source",
        OpenOptions::new(Arc::new(SqliteDialect))
            .db_opts(DatabaseOpts::default().with_index_method(true)),
    )
    .expect("open FTS database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE articles(id INTEGER PRIMARY KEY, content TEXT); \
             CREATE TABLE notes(id INTEGER PRIMARY KEY, content TEXT)",
        )
        .expect("create text sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "library".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "articles_src".to_owned(),
                    table: "articles".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "notes_src".to_owned(),
                    table: "notes".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: Vec::new(),
        },
    )
    .expect("register graph");
    register_semantic_schema(
        &connection,
        "library",
        &SemanticSchemaRegistration {
            node_types: vec![
                SemanticNodeType {
                    name: "Article".to_owned(),
                    source: "articles_src".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "content".to_owned(),
                        column: "content".to_owned(),
                    }],
                },
                SemanticNodeType {
                    name: "Note".to_owned(),
                    source: "notes_src".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "content".to_owned(),
                        column: "content".to_owned(),
                    }],
                },
            ],
            relationship_types: Vec::new(),
        },
    )
    .expect("register semantic schema");
    let session = GraphConnection::open(connection, "library").expect("open graph");
    session
        .execute(
            "CREATE (:Article {content: 'Database article'}), \
                    (:Note {content: 'Database note'})",
            &Parameters::new(),
        )
        .expect("seed colliding identities");
    for (name, source) in [
        ("article_search", "articles_src"),
        ("note_search", "notes_src"),
    ] {
        session
            .create_fts_index(&GraphFtsIndexSpec {
                name: name.to_owned(),
                entity: GraphFtsEntityKind::Node,
                source: source.to_owned(),
                properties: vec!["content".to_owned()],
                tokenizer: GraphFtsTokenizer::Default,
                weights: Vec::new(),
            })
            .expect("create source-specific FTS index");
    }

    assert_eq!(
        session
            .query(
                "MATCH (n) WHERE fts_match(n.content, 'database') \
                 RETURN n.content ORDER BY n.content",
                &Parameters::new(),
            )
            .expect("dispatch FTS by source coordinate"),
        vec![
            vec![Value::build_text("Database article")],
            vec![Value::build_text("Database note")],
        ]
    );
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
    assert_eq!(metadata.graph_id, session.graph_id());
    assert_eq!(metadata.catalog_version, GRAPH_CATALOG_VERSION);
    assert_eq!(
        metadata.source_generation,
        graph_generation(&fixture::second_connection(&database), "social").unwrap()
    );
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
        current_catalog_version,
        current_generation,
    } = stale.status
    else {
        panic!("diagnostics must observe stale state without refreshing")
    };
    assert_eq!(current_catalog_version, GRAPH_CATALOG_VERSION);
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
    assert_eq!(
        session
            .query(
                "MATCH (n:Person {id: 2}) \
                 OPTIONAL MATCH (n)-[r:KNOWS]->() \
                 RETURN startNode(r), endNode(r)",
                &Parameters::new(),
            )
            .expect("nullable relationship endpoints"),
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
    for query in [
        "MATCH (n:Person) RETURN startNode(n)",
        "MATCH ()-[r:KNOWS]->() RETURN endNode([r])",
        "MATCH p = ()-[:KNOWS]->() RETURN startNode(p)",
    ] {
        let error = session
            .query(query, &Parameters::new())
            .expect_err("nodes and relationship lists are not scalar relationships");
        assert!(
            error
                .to_string()
                .contains("require a relationship argument"),
            "unexpected error for {query}: {error}"
        );
    }
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
    assert_eq!(
        session
            .query("CALL DB.Labels()", &Parameters::new())
            .expect("bare call uses the descriptor's default yield"),
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
                 cyprop_id TEXT,\
                 \"owner's_note\" TEXT\
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
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
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
        [
            "empty_declared",
            "id",
            "name",
            "owner's_note",
            "score",
            "since",
            "src",
        ]
        .into_iter()
        .map(|name| vec![Value::Text(name.into())])
        .collect::<Vec<_>>()
    );
}

/// After main's covering-index / column-free scan work, pure Cypher
/// `count(*)` over labeled nodes must lower to a junction membership count
/// that uses the semantic-type-first complete index (not a full node-table
/// scan). This exercises the real EXPLAIN QUERY PLAN prepare path.
#[test]
fn pure_count_star_uses_junction_covering_index() {
    let (_database, session) = fixture::social_graph_connection();
    session
        .execute(
            "CREATE (:Person {id: 3, name: 'Alan', age: 41})",
            &Parameters::new(),
        )
        .expect("seed a third person");

    assert_eq!(
        session
            .query("MATCH (n:Person) RETURN count(*) AS c", &Parameters::new(),)
            .expect("count people"),
        vec![vec![Value::from_i64(3)]]
    );

    let plan = session
        .query(
            "EXPLAIN MATCH (n:Person) RETURN count(*) AS c",
            &Parameters::new(),
        )
        .expect("explain pure count");
    let plan_text = plan
        .iter()
        .flatten()
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.as_str().to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        plan_text.contains("__turso_graph_node_labels_")
            && (plan_text.contains("USING INDEX")
                || plan_text.contains("USING COVERING INDEX")
                || plan_text.contains("SEARCH")),
        "labeled count(*) must plan through the label junction index, got:\n{plan_text}"
    );
    assert!(
        !plan_text.contains("SCAN people")
            || plan_text.contains("USING COVERING INDEX")
            || plan_text.contains("USING INDEX"),
        "labeled count(*) must not fall back to an unindexed people table scan:\n{plan_text}"
    );
}

/// Relationship tables get complete endpoint indexes at registration; bare
/// `count(*)` over those tables must be able to use a covering index (core
/// column-free covering path). Drive via the same connection the graph
/// session owns so registration-created indexes are present.
#[test]
fn relationship_table_count_uses_registration_covering_index() {
    let (database, session) = fixture::social_graph_connection();
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) CREATE (a)-[:KNOWS]->(b)",
            &Parameters::new(),
        )
        .expect("seed relationship");
    let connection = fixture::second_connection(&database);
    let plan = connection
        .prepare("EXPLAIN QUERY PLAN SELECT count(*) FROM relationships")
        .expect("prepare count plan")
        .run_collect_rows()
        .expect("run plan");
    let plan_text = plan
        .iter()
        .flatten()
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.as_str().to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        plan_text.contains("USING COVERING INDEX") || plan_text.contains("USING INDEX"),
        "count(*) over graph-indexed relationship table must use a complete index:\n{plan_text}"
    );
}

/// Main's aggregate-collation + sorter/NULLS work must keep Cypher text
/// ORDER BY / min / max stable on the real prepare+execute path. Graph
/// already emits NULLS FIRST/LAST; no extra COLLATE is required for default
/// BINARY property columns, but regressions here would surface as reorderings.
#[test]
fn text_order_by_and_min_max_follow_sqlite_binary_collation() {
    let (_database, session) = fixture::social_graph_connection();
    session
        .execute(
            "CREATE (:Person {id: 3, name: 'alan', age: 20}), \
             (:Person {id: 4, name: 'Zed', age: 30}), \
             (:Person {id: 5, name: NULL, age: 10})",
            &Parameters::new(),
        )
        .expect("seed mixed-case and null names");

    // Cypher ORDER BY has no NULLS FIRST/LAST surface syntax; the binder maps
    // ASC → NULLS LAST and DESC → NULLS FIRST before lowering to SQL.
    let ordered_asc = session
        .query(
            "MATCH (n:Person) RETURN n.name AS name ORDER BY n.name ASC",
            &Parameters::new(),
        )
        .expect("order by name asc");
    assert_eq!(
        ordered_asc,
        vec![
            vec![Value::build_text("Ada")],
            vec![Value::build_text("Grace")],
            vec![Value::build_text("Zed")],
            vec![Value::build_text("alan")],
            vec![Value::Null],
        ],
        "BINARY ASC places uppercase before lowercase; binder ASC uses NULLS LAST"
    );

    let ordered_desc = session
        .query(
            "MATCH (n:Person) RETURN n.name AS name ORDER BY n.name DESC",
            &Parameters::new(),
        )
        .expect("order by name desc");
    assert_eq!(
        ordered_desc,
        vec![
            vec![Value::Null],
            vec![Value::build_text("alan")],
            vec![Value::build_text("Zed")],
            vec![Value::build_text("Grace")],
            vec![Value::build_text("Ada")],
        ],
        "binder DESC uses NULLS FIRST with BINARY reverse order"
    );

    let mins = session
        .query(
            "MATCH (n:Person) RETURN min(n.name) AS lo, max(n.name) AS hi",
            &Parameters::new(),
        )
        .expect("min/max name");
    assert_eq!(
        mins,
        vec![vec![Value::build_text("Ada"), Value::build_text("alan"),]],
        "min/max on text properties must use SQLite BINARY aggregate collation"
    );

    let plan = session
        .query(
            "EXPLAIN MATCH (n:Person) RETURN n.name ORDER BY n.name DESC",
            &Parameters::new(),
        )
        .expect("explain ordered projection");
    let plan_text = plan
        .iter()
        .flatten()
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.as_str().to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !plan_text.is_empty(),
        "ordered text projection must produce an EXPLAIN QUERY PLAN"
    );
}

/// Vector properties are ordinary BLOB columns, so their bytes are whatever a
/// writer put there: an unrelated blob, a truncated write, or a hand-crafted
/// value. `vector_extract` and friends decode those bytes into a dense
/// `dims`-element buffer, and before main's parse-boundary validation a
/// malformed sparse index or a trailing-marker byte that underflowed the size
/// arithmetic aborted the process instead of failing the statement. A Cypher
/// query must never take down the database on stored data it did not write.
#[test]
fn malformed_vector_property_fails_the_query_instead_of_aborting() {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open_file(
        io,
        ":memory:native-capabilities-vector-guard",
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, embedding BLOB);\
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "vectors".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");
    let session = GraphConnection::open(connection, "vectors").expect("open graph session");

    // (id, blob bytes, why the blob is malformed)
    let malformed: [(i64, &[u8], &str); 3] = [
        // f32 sparse (type 0x09): one value, idx = 0xFFFFFFFF, dims = 1. The
        // index is used to address a dense 1-element buffer.
        (
            1,
            &[
                0x00, 0x00, 0x80, 0x3F, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x09,
            ],
            "sparse index past the dimension count",
        ),
        // float1bit (type 0x03): trailing_bits = 0xFF names more padding than
        // the two data bytes hold, so dims underflows.
        (
            2,
            &[0x00, 0xFF, 0x03],
            "float1bit trailing bits exceed the blob",
        ),
        // float8 (type 0x04): shorter than the mandatory alpha/shift header.
        (
            3,
            &[0x00, 0x00, 0x04],
            "float8 blob shorter than its header",
        ),
    ];
    for (id, blob, _) in malformed {
        session
            .execute(
                "CREATE (:Person {id: $id, embedding: $embedding})",
                &Parameters::from([
                    ("id".to_owned(), Value::from_i64(id)),
                    ("embedding".to_owned(), Value::from_blob(blob.to_vec())),
                ]),
            )
            .expect("store raw blob property");
    }

    for (id, _, why) in malformed {
        let result = session.query(
            &format!("MATCH (n:Person) WHERE n.id = {id} RETURN vector_extract(n.embedding)"),
            &Parameters::new(),
        );
        assert!(
            result.is_err(),
            "{why}: vector_extract must report an error, got {result:?}"
        );
    }

    // The session survives the failures and still serves the graph.
    assert_eq!(
        session
            .query("MATCH (n:Person) RETURN count(*)", &Parameters::new())
            .expect("count after malformed vector errors"),
        vec![vec![Value::from_i64(3)]]
    );
}

/// A literal sort key orders nothing.
///
/// Cypher `ORDER BY` takes expressions, so a bare literal is a constant every
/// row shares and the result keeps the order it already had. SQL reads a small
/// integer there as a column position instead, which made `ORDER BY 1 DESC`
/// reverse the projection and `ORDER BY 2` over a one-column projection fail
/// with "1st ORDER BY term out of range". Core only ever sees literal sort keys
/// that the graph lowering chose to emit, so the fix belongs here.
#[test]
fn literal_sort_keys_neither_reorder_nor_fail() {
    let (_database, session) = fixture::social_graph_connection();

    let unordered = session
        .query("MATCH (n:Person) RETURN n.name AS a", &Parameters::new())
        .expect("unordered projection");
    assert_eq!(
        unordered,
        vec![
            vec![Value::build_text("Ada")],
            vec![Value::build_text("Grace")],
        ],
        "fixture order the literal sort keys must preserve"
    );

    // Descending position 1 would put Grace first if the literal were read as
    // a column position.
    for query in [
        "MATCH (n:Person) RETURN n.name AS a ORDER BY 1 DESC",
        "MATCH (n:Person) RETURN n.name AS a ORDER BY -1",
        "MATCH (n:Person) RETURN n.name AS a ORDER BY 'x'",
        // Past the end of the projection: a column position would be an error.
        "MATCH (n:Person) RETURN n.name AS a ORDER BY 2",
    ] {
        assert_eq!(
            session
                .query(query, &Parameters::new())
                .unwrap_or_else(|error| panic!("{query} must not fail: {error:?}")),
            unordered,
            "{query} must leave the order alone"
        );
    }

    // A literal alongside a real key drops out without disturbing the key.
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) RETURN n.name AS a ORDER BY 1, n.name DESC",
                &Parameters::new(),
            )
            .expect("literal beside a real sort key"),
        vec![
            vec![Value::build_text("Grace")],
            vec![Value::build_text("Ada")],
        ]
    );
}

/// `reduce()` folds a list of any length.
///
/// Before core gained recursive CTEs the fold was an unrolled ladder of ten
/// sibling CTEs, so a valid openCypher `reduce()` over an eleventh element
/// raised "reduce() list exceeds 10 elements" instead of folding. The list
/// length is query data, never a compile-time constant, so the cap was a
/// semantic hole rather than a resource limit. The recursive lowering also has
/// to keep the two boundary cases the ladder encoded explicitly: a null list
/// folds to null, an empty list to the initializer.
#[test]
fn reduce_folds_lists_longer_than_the_former_unroll_cap() {
    let (_database, session) = fixture::social_graph_connection();

    // 25 elements: well past the ten the ladder could unroll.
    assert_eq!(
        session
            .query(
                "RETURN reduce(acc = 0, x IN range(1, 25) | acc + x) AS total",
                &Parameters::new(),
            )
            .expect("fold a 25-element list"),
        vec![vec![Value::from_i64(325)]]
    );

    // A fold whose body is not associative proves the elements are visited in
    // list order, not merely all visited.
    assert_eq!(
        session
            .query(
                "RETURN reduce(acc = '', x IN ['a', 'b', 'c', 'd', 'e', 'f', \
                 'g', 'h', 'i', 'j', 'k', 'l'] | acc + x) AS word",
                &Parameters::new(),
            )
            .expect("fold twelve strings in order"),
        vec![vec![Value::build_text("abcdefghijkl")]]
    );

    assert_eq!(
        session
            .query(
                "RETURN reduce(acc = 7, x IN [] | acc + x) AS empty, \
                 reduce(acc = 7, x IN null | acc + x) AS missing",
                &Parameters::new(),
            )
            .expect("fold the empty and null lists"),
        vec![vec![Value::from_i64(7), Value::Null]]
    );

    // A nested fold puts a whole recursive CTE inside the outer recursive arm.
    assert_eq!(
        session
            .query(
                "RETURN reduce(acc = 0, x IN range(1, 12) | \
                 acc + reduce(inner = 0, y IN range(1, x) | inner + y)) AS total",
                &Parameters::new(),
            )
            .expect("fold a nested reduce"),
        vec![vec![Value::from_i64(364)]]
    );
}

/// An aggregate inside a `reduce()` cannot mean what it reads as: the fold is a
/// recursive CTE, so the aggregate would range over the fold's own rows rather
/// than the outer rows it is written against. Each position failed differently
/// and none failed usefully — the body leaked core's "recursive aggregate
/// queries not supported", the list leaked "no such function: collect", and the
/// seed silently answered from a bogus grouping. One Cypher-level rejection
/// covers all three, and matches the message AGE and Neo4j give.
#[test]
fn aggregates_inside_reduce_are_rejected_in_cypher_terms() {
    let (_database, session) = fixture::social_graph_connection();

    for query in [
        "RETURN reduce(s = 0, x IN [1, 2] | s + count(x)) AS folded",
        "MATCH (n:Person) RETURN reduce(s = 0, x IN collect(n.name) | s + 1) AS folded",
        "MATCH (n:Person) RETURN reduce(s = count(n), x IN [1, 2] | s + x) AS folded",
    ] {
        let error = session
            .query(query, &Parameters::new())
            .expect_err("reject an aggregate inside reduce()")
            .to_string();
        assert!(
            error.contains("aggregate functions are not supported in a reduce() expression"),
            "{query} reported {error}"
        );
    }

    // The rejection is about the reduce() scope, not about aggregates: the same
    // aggregate outside the fold still works, and folding its result does too.
    assert_eq!(
        session
            .query(
                "MATCH (n:Person) WITH collect(n.name) AS names \
                 RETURN reduce(acc = '', x IN names | acc + x) AS joined",
                &Parameters::new(),
            )
            .expect("fold an aggregate computed in an earlier clause"),
        vec![vec![Value::build_text("AdaGrace")]]
    );
}
