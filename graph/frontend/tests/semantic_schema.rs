use std::sync::Arc;

use turso_graph_frontend::core::{Database, MemoryIO, SqliteDialect};
use turso_graph_frontend::{
    bind, graph_generation, labels_table_name, load_registered_graph, load_semantic_snapshot,
    register_graph, register_semantic_constraints, register_semantic_schema,
    register_semantic_schema_with_fragments, relationship_types_table_name, BindError,
    CatalogEntity, Error as FrontendError, GraphCatalogSnapshot, GraphRegistration, MutationError,
    NodeSourceRegistration, PropertyResolution, RelationalCatalogSnapshot,
    RelationshipSourceRegistration, SchemaCatalog, SemanticCatalogError,
    SemanticConstraintRegistration, SemanticEndpoint, SemanticFragment, SemanticFragmentMember,
    SemanticFragmentRegistration, SemanticKeyConstraint, SemanticNodeType, SemanticProperty,
    SemanticPropertyValueConstraint, SemanticRangeBound, SemanticRelationshipCardinality,
    SemanticRelationshipType, SemanticRequiredProperty, SemanticScalar, SemanticSchemaRegistration,
    SemanticUniqueProperty, SemanticValuePredicate, SnapshotStatus, SnapshotStore,
};

fn role_node_ids(type_info: &turso_graph_frontend::SemanticTypeInfo, role_name: &str) -> Vec<u32> {
    type_info
        .role(role_name)
        .expect("role")
        .targets
        .iter()
        .map(|target| match target {
            turso_graph_ir::RoleTarget::Node(label) => label.get(),
            turso_graph_ir::RoleTarget::Relation(_) => panic!("expected node target"),
        })
        .collect()
}

fn connection() -> Arc<turso_graph_frontend::core::Connection> {
    let io = Arc::new(MemoryIO::new());
    Database::open_file(io, ":memory:semantic-schema", Arc::new(SqliteDialect))
        .expect("open database")
        .connect()
        .expect("connect")
}

fn registered_graph(connection: &Arc<turso_graph_frontend::core::Connection>) {
    connection
        .execute(
            "CREATE TABLE tbl_people(\
                 pk INTEGER PRIMARY KEY, \
                 full_name TEXT, \
                 supplier_name TEXT, \
                 birth_year INTEGER, \
                 active INTEGER, \
                 score REAL\
             ); \
             CREATE TABLE tbl_edges(\
                 pk INTEGER PRIMARY KEY, \
                 a INTEGER, \
                 b INTEGER, \
                 since INTEGER\
             );",
        )
        .expect("create sources");
    register_graph(
        connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "people_src".to_owned(),
                table: "tbl_people".to_owned(),
                identity_column: "pk".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "edges_src",
                "tbl_edges",
                "pk",
                "a",
                "b",
                "people_src",
                "people_src",
            )],
        },
    )
    .expect("register graph");
}

fn semantic_registration() -> SemanticSchemaRegistration {
    SemanticSchemaRegistration {
        node_types: vec![
            SemanticNodeType {
                name: "Customer".to_owned(),
                source: "people_src".to_owned(),
                properties: vec![
                    SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    },
                    SemanticProperty {
                        name: "born".to_owned(),
                        column: "birth_year".to_owned(),
                    },
                    SemanticProperty {
                        name: "active".to_owned(),
                        column: "active".to_owned(),
                    },
                    SemanticProperty {
                        name: "score".to_owned(),
                        column: "score".to_owned(),
                    },
                    SemanticProperty {
                        name: "category".to_owned(),
                        column: "supplier_name".to_owned(),
                    },
                ],
            },
            SemanticNodeType {
                name: "Supplier".to_owned(),
                source: "people_src".to_owned(),
                properties: vec![SemanticProperty {
                    name: "displayName".to_owned(),
                    column: "full_name".to_owned(),
                }],
            },
        ],
        relationship_types: vec![SemanticRelationshipType::binary(
            "TRADES_WITH",
            "edges_src",
            vec!["Customer".to_owned()],
            vec!["Supplier".to_owned()],
            vec![SemanticProperty {
                name: "since".to_owned(),
                column: "since".to_owned(),
            }],
        )],
    }
}

#[test]
fn property_keys_enumerates_semantic_names_instead_of_physical_columns() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    let session =
        turso_graph_frontend::Connection::open(connection, "social").expect("open semantic graph");

    let rows = session
        .query(
            "CALL db.propertyKeys() YIELD propertyKey \
             RETURN propertyKey ORDER BY propertyKey",
            &Default::default(),
        )
        .expect("semantic property keys");
    assert_eq!(
        rows,
        [
            "active",
            "born",
            "category",
            "displayName",
            "score",
            "since"
        ]
        .into_iter()
        .map(|name| vec![turso_graph_frontend::Value::build_text(name)])
        .collect::<Vec<_>>()
    );
    assert_eq!(
        session
            .query(
                "CALL db.labels() YIELD label RETURN label ORDER BY label",
                &Default::default(),
            )
            .expect("semantic labels"),
        ["Customer", "Supplier"]
            .into_iter()
            .map(|name| vec![turso_graph_frontend::Value::build_text(name)])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        session
            .query(
                "CALL db.relationshipTypes() YIELD relationshipType \
                 RETURN relationshipType",
                &Default::default(),
            )
            .expect("semantic relationship types"),
        vec![vec![turso_graph_frontend::Value::build_text("TRADES_WITH")]]
    );
}

fn multi_source_session() -> turso_graph_frontend::Connection {
    let connection = connection();
    connection
        .execute(
            "CREATE TABLE people(\
                 id INTEGER PRIMARY KEY, \
                 display_name TEXT, \
                 age INTEGER\
             ); \
             CREATE TABLE companies(\
                 id INTEGER PRIMARY KEY, \
                 legal_name TEXT\
             ); \
             CREATE TABLE employment(\
                 id INTEGER PRIMARY KEY, \
                 person_id INTEGER, \
                 company_id INTEGER, \
                 since INTEGER\
             ); \
             CREATE TABLE ownership(\
                 id INTEGER PRIMARY KEY, \
                 company_id INTEGER, \
                 person_id INTEGER, \
                 share INTEGER\
             );",
        )
        .expect("create multi-source tables");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "multi".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "people_src".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "companies_src".to_owned(),
                    table: "companies".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: vec![
                RelationshipSourceRegistration::binary(
                    "employment_src",
                    "employment",
                    "id",
                    "person_id",
                    "company_id",
                    "people_src",
                    "companies_src",
                ),
                RelationshipSourceRegistration::binary(
                    "ownership_src",
                    "ownership",
                    "id",
                    "company_id",
                    "person_id",
                    "companies_src",
                    "people_src",
                ),
            ],
        },
    )
    .expect("register multi-source graph");
    register_semantic_schema(
        &connection,
        "multi",
        &SemanticSchemaRegistration {
            node_types: vec![
                SemanticNodeType {
                    name: "Person".to_owned(),
                    source: "people_src".to_owned(),
                    properties: vec![
                        SemanticProperty {
                            name: "displayName".to_owned(),
                            column: "display_name".to_owned(),
                        },
                        SemanticProperty {
                            name: "age".to_owned(),
                            column: "age".to_owned(),
                        },
                    ],
                },
                SemanticNodeType {
                    name: "Company".to_owned(),
                    source: "companies_src".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "legal_name".to_owned(),
                    }],
                },
            ],
            relationship_types: vec![
                SemanticRelationshipType::binary(
                    "WORKS_AT",
                    "employment_src",
                    vec!["Person".to_owned()],
                    vec!["Company".to_owned()],
                    vec![SemanticProperty {
                        name: "weight".to_owned(),
                        column: "since".to_owned(),
                    }],
                ),
                SemanticRelationshipType::binary(
                    "OWNS",
                    "ownership_src",
                    vec!["Company".to_owned()],
                    vec!["Person".to_owned()],
                    vec![SemanticProperty {
                        name: "weight".to_owned(),
                        column: "share".to_owned(),
                    }],
                ),
            ],
        },
    )
    .expect("register multi-source semantic schema");
    turso_graph_frontend::Connection::open(connection, "multi").expect("open multi-source graph")
}

#[test]
fn multi_source_semantic_types_route_reads_and_writes_to_their_sources() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (p:Person {displayName: 'Ada'})\
             -[:WORKS_AT {weight: 1843}]->\
             (c:Company {displayName: 'Analytical Engines'})",
            &Default::default(),
        )
        .expect("create routed path");
    session
        .execute(
            "MATCH (c:Company) SET c.displayName = 'Difference Engines'",
            &Default::default(),
        )
        .expect("update company source");

    let rows = session
        .query(
            "MATCH (p:Person)-[r:WORKS_AT]->(c:Company) \
             RETURN p.displayName, r.weight, c.displayName",
            &Default::default(),
        )
        .expect("read routed path");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0][2],
        turso_graph_frontend::Value::Text("Difference Engines".into())
    );
}

#[test]
fn unlabeled_node_scan_unions_sources_with_owner_specific_columns() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada'}) \
             CREATE (:Company {displayName: 'Analytical Engines'})",
            &Default::default(),
        )
        .expect("create colliding local identities");

    let rows = session
        .query(
            "MATCH (n) RETURN n.displayName ORDER BY n.displayName",
            &Default::default(),
        )
        .expect("union unlabeled sources");
    assert_eq!(
        rows,
        vec![
            vec![turso_graph_frontend::Value::Text("Ada".into())],
            vec![turso_graph_frontend::Value::Text(
                "Analytical Engines".into()
            )],
        ]
    );

    let rows = session
        .query(
            "MATCH (n) RETURN n.displayName, properties(n) ORDER BY n.displayName",
            &Default::default(),
        )
        .expect("dispatch whole properties by source");
    assert_eq!(
        rows,
        vec![
            vec![
                turso_graph_frontend::Value::Text("Ada".into()),
                turso_graph_frontend::Value::Text("{\"displayName\":\"Ada\"}".into()),
            ],
            vec![
                turso_graph_frontend::Value::Text("Analytical Engines".into()),
                turso_graph_frontend::Value::Text(
                    "{\"displayName\":\"Analytical Engines\"}".into()
                ),
            ],
        ]
    );
}

#[test]
fn unlabeled_properties_must_be_owned_by_every_possible_source_type() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada', age: 36}) \
             CREATE (:Company {displayName: 'Engines'})",
            &Default::default(),
        )
        .expect("create owners with different property sets");

    let read_error = session
        .query("MATCH (n) RETURN n.age", &Default::default())
        .expect_err("unlabeled read must reject a partially owned property");
    assert!(
        read_error
            .to_string()
            .contains("owned by [\"Person\"] but not by [\"Company\"]"),
        "{read_error}"
    );

    let write_error = session
        .execute("MATCH (n) SET n.age = 37", &Default::default())
        .expect_err("unlabeled write must reject a partially owned property");
    assert!(
        write_error
            .to_string()
            .contains("owned by [\"Person\"] but not by [\"Company\"]"),
        "{write_error}"
    );

    let rows = session
        .query("MATCH (p:Person) RETURN p.age", &Default::default())
        .expect("failed mutation rolled back atomically");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Numeric(
            turso_graph_frontend::Numeric::Integer(36)
        )]]
    );
}

#[test]
fn untyped_relationship_scan_unions_sources_and_honors_endpoint_layouts() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada'})\
             -[:WORKS_AT {weight: 1843}]->\
             (:Company {displayName: 'Engines'}) \
             CREATE (:Company {displayName: 'Foundry'})\
             -[:OWNS {weight: 75}]->\
             (:Person {displayName: 'Charles'})",
            &Default::default(),
        )
        .expect("create both relationship sources");

    let rows = session
        .query(
            "MATCH (a)-[r]->(b) \
             RETURN a.displayName, r.weight, b.displayName, type(r) ORDER BY r.weight",
            &Default::default(),
        )
        .expect("union relationship sources");
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0],
        vec![
            turso_graph_frontend::Value::Text("Foundry".into()),
            turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(75)),
            turso_graph_frontend::Value::Text("Charles".into()),
            turso_graph_frontend::Value::Text("OWNS".into()),
        ]
    );
    assert_eq!(
        rows[1],
        vec![
            turso_graph_frontend::Value::Text("Ada".into()),
            turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1843)),
            turso_graph_frontend::Value::Text("Engines".into()),
            turso_graph_frontend::Value::Text("WORKS_AT".into()),
        ]
    );

    let endpoints = session
        .query(
            "MATCH (a)-[r]->(b) \
             RETURN type(r), a, b, startNode(r), endNode(r) ORDER BY type(r)",
            &Default::default(),
        )
        .expect("resolve endpoints for every relationship source");
    assert_eq!(endpoints.len(), 2);
    for row in endpoints {
        assert_eq!(
            row[1], row[3],
            "startNode must use the matched relationship source layout"
        );
        assert_eq!(
            row[2], row[4],
            "endNode must use the matched relationship source layout"
        );
    }
}

#[test]
fn multi_source_traversal_honors_incoming_and_undirected_endpoint_orientation() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada'})\
             -[:WORKS_AT {weight: 1843}]->\
             (:Company {displayName: 'Engines'}) \
             CREATE (:Company {displayName: 'Foundry'})\
             -[:OWNS {weight: 75}]->\
             (:Person {displayName: 'Charles'})",
            &Default::default(),
        )
        .expect("seed both relationship orientations");

    let incoming = session
        .query(
            "MATCH (p:Person)<-[r:OWNS]-(c:Company) \
             RETURN p.displayName, type(r), c.displayName",
            &Default::default(),
        )
        .expect("traverse an incoming cross-source relationship");
    assert_eq!(
        incoming,
        vec![vec![
            turso_graph_frontend::Value::Text("Charles".into()),
            turso_graph_frontend::Value::Text("OWNS".into()),
            turso_graph_frontend::Value::Text("Foundry".into()),
        ]]
    );

    let undirected = session
        .query(
            "MATCH (p:Person)-[r]-(c:Company) \
             RETURN p.displayName, type(r), c.displayName ORDER BY type(r)",
            &Default::default(),
        )
        .expect("traverse both physical orientations without duplication");
    assert_eq!(
        undirected,
        vec![
            vec![
                turso_graph_frontend::Value::Text("Charles".into()),
                turso_graph_frontend::Value::Text("OWNS".into()),
                turso_graph_frontend::Value::Text("Foundry".into()),
            ],
            vec![
                turso_graph_frontend::Value::Text("Ada".into()),
                turso_graph_frontend::Value::Text("WORKS_AT".into()),
                turso_graph_frontend::Value::Text("Engines".into()),
            ],
        ]
    );

    let filtered = session
        .query(
            "MATCH (n) OPTIONAL MATCH (n)-[r]->(m) WHERE r.weight > 100 \
             RETURN n.displayName, type(r), m.displayName ORDER BY n.displayName",
            &Default::default(),
        )
        .expect("fold optional predicates into every source branch");
    assert_eq!(
        filtered,
        vec![
            vec![
                turso_graph_frontend::Value::Text("Ada".into()),
                turso_graph_frontend::Value::Text("WORKS_AT".into()),
                turso_graph_frontend::Value::Text("Engines".into()),
            ],
            vec![
                turso_graph_frontend::Value::Text("Charles".into()),
                turso_graph_frontend::Value::Null,
                turso_graph_frontend::Value::Null,
            ],
            vec![
                turso_graph_frontend::Value::Text("Engines".into()),
                turso_graph_frontend::Value::Null,
                turso_graph_frontend::Value::Null,
            ],
            vec![
                turso_graph_frontend::Value::Text("Foundry".into()),
                turso_graph_frontend::Value::Null,
                turso_graph_frontend::Value::Null,
            ],
        ]
    );
}

#[test]
fn colliding_local_node_ids_keep_source_qualified_labels() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada'}) \
             CREATE (:Company {displayName: 'Engines'})",
            &Default::default(),
        )
        .expect("create colliding identities");

    let rows = session
        .query(
            "MATCH (n) WHERE n:Person RETURN n.displayName",
            &Default::default(),
        )
        .expect("filter source-qualified labels");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Text("Ada".into())]]
    );
}

#[test]
fn semantic_endpoints_must_match_relationship_source_layouts() {
    let connection = connection();
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY); \
             CREATE TABLE companies(id INTEGER PRIMARY KEY); \
             CREATE TABLE employment(\
                 id INTEGER PRIMARY KEY, person_id INTEGER, company_id INTEGER\
             );",
        )
        .expect("create endpoint sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "endpoints".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "people".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "companies".to_owned(),
                    table: "companies".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "employment",
                "employment",
                "id",
                "person_id",
                "company_id",
                "people",
                "companies",
            )],
        },
    )
    .expect("register endpoint graph");

    let error = register_semantic_schema(
        &connection,
        "endpoints",
        &SemanticSchemaRegistration {
            node_types: vec![
                SemanticNodeType {
                    name: "Person".to_owned(),
                    source: "people".to_owned(),
                    properties: Vec::new(),
                },
                SemanticNodeType {
                    name: "Company".to_owned(),
                    source: "companies".to_owned(),
                    properties: Vec::new(),
                },
            ],
            relationship_types: vec![SemanticRelationshipType::binary(
                "WORKS_AT",
                "employment",
                vec!["Company".to_owned()],
                vec!["Person".to_owned()],
                Vec::new(),
            )],
        },
    )
    .expect_err("semantic endpoints conflict with physical source layout");
    assert!(matches!(
        error,
        SemanticCatalogError::RoleSourceMismatch { .. }
    ));
}

