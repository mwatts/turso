use std::sync::Arc;

use turso_graph_frontend::core::{Database, MemoryIO, SqliteDialect};
use turso_graph_frontend::{
    load_registered_graph, load_semantic_snapshot, register_graph, register_semantic_schema,
    CatalogEntity, GraphCatalogSnapshot, GraphRegistration, NodeSourceRegistration,
    PropertyResolution, RelationalCatalogSnapshot, RelationshipSourceRegistration, SchemaCatalog,
    SemanticCatalogError, SemanticNodeType, SemanticProperty, SemanticRelationshipType,
    SemanticSchemaRegistration,
};

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
                 birth_year INTEGER\
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
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "edges_src".to_owned(),
                table: "tbl_edges".to_owned(),
                identity_column: "pk".to_owned(),
                start_column: "a".to_owned(),
                end_column: "b".to_owned(),
                start_node_source: "people_src".to_owned(),
                end_node_source: "people_src".to_owned(),
            }],
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
        relationship_types: vec![SemanticRelationshipType {
            name: "TRADES_WITH".to_owned(),
            source: "edges_src".to_owned(),
            start: vec!["Customer".to_owned()],
            end: vec!["Supplier".to_owned()],
            properties: vec![SemanticProperty {
                name: "since".to_owned(),
                column: "since".to_owned(),
            }],
        }],
    }
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
    let connection = connection();
    registered_graph(&connection);
    register_semantic_schema(&connection, "social", &semantic_registration()).expect("register");
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
    let second = load_semantic_snapshot(&connection, &graph)
        .expect("reload")
        .expect("semantic mode");
    assert_eq!(
        second.node_type("Customer").expect("customer").type_id,
        customer_id
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
fn create_with_multiple_labels_is_rejected_before_fragment_polymorphism() {
    let session = semantic_session();
    let error = session
        .execute("CREATE (n:Customer:Supplier)", &Default::default())
        .expect_err("reject multiple semantic labels");

    assert!(
        error.to_string().contains("multiple semantic labels"),
        "{error}"
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
            .contains("not allowed as the start endpoint"),
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
    assert!(error.to_string().contains("endpoint"), "{error}");
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
