//! Shared `GraphConnection` fixture for `graph_frontend` integration tests.
//!
//! Mirrors the "social" graph (`Person` nodes over `people`, `KNOWS`
//! relationships over `relationships`) `session.rs`'s own unit tests
//! install, but backed by `SchemaCatalog` rather than a private catalog
//! stub so this can run as an external integration-test crate.

use std::sync::Arc;

use turso_core::{Connection, Database, MemoryIO, SqliteDialect};
use turso_graph_frontend::{
    register_graph, GraphCompilationCatalog, GraphConnection, GraphRegistration,
    NodeSourceRegistration, ParameterTypes, RelationshipSourceRegistration, SchemaCatalog,
    SnapshotStore,
};
use turso_graph_runtime::{BuildLimits, NeverCancelled};

/// Installs a `GraphConnection` over a fresh in-memory "social" graph:
/// `Person` nodes (`people(id, name, age)`), `KNOWS` relationships
/// (`relationships(id, src, dst)`).
pub fn social_graph_connection() -> (Arc<Connection>, GraphConnection) {
    let io = Arc::new(MemoryIO::new());
    let connection = Database::open_file(io, ":memory:fixture-social", Arc::new(SqliteDialect))
        .expect("open database")
        .connect()
        .expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
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
    let catalog: Arc<dyn GraphCompilationCatalog> =
        Arc::new(SchemaCatalog::new(connection.clone(), registered.clone()));
    let shared_snapshots = Arc::new(SnapshotStore::default());
    shared_snapshots
        .refresh(
            &connection,
            &registered.name,
            BuildLimits::default(),
            &NeverCancelled,
        )
        .expect("build initial traversal snapshot");
    let session = GraphConnection::install(
        connection.clone(),
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session");
    (connection, session)
}