#[test]
fn detach_delete_uses_endpoint_sources_when_local_ids_collide() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (p:Person {displayName: 'Person One'})\
             -[:WORKS_AT]->\
             (c:Company {displayName: 'Company One'}) \
             CREATE (:Company {displayName: 'Company Two'})\
             -[:OWNS]->\
             (p)",
            &Default::default(),
        )
        .expect("seed colliding endpoint identities");
    session
        .execute(
            "MATCH (c:Company {displayName: 'Company One'}) DETACH DELETE c",
            &Default::default(),
        )
        .expect("detach exact company source");

    let rows = session
        .query(
            "MATCH (:Company)-[r:OWNS]->(:Person) RETURN count(r)",
            &Default::default(),
        )
        .expect("unrelated relationship survives");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Numeric(
            turso_graph_frontend::Numeric::Integer(1)
        )]]
    );
}

#[test]
fn unlabeled_mutations_dispatch_to_each_row_source() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada'}) \
             CREATE (:Company {displayName: 'Engines'})",
            &Default::default(),
        )
        .expect("seed colliding node identities");

    session
        .execute(
            "MATCH (n) SET n = {displayName: n.displayName + '!'}",
            &Default::default(),
        )
        .expect("replace properties through runtime source provenance");
    let rows = session
        .query(
            "MATCH (n) RETURN n.displayName ORDER BY n.displayName",
            &Default::default(),
        )
        .expect("read both updated sources");
    assert_eq!(
        rows,
        vec![
            vec![turso_graph_frontend::Value::Text("Ada!".into())],
            vec![turso_graph_frontend::Value::Text("Engines!".into())],
        ]
    );
    session
        .execute("MATCH (n) SET n = properties(n)", &Default::default())
        .expect("replace dynamic maps through runtime source provenance");

    session
        .execute(
            "MATCH (n) WHERE n.displayName = 'Ada!' DELETE n",
            &Default::default(),
        )
        .expect("delete only the runtime-selected source row");
    let rows = session
        .query("MATCH (n) RETURN n.displayName", &Default::default())
        .expect("colliding identity in the other source survives");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Text("Engines!".into())]]
    );
    session
        .execute("MATCH (n) REMOVE n.displayName", &Default::default())
        .expect("remove a property through runtime source provenance");
    let rows = session
        .query("MATCH (n) RETURN n.displayName", &Default::default())
        .expect("removed property reads as null");
    assert_eq!(rows, vec![vec![turso_graph_frontend::Value::Null]]);
}

#[test]
fn untyped_relationship_mutations_dispatch_to_each_row_source() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada'})\
             -[:WORKS_AT {weight: 1843}]->\
             (:Company {displayName: 'Engines'}) \
             CREATE (:Company {displayName: 'Foundry'})\
             -[:OWNS {weight: 75}]->\
             (:Person {displayName: 'Charles'})",
            &Default::default(),
        )
        .expect("seed colliding relationship identities");

    session
        .execute(
            "MATCH ()-[r]->() SET r.weight = r.weight + 1",
            &Default::default(),
        )
        .expect("update both relationship sources");
    let rows = session
        .query(
            "MATCH ()-[r]->() RETURN r.weight ORDER BY r.weight",
            &Default::default(),
        )
        .expect("read updated relationship sources");
    assert_eq!(
        rows,
        vec![
            vec![turso_graph_frontend::Value::Numeric(
                turso_graph_frontend::Numeric::Integer(76)
            )],
            vec![turso_graph_frontend::Value::Numeric(
                turso_graph_frontend::Numeric::Integer(1844)
            )],
        ]
    );

    session
        .execute(
            "MATCH ()-[r]->() WHERE r.weight = 76 DELETE r",
            &Default::default(),
        )
        .expect("delete only the selected relationship source");
    let rows = session
        .query(
            "MATCH ()-[r]->() RETURN type(r), r.weight",
            &Default::default(),
        )
        .expect("colliding relationship identity in other source survives");
    assert_eq!(
        rows,
        vec![vec![
            turso_graph_frontend::Value::Text("WORKS_AT".into()),
            turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1844)),
        ]]
    );
}

#[test]
fn optional_expand_partitions_multi_source_inputs_before_unioning_branches() {
    let session = multi_source_session();
    session
        .execute(
            "CREATE (:Person {displayName: 'Ada'})\
             -[:WORKS_AT]->\
             (:Company {displayName: 'Engines'}) \
             CREATE (:Company {displayName: 'Foundry'})\
             -[:OWNS]->\
             (:Person {displayName: 'Charles'})",
            &Default::default(),
        )
        .expect("seed both source orientations");

    let rows = session
        .query(
            "MATCH (n) OPTIONAL MATCH (n)-[r]->(m) \
             RETURN n.displayName, type(r), m.displayName ORDER BY n.displayName",
            &Default::default(),
        )
        .expect("optional expansion over source-partitioned branches");
    assert_eq!(
        rows,
        vec![
            vec![
                turso_graph_frontend::Value::Text("Ada".into()),
                turso_graph_frontend::Value::Text("WORKS_AT".into()),
                turso_graph_frontend::Value::Text("Engines".into()),
            ],
            vec![
                turso_graph_frontend::Value::Text("Charles".into()),
                turso_graph_frontend::Value::Null,
                turso_graph_frontend::Value::Null,
            ],
            vec![
                turso_graph_frontend::Value::Text("Engines".into()),
                turso_graph_frontend::Value::Null,
                turso_graph_frontend::Value::Null,
            ],
            vec![
                turso_graph_frontend::Value::Text("Foundry".into()),
                turso_graph_frontend::Value::Text("OWNS".into()),
                turso_graph_frontend::Value::Text("Charles".into()),
            ],
        ]
    );
}

#[test]
fn registration_is_idempotent_for_identical_input() {
    let connection = connection();
    registered_graph(&connection);

    register_semantic_schema(&connection, "social", &semantic_registration()).expect("first");
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("identical replay");
}

#[test]
fn conflicting_replay_is_rejected_and_leaves_rows_unchanged() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("first");
    let mut conflicting = semantic_registration();
    // Keep the alternative definition physically valid so this exercises
    // schema identity rather than the independent shared-type validator.
    conflicting.node_types[0].properties[1].column = "full_name".to_owned();

    assert!(matches!(
        register_semantic_schema(&connection, "social", &conflicting),
        Err(SemanticCatalogError::ConflictingSchema(name)) if name == "social"
    ));
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("catalog remained unchanged");
}

#[test]
fn registration_rejects_structural_missing_and_wrong_kind_mappings() {
    let connection = connection();
    registered_graph(&connection);

    let mut structural = semantic_registration();
    structural.node_types[0].properties[0].column = "pk".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &structural),
        Err(SemanticCatalogError::StructuralColumn { .. })
    ));

    let mut endpoint_column = semantic_registration();
    endpoint_column.relationship_types[0].properties[0].column = "a".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &endpoint_column),
        Err(SemanticCatalogError::StructuralColumn { .. })
    ));

    let mut missing = semantic_registration();
    missing.node_types[0].properties[0].column = "ghost".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &missing),
        Err(SemanticCatalogError::ColumnMissing { .. })
    ));

    let mut bad_source = semantic_registration();
    bad_source.node_types[0].source = "nope".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &bad_source),
        Err(SemanticCatalogError::UnknownSource { .. })
    ));

    let mut kind_mismatch = semantic_registration();
    kind_mismatch.node_types[0].source = "edges_src".to_owned();
    assert!(matches!(
        register_semantic_schema(&connection, "social", &kind_mismatch),
        Err(SemanticCatalogError::UnknownSource { kind: "node", .. })
    ));
}

#[test]
fn shared_property_with_incompatible_column_types_is_rejected() {
    let connection = connection();
    registered_graph(&connection);
    let mut registration = semantic_registration();
    registration.node_types[1].properties[0].column = "birth_year".to_owned();

    assert!(matches!(
        register_semantic_schema(&connection, "social", &registration),
        Err(SemanticCatalogError::IncompatiblePropertyType { .. })
    ));
}

#[test]
fn failed_registration_writes_no_catalog_rows() {
    let connection = connection();
    registered_graph(&connection);
    let mut invalid = semantic_registration();
    invalid.node_types[1].properties[0].column = "ghost".to_owned();

    assert!(register_semantic_schema(&connection, "social", &invalid).is_err());
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("valid registration starts from a clean catalog");
}

#[test]
fn snapshot_reloads_identical_identities() {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:semantic-schema-reopen",
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("first connection");
    registered_graph(&connection);
    let fragments = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Named".to_owned(),
            properties: vec!["displayName".to_owned()],
            members: vec![
                SemanticFragmentMember {
                    node_type: "Customer".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    }],
                },
                SemanticFragmentMember {
                    node_type: "Supplier".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    }],
                },
            ],
        }],
    };
    register_semantic_schema_with_fragments(
        &connection,
        "social",
        &semantic_registration(),
        &fragments,
    )
    .expect("register");
    let graph = load_registered_graph(&connection, "social").expect("load graph");

    let first = load_semantic_snapshot(&connection, &graph)
        .expect("load snapshot")
        .expect("semantic mode");
    let customer = first
        .node_type("customer")
        .expect("case-insensitive lookup");
    let supplier = first.node_type("Supplier").expect("supplier");
    assert_ne!(customer.type_id, supplier.type_id);
    assert_eq!(
        customer.property("displayname").expect("owned").id,
        supplier.property("displayName").expect("owned").id,
    );
    assert_eq!(customer.property("displayName").expect("owned").id.get(), 1);

    let customer_id = customer.type_id;
    let fragment_id = first.fragment("Named").expect("fragment").fragment_id;
    drop(connection);
    let reopened = database.connect().expect("reopen connection");
    let reopened_graph = load_registered_graph(&reopened, "social").expect("reload graph");
    let second = load_semantic_snapshot(&reopened, &reopened_graph)
        .expect("reload")
        .expect("semantic mode");
    assert_eq!(
        second.node_type("Customer").expect("customer").type_id,
        customer_id
    );
    let reopened_fragment = second.fragment("named").expect("reopened fragment");
    assert_eq!(reopened_fragment.fragment_id, fragment_id);
    assert_eq!(reopened_fragment.member_type_ids(), &[1, 2]);
}

#[test]
fn semantic_registration_invalidates_and_retypes_traversal_snapshots() {
    let connection = connection();
    registered_graph(&connection);
    let graph = load_registered_graph(&connection, "social").expect("load graph");
    connection
        .execute(format!(
            "INSERT INTO tbl_people(pk, full_name) VALUES (1, 'Ada'), (2, 'Iron Co'); \
             INSERT INTO tbl_edges(pk, a, b, since) VALUES (10, 1, 2, 1840); \
             INSERT INTO \"{}\"(source_id, relationship_id, type) \
             VALUES ({}, 10, 'TRADES_WITH')",
            relationship_types_table_name(graph.id),
            graph.relationship_sources[0].id.get(),
        ))
        .expect("seed graph");

    let store = SnapshotStore::default();
    store
        .refresh(
            &connection,
            "social",
            turso_graph_runtime::BuildLimits::default(),
            &turso_graph_runtime::NeverCancelled,
        )
        .expect("publish legacy snapshot");
    let SnapshotStatus::Current(before) = store.status(&connection, "social").expect("status")
    else {
        panic!("legacy snapshot must initially be current");
    };

    let mut registration = semantic_registration();
    let mut supplies = registration.relationship_types[0].clone();
    supplies.name = "SUPPLIES_TO".to_owned();
    registration.relationship_types.insert(0, supplies);
    register_semantic_schema(&connection, "social", &registration).expect("register semantic");

    assert!(matches!(
        store.status(&connection, "social").expect("stale status"),
        SnapshotStatus::Stale {
            snapshot,
            current_generation,
            ..
        } if snapshot.source_generation == before.source_generation
            && current_generation == before.source_generation + 1
    ));
    assert!(store
        .get_current(&connection, "social")
        .expect("current lookup")
        .is_none());

    store
        .refresh(
            &connection,
            "social",
            turso_graph_runtime::BuildLimits::default(),
            &turso_graph_runtime::NeverCancelled,
        )
        .expect("rebuild semantic snapshot");
    let rebuilt = store
        .get_current(&connection, "social")
        .expect("current lookup")
        .expect("rebuilt snapshot");
    assert_eq!(
        rebuilt
            .relationship(turso_graph_ir::RelationshipId::new(1).expect("relationship id"))
            .expect("relationship")
            .relationship_type,
        turso_graph_ir::RelationshipTypeId::new(2).expect("semantic type id")
    );
}

#[test]
fn owner_specific_columns_preserve_one_stable_property_identity() {
    let connection = connection();
    registered_graph(&connection);
    let mut registration = semantic_registration();
    registration.node_types[1].properties[0].column = "supplier_name".to_owned();
    register_semantic_schema(&connection, "social", &registration).expect("register");

    let graph = load_registered_graph(&connection, "social").expect("load graph");
    let snapshot = load_semantic_snapshot(&connection, &graph)
        .expect("load snapshot")
        .expect("semantic mode");
    let customer_property = snapshot
        .node_type("Customer")
        .and_then(|type_info| type_info.property("displayName"))
        .expect("customer property");
    let supplier_property = snapshot
        .node_type("Supplier")
        .and_then(|type_info| type_info.property("displayName"))
        .expect("supplier property");
    assert_eq!(customer_property.id, supplier_property.id);
    assert_eq!(customer_property.column, "full_name");
    assert_eq!(supplier_property.column, "supplier_name");

    let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
        .expect("open semantic graph");
    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada'}) \
             CREATE (:Supplier {displayName: 'Iron Co'})",
            &Default::default(),
        )
        .expect("create both owners");
    session
        .execute(
            "MATCH (s:Supplier) SET s.displayName = 'Steel Co'",
            &Default::default(),
        )
        .expect("update supplier mapping");

    let rows = connection
        .prepare(
            "SELECT full_name, supplier_name FROM tbl_people \
             ORDER BY pk",
        )
        .expect("prepare physical verification")
        .run_collect_rows()
        .expect("read physical columns");
    assert_eq!(
        rows,
        vec![
            vec![
                turso_graph_frontend::Value::Text("Ada".into()),
                turso_graph_frontend::Value::Null,
            ],
            vec![
                turso_graph_frontend::Value::Null,
                turso_graph_frontend::Value::Text("Steel Co".into()),
            ],
        ]
    );
}

#[test]
fn legacy_graph_without_semantic_rows_loads_none() {
    let connection = connection();
    registered_graph(&connection);
    let graph = load_registered_graph(&connection, "social").expect("load graph");

    assert!(load_semantic_snapshot(&connection, &graph)
        .expect("load legacy mode")
        .is_none());
}

#[test]
fn semantic_catalog_resolves_conceptual_names_and_owned_properties() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("register");
    let graph = load_registered_graph(&connection, "social").expect("load graph");
    let semantic = load_semantic_snapshot(&connection, &graph)
        .expect("load snapshot")
        .map(Arc::new);
    let catalog = SchemaCatalog::with_semantic(connection, graph.clone(), semantic);

    let customer = catalog
        .label(graph.id, "Customer")
        .expect("semantic customer label");
    assert!(catalog.label(graph.id, "people_src").is_none());
    let relationship = catalog
        .relationship_type(graph.id, "TRADES_WITH")
        .expect("semantic relationship type");
    let display_name = match catalog
        .resolve_owned_property(
            graph.id,
            CatalogEntity::Node,
            &["Customer".to_owned()],
            "displayName",
        )
        .expect("property resolution")
    {
        PropertyResolution::Resolved(property) => property,
        other => panic!("expected resolved property, got {other:?}"),
    };
    assert!(matches!(
        catalog.resolve_owned_property(
            graph.id,
            CatalogEntity::Node,
            &["Supplier".to_owned()],
            "born",
        ),
        Some(PropertyResolution::NotOwned { .. })
    ));
    assert!(matches!(
        catalog.resolve_owned_property(graph.id, CatalogEntity::Node, &[], "born"),
        Some(PropertyResolution::Ambiguous { .. })
    ));
    assert_eq!(
        catalog.property_column(graph.node_sources[0].id, display_name.id),
        Some("full_name".to_owned())
    );
    assert_eq!(catalog.label_name(customer).as_deref(), Some("Customer"));
    assert_eq!(
        catalog.relationship_type_name(relationship).as_deref(),
        Some("TRADES_WITH")
    );
}

fn semantic_session() -> turso_graph_frontend::Connection {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("register");
    turso_graph_frontend::Connection::open(connection, "social").expect("open semantic graph")
}

#[test]
fn create_without_a_label_is_rejected_in_semantic_mode() {
    let session = semantic_session();
    let error = session
        .execute("CREATE (n {displayName: 'Ada'})", &Default::default())
        .expect_err("reject untyped create");

    assert!(
        error.to_string().contains("exactly one semantic type"),
        "{error}"
    );
}

#[test]
fn create_with_multiple_concrete_labels_is_rejected() {
    let session = semantic_session();
    let error = session
        .execute("CREATE (n:Customer:Supplier)", &Default::default())
        .expect_err("reject multiple semantic labels");

    assert!(
        error
            .to_string()
            .contains("multiple concrete semantic labels"),
        "{error}"
    );
}

#[test]
fn relationship_mutations_require_one_known_semantic_type() {
    let session = semantic_session();

    let missing = session
        .execute("CREATE (:Customer)-[]->(:Supplier)", &Default::default())
        .expect_err("reject missing relationship type");
    assert!(
        missing.to_string().contains("exactly one type"),
        "{missing}"
    );

    let unknown = session
        .execute(
            "CREATE (:Customer)-[:UNKNOWN]->(:Supplier)",
            &Default::default(),
        )
        .expect_err("reject unknown relationship type");
    assert!(
        unknown.to_string().contains("unknown relationship type"),
        "{unknown}"
    );

    let multiple = session
        .execute(
            "CREATE (:Customer)-[:TRADES_WITH|TRADES_WITH]->(:Supplier)",
            &Default::default(),
        )
        .expect_err("reject multiple relationship types");
    assert!(
        multiple.to_string().contains("exactly one type"),
        "{multiple}"
    );
}

#[test]
fn create_uses_one_semantic_type_and_rejects_a_source_name_as_a_label() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (n:Customer {displayName: 'Ada'})",
            &Default::default(),
        )
        .expect("typed create");
    let error = session
        .execute(
            "CREATE (n:people_src {displayName: 'Bob'})",
            &Default::default(),
        )
        .expect_err("source name is not a semantic label");

    assert!(error.to_string().contains("unknown label"), "{error}");
}

#[test]
fn legacy_graph_still_creates_untyped_nodes() {
    let connection = connection();
    registered_graph(&connection);
    let session =
        turso_graph_frontend::Connection::open(connection, "social").expect("open legacy graph");

    session
        .execute("CREATE (n {full_name: 'Ada'})", &Default::default())
        .expect("legacy untyped create");
}

#[test]
fn reads_reject_unowned_and_ambiguous_properties() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (n:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect("seed customer");

    let error = session
        .query("MATCH (n:Supplier) RETURN n.born", &Default::default())
        .expect_err("supplier does not own born");
    assert!(error.to_string().contains("not owned"), "{error}");

    let error = session
        .query("MATCH (n) RETURN n.born", &Default::default())
        .expect_err("unlabeled born access is ambiguous");
    assert!(error.to_string().contains("owned by"), "{error}");

    session
        .query("MATCH (n) RETURN n.displayName", &Default::default())
        .expect("displayName is owned by every node type");
}

#[test]
fn label_predicates_narrow_semantic_property_ownership() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815}), \
             (:Supplier {displayName: 'Iron Co'})",
            &Default::default(),
        )
        .expect("seed both possible semantic types");

    let rows = session
        .query(
            "MATCH (n) WHERE n:Customer RETURN n.born",
            &Default::default(),
        )
        .expect("the label predicate narrows n before binding n.born");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Numeric(
            turso_graph_frontend::Numeric::Integer(1815),
        )]]
    );

    session
        .execute(
            "MATCH (n) WHERE n:Customer SET n.born = 1816",
            &Default::default(),
        )
        .expect("the narrowed semantic type flows into mutation binding");
    let rows = session
        .query(
            "MATCH (n) WHERE n:Customer RETURN n.born",
            &Default::default(),
        )
        .expect("read the narrowed update");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Numeric(
            turso_graph_frontend::Numeric::Integer(1816),
        )]]
    );
}

#[test]
fn semantic_bind_errors_preserve_variants_payloads_and_source_spans() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("register");
    let graph = load_registered_graph(&connection, "social").expect("load graph");
    let semantic = load_semantic_snapshot(&connection, &graph)
        .expect("load snapshot")
        .map(Arc::new);
    let catalog = SchemaCatalog::with_semantic(connection.clone(), graph.clone(), semantic);

    let not_owned_query = "MATCH (n:Supplier) RETURN n.born";
    let property_start = not_owned_query.find("born").expect("property spelling");
    let syntax = turso_graph_cypher::parse(not_owned_query).expect("parse query");
    let error = bind(&syntax, graph.id, &catalog, &Default::default())
        .expect_err("Supplier does not own born");
    assert!(matches!(
        error,
        BindError::PropertyNotOwned {
            name,
            types,
            span_start,
            span_end,
        } if name == "born"
            && types == vec!["Supplier"]
            && span_start == property_start
            && span_end == property_start + "born".len()
    ));

    let ambiguous_query = "MATCH (n) RETURN n.born";
    let property_start = ambiguous_query.find("born").expect("property spelling");
    let syntax = turso_graph_cypher::parse(ambiguous_query).expect("parse query");
    let error = bind(&syntax, graph.id, &catalog, &Default::default())
        .expect_err("only Customer owns born");
    assert!(matches!(
        error,
        BindError::AmbiguousProperty {
            name,
            owners,
            non_owners,
            span_start,
            span_end,
        } if name == "born"
            && owners == vec!["Customer"]
            && non_owners == vec!["Supplier"]
            && span_start == property_start
            && span_end == property_start + "born".len()
    ));

    let session =
        turso_graph_frontend::Connection::open(connection, "social").expect("open semantic graph");
    let incompatible_query = "CREATE (:Customer {displayName: 'Ada', born: 'not an integer'})";
    let value_start = incompatible_query
        .find("'not an integer'")
        .expect("value spelling");
    let error = session
        .execute(incompatible_query, &Default::default())
        .expect_err("the literal has the wrong static type");
    assert!(matches!(
        error,
        FrontendError::Mutation(MutationError::Bind(
            BindError::IncompatiblePropertyValue {
                property,
                expected: turso_graph_ir::ValueType::Integer,
                actual: turso_graph_ir::ValueType::Text,
                span_start,
                span_end,
            }
        )) if property == "born"
            && span_start == value_start
            && span_end == value_start + "'not an integer'".len()
    ));
}

#[test]
fn writes_reject_unowned_properties_on_every_static_route() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (n:Customer {displayName: 'Ada'})",
            &Default::default(),
        )
        .expect("seed customer");

    for query in [
        "CREATE (n:Supplier {born: 1815})",
        "MATCH (n:Supplier) SET n.born = 1815",
        "MATCH (n:Supplier) REMOVE n.born",
        "MATCH (n:Supplier) SET n = {born: 1815}",
        "MERGE (n:Supplier {displayName: 'S'}) ON CREATE SET n.born = 1",
        "MERGE (n:Supplier {displayName: 'S'}) ON MATCH SET n.born = 1",
    ] {
        let error = session
            .execute(query, &Default::default())
            .expect_err(query);
        assert!(error.to_string().contains("not owned"), "{query}: {error}");
    }
}

#[test]
fn typed_reads_and_writes_work_in_semantic_mode() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (c:Customer {displayName: 'Ada', born: 1815})\
             -[:TRADES_WITH {since: 1840}]->\
             (s:Supplier {displayName: 'Iron Co'})",
            &Default::default(),
        )
        .expect("typed path");
    let rows = session
        .query(
            "MATCH (c:Customer)-[t:TRADES_WITH]->(s:Supplier) \
             RETURN c.displayName, t.since, s.displayName",
            &Default::default(),
        )
        .expect("typed read");

    assert_eq!(rows.len(), 1);
}

#[test]
fn every_typed_property_mutation_route_executes_in_semantic_mode() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect("create customer");
    session
        .execute(
            "MATCH (n:Customer {displayName: 'Ada'}) SET n.born = 1816",
            &Default::default(),
        )
        .expect("set one property");
    session
        .execute(
            "MATCH (n:Customer {displayName: 'Ada'}) \
             SET n += {displayName: 'Ada Lovelace'}",
            &Default::default(),
        )
        .expect("merge literal properties");
    session
        .execute(
            "MATCH (n:Customer {displayName: 'Ada Lovelace'}) REMOVE n.born",
            &Default::default(),
        )
        .expect("remove one property");
    session
        .execute(
            "MATCH (n:Customer {displayName: 'Ada Lovelace'}) \
             SET n = {displayName: 'Ada'}",
            &Default::default(),
        )
        .expect("replace literal properties");
    session
        .execute(
            "MERGE (n:Customer {displayName: 'Ada'}) \
             ON MATCH SET n.born = 1815",
            &Default::default(),
        )
        .expect("merge on match");
    session
        .execute(
            "MERGE (n:Customer {displayName: 'Grace'}) \
             ON CREATE SET n.born = 1906",
            &Default::default(),
        )
        .expect("merge on create");

    session
        .execute(
            "CREATE (:Customer {displayName: 'Buyer'})\
             -[:TRADES_WITH {since: 1840}]->\
             (:Supplier {displayName: 'Seller'})",
            &Default::default(),
        )
        .expect("create typed relationship");
    session
        .execute(
            "MATCH (:Customer)-[r:TRADES_WITH]->(:Supplier) SET r.since = 1841",
            &Default::default(),
        )
        .expect("set relationship property");
    session
        .execute(
            "MATCH (:Customer)-[r:TRADES_WITH]->(:Supplier) REMOVE r.since",
            &Default::default(),
        )
        .expect("remove relationship property");

    let customers = session
        .query(
            "MATCH (n:Customer) RETURN n.displayName, n.born",
            &Default::default(),
        )
        .expect("read customers");
    assert_eq!(
        customers,
        vec![
            vec![
                turso_graph_frontend::Value::Text("Ada".into()),
                turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1815),),
            ],
            vec![
                turso_graph_frontend::Value::Text("Grace".into()),
                turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1906),),
            ],
            vec![
                turso_graph_frontend::Value::Text("Buyer".into()),
                turso_graph_frontend::Value::Null,
            ],
        ]
    );
    let relationships = session
        .query(
            "MATCH (:Customer)-[r:TRADES_WITH]->(:Supplier) RETURN r.since",
            &Default::default(),
        )
        .expect("read relationship");
    assert_eq!(relationships, vec![vec![turso_graph_frontend::Value::Null]]);
}

#[test]
fn endpoint_validation_covers_both_directions() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (:Customer {displayName: 'A'})\
             -[:TRADES_WITH]->\
             (:Supplier {displayName: 'B'})",
            &Default::default(),
        )
        .expect("valid outgoing");
    session
        .execute(
            "CREATE (:Supplier {displayName: 'C'})\
             <-[:TRADES_WITH]-\
             (:Customer {displayName: 'D'})",
            &Default::default(),
        )
        .expect("valid incoming syntax");

    let error = session
        .execute(
            "CREATE (:Supplier {displayName: 'E'})\
             -[:TRADES_WITH]->\
             (:Customer {displayName: 'F'})",
            &Default::default(),
        )
        .expect_err("start endpoint must be Customer");
    assert!(
        error
            .to_string()
            .contains("role `start` of relationship type `TRADES_WITH` does not accept"),
        "{error}"
    );

    let error = session
        .execute(
            "CREATE (:Customer {displayName: 'G'})\
             <-[:TRADES_WITH]-\
             (:Supplier {displayName: 'H'})",
            &Default::default(),
        )
        .expect_err("incoming reversal must be checked");
    assert!(error.to_string().contains("does not accept"), "{error}");
}

#[test]
fn semantic_relationship_merge_enforces_types_endpoints_and_idempotency() {
    let session = semantic_session();
    let merge = "MERGE (:Customer {displayName: 'Buyer'})\
               -[:TRADES_WITH {since: 1840}]->\
               (:Supplier {displayName: 'Seller'})";
    session
        .execute(merge, &Default::default())
        .expect("first merge");
    session
        .execute(merge, &Default::default())
        .expect("matching merge");

    let rows = session
        .query(
            "MATCH (:Customer)-[r:TRADES_WITH]->(:Supplier) RETURN r.since",
            &Default::default(),
        )
        .expect("read merged relationship");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Numeric(
            turso_graph_frontend::Numeric::Integer(1840),
        )]]
    );

    let invalid = "MERGE (:Supplier {displayName: 'Wrong'})\
               -[:TRADES_WITH]->\
               (:Customer {displayName: 'Direction'})";
    let error = session
        .execute(invalid, &Default::default())
        .expect_err("MERGE must enforce the relationship endpoints");
    assert!(matches!(
        error,
        FrontendError::Mutation(MutationError::Bind(BindError::RoleTargetTypeViolation {
            relationship_type,
            role,
            found,
            ..
        })) if relationship_type == "TRADES_WITH" && role == "start" && found == "Supplier"
    ));
}

#[test]
fn dynamic_relationship_maps_validate_keys_and_values_atomically() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (:Customer {displayName: 'Buyer'})\
                    -[:TRADES_WITH {since: 1840}]->\
                    (:Supplier {displayName: 'Seller'})",
            &Default::default(),
        )
        .expect("seed typed relationship");

    let unknown_key = turso_graph_frontend::Parameters::from([(
        "properties".to_owned(),
        turso_graph_frontend::Value::Text(r#"{"ghost":1}"#.into()),
    )]);
    let error = session
        .execute(
            "MATCH ()-[r:TRADES_WITH]->() SET r = $properties",
            &unknown_key,
        )
        .expect_err("unknown relationship keys must fail");
    assert!(error.to_string().contains("ghost"), "{error}");

    let wrong_value = turso_graph_frontend::Parameters::from([(
        "properties".to_owned(),
        turso_graph_frontend::Value::Text(r#"{"since":"old"}"#.into()),
    )]);
    let error = session
        .execute(
            "MATCH ()-[r:TRADES_WITH]->() SET r += $properties",
            &wrong_value,
        )
        .expect_err("relationship values must retain semantic types");
    assert!(matches!(
        error,
        FrontendError::Mutation(MutationError::IncompatibleRuntimeValue {
            property,
            expected: turso_graph_ir::ValueType::Integer,
        }) if property == "since"
    ));

    let rows = session
        .query(
            "MATCH ()-[r:TRADES_WITH]->() RETURN r.since",
            &Default::default(),
        )
        .expect("the failed replacements leave the relationship unchanged");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Numeric(
            turso_graph_frontend::Numeric::Integer(1840),
        )]]
    );
}

#[test]
fn ambiguous_matched_bindings_cannot_create_semantic_relationship_endpoints() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (:Customer {displayName: 'Customer'}), \
             (:Supplier {displayName: 'Supplier'})",
            &Default::default(),
        )
        .expect("seed both endpoint types");

    let error = session
        .execute(
            "MATCH (start), (end) CREATE (start)-[:TRADES_WITH]->(end)",
            &Default::default(),
        )
        .expect_err("untyped bindings cannot prove either endpoint constraint");
    assert!(
        matches!(
            &error,
            FrontendError::Mutation(MutationError::Bind(BindError::RoleTargetTypeViolation {
                relationship_type,
                role,
                found,
                ..
            })) if relationship_type == "TRADES_WITH"
                && role == "start"
                && found == "Customer, Supplier"
        ),
        "{error:?}"
    );

    let rows = session
        .query("MATCH ()-[r:TRADES_WITH]->() RETURN r", &Default::default())
        .expect("read relationships after rejected mutation");
    assert!(rows.is_empty());
}

#[test]
fn statically_wrong_property_value_types_fail_during_binding() {
    let session = semantic_session();
    let error = session
        .execute(
            "CREATE (n:Customer {displayName: 'Ada', born: 'yesterday'})",
            &Default::default(),
        )
        .expect_err("Text cannot be assigned to Integer");
    assert!(error.to_string().contains("not assignable"), "{error}");

    session
        .execute(
            "CREATE (n:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect("seed typed customer");
    let error = session
        .execute("MATCH (n:Customer) SET n.born = 'old'", &Default::default())
        .expect_err("SET Text cannot be assigned to Integer");
    assert!(error.to_string().contains("not assignable"), "{error}");
}

#[test]
fn wrong_parameter_values_leave_zero_partial_writes() {
    let session = semantic_session();
    let parameters = turso_graph_frontend::Parameters::from([(
        "b".to_owned(),
        turso_graph_frontend::Value::Text("old".into()),
    )]);
    let error = session
        .execute(
            "CREATE (a:Customer {displayName: 'First'}) \
             CREATE (b:Customer {displayName: 'Second', born: $b})",
            &parameters,
        )
        .expect_err("Text parameter cannot be assigned to Integer");
    assert!(error.to_string().contains("not assignable"), "{error}");

    let rows = session
        .query(
            "MATCH (n:Customer) RETURN n.displayName",
            &Default::default(),
        )
        .expect("read after failed mutation");
    assert!(rows.is_empty(), "partial write leaked: {rows:?}");
}

#[test]
fn dynamic_map_replacement_rejects_unknown_keys_atomically() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (n:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect("seed customer");
    let parameters = turso_graph_frontend::Parameters::from([(
        "m".to_owned(),
        turso_graph_frontend::Value::Text(r#"{"displayName":"Eve","ghost":1}"#.into()),
    )]);

    let error = session
        .execute("MATCH (n:Customer) SET n = $m", &parameters)
        .expect_err("unknown dynamic property");
    assert!(error.to_string().contains("ghost"), "{error}");

    let bad_value = turso_graph_frontend::Parameters::from([(
        "m".to_owned(),
        turso_graph_frontend::Value::Text(r#"{"displayName":"Eve","born":"old"}"#.into()),
    )]);
    let error = session
        .execute("MATCH (n:Customer) SET n += $m", &bad_value)
        .expect_err("dynamic value must match the owned property type");
    assert!(
        error.to_string().contains("runtime value"),
        "expected runtime type validation: {error}"
    );

    let rows = session
        .query(
            "MATCH (n:Customer) RETURN n.displayName, n.born",
            &Default::default(),
        )
        .expect("read unchanged customer");
    assert_eq!(
        rows,
        vec![vec![
            turso_graph_frontend::Value::Text("Ada".into()),
            turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1815),),
        ]]
    );
}

#[test]
fn staged_runtime_failure_aborts_every_row() {
    let session = semantic_session();
    let parameters = turso_graph_frontend::Parameters::from([(
        "bad".to_owned(),
        turso_graph_frontend::Value::Text("x".into()),
    )]);

    let error = session
        .execute(
            "WITH [1, 2] AS rows \
             UNWIND rows AS i \
             CREATE (n:Customer {\
                 displayName: 'row', \
                 born: CASE i WHEN 1 THEN 1815 ELSE $bad END\
             })",
            &parameters,
        )
        .expect_err("second row must fail");
    assert!(
        error.to_string().contains("runtime value"),
        "expected deferred runtime validation: {error}"
    );
    let rows = session
        .query("MATCH (n:Customer) RETURN n", &Default::default())
        .expect("read after staged rollback");
    assert!(rows.is_empty(), "staged mutation leaked rows: {rows:?}");
}

#[test]
fn foreach_runtime_failure_aborts_every_iteration() {
    let session = semantic_session();
    let parameters = turso_graph_frontend::Parameters::from([(
        "bad".to_owned(),
        turso_graph_frontend::Value::Text("x".into()),
    )]);

    let error = session
        .execute(
            "FOREACH (i IN [1, 2] | \
                 CREATE (:Customer {\
                     displayName: 'row', \
                     born: CASE i WHEN 1 THEN 1815 ELSE $bad END\
                 })\
             )",
            &parameters,
        )
        .expect_err("second iteration must fail");
    assert!(
        error.to_string().contains("runtime value"),
        "expected deferred runtime validation: {error}"
    );
    let rows = session
        .query("MATCH (n:Customer) RETURN n", &Default::default())
        .expect("read after FOREACH rollback");
    assert!(rows.is_empty(), "FOREACH mutation leaked rows: {rows:?}");
}

#[test]
fn one_invalid_matched_row_rolls_back_prior_row_updates() {
    let session = semantic_session();
    session
        .execute(
            "CREATE (:Customer {displayName: 'first'}), \
             (:Customer {displayName: 'second'})",
            &Default::default(),
        )
        .expect("seed matched rows");
    let parameters = turso_graph_frontend::Parameters::from([(
        "bad".to_owned(),
        turso_graph_frontend::Value::Text("x".into()),
    )]);

    let error = session
        .execute(
            "MATCH (n:Customer) \
             SET n.born = CASE n.displayName \
                 WHEN 'first' THEN 1815 ELSE $bad END",
            &parameters,
        )
        .expect_err("one invalid matched row must fail the mutation");
    assert!(
        error.to_string().contains("runtime value"),
        "expected deferred runtime validation: {error}"
    );
    let rows = session
        .query(
            "MATCH (n:Customer) RETURN n.displayName, n.born",
            &Default::default(),
        )
        .expect("read after matched-row rollback");
    assert_eq!(
        rows,
        vec![
            vec![
                turso_graph_frontend::Value::Text("first".into()),
                turso_graph_frontend::Value::Null,
            ],
            vec![
                turso_graph_frontend::Value::Text("second".into()),
                turso_graph_frontend::Value::Null,
            ],
        ]
    );
}

fn fragment_registration() -> SemanticFragmentRegistration {
    SemanticFragmentRegistration {
        fragments: vec![
            SemanticFragment {
                name: "Nameable".to_owned(),
                properties: vec!["displayName".to_owned()],
                members: vec![
                    SemanticFragmentMember {
                        node_type: "Person".to_owned(),
                        properties: vec![SemanticProperty {
                            name: "displayName".to_owned(),
                            column: "person_name".to_owned(),
                        }],
                    },
                    SemanticFragmentMember {
                        node_type: "Company".to_owned(),
                        properties: vec![SemanticProperty {
                            name: "displayName".to_owned(),
                            column: "company_name".to_owned(),
                        }],
                    },
                    SemanticFragmentMember {
                        node_type: "Alias".to_owned(),
                        properties: vec![SemanticProperty {
                            name: "displayName".to_owned(),
                            column: "alias_name".to_owned(),
                        }],
                    },
                ],
            },
            SemanticFragment {
                name: "NaturalPerson".to_owned(),
                properties: Vec::new(),
                members: vec![SemanticFragmentMember {
                    node_type: "Person".to_owned(),
                    properties: Vec::new(),
                }],
            },
        ],
    }
}

fn fragment_schema() -> SemanticSchemaRegistration {
    SemanticSchemaRegistration {
        node_types: vec![
            SemanticNodeType {
                name: "Person".to_owned(),
                source: "people_src".to_owned(),
                properties: vec![SemanticProperty {
                    name: "age".to_owned(),
                    column: "age".to_owned(),
                }],
            },
            SemanticNodeType {
                name: "Company".to_owned(),
                source: "companies_src".to_owned(),
                properties: Vec::new(),
            },
            SemanticNodeType {
                name: "Alias".to_owned(),
                source: "people_src".to_owned(),
                properties: Vec::new(),
            },
        ],
        relationship_types: vec![SemanticRelationshipType::binary(
            "WORKS_AT",
            "employment_src",
            vec!["Person".to_owned()],
            vec!["Company".to_owned()],
            Vec::new(),
        )],
    }
}

fn register_fragment_graph(connection: &Arc<turso_graph_frontend::core::Connection>) {
    connection
        .execute(
            "CREATE TABLE people(\
                 id INTEGER PRIMARY KEY, \
                 person_name TEXT, \
                 alias_name TEXT, \
                 age INTEGER\
             ); \
             CREATE TABLE companies(id INTEGER PRIMARY KEY, company_name TEXT); \
             CREATE TABLE employment(\
                 id INTEGER PRIMARY KEY, \
                 person_id INTEGER, \
                 company_id INTEGER\
             );",
        )
        .expect("create fragment sources");
    register_graph(
        connection,
        &GraphRegistration {
            name: "fragments".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "people_src".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "companies_src".to_owned(),
                    table: "companies".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "employment_src",
                "employment",
                "id",
                "person_id",
                "company_id",
                "people_src",
                "companies_src",
            )],
        },
    )
    .expect("register fragment graph");
}

fn fragment_connection() -> Arc<turso_graph_frontend::core::Connection> {
    let connection = connection();
    register_fragment_graph(&connection);
    register_semantic_schema_with_fragments(
        &connection,
        "fragments",
        &fragment_schema(),
        &fragment_registration(),
    )
    .expect("register semantic fragments");
    connection
}

fn fragment_session() -> turso_graph_frontend::Connection {
    let connection = fragment_connection();
    turso_graph_frontend::Connection::open(connection, "fragments").expect("open fragment graph")
}

#[test]
fn catalog_procedures_include_fragment_labels_and_contributed_properties() {
    let session = fragment_session();
    assert_eq!(
        session
            .query(
                "CALL db.labels() YIELD label RETURN label ORDER BY label",
                &Default::default(),
            )
            .expect("fragment labels"),
        ["Alias", "Company", "Nameable", "NaturalPerson", "Person"]
            .into_iter()
            .map(|name| vec![turso_graph_frontend::Value::build_text(name)])
            .collect::<Vec<_>>()
    );
    assert_eq!(
        session
            .query(
                "CALL db.propertyKeys() YIELD propertyKey \
                 RETURN propertyKey ORDER BY propertyKey",
                &Default::default(),
            )
            .expect("fragment properties"),
        ["age", "displayName"]
            .into_iter()
            .map(|name| vec![turso_graph_frontend::Value::build_text(name)])
            .collect::<Vec<_>>()
    );
}

fn first_union_inputs(plan: &turso_graph_ir::Plan) -> Option<&[turso_graph_ir::Plan]> {
    use turso_graph_ir::PlanKind;

    match plan.kind() {
        PlanKind::Union(union) => Some(union.inputs()),
        PlanKind::RoleExpand(expand) => first_union_inputs(&expand.input),
        PlanKind::GraphExpand(expand) => first_union_inputs(&expand.input),
        PlanKind::Filter(filter) => first_union_inputs(&filter.input),
        PlanKind::Project(project) => first_union_inputs(&project.input),
        PlanKind::Aggregate(aggregate) => first_union_inputs(&aggregate.input),
        PlanKind::Distinct(distinct) => first_union_inputs(&distinct.input),
        PlanKind::Sort(sort) => first_union_inputs(&sort.input),
        PlanKind::Skip(skip) => first_union_inputs(&skip.input),
        PlanKind::Limit(limit) => first_union_inputs(&limit.input),
        PlanKind::LeftApply(apply) => {
            first_union_inputs(&apply.left).or_else(|| first_union_inputs(&apply.right))
        }
        PlanKind::Unwind(unwind) => first_union_inputs(&unwind.input),
        PlanKind::ProcedureCall(call) => first_union_inputs(&call.input),
        PlanKind::Join(join) => {
            first_union_inputs(&join.left).or_else(|| first_union_inputs(&join.right))
        }
        PlanKind::Unit(_) | PlanKind::NodeScan(_) | PlanKind::RelationScan(_) => None,
        PlanKind::RoleJoin(join) => first_union_inputs(&join.input),
    }
}

#[test]
fn fragment_scan_unions_concrete_members_across_sources() {
    let session = fragment_session();
    session
        .execute(
            "CREATE (:Person:Nameable:NaturalPerson {displayName: 'Ada'}), \
             (:Company:Nameable {displayName: 'Analytical Engines'}), \
             (:Alias:Nameable {displayName: 'Enchantress of Numbers'})",
            &Default::default(),
        )
        .expect("create fragment members");
    session
        .execute(
            "MATCH (p:Person {displayName: 'Ada'}), \
                   (c:Company {displayName: 'Analytical Engines'}) \
             CREATE (p)-[:WORKS_AT]->(c)",
            &Default::default(),
        )
        .expect("connect fragment members");

    let rows = session
        .query(
            "MATCH (n:Nameable) RETURN n.displayName ORDER BY n.displayName",
            &Default::default(),
        )
        .expect("scan fragment members");
    assert_eq!(
        rows,
        vec![
            vec![turso_graph_frontend::Value::Text("Ada".into())],
            vec![turso_graph_frontend::Value::Text(
                "Analytical Engines".into()
            )],
            vec![turso_graph_frontend::Value::Text(
                "Enchantress of Numbers".into()
            )],
        ]
    );

    let people = session
        .query(
            "MATCH (n:Person:Nameable) RETURN n.displayName",
            &Default::default(),
        )
        .expect("conjoin concrete and fragment labels");
    assert_eq!(
        people,
        vec![vec![turso_graph_frontend::Value::Text("Ada".into())]]
    );
    let natural_nameables = session
        .query(
            "MATCH (n:Nameable:NaturalPerson) RETURN n.displayName",
            &Default::default(),
        )
        .expect("intersect two fragment membership sets");
    assert_eq!(
        natural_nameables,
        vec![vec![turso_graph_frontend::Value::Text("Ada".into())]]
    );
    let expanded = session
        .query(
            "MATCH (:Person)-[:WORKS_AT]->(n:Nameable) RETURN n.displayName",
            &Default::default(),
        )
        .expect("enforce a fragment label on an expansion target");
    assert_eq!(
        expanded,
        vec![vec![turso_graph_frontend::Value::Text(
            "Analytical Engines".into()
        )]]
    );

    let partial_property =
        match session.prepare("MATCH (n:Nameable) RETURN n.age", &Default::default()) {
            Ok(_) => panic!("a property owned by only one fragment member must be ambiguous"),
            Err(error) => error,
        };
    assert!(
        partial_property
            .to_string()
            .contains("owned by [\"Person\"] but not by"),
        "{partial_property}"
    );
}

#[test]
fn fragment_scans_bind_as_union_of_concrete_node_scans() {
    let connection = fragment_connection();
    let graph = load_registered_graph(&connection, "fragments").expect("load graph");
    let semantic = Arc::new(
        load_semantic_snapshot(&connection, &graph)
            .expect("load snapshot")
            .expect("semantic schema"),
    );
    let fragment_id = semantic
        .fragment("Nameable")
        .expect("fragment identity")
        .fragment_id;
    let expected_type_ids = ["Person", "Company", "Alias"]
        .map(|name| semantic.node_type(name).expect("concrete type").type_id);
    let catalog = SchemaCatalog::with_semantic(connection, graph.clone(), Some(semantic));
    let syntax = turso_graph_cypher::parse("MATCH (n:Nameable) RETURN n").expect("parse query");
    let bound = bind(&syntax, graph.id, &catalog, &Default::default()).expect("bind query");

    let inputs = first_union_inputs(&bound.plan).expect("fragment scan must use Union");
    assert_eq!(inputs.len(), expected_type_ids.len());
    let mut actual_type_ids = inputs
        .iter()
        .map(|input| match input.kind() {
            turso_graph_ir::PlanKind::NodeScan(scan) => {
                assert_eq!(scan.labels.len(), 1);
                assert_ne!(scan.labels[0].get(), fragment_id);
                scan.labels[0].get()
            }
            other => panic!("Union branch must be a concrete NodeScan, got {other:?}"),
        })
        .collect::<Vec<_>>();
    actual_type_ids.sort_unstable();
    let mut expected_type_ids = expected_type_ids.to_vec();
    expected_type_ids.sort_unstable();
    assert_eq!(actual_type_ids, expected_type_ids);
}

#[test]
fn fragments_are_not_instantiable_and_membership_is_checked() {
    let session = fragment_session();
    let fragment_only = session
        .execute(
            "CREATE (:Nameable {displayName: 'invalid'})",
            &Default::default(),
        )
        .expect_err("fragment-only creation must fail");
    assert!(
        fragment_only
            .to_string()
            .contains("exactly one semantic type"),
        "{fragment_only}"
    );

    let unrelated = session
        .execute(
            "CREATE (:Company:NaturalPerson {displayName: 'invalid'})",
            &Default::default(),
        )
        .expect_err("concrete type must carry every written fragment");
    assert!(
        unrelated
            .to_string()
            .contains("have no common concrete node type"),
        "{unrelated}"
    );
}

#[test]
fn fragment_merge_requires_one_concrete_type_and_carried_fragments() {
    let session = fragment_session();
    let merge = "MERGE (:Person:Nameable:NaturalPerson {displayName: 'Ada', age: 1815})";
    session
        .execute(merge, &Default::default())
        .expect("first merge");
    session
        .execute(merge, &Default::default())
        .expect("matching merge");
    let rows = session
        .query(
            "MATCH (n:Nameable:NaturalPerson) RETURN n.displayName, n.age",
            &Default::default(),
        )
        .expect("read merged fragment member");
    assert_eq!(
        rows,
        vec![vec![
            turso_graph_frontend::Value::Text("Ada".into()),
            turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1815),),
        ]]
    );

    let fragment_only = session
        .execute(
            "MERGE (:Nameable {displayName: 'invalid'})",
            &Default::default(),
        )
        .expect_err("a fragment cannot become the MERGE instance type");
    assert!(matches!(
        fragment_only,
        FrontendError::Mutation(MutationError::Bind(BindError::MissingSemanticType {
            entity: "node",
            ..
        }))
    ));

    let unrelated = session
        .execute(
            "MERGE (:Company:NaturalPerson {displayName: 'invalid'})",
            &Default::default(),
        )
        .expect_err("the concrete type must carry every MERGE fragment");
    assert!(matches!(
        unrelated,
        FrontendError::Mutation(MutationError::Bind(
            BindError::IncompatibleSemanticLabels { names, .. }
        )) if names == vec!["Company", "NaturalPerson"]
    ));
}

#[test]
fn fragment_snapshot_precomputes_members_properties_and_endpoint_expansion() {
    let connection = connection();
    registered_graph(&connection);
    let schema = SemanticSchemaRegistration {
        node_types: semantic_registration().node_types,
        relationship_types: vec![SemanticRelationshipType::binary(
            "TRADES_WITH",
            "edges_src",
            vec!["Party".to_owned()],
            vec!["Party".to_owned()],
            Vec::new(),
        )],
    };
    let fragments = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Party".to_owned(),
            properties: vec!["displayName".to_owned()],
            members: vec![
                SemanticFragmentMember {
                    node_type: "Customer".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    }],
                },
                SemanticFragmentMember {
                    node_type: "Supplier".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    }],
                },
            ],
        }],
    };

    register_semantic_schema_with_fragments(&connection, "social", &schema, &fragments)
        .expect("register fragment endpoint schema");
    register_semantic_schema_with_fragments(&connection, "social", &schema, &fragments)
        .expect("identical replay is idempotent");
    let mut conflicting = fragments.clone();
    conflicting.fragments[0].members.pop();
    assert!(matches!(
        register_semantic_schema_with_fragments(&connection, "social", &schema, &conflicting),
        Err(SemanticCatalogError::ConflictingSchema(name)) if name == "social"
    ));
    register_semantic_schema_with_fragments(&connection, "social", &schema, &fragments)
        .expect("conflicting replay left the catalog unchanged");
    let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
        .expect("open fragment endpoint graph");
    session
        .execute(
            "CREATE (:Supplier:Party {displayName: 'Supplier'})\
                    -[:TRADES_WITH]->\
                    (:Customer:Party {displayName: 'Customer'})",
            &Default::default(),
        )
        .expect("expanded fragment endpoints permit every member type");
    drop(session);
    let graph = load_registered_graph(&connection, "social").expect("reload graph");
    let snapshot = load_semantic_snapshot(&connection, &graph)
        .expect("load semantic snapshot")
        .expect("semantic schema exists");
    let party = snapshot.fragment("party").expect("fragment resolves");
    assert_eq!(party.member_type_ids(), &[1, 2]);
    assert!(snapshot
        .node_type("Customer")
        .expect("customer type")
        .property("displayName")
        .is_some());
    let trades_with = snapshot
        .relationship_type_by_id(
            turso_graph_ir::RelationshipTypeId::new(1).expect("relationship type id"),
        )
        .expect("relationship type");
    assert_eq!(role_node_ids(trades_with, "start"), vec![1, 2]);
    assert_eq!(role_node_ids(trades_with, "end"), vec![1, 2]);
}

#[test]
fn fragment_reopen_preserves_noncolliding_ids_properties_memberships_and_endpoints() {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:fragment-schema-reopen",
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("first connection");
    register_fragment_graph(&connection);
    let mut schema = fragment_schema();
    schema.relationship_types[0]
        .roles
        .iter_mut()
        .find(|role| role.name == "start")
        .expect("start role")
        .targets = vec!["NaturalPerson".to_owned()];
    schema.relationship_types[0]
        .roles
        .iter_mut()
        .find(|role| role.name == "end")
        .expect("end role")
        .targets = vec!["Employer".to_owned()];
    let mut fragments = fragment_registration();
    fragments.fragments.push(SemanticFragment {
        name: "Employer".to_owned(),
        properties: Vec::new(),
        members: vec![SemanticFragmentMember {
            node_type: "Company".to_owned(),
            properties: Vec::new(),
        }],
    });
    register_semantic_schema_with_fragments(&connection, "fragments", &schema, &fragments)
        .expect("register fragment schema");

    let graph = load_registered_graph(&connection, "fragments").expect("load graph");
    let first = load_semantic_snapshot(&connection, &graph)
        .expect("load snapshot")
        .expect("semantic schema");
    let concrete_ids = ["Person", "Company", "Alias"]
        .map(|name| first.node_type(name).expect("concrete type").type_id);
    let nameable_id = first
        .fragment("Nameable")
        .expect("Nameable fragment")
        .fragment_id;
    let natural_person_id = first
        .fragment("NaturalPerson")
        .expect("NaturalPerson fragment")
        .fragment_id;
    let employer_id = first
        .fragment("Employer")
        .expect("Employer fragment")
        .fragment_id;
    for fragment_id in [nameable_id, natural_person_id, employer_id] {
        assert!(
            !concrete_ids.contains(&fragment_id),
            "fragment identity {fragment_id} collided with a concrete type"
        );
    }
    assert_ne!(nameable_id, natural_person_id);
    assert_ne!(nameable_id, employer_id);
    assert_ne!(natural_person_id, employer_id);

    let person_property = first
        .node_type("Person")
        .and_then(|type_info| type_info.property("displayName"))
        .expect("fragment-contributed Person property");
    let company_property = first
        .node_type("Company")
        .and_then(|type_info| type_info.property("displayName"))
        .expect("fragment-contributed Company property");
    let display_name_id = person_property.id;
    assert_eq!(company_property.id, display_name_id);
    assert_eq!(person_property.column, "person_name");
    assert_eq!(company_property.column, "company_name");
    let relationship_id = first
        .relationship_type("WORKS_AT")
        .expect("relationship type")
        .type_id;
    let works_at = first
        .relationship_type_by_id(
            turso_graph_ir::RelationshipTypeId::new(relationship_id).expect("relationship type id"),
        )
        .expect("relationship type");
    assert_eq!(role_node_ids(works_at, "start"), vec![concrete_ids[0]]);
    assert_eq!(role_node_ids(works_at, "end"), vec![concrete_ids[1]]);

    drop(connection);
    let reopened = database.connect().expect("reopen connection");
    let graph = load_registered_graph(&reopened, "fragments").expect("reload graph");
    let second = load_semantic_snapshot(&reopened, &graph)
        .expect("reload snapshot")
        .expect("semantic schema");
    assert_eq!(
        second
            .fragment("nameable")
            .expect("reopened Nameable")
            .fragment_id,
        nameable_id
    );
    assert_eq!(
        second
            .fragment("NaturalPerson")
            .expect("reopened NaturalPerson")
            .member_type_ids(),
        &[concrete_ids[0]]
    );
    assert_eq!(
        second
            .fragment("Employer")
            .expect("reopened Employer")
            .member_type_ids(),
        &[concrete_ids[1]]
    );
    let reopened_person_property = second
        .node_type("Person")
        .and_then(|type_info| type_info.property("displayName"))
        .expect("reopened Person property");
    let reopened_company_property = second
        .node_type("Company")
        .and_then(|type_info| type_info.property("displayName"))
        .expect("reopened Company property");
    assert_eq!(reopened_person_property.id, display_name_id);
    assert_eq!(reopened_company_property.id, display_name_id);
    assert_eq!(reopened_person_property.column, "person_name");
    assert_eq!(reopened_company_property.column, "company_name");
    let works_at = second
        .relationship_type_by_id(
            turso_graph_ir::RelationshipTypeId::new(relationship_id).expect("relationship type id"),
        )
        .expect("reopened relationship type");
    assert_eq!(role_node_ids(works_at, "start"), vec![concrete_ids[0]]);
    assert_eq!(role_node_ids(works_at, "end"), vec![concrete_ids[1]]);
}

#[test]
fn fragment_endpoint_expansion_rejects_physical_source_mismatches() {
    let connection = connection();
    register_fragment_graph(&connection);
    let mut schema = fragment_schema();
    schema.relationship_types[0]
        .roles
        .iter_mut()
        .find(|role| role.name == "start")
        .expect("start role")
        .targets = vec!["Nameable".to_owned()];

    let error = register_semantic_schema_with_fragments(
        &connection,
        "fragments",
        &schema,
        &fragment_registration(),
    )
    .expect_err("Nameable includes Company, which cannot occupy employment.person_id");
    assert!(
        matches!(
            &error,
            SemanticCatalogError::RoleSourceMismatch {
                relationship_type,
                role,
                node_type,
                actual_source,
                relationship_source,
                required_source,
            } if relationship_type.as_ref() == "WORKS_AT"
                && role.as_ref() == "start"
                && node_type.as_ref() == "Company"
                && actual_source.as_ref() == "companies_src"
                && relationship_source.as_ref() == "employment_src"
                && required_source.as_ref() == "people_src"
        ),
        "{error:?}"
    );
}

#[test]
fn fragment_registration_upgrades_an_identical_fragment_free_schema() {
    let connection = connection();
    registered_graph(&connection);
    let schema = semantic_registration();
    register_semantic_schema(&connection, "social", &schema).expect("register base schema");
    let fragments = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Party".to_owned(),
            properties: vec!["displayName".to_owned()],
            members: vec![
                SemanticFragmentMember {
                    node_type: "Customer".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    }],
                },
                SemanticFragmentMember {
                    node_type: "Supplier".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "full_name".to_owned(),
                    }],
                },
            ],
        }],
    };

    register_semantic_schema_with_fragments(&connection, "social", &schema, &fragments)
        .expect("add first fragment definition");
    register_semantic_schema_with_fragments(&connection, "social", &schema, &fragments)
        .expect("fragment upgrade replay is idempotent");
    register_semantic_schema(&connection, "social", &schema)
        .expect("legacy base-schema replay remains idempotent");
    let graph = load_registered_graph(&connection, "social").expect("reload graph");
    let snapshot = load_semantic_snapshot(&connection, &graph)
        .expect("load upgraded snapshot")
        .expect("semantic schema");
    assert_eq!(
        snapshot
            .fragment("Party")
            .expect("upgraded fragment")
            .member_type_ids(),
        &[1, 2]
    );
}

#[test]
fn fragment_registration_rejects_collisions_and_invalid_property_mappings() {
    let connection = connection();
    registered_graph(&connection);
    let schema = semantic_registration();

    let collision = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Customer".to_owned(),
            properties: Vec::new(),
            members: vec![SemanticFragmentMember {
                node_type: "Customer".to_owned(),
                properties: Vec::new(),
            }],
        }],
    };
    assert!(matches!(
        register_semantic_schema_with_fragments(&connection, "social", &schema, &collision),
        Err(SemanticCatalogError::DuplicateFragmentName { .. })
    ));

    let missing = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Named".to_owned(),
            properties: vec!["displayName".to_owned()],
            members: vec![SemanticFragmentMember {
                node_type: "Customer".to_owned(),
                properties: Vec::new(),
            }],
        }],
    };
    assert!(matches!(
        register_semantic_schema_with_fragments(&connection, "social", &schema, &missing),
        Err(SemanticCatalogError::MissingFragmentProperty { .. })
    ));

    let valid = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Named".to_owned(),
            properties: vec!["displayName".to_owned()],
            members: vec![SemanticFragmentMember {
                node_type: "Customer".to_owned(),
                properties: vec![SemanticProperty {
                    name: "displayName".to_owned(),
                    column: "full_name".to_owned(),
                }],
            }],
        }],
    };
    let mut unknown_member = valid.clone();
    unknown_member.fragments[0].members[0].node_type = "Ghost".to_owned();
    assert!(matches!(
        register_semantic_schema_with_fragments(
            &connection,
            "social",
            &schema,
            &unknown_member
        ),
        Err(SemanticCatalogError::UnknownFragmentMember {
            fragment,
            node_type,
        }) if fragment == "Named" && node_type == "Ghost"
    ));

    let mut duplicate_member = valid.clone();
    let duplicate = duplicate_member.fragments[0].members[0].clone();
    duplicate_member.fragments[0].members.push(duplicate);
    assert!(matches!(
        register_semantic_schema_with_fragments(
            &connection,
            "social",
            &schema,
            &duplicate_member
        ),
        Err(SemanticCatalogError::DuplicateFragmentMember {
            fragment,
            node_type,
        }) if fragment == "Named" && node_type == "Customer"
    ));

    let mut empty_fragment = valid.clone();
    empty_fragment.fragments[0].members.clear();
    assert!(matches!(
        register_semantic_schema_with_fragments(
            &connection,
            "social",
            &schema,
            &empty_fragment
        ),
        Err(SemanticCatalogError::EmptyFragment { fragment }) if fragment == "Named"
    ));

    let mut duplicate_fragment = valid.clone();
    let mut duplicate = duplicate_fragment.fragments[0].clone();
    duplicate.name = "nAmEd".to_owned();
    duplicate_fragment.fragments.push(duplicate);
    assert!(matches!(
        register_semantic_schema_with_fragments(
            &connection,
            "social",
            &schema,
            &duplicate_fragment
        ),
        Err(SemanticCatalogError::DuplicateFragmentName { name }) if name == "nAmEd"
    ));

    let mut extra_mapping = valid.clone();
    extra_mapping.fragments[0].members[0]
        .properties
        .push(SemanticProperty {
            name: "ghost".to_owned(),
            column: "birth_year".to_owned(),
        });
    assert!(matches!(
        register_semantic_schema_with_fragments(
            &connection,
            "social",
            &schema,
            &extra_mapping
        ),
        Err(SemanticCatalogError::UndeclaredFragmentProperty {
            fragment,
            node_type,
            property,
        }) if fragment.as_ref() == "Named"
            && node_type.as_ref() == "Customer"
            && property.as_ref() == "ghost"
    ));

    let conflicting = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Named".to_owned(),
            properties: vec!["displayName".to_owned()],
            members: vec![SemanticFragmentMember {
                node_type: "Customer".to_owned(),
                properties: vec![SemanticProperty {
                    name: "displayName".to_owned(),
                    column: "supplier_name".to_owned(),
                }],
            }],
        }],
    };
    assert!(matches!(
        register_semantic_schema_with_fragments(&connection, "social", &schema, &conflicting),
        Err(SemanticCatalogError::ConflictingPropertyMapping { .. })
    ));

    let incompatible = SemanticFragmentRegistration {
        fragments: vec![SemanticFragment {
            name: "Shared".to_owned(),
            properties: vec!["shared".to_owned()],
            members: vec![
                SemanticFragmentMember {
                    node_type: "Customer".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "shared".to_owned(),
                        column: "full_name".to_owned(),
                    }],
                },
                SemanticFragmentMember {
                    node_type: "Supplier".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "shared".to_owned(),
                        column: "birth_year".to_owned(),
                    }],
                },
            ],
        }],
    };
    assert!(matches!(
        register_semantic_schema_with_fragments(&connection, "social", &schema, &incompatible),
        Err(SemanticCatalogError::IncompatiblePropertyType { .. })
    ));

    register_semantic_schema_with_fragments(&connection, "social", &schema, &valid)
        .expect("all rejected registrations left the catalog clean");
}

fn required_display_name() -> SemanticConstraintRegistration {
    SemanticConstraintRegistration {
        required: vec![SemanticRequiredProperty {
            owner: "Customer".to_owned(),
            property: "displayName".to_owned(),
        }],
        ..SemanticConstraintRegistration::default()
    }
}

fn constrained_session(
    constraints: &SemanticConstraintRegistration,
) -> turso_graph_frontend::Connection {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    register_semantic_constraints(&connection, "social", constraints)
        .expect("register semantic constraints");
    turso_graph_frontend::Connection::open(connection, "social").expect("open constrained graph")
}

fn assert_catalog_constraint_violation(error: SemanticCatalogError) {
    let SemanticCatalogError::ConstraintViolation { constraint, detail } = error else {
        panic!("expected a typed catalog constraint violation, got {error:?}");
    };
    assert!(
        !constraint.is_empty(),
        "constraint identity must be preserved"
    );
    assert!(!detail.is_empty(), "constraint detail must be preserved");
}

fn assert_mutation_constraint_violation(error: FrontendError) {
    let FrontendError::Mutation(MutationError::SemanticConstraintViolation(detail)) = error else {
        panic!("expected a typed runtime constraint violation, got {error:?}");
    };
    assert!(!detail.is_empty(), "runtime detail must be preserved");
}

fn assert_bind_constraint_violation(error: FrontendError) {
    let FrontendError::Mutation(MutationError::Bind(BindError::SemanticConstraintViolation {
        detail,
        span_start,
        span_end,
    })) = error
    else {
        panic!("expected a typed bind-time constraint violation, got {error:?}");
    };
    assert!(!detail.is_empty(), "bind detail must be preserved");
    assert!(span_start < span_end, "bind source span must be non-empty");
}

#[test]
fn required_constraint_is_persisted_idempotently_and_rejects_invalid_existing_data() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
        .expect("open unconstrained graph");
    session
        .execute("CREATE (:Customer {born: 1815})", &Default::default())
        .expect("create data that violates the future constraint");
    let before = graph_generation(&connection, "social").expect("generation before constraints");

    let error = register_semantic_constraints(&connection, "social", &required_display_name())
        .expect_err("invalid existing data must reject activation");
    assert!(matches!(
        error,
        SemanticCatalogError::ConstraintViolation { .. }
    ));
    assert_eq!(
        graph_generation(&connection, "social").expect("generation after rejection"),
        before
    );
    let constraint_rows = connection
        .prepare(
            "SELECT COUNT(*) FROM sqlite_schema \
             WHERE type = 'table' \
               AND name = '__turso_internal_graph_semantic_property_constraints'",
        )
        .and_then(|mut statement| statement.run_collect_rows())
        .expect("constraint catalog remains readable");
    assert!(matches!(
        constraint_rows[0][0],
        turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(0))
    ));

    session
        .execute(
            "MATCH (c:Customer) SET c.displayName = 'Ada'",
            &Default::default(),
        )
        .expect("repair existing data");
    let before_activation =
        graph_generation(&connection, "social").expect("generation before activation");
    register_semantic_constraints(&connection, "social", &required_display_name())
        .expect("activate after repair");
    let activated = graph_generation(&connection, "social").expect("generation after activation");
    assert_eq!(activated, before_activation + 1);
    let persisted = connection
        .prepare(
            "SELECT owner_type_id, property_id \
             FROM __turso_internal_graph_semantic_property_constraints \
             WHERE kind = 'required'",
        )
        .and_then(|mut statement| statement.run_collect_rows())
        .expect("read persisted conceptual constraint identities");
    assert!(matches!(
        persisted.as_slice(),
        [row]
            if matches!(
                row.as_slice(),
                [
                    turso_graph_frontend::Value::Numeric(
                        turso_graph_frontend::Numeric::Integer(1)
                    ),
                    turso_graph_frontend::Value::Numeric(
                        turso_graph_frontend::Numeric::Integer(1)
                    )
                ]
            )
    ));
    register_semantic_constraints(&connection, "social", &required_display_name())
        .expect("identical replay");
    assert_eq!(
        graph_generation(&connection, "social").expect("generation after replay"),
        activated,
        "idempotent replay must not publish a new catalog generation"
    );
}

#[test]
fn every_constraint_kind_rejects_invalid_existing_data_atomically() {
    let cases = [
        (
            "CREATE (:Customer {displayName: 'Ada', born: 1815}), \
             (:Customer {displayName: 'Ada', born: 1816}), \
             (:Supplier {displayName: 'Iron'})",
            SemanticConstraintRegistration {
                required: vec![SemanticRequiredProperty {
                    owner: "Supplier".to_owned(),
                    property: "displayName".to_owned(),
                }],
                unique: vec![SemanticUniqueProperty {
                    owner: "Customer".to_owned(),
                    property: "displayName".to_owned(),
                }],
                ..SemanticConstraintRegistration::default()
            },
        ),
        (
            "CREATE (:Customer {displayName: 'Ada', born: 1815}), \
             (:Customer {displayName: 'Ada', born: 1815})",
            SemanticConstraintRegistration {
                keys: vec![SemanticKeyConstraint {
                    owner: "Customer".to_owned(),
                    properties: vec!["displayName".to_owned(), "born".to_owned()],
                }],
                ..SemanticConstraintRegistration::default()
            },
        ),
        (
            "CREATE (:Customer {displayName: 'Ada', born: 1700})",
            SemanticConstraintRegistration {
                values: vec![SemanticPropertyValueConstraint {
                    owner: "Customer".to_owned(),
                    property: "born".to_owned(),
                    predicate: SemanticValuePredicate::Range {
                        minimum: Some(SemanticRangeBound {
                            value: SemanticScalar::Integer(1800),
                            inclusive: true,
                        }),
                        maximum: None,
                    },
                }],
                ..SemanticConstraintRegistration::default()
            },
        ),
        (
            "CREATE (:Customer {displayName: 'Ada', active: 0})",
            SemanticConstraintRegistration {
                values: vec![SemanticPropertyValueConstraint {
                    owner: "Customer".to_owned(),
                    property: "active".to_owned(),
                    predicate: SemanticValuePredicate::Allowed {
                        values: vec![SemanticScalar::Boolean(true)],
                    },
                }],
                ..SemanticConstraintRegistration::default()
            },
        ),
        (
            "CREATE (:Customer {displayName: 'ada'})",
            SemanticConstraintRegistration {
                values: vec![SemanticPropertyValueConstraint {
                    owner: "Customer".to_owned(),
                    property: "displayName".to_owned(),
                    predicate: SemanticValuePredicate::Regex {
                        pattern: "^[A-Z]".to_owned(),
                    },
                }],
                ..SemanticConstraintRegistration::default()
            },
        ),
        (
            "CREATE (:Customer {displayName: 'Ada'})",
            SemanticConstraintRegistration {
                cardinalities: vec![SemanticRelationshipCardinality {
                    relationship_type: "TRADES_WITH".to_owned(),
                    endpoint: SemanticEndpoint::Start,
                    minimum: 1,
                    maximum: None,
                }],
                ..SemanticConstraintRegistration::default()
            },
        ),
        (
            "CREATE (c:Customer {displayName: 'Ada'})\
             -[:TRADES_WITH {since: 1840}]->(:Supplier {displayName: 'Iron'}), \
             (c)-[:TRADES_WITH {since: 1850}]->(:Supplier {displayName: 'Steel'})",
            SemanticConstraintRegistration {
                cardinalities: vec![SemanticRelationshipCardinality {
                    relationship_type: "TRADES_WITH".to_owned(),
                    endpoint: SemanticEndpoint::Start,
                    minimum: 0,
                    maximum: Some(1),
                }],
                ..SemanticConstraintRegistration::default()
            },
        ),
    ];

    for (seed, constraints) in cases {
        let connection = connection();
        registered_graph(&connection);
        register_semantic_schema(&connection, "social", &semantic_registration())
            .expect("register semantic schema");
        let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
            .expect("open graph before activation");
        session
            .execute(seed, &Default::default())
            .expect("seed invalid future state");
        let before = graph_generation(&connection, "social").expect("generation before rejection");

        let error = register_semantic_constraints(&connection, "social", &constraints)
            .expect_err("invalid existing state must reject activation");
        assert_catalog_constraint_violation(error);
        assert_eq!(
            graph_generation(&connection, "social").expect("generation after rejection"),
            before,
            "failed activation must not publish a generation"
        );
        for table in [
            "__turso_internal_graph_semantic_property_constraints",
            "__turso_internal_graph_semantic_key_constraints",
            "__turso_internal_graph_semantic_cardinality_constraints",
        ] {
            let rows = connection
                .prepare(format!(
                    "SELECT COUNT(*) FROM sqlite_schema \
                     WHERE type = 'table' AND name = '{table}'"
                ))
                .and_then(|mut statement| statement.run_collect_rows())
                .expect("inspect rolled-back constraint catalog");
            assert!(matches!(
                rows[0][0],
                turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(0))
            ));
        }
    }

    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    let supplier_required = SemanticConstraintRegistration {
        required: vec![SemanticRequiredProperty {
            owner: "Supplier".to_owned(),
            property: "displayName".to_owned(),
        }],
        ..SemanticConstraintRegistration::default()
    };
    register_semantic_constraints(&connection, "social", &supplier_required)
        .expect("activate an existing constraint");
    let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
        .expect("open graph with the existing constraint");
    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815}), \
             (:Customer {displayName: 'Ada', born: 1816})",
            &Default::default(),
        )
        .expect("seed data invalid only for the proposed constraint");
    let before =
        graph_generation(&connection, "social").expect("generation before failed addition");
    let customer_unique = SemanticConstraintRegistration {
        unique: vec![SemanticUniqueProperty {
            owner: "Customer".to_owned(),
            property: "displayName".to_owned(),
        }],
        ..SemanticConstraintRegistration::default()
    };
    assert_catalog_constraint_violation(
        register_semantic_constraints(&connection, "social", &customer_unique)
            .expect_err("invalid additive constraint must roll back"),
    );
    assert_eq!(
        graph_generation(&connection, "social").expect("generation after failed addition"),
        before
    );
    let existing = session
        .execute("CREATE (:Supplier)", &Default::default())
        .expect_err("the prior required constraint must remain active");
    assert_mutation_constraint_violation(existing);
}

#[test]
fn required_unique_key_and_cardinality_constraints_enforce_after_reopen() {
    let cases = [
        (
            required_display_name(),
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
            "CREATE (:Customer {born: 1816})",
        ),
        (
            SemanticConstraintRegistration {
                unique: vec![SemanticUniqueProperty {
                    owner: "Customer".to_owned(),
                    property: "displayName".to_owned(),
                }],
                ..SemanticConstraintRegistration::default()
            },
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
            "CREATE (:Customer {displayName: 'Ada', born: 1816})",
        ),
        (
            SemanticConstraintRegistration {
                keys: vec![SemanticKeyConstraint {
                    owner: "Customer".to_owned(),
                    properties: vec!["displayName".to_owned(), "born".to_owned()],
                }],
                ..SemanticConstraintRegistration::default()
            },
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
        ),
        (
            SemanticConstraintRegistration {
                cardinalities: vec![SemanticRelationshipCardinality {
                    relationship_type: "TRADES_WITH".to_owned(),
                    endpoint: SemanticEndpoint::Start,
                    minimum: 0,
                    maximum: Some(1),
                }],
                ..SemanticConstraintRegistration::default()
            },
            "CREATE (c:Customer {displayName: 'Ada'})\
             -[:TRADES_WITH {since: 1840}]->(:Supplier {displayName: 'Iron'})",
            "MATCH (c:Customer {displayName: 'Ada'}) \
             CREATE (c)-[:TRADES_WITH {since: 1850}]->(:Supplier {displayName: 'Steel'})",
        ),
    ];

    for (constraints, seed, violation) in cases {
        let connection = connection();
        registered_graph(&connection);
        register_semantic_schema(&connection, "social", &semantic_registration())
            .expect("register semantic schema");
        register_semantic_constraints(&connection, "social", &constraints)
            .expect("register constraint");
        let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
            .expect("open constrained graph");
        session
            .execute(seed, &Default::default())
            .expect("seed valid constrained state");
        drop(session);

        let reopened = turso_graph_frontend::Connection::open(connection, "social")
            .expect("reopen constrained graph");
        let error = reopened
            .execute(violation, &Default::default())
            .expect_err("reopened constraint must remain active");
        assert_mutation_constraint_violation(error);
    }
}

#[test]
fn an_open_session_observes_newly_activated_constraints() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
        .expect("open session before constraint activation");

    register_semantic_constraints(&connection, "social", &required_display_name())
        .expect("activate required property");
    let error = session
        .execute("CREATE (:Customer {born: 1815})", &Default::default())
        .expect_err("the already-open session must refresh its semantic snapshot");
    assert_mutation_constraint_violation(error);
}

#[test]
fn an_open_legacy_session_refreshes_a_new_semantic_schema() {
    let connection = connection();
    registered_graph(&connection);
    let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
        .expect("open legacy session before semantic registration");

    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect("the existing session must compile mutations with the new schema");
    assert_eq!(
        session
            .query(
                "MATCH (c:Customer) RETURN c.displayName",
                &Default::default(),
            )
            .expect("the registered compiler must use the refreshed schema"),
        vec![vec![turso_graph_frontend::Value::Text("Ada".into())]]
    );
}

#[test]
fn relationship_required_unique_and_key_constraints_are_enforced() {
    let required = constrained_session(&SemanticConstraintRegistration {
        required: vec![SemanticRequiredProperty {
            owner: "TRADES_WITH".to_owned(),
            property: "since".to_owned(),
        }],
        ..SemanticConstraintRegistration::default()
    });
    let error = required
        .execute(
            "CREATE (:Customer {displayName: 'Ada'})\
             -[:TRADES_WITH]->(:Supplier {displayName: 'Iron'})",
            &Default::default(),
        )
        .expect_err("relationship required property must be enforced");
    assert_mutation_constraint_violation(error);

    for constraints in [
        SemanticConstraintRegistration {
            unique: vec![SemanticUniqueProperty {
                owner: "TRADES_WITH".to_owned(),
                property: "since".to_owned(),
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            keys: vec![SemanticKeyConstraint {
                owner: "TRADES_WITH".to_owned(),
                properties: vec!["since".to_owned()],
            }],
            ..SemanticConstraintRegistration::default()
        },
    ] {
        let session = constrained_session(&constraints);
        session
            .execute(
                "CREATE (:Customer {displayName: 'Ada'})\
                 -[:TRADES_WITH {since: 1840}]->(:Supplier {displayName: 'Iron'})",
                &Default::default(),
            )
            .expect("first relationship value");
        let duplicate = session
            .execute(
                "CREATE (:Customer {displayName: 'Grace'})\
                 -[:TRADES_WITH {since: 1840}]->(:Supplier {displayName: 'Steel'})",
                &Default::default(),
            )
            .expect_err("duplicate relationship property must fail");
        assert_mutation_constraint_violation(duplicate);
    }

    let key = constrained_session(&SemanticConstraintRegistration {
        keys: vec![SemanticKeyConstraint {
            owner: "TRADES_WITH".to_owned(),
            properties: vec!["since".to_owned()],
        }],
        ..SemanticConstraintRegistration::default()
    });
    let missing = key
        .execute(
            "CREATE (:Customer {displayName: 'Ada'})\
             -[:TRADES_WITH]->(:Supplier {displayName: 'Iron'})",
            &Default::default(),
        )
        .expect_err("relationship key member must be non-NULL");
    assert_mutation_constraint_violation(missing);
}

#[test]
fn constraint_registration_respects_the_callers_transaction_boundary() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    connection
        .execute("BEGIN IMMEDIATE")
        .expect("begin caller transaction");
    register_semantic_constraints(&connection, "social", &required_display_name())
        .expect("register inside caller transaction");
    connection
        .execute("ROLLBACK")
        .expect("rollback caller transaction");

    let session = turso_graph_frontend::Connection::open(connection, "social")
        .expect("open graph after rollback");
    session
        .execute("CREATE (:Customer {born: 1815})", &Default::default())
        .expect("rolled-back constraint must not remain active");
}

#[test]
fn required_constraint_covers_create_remove_and_dynamic_replacement_atomically() {
    let session = constrained_session(&required_display_name());

    let create = session
        .execute("CREATE (:Customer {born: 1815})", &Default::default())
        .expect_err("required property is missing");
    assert!(create.to_string().contains("required"), "{create}");

    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect("valid create");
    session
        .execute(
            "MATCH (c:Customer {displayName: 'Ada'}) \
             REMOVE c.displayName \
             SET c.displayName = 'Ada'",
            &Default::default(),
        )
        .expect("final valid state may repair an intermediate required-property violation");
    session
        .execute(
            "MERGE (c:Customer {born: 1906}) \
             ON CREATE SET c.displayName = 'Grace'",
            &Default::default(),
        )
        .expect("ON CREATE may satisfy a required property");
    session
        .execute(
            "MERGE (c:Customer {born: 1906}) \
             ON MATCH SET c.active = 1",
            &Default::default(),
        )
        .expect("ON MATCH preserves an existing required property");
    let remove = session
        .execute(
            "MATCH (c:Customer) REMOVE c.displayName",
            &Default::default(),
        )
        .expect_err("required property cannot be removed");
    assert!(remove.to_string().contains("is required"), "{remove}");

    let parameters = turso_graph_frontend::Parameters::from([(
        "replacement".to_owned(),
        turso_graph_frontend::Value::Text(r#"{"born":1816}"#.into()),
    )]);
    let replacement = session
        .execute("MATCH (c:Customer) SET c = $replacement", &parameters)
        .expect_err("dynamic replacement must validate required properties");
    assert!(
        replacement.to_string().contains("required"),
        "{replacement}"
    );
    let rows = session
        .query(
            "MATCH (c:Customer {displayName: 'Ada'}) RETURN c.displayName, c.born",
            &Default::default(),
        )
        .expect("read after rejected mutations");
    assert_eq!(
        rows,
        vec![vec![
            turso_graph_frontend::Value::Text("Ada".into()),
            turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1815)),
        ]]
    );
}

#[test]
fn required_constraint_covers_nulls_merge_replacements_and_staged_routes() {
    let session = constrained_session(&required_display_name());
    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815}), \
             (:Customer {displayName: 'Grace', born: 1906})",
            &Default::default(),
        )
        .expect("seed valid customers");

    let literal_null = session
        .execute(
            "MATCH (c:Customer {displayName: 'Ada'}) SET c.displayName = null",
            &Default::default(),
        )
        .expect_err("literal NULL must be rejected during binding");
    assert_bind_constraint_violation(literal_null);

    let null_parameter = turso_graph_frontend::Parameters::from([(
        "name".to_owned(),
        turso_graph_frontend::Value::Null,
    )]);
    let runtime_null = session
        .execute(
            "MATCH (c:Customer {displayName: 'Ada'}) SET c.displayName = $name",
            &null_parameter,
        )
        .expect_err("parameterized NULL must be rejected at runtime");
    assert_mutation_constraint_violation(runtime_null);

    let static_replacement = session
        .execute(
            "MATCH (c:Customer {displayName: 'Ada'}) SET c = {born: 1816}",
            &Default::default(),
        )
        .expect_err("static whole-entity replacement must preserve required properties");
    assert_mutation_constraint_violation(static_replacement);

    let on_create = session
        .execute(
            "MERGE (c:Customer {born: 1912}) ON CREATE SET c.active = 1",
            &Default::default(),
        )
        .expect_err("ON CREATE must satisfy every required property");
    assert_mutation_constraint_violation(on_create);

    let on_match = session
        .execute(
            "MERGE (c:Customer {displayName: 'Grace', born: 1906}) \
             ON MATCH SET c.displayName = null",
            &Default::default(),
        )
        .expect_err("ON MATCH cannot clear a required property");
    assert_bind_constraint_violation(on_match);

    let unwind = session
        .execute(
            "WITH [1, 0] AS keeps \
             UNWIND keeps AS keep \
             CREATE (:Customer {\
                 displayName: CASE WHEN keep = 1 THEN 'Row' ELSE null END\
             })",
            &Default::default(),
        )
        .expect_err("one invalid UNWIND row must roll back all rows");
    assert_mutation_constraint_violation(unwind);

    let foreach = session
        .execute(
            "FOREACH (keep IN [1, 0] | \
                 CREATE (:Customer {\
                     displayName: CASE WHEN keep = 1 THEN 'Loop' ELSE null END\
                 })\
             )",
            &Default::default(),
        )
        .expect_err("one invalid FOREACH iteration must roll back all iterations");
    assert_mutation_constraint_violation(foreach);

    assert_eq!(
        session
            .query(
                "MATCH (c:Customer) RETURN c.displayName ORDER BY c.displayName",
                &Default::default(),
            )
            .expect("read after rejected required-property mutations"),
        vec![
            vec![turso_graph_frontend::Value::Text("Ada".into())],
            vec![turso_graph_frontend::Value::Text("Grace".into())],
        ]
    );
}

#[test]
fn direct_sql_can_bypass_semantic_constraints_but_later_cypher_detects_the_violation() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    register_semantic_constraints(&connection, "social", &required_display_name())
        .expect("register required property");
    let graph = load_registered_graph(&connection, "social").expect("load graph");
    let source = graph.node_sources[0].id;
    let labels = labels_table_name(graph.id);
    connection
        .execute(format!(
            "INSERT INTO tbl_people(pk, birth_year) VALUES (1, 1815); \
             INSERT INTO \"{labels}\"(source_id, node_id, label) \
             VALUES ({}, 1, 'Customer')",
            source.get()
        ))
        .expect("direct SQL is outside semantic enforcement");

    let session = turso_graph_frontend::Connection::open(connection, "social")
        .expect("open graph containing direct-SQL violation");
    let error = session
        .execute(
            "CREATE (:Supplier {displayName: 'Iron Co'})",
            &Default::default(),
        )
        .expect_err("the next Cypher mutation validates the complete active constraint state");
    assert!(error.to_string().contains("is required"), "{error}");
    assert!(
        session
            .query("MATCH (s:Supplier) RETURN s", &Default::default())
            .expect("read after rollback")
            .is_empty(),
        "the unrelated Cypher write must roll back"
    );
}

#[test]
fn keys_and_unique_values_are_scoped_to_one_semantic_type() {
    let session = constrained_session(&SemanticConstraintRegistration {
        unique: vec![SemanticUniqueProperty {
            owner: "Customer".to_owned(),
            property: "displayName".to_owned(),
        }],
        ..SemanticConstraintRegistration::default()
    });
    session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815}), \
             (:Supplier {displayName: 'Ada'})",
            &Default::default(),
        )
        .expect("another type sharing the source does not contaminate uniqueness");

    let duplicate = session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1816})",
            &Default::default(),
        )
        .expect_err("unique value must reject a second customer");
    assert!(
        duplicate.to_string().contains("duplicate value"),
        "{duplicate}"
    );
    let rows = session
        .query(
            "MATCH (c:Customer) RETURN c.displayName",
            &Default::default(),
        )
        .expect("read surviving customer");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Text("Ada".into())]]
    );

    let key_session = constrained_session(&SemanticConstraintRegistration {
        keys: vec![SemanticKeyConstraint {
            owner: "Customer".to_owned(),
            properties: vec!["displayName".to_owned(), "born".to_owned()],
        }],
        ..SemanticConstraintRegistration::default()
    });
    let duplicate_key = key_session
        .execute(
            "CREATE (:Customer {displayName: 'Grace', born: 1906}), \
             (:Customer {displayName: 'Grace', born: 1906})",
            &Default::default(),
        )
        .expect_err("duplicate composite key must roll back the whole mutation");
    assert!(
        duplicate_key.to_string().contains("duplicate tuple"),
        "{duplicate_key}"
    );
    let missing_key_member = key_session
        .execute(
            "CREATE (:Customer {displayName: 'Katherine'})",
            &Default::default(),
        )
        .expect_err("key members are required");
    assert!(
        missing_key_member.to_string().contains("NULL member"),
        "{missing_key_member}"
    );
    let rows = key_session
        .query(
            "MATCH (c:Customer) RETURN c.displayName",
            &Default::default(),
        )
        .expect("read surviving customer");
    assert_eq!(rows, Vec::<Vec<turso_graph_frontend::Value>>::new());
}

#[test]
fn unique_and_composite_keys_cover_null_update_merge_and_dynamic_routes() {
    let unique = constrained_session(&SemanticConstraintRegistration {
        unique: vec![SemanticUniqueProperty {
            owner: "Customer".to_owned(),
            property: "displayName".to_owned(),
        }],
        ..SemanticConstraintRegistration::default()
    });
    unique
        .execute(
            "CREATE (:Customer {born: 1800}), (:Customer {born: 1801}), \
             (:Customer {displayName: 'Ada', born: 1815}), \
             (:Customer {displayName: 'Grace', born: 1906})",
            &Default::default(),
        )
        .expect("unique permits multiple NULL values");
    let update = unique
        .execute(
            "MATCH (c:Customer {displayName: 'Grace'}) SET c.displayName = 'Ada'",
            &Default::default(),
        )
        .expect_err("SET cannot introduce a duplicate unique value");
    assert_mutation_constraint_violation(update);
    let replacement = turso_graph_frontend::Parameters::from([(
        "replacement".to_owned(),
        turso_graph_frontend::Value::Text(r#"{"displayName":"Ada","born":1906}"#.into()),
    )]);
    let dynamic = unique
        .execute(
            "MATCH (c:Customer {displayName: 'Grace'}) SET c = $replacement",
            &replacement,
        )
        .expect_err("dynamic replacement cannot introduce a duplicate unique value");
    assert_mutation_constraint_violation(dynamic);

    let key = constrained_session(&SemanticConstraintRegistration {
        keys: vec![SemanticKeyConstraint {
            owner: "Customer".to_owned(),
            properties: vec!["displayName".to_owned(), "born".to_owned()],
        }],
        ..SemanticConstraintRegistration::default()
    });
    key.execute(
        "CREATE (:Customer {displayName: 'Ada', born: 1815}), \
         (:Customer {displayName: 'Grace', born: 1906}), \
         (:Supplier {displayName: 'Ada'})",
        &Default::default(),
    )
    .expect("another semantic type sharing the table does not contaminate a key");

    let update = key
        .execute(
            "MATCH (c:Customer {displayName: 'Grace'}) \
             SET c.displayName = 'Ada', c.born = 1815",
            &Default::default(),
        )
        .expect_err("SET cannot introduce a duplicate composite key");
    assert_mutation_constraint_violation(update);
    let removal = key
        .execute(
            "MATCH (c:Customer {displayName: 'Grace'}) REMOVE c.born",
            &Default::default(),
        )
        .expect_err("REMOVE cannot clear a key member");
    assert_mutation_constraint_violation(removal);
    let replacement = turso_graph_frontend::Parameters::from([(
        "replacement".to_owned(),
        turso_graph_frontend::Value::Text(r#"{"displayName":"Grace"}"#.into()),
    )]);
    let dynamic = key
        .execute(
            "MATCH (c:Customer {displayName: 'Grace'}) SET c = $replacement",
            &replacement,
        )
        .expect_err("dynamic replacement cannot omit a key member");
    assert_mutation_constraint_violation(dynamic);
    let on_match = key
        .execute(
            "MERGE (c:Customer {displayName: 'Grace', born: 1906}) \
             ON MATCH SET c.displayName = 'Ada', c.born = 1815",
            &Default::default(),
        )
        .expect_err("MERGE ON MATCH cannot introduce a duplicate key");
    assert_mutation_constraint_violation(on_match);

    assert_eq!(
        key.query(
            "MATCH (c:Customer) RETURN c.displayName, c.born ORDER BY c.born",
            &Default::default(),
        )
        .expect("read after rejected key mutations"),
        vec![
            vec![
                turso_graph_frontend::Value::Text("Ada".into()),
                turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1815)),
            ],
            vec![
                turso_graph_frontend::Value::Text("Grace".into()),
                turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1906)),
            ],
        ]
    );
}

#[test]
fn range_allowed_and_regex_constraints_validate_literals_parameters_and_reopen() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    let constraints = SemanticConstraintRegistration {
        values: vec![
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1800),
                        inclusive: true,
                    }),
                    maximum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1900),
                        inclusive: false,
                    }),
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "displayName".to_owned(),
                predicate: SemanticValuePredicate::Regex {
                    pattern: "^[A-Z][A-Za-z ]+$".to_owned(),
                },
            },
            SemanticPropertyValueConstraint {
                owner: "TRADES_WITH".to_owned(),
                property: "since".to_owned(),
                predicate: SemanticValuePredicate::Allowed {
                    values: vec![SemanticScalar::Integer(1840), SemanticScalar::Integer(1850)],
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "active".to_owned(),
                predicate: SemanticValuePredicate::Allowed {
                    values: vec![SemanticScalar::Boolean(true)],
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Supplier".to_owned(),
                property: "displayName".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Text("A".to_owned()),
                        inclusive: true,
                    }),
                    maximum: Some(SemanticRangeBound {
                        value: SemanticScalar::Text("Zzz".to_owned()),
                        inclusive: false,
                    }),
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "score".to_owned(),
                predicate: SemanticValuePredicate::Allowed {
                    values: vec![SemanticScalar::Real(1.5)],
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "category".to_owned(),
                predicate: SemanticValuePredicate::Allowed {
                    values: vec![
                        SemanticScalar::Text("primary".to_owned()),
                        SemanticScalar::Text("secondary".to_owned()),
                    ],
                },
            },
        ],
        ..SemanticConstraintRegistration::default()
    };
    register_semantic_constraints(&connection, "social", &constraints)
        .expect("register value constraints");
    let session = turso_graph_frontend::Connection::open(connection.clone(), "social")
        .expect("open constrained graph");

    let literal_source = "CREATE (:Customer {displayName: 'ada', born: 1815})";
    let literal = session
        .execute(literal_source, &Default::default())
        .expect_err("literal regex failure binds early");
    let FrontendError::Mutation(MutationError::Bind(BindError::SemanticConstraintViolation {
        detail,
        span_start,
        span_end,
    })) = literal
    else {
        panic!("expected a typed regex bind failure, got {literal:?}");
    };
    assert!(detail.contains("regular expression"), "{detail}");
    assert_eq!(&literal_source[span_start..span_end], "'ada'");
    let parameters = turso_graph_frontend::Parameters::from([
        (
            "name".to_owned(),
            turso_graph_frontend::Value::Text("Ada".into()),
        ),
        (
            "born".to_owned(),
            turso_graph_frontend::Value::Numeric(turso_graph_frontend::Numeric::Integer(1900)),
        ),
    ]);
    let runtime = session
        .execute(
            "CREATE (:Customer {displayName: $name, born: $born})",
            &parameters,
        )
        .expect_err("exclusive upper bound rejects a parameter");
    assert!(
        runtime.to_string().contains("configured range"),
        "{runtime}"
    );
    let staged = session
        .execute(
            "WITH [1815, 1900] AS years \
             UNWIND years AS year \
             CREATE (:Customer {displayName: 'Row', born: year})",
            &Default::default(),
        )
        .expect_err("one invalid staged row rolls back every row");
    assert!(staged.to_string().contains("configured range"), "{staged}");
    let foreach = session
        .execute(
            "FOREACH (year IN [1815, 1900] | \
                 CREATE (:Customer {displayName: 'Loop', born: year})\
             )",
            &Default::default(),
        )
        .expect_err("one invalid FOREACH iteration rolls back every iteration");
    assert!(
        foreach.to_string().contains("configured range"),
        "{foreach}"
    );
    assert!(
        session
            .query("MATCH (c:Customer) RETURN c", &Default::default())
            .expect("read after staged failures")
            .is_empty(),
        "constraint failures must leave zero partial rows"
    );
    session
        .execute(
            "CREATE (:Customer {displayName: 'Minimum', born: 1800, active: 1})",
            &Default::default(),
        )
        .expect("inclusive numeric lower bound accepts equality");
    let boolean = session
        .execute(
            "CREATE (:Customer {displayName: 'Grace', born: 1815, active: 0})",
            &Default::default(),
        )
        .expect_err("Boolean allowed-value predicate rejects false");
    assert!(boolean.to_string().contains("allowed-value"), "{boolean}");
    let real = session
        .execute(
            "CREATE (:Customer {displayName: 'Real', born: 1815, score: 1.6})",
            &Default::default(),
        )
        .expect_err("real allowed-value predicate rejects an unlisted value");
    assert!(real.to_string().contains("allowed-value"), "{real}");
    let text_allowed = session
        .execute(
            "CREATE (:Customer {displayName: 'Category', born: 1815, category: 'other'})",
            &Default::default(),
        )
        .expect_err("text allowed-value predicate rejects an unlisted value");
    assert!(
        text_allowed.to_string().contains("allowed-value"),
        "{text_allowed}"
    );

    drop(session);
    let reopened = turso_graph_frontend::Connection::open(connection, "social")
        .expect("reopen constrained graph");
    reopened
        .execute(
            "CREATE (c:Customer {\
                 displayName: 'Ada Lovelace', born: 1815, active: 1, score: 1.5, category: 'primary'\
             })\
             -[:TRADES_WITH {since: 1840}]->\
             (:Supplier {displayName: 'Iron Co'})",
            &Default::default(),
        )
        .expect("all persisted predicates accept valid values");
    let text_range = reopened
        .execute(
            "CREATE (:Supplier {displayName: 'zzz'})",
            &Default::default(),
        )
        .expect_err("text range rejects a value above its upper bound");
    assert!(
        text_range.to_string().contains("configured range"),
        "{text_range}"
    );
    let disallowed = reopened
        .execute(
            "MATCH (c:Customer), (s:Supplier) \
             CREATE (c)-[:TRADES_WITH {since: 1841}]->(s)",
            &Default::default(),
        )
        .expect_err("allowed values survive reopen");
    assert!(
        disallowed.to_string().contains("allowed-value"),
        "{disallowed}"
    );
}

#[test]
fn value_predicates_cover_all_boundaries_nulls_and_numeric_coercions() {
    let session = constrained_session(&SemanticConstraintRegistration {
        values: vec![
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1800),
                        inclusive: false,
                    }),
                    maximum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1900),
                        inclusive: true,
                    }),
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "score".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1),
                        inclusive: true,
                    }),
                    maximum: Some(SemanticRangeBound {
                        value: SemanticScalar::Real(2.5),
                        inclusive: true,
                    }),
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "active".to_owned(),
                predicate: SemanticValuePredicate::Allowed {
                    values: vec![SemanticScalar::Boolean(true)],
                },
            },
            SemanticPropertyValueConstraint {
                owner: "Supplier".to_owned(),
                property: "displayName".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Text("M".to_owned()),
                        inclusive: true,
                    }),
                    maximum: None,
                },
            },
        ],
        ..SemanticConstraintRegistration::default()
    });

    session
        .execute(
            "CREATE (:Customer {displayName: 'Nulls'}), (:Supplier)",
            &Default::default(),
        )
        .expect("value predicates ignore NULL values");
    for query in [
        "CREATE (:Customer {displayName: 'Lower', born: 1800})",
        "CREATE (:Customer {displayName: 'Above', born: 1901})",
        "CREATE (:Customer {displayName: 'Real', score: 2.6})",
        "CREATE (:Supplier {displayName: 'A'})",
    ] {
        let error = session
            .execute(query, &Default::default())
            .expect_err("out-of-range boundary must fail");
        assert_bind_constraint_violation(error);
    }
    session
        .execute(
            "CREATE (:Customer {displayName: 'Inside', born: 1801}), \
             (:Customer {displayName: 'Upper', born: 1900}), \
             (:Customer {displayName: 'IntegerReal', score: 1}), \
             (:Customer {displayName: 'RealUpper', score: 2.5}), \
             (:Supplier {displayName: 'M'})",
            &Default::default(),
        )
        .expect("valid exact and cross-numeric boundaries");
}

#[test]
fn constraint_evolution_and_invalid_predicates_fail_without_changes() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");
    let range = |maximum| SemanticConstraintRegistration {
        values: vec![SemanticPropertyValueConstraint {
            owner: "Customer".to_owned(),
            property: "born".to_owned(),
            predicate: SemanticValuePredicate::Range {
                minimum: None,
                maximum: Some(SemanticRangeBound {
                    value: SemanticScalar::Integer(maximum),
                    inclusive: true,
                }),
            },
        }],
        ..SemanticConstraintRegistration::default()
    };
    register_semantic_constraints(&connection, "social", &range(1900)).expect("first range");
    let before = graph_generation(&connection, "social").expect("generation before conflict");
    assert!(matches!(
        register_semantic_constraints(&connection, "social", &range(2000)),
        Err(SemanticCatalogError::ConstraintEvolutionUnsupported { .. })
    ));
    assert_eq!(
        graph_generation(&connection, "social").expect("generation after conflict"),
        before
    );
    let invalid_regex = SemanticConstraintRegistration {
        values: vec![SemanticPropertyValueConstraint {
            owner: "Customer".to_owned(),
            property: "displayName".to_owned(),
            predicate: SemanticValuePredicate::Regex {
                pattern: "(".to_owned(),
            },
        }],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&connection, "social", &invalid_regex),
        Err(SemanticCatalogError::InvalidConstraint { .. })
    ));

    let allowed = |values| SemanticConstraintRegistration {
        values: vec![SemanticPropertyValueConstraint {
            owner: "Customer".to_owned(),
            property: "active".to_owned(),
            predicate: SemanticValuePredicate::Allowed { values },
        }],
        ..SemanticConstraintRegistration::default()
    };
    register_semantic_constraints(
        &connection,
        "social",
        &allowed(vec![SemanticScalar::Integer(1), SemanticScalar::Integer(2)]),
    )
    .expect("register allowed values");
    let before_reordered =
        graph_generation(&connection, "social").expect("generation before reordered replay");
    register_semantic_constraints(
        &connection,
        "social",
        &allowed(vec![SemanticScalar::Integer(2), SemanticScalar::Integer(1)]),
    )
    .expect("allowed-value ordering is not semantic evolution");
    assert_eq!(
        graph_generation(&connection, "social").expect("generation after reordered replay"),
        before_reordered
    );

    let cardinality = |maximum| SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 0,
            maximum: Some(maximum),
        }],
        ..SemanticConstraintRegistration::default()
    };
    register_semantic_constraints(&connection, "social", &cardinality(2))
        .expect("register cardinality");
    let before_cardinality =
        graph_generation(&connection, "social").expect("generation before cardinality conflict");
    assert!(matches!(
        register_semantic_constraints(&connection, "social", &cardinality(3)),
        Err(SemanticCatalogError::ConstraintEvolutionUnsupported { .. })
    ));
    assert_eq!(
        graph_generation(&connection, "social")
            .expect("generation after cardinality evolution rejection"),
        before_cardinality
    );
}

#[test]
fn constraint_registration_rejects_unknown_duplicate_and_incompatible_definitions() {
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration())
        .expect("register semantic schema");

    let unknown = SemanticConstraintRegistration {
        required: vec![SemanticRequiredProperty {
            owner: "Customer".to_owned(),
            property: "missing".to_owned(),
        }],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&connection, "social", &unknown),
        Err(SemanticCatalogError::UnknownConstraintProperty { .. })
    ));

    let duplicate = SemanticConstraintRegistration {
        required: vec![
            SemanticRequiredProperty {
                owner: "Customer".to_owned(),
                property: "displayName".to_owned(),
            },
            SemanticRequiredProperty {
                owner: "customer".to_owned(),
                property: "DISPLAYNAME".to_owned(),
            },
        ],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&connection, "social", &duplicate),
        Err(SemanticCatalogError::DuplicateConstraint { .. })
    ));

    let incompatible = SemanticConstraintRegistration {
        values: vec![SemanticPropertyValueConstraint {
            owner: "Customer".to_owned(),
            property: "born".to_owned(),
            predicate: SemanticValuePredicate::Regex {
                pattern: "^[0-9]+$".to_owned(),
            },
        }],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&connection, "social", &incompatible),
        Err(SemanticCatalogError::InvalidConstraint { .. })
    ));

    let impossible_cardinality = SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 2,
            maximum: Some(1),
        }],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&connection, "social", &impossible_cardinality),
        Err(SemanticCatalogError::InvalidConstraint { .. })
    ));
}

#[test]
fn constraint_registration_rejects_every_invalid_public_shape() {
    let invalid_constraints = [
        SemanticConstraintRegistration {
            values: vec![SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: None,
                    maximum: None,
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            values: vec![SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Allowed { values: Vec::new() },
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            values: vec![SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Boolean(false),
                        inclusive: true,
                    }),
                    maximum: None,
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            values: vec![SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1),
                        inclusive: true,
                    }),
                    maximum: Some(SemanticRangeBound {
                        value: SemanticScalar::Text("z".to_owned()),
                        inclusive: true,
                    }),
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            values: vec![SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1),
                        inclusive: false,
                    }),
                    maximum: Some(SemanticRangeBound {
                        value: SemanticScalar::Integer(1),
                        inclusive: true,
                    }),
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            values: vec![SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "score".to_owned(),
                predicate: SemanticValuePredicate::Range {
                    minimum: Some(SemanticRangeBound {
                        value: SemanticScalar::Real(f64::NAN),
                        inclusive: true,
                    }),
                    maximum: None,
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            values: vec![SemanticPropertyValueConstraint {
                owner: "Customer".to_owned(),
                property: "born".to_owned(),
                predicate: SemanticValuePredicate::Allowed {
                    values: vec![SemanticScalar::Text("wrong".to_owned())],
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            keys: vec![SemanticKeyConstraint {
                owner: "Customer".to_owned(),
                properties: Vec::new(),
            }],
            ..SemanticConstraintRegistration::default()
        },
        SemanticConstraintRegistration {
            keys: vec![SemanticKeyConstraint {
                owner: "Customer".to_owned(),
                properties: vec!["born".to_owned(), "BORN".to_owned()],
            }],
            ..SemanticConstraintRegistration::default()
        },
    ];

    for invalid in invalid_constraints {
        let connection = connection();
        registered_graph(&connection);
        register_semantic_schema(&connection, "social", &semantic_registration())
            .expect("register semantic schema");
        assert!(matches!(
            register_semantic_constraints(&connection, "social", &invalid),
            Err(SemanticCatalogError::InvalidConstraint { .. })
        ));
    }

    let definition_connection = connection();
    registered_graph(&definition_connection);
    register_semantic_schema(&definition_connection, "social", &semantic_registration())
        .expect("register semantic schema");
    let duplicate_key = SemanticConstraintRegistration {
        keys: vec![
            SemanticKeyConstraint {
                owner: "Customer".to_owned(),
                properties: vec!["displayName".to_owned(), "born".to_owned()],
            },
            SemanticKeyConstraint {
                owner: "customer".to_owned(),
                properties: vec!["BORN".to_owned(), "DISPLAYNAME".to_owned()],
            },
        ],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&definition_connection, "social", &duplicate_key),
        Err(SemanticCatalogError::DuplicateConstraint { .. })
    ));
    let duplicate_cardinality = SemanticConstraintRegistration {
        cardinalities: vec![
            SemanticRelationshipCardinality {
                relationship_type: "TRADES_WITH".to_owned(),
                endpoint: SemanticEndpoint::End,
                minimum: 0,
                maximum: Some(1),
            },
            SemanticRelationshipCardinality {
                relationship_type: "trades_with".to_owned(),
                endpoint: SemanticEndpoint::End,
                minimum: 0,
                maximum: Some(2),
            },
        ],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&definition_connection, "social", &duplicate_cardinality),
        Err(SemanticCatalogError::DuplicateConstraint { .. })
    ));
    let unknown_cardinality = SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "MISSING".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 0,
            maximum: None,
        }],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&definition_connection, "social", &unknown_cardinality),
        Err(SemanticCatalogError::UnknownConstraintOwner { .. })
    ));

    let endpoint_connection = connection();
    registered_graph(&endpoint_connection);
    let mut endpoint_free = semantic_registration();
    endpoint_free.relationship_types[0]
        .roles
        .iter_mut()
        .find(|role| role.name == "start")
        .expect("start role")
        .targets
        .clear();
    register_semantic_schema(&endpoint_connection, "social", &endpoint_free)
        .expect("register relationship without a permitted start type");
    let missing_endpoint = SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 0,
            maximum: None,
        }],
        ..SemanticConstraintRegistration::default()
    };
    assert!(matches!(
        register_semantic_constraints(&endpoint_connection, "social", &missing_endpoint),
        Err(SemanticCatalogError::InvalidConstraint { .. })
    ));
}

#[test]
fn endpoint_cardinality_validates_final_transaction_state_and_rolls_back_overflow() {
    let session = constrained_session(&SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 1,
            maximum: Some(1),
        }],
        ..SemanticConstraintRegistration::default()
    });

    let isolated = session
        .execute(
            "CREATE (:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect_err("minimum participation rejects an isolated customer");
    assert!(isolated.to_string().contains("participation"), "{isolated}");
    session
        .execute(
            "CREATE (c:Customer {displayName: 'Ada', born: 1815})\
             -[:TRADES_WITH {since: 1840}]->\
             (:Supplier {displayName: 'Iron Co'})",
            &Default::default(),
        )
        .expect("one query can create the node and satisfy its minimum");
    let deletion = session
        .execute(
            "MATCH (:Customer)-[r:TRADES_WITH]->(:Supplier) DELETE r",
            &Default::default(),
        )
        .expect_err("deleting the only relationship violates the minimum");
    assert!(deletion.to_string().contains("participation"), "{deletion}");
    let overflow = session
        .execute(
            "MATCH (c:Customer {displayName: 'Ada'}) \
             CREATE (c)-[:TRADES_WITH {since: 1850}]->\
             (:Supplier {displayName: 'Steel Co'})",
            &Default::default(),
        )
        .expect_err("maximum participation rejects the second relationship");
    assert!(overflow.to_string().contains("participation"), "{overflow}");
    let rows = session
        .query(
            "MATCH (c:Customer)-[r:TRADES_WITH]->(s:Supplier) \
             RETURN s.displayName",
            &Default::default(),
        )
        .expect("read after overflow rollback");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Text("Iron Co".into())]]
    );
    assert!(
        session
            .query(
                "MATCH (s:Supplier {displayName: 'Steel Co'}) RETURN s",
                &Default::default(),
            )
            .expect("read endpoint created by rejected overflow")
            .is_empty(),
        "the endpoint node created by the rejected relationship must roll back"
    );
}

#[test]
fn end_cardinality_uses_stored_direction_for_incoming_cypher_syntax() {
    let session = constrained_session(&SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::End,
            minimum: 0,
            maximum: Some(1),
        }],
        ..SemanticConstraintRegistration::default()
    });
    session
        .execute(
            "CREATE (s:Supplier {displayName: 'Iron Co'})\
             <-[:TRADES_WITH {since: 1840}]-\
             (:Customer {displayName: 'Ada', born: 1815})",
            &Default::default(),
        )
        .expect("incoming syntax stores Customer at start and Supplier at end");
    let error = session
        .execute(
            "MATCH (s:Supplier {displayName: 'Iron Co'}) \
             CREATE (s)<-[:TRADES_WITH {since: 1850}]-\
             (:Customer {displayName: 'Grace', born: 1816})",
            &Default::default(),
        )
        .expect_err("stored end cardinality rejects a second incoming relationship");
    assert!(error.to_string().contains("participation"), "{error}");
    let rows = session
        .query(
            "MATCH (s:Supplier)<-[r:TRADES_WITH]-(c:Customer) \
             RETURN c.displayName",
            &Default::default(),
        )
        .expect("read after incoming overflow rollback");
    assert_eq!(
        rows,
        vec![vec![turso_graph_frontend::Value::Text("Ada".into())]]
    );
    assert!(
        session
            .query(
                "MATCH (c:Customer {displayName: 'Grace'}) RETURN c",
                &Default::default(),
            )
            .expect("read endpoint created by rejected incoming overflow")
            .is_empty(),
        "the rejected incoming relationship must not leave its start node behind"
    );
}

#[test]
fn cardinality_covers_end_minimum_zero_maximum_and_merge() {
    let end_minimum = constrained_session(&SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::End,
            minimum: 1,
            maximum: None,
        }],
        ..SemanticConstraintRegistration::default()
    });
    let isolated = end_minimum
        .execute(
            "CREATE (:Supplier {displayName: 'Iron'})",
            &Default::default(),
        )
        .expect_err("end minimum rejects an isolated permitted endpoint");
    assert_mutation_constraint_violation(isolated);
    end_minimum
        .execute(
            "CREATE (:Customer {displayName: 'Ada'})\
             -[:TRADES_WITH {since: 1840}]->(:Supplier {displayName: 'Iron'})",
            &Default::default(),
        )
        .expect("one query may satisfy an end minimum");

    let zero_maximum = constrained_session(&SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 0,
            maximum: Some(0),
        }],
        ..SemanticConstraintRegistration::default()
    });
    zero_maximum
        .execute(
            "CREATE (:Customer {displayName: 'Ada'}), \
             (:Supplier {displayName: 'Iron'})",
            &Default::default(),
        )
        .expect("zero maximum permits isolated nodes");
    let forbidden = zero_maximum
        .execute(
            "MATCH (c:Customer), (s:Supplier) \
             CREATE (c)-[:TRADES_WITH {since: 1840}]->(s)",
            &Default::default(),
        )
        .expect_err("zero maximum rejects any relationship");
    assert_mutation_constraint_violation(forbidden);

    let merge = constrained_session(&SemanticConstraintRegistration {
        cardinalities: vec![SemanticRelationshipCardinality {
            relationship_type: "TRADES_WITH".to_owned(),
            endpoint: SemanticEndpoint::Start,
            minimum: 0,
            maximum: Some(1),
        }],
        ..SemanticConstraintRegistration::default()
    });
    merge
        .execute(
            "CREATE (c:Customer {displayName: 'Ada'})\
             -[:TRADES_WITH {since: 1840}]->(s:Supplier {displayName: 'Iron'})",
            &Default::default(),
        )
        .expect("seed one relationship");
    merge
        .execute(
            "MATCH (c:Customer {displayName: 'Ada'}), \
                   (s:Supplier {displayName: 'Iron'}) \
             MERGE (c)-[:TRADES_WITH {since: 1840}]->(s)",
            &Default::default(),
        )
        .expect("MERGE matching the existing relationship does not increase cardinality");
    let overflow = merge
        .execute(
            "MATCH (c:Customer {displayName: 'Ada'}), \
                   (s:Supplier {displayName: 'Iron'}) \
             MERGE (c)-[:TRADES_WITH {since: 1850}]->(s)",
            &Default::default(),
        )
        .expect_err("MERGE creating a second relationship must honor maximum cardinality");
    assert_mutation_constraint_violation(overflow);
}

#[test]
fn cardinality_expands_fragment_endpoints_and_handles_colliding_source_ids() {
    let connection = connection();
    register_fragment_graph(&connection);
    let mut fragments = fragment_registration();
    fragments.fragments.push(SemanticFragment {
        name: "Worker".to_owned(),
        properties: Vec::new(),
        members: vec![
            SemanticFragmentMember {
                node_type: "Person".to_owned(),
                properties: Vec::new(),
            },
            SemanticFragmentMember {
                node_type: "Alias".to_owned(),
                properties: Vec::new(),
            },
        ],
    });
    let mut schema = fragment_schema();
    schema.relationship_types[0]
        .roles
        .iter_mut()
        .find(|role| role.name == "start")
        .expect("start role")
        .targets = vec!["Worker".to_owned()];
    register_semantic_schema_with_fragments(&connection, "fragments", &schema, &fragments)
        .expect("register fragment-expanded endpoint");
    register_semantic_constraints(
        &connection,
        "fragments",
        &SemanticConstraintRegistration {
            cardinalities: vec![SemanticRelationshipCardinality {
                relationship_type: "WORKS_AT".to_owned(),
                endpoint: SemanticEndpoint::Start,
                minimum: 1,
                maximum: Some(1),
            }],
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register fragment-expanded cardinality");
    let session = turso_graph_frontend::Connection::open(connection, "fragments")
        .expect("open fragment-cardinality graph");
    let isolated_alias = session
        .execute("CREATE (:Alias:Worker)", &Default::default())
        .expect_err("every concrete fragment member must satisfy the minimum");
    assert_mutation_constraint_violation(isolated_alias);
    session
        .execute(
            "CREATE (:Alias:Worker)\
             -[:WORKS_AT]->(:Company:Nameable)",
            &Default::default(),
        )
        .expect("a concrete fragment member may satisfy the expanded endpoint cardinality");

    let connection = fragment_connection();
    register_semantic_constraints(
        &connection,
        "fragments",
        &SemanticConstraintRegistration {
            cardinalities: vec![
                SemanticRelationshipCardinality {
                    relationship_type: "WORKS_AT".to_owned(),
                    endpoint: SemanticEndpoint::Start,
                    minimum: 1,
                    maximum: Some(1),
                },
                SemanticRelationshipCardinality {
                    relationship_type: "WORKS_AT".to_owned(),
                    endpoint: SemanticEndpoint::End,
                    minimum: 1,
                    maximum: Some(1),
                },
            ],
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register cardinality on both physical endpoint sources");
    let session = turso_graph_frontend::Connection::open(connection.clone(), "fragments")
        .expect("open multi-source cardinality graph");
    session
        .execute(
            "CREATE (:Person:Nameable {displayName: 'Ada', age: 36})\
             -[:WORKS_AT]->(:Company:Nameable {displayName: 'Turso'})",
            &Default::default(),
        )
        .expect("colliding source-local identities satisfy their own endpoint constraints");
    let ids = connection
        .prepare("SELECT people.id, companies.id FROM people CROSS JOIN companies")
        .and_then(|mut statement| statement.run_collect_rows())
        .expect("read source-local identities");
    assert!(matches!(
        ids.as_slice(),
        [row] if matches!(
            row.as_slice(),
            [
                turso_graph_frontend::Value::Numeric(
                    turso_graph_frontend::Numeric::Integer(1)
                ),
                turso_graph_frontend::Value::Numeric(
                    turso_graph_frontend::Numeric::Integer(1)
                )
            ]
        )
    ));
    let deletion = session
        .execute(
            "MATCH (:Person)-[r:WORKS_AT]->(:Company) DELETE r",
            &Default::default(),
        )
        .expect_err("both source-qualified endpoint minimums remain active");
    assert_mutation_constraint_violation(deletion);
}

#[test]
fn constraints_resolve_fragment_contributed_properties_through_concrete_owners() {
    let connection = fragment_connection();
    register_semantic_constraints(
        &connection,
        "fragments",
        &SemanticConstraintRegistration {
            required: vec![SemanticRequiredProperty {
                owner: "Person".to_owned(),
                property: "displayName".to_owned(),
            }],
            values: vec![SemanticPropertyValueConstraint {
                owner: "Person".to_owned(),
                property: "displayName".to_owned(),
                predicate: SemanticValuePredicate::Regex {
                    pattern: "^[A-Z]".to_owned(),
                },
            }],
            ..SemanticConstraintRegistration::default()
        },
    )
    .expect("register constraint on fragment-contributed ownership");
    let session = turso_graph_frontend::Connection::open(connection, "fragments")
        .expect("open constrained fragment graph");
    let missing = session
        .execute("CREATE (:Person {age: 36})", &Default::default())
        .expect_err("required contributed property is enforced");
    assert!(missing.to_string().contains("is required"), "{missing}");
    session
        .execute(
            "CREATE (:Person:Nameable {displayName: 'Ada', age: 36})",
            &Default::default(),
        )
        .expect("valid contributed property");
    assert_eq!(
        session
            .query(
                "MATCH (n:Nameable) RETURN n.displayName",
                &Default::default()
            )
            .expect("fragment scan after constrained write"),
        vec![vec![turso_graph_frontend::Value::Text("Ada".into())]]
    );
}
