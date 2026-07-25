//! Shared `GraphConnection` fixture for `graph_frontend` integration tests.
//!
//! Mirrors the "social" graph (`Person` nodes over `people`, `KNOWS`
//! relationships over `relationships`) `session.rs`'s own unit tests
//! install, but backed by `SchemaCatalog` rather than a private catalog
//! stub so this can run as an external integration-test crate.

use std::sync::Arc;

use turso_core::{Connection, Database, DatabaseOpts, MemoryIO, OpenOptions, SqliteDialect};
use turso_graph_frontend::{
    register_graph, GraphCompilationCatalog, GraphConnection, GraphRegistration,
    NodeSourceRegistration, ParameterTypes, Parameters, RelationshipSourceRegistration,
    SchemaCatalog, SnapshotStore,
};
use turso_graph_runtime::{BuildLimits, NeverCancelled};

/// Installs a `GraphConnection` over a fresh in-memory "social" graph:
/// `Person` nodes (`people(id, name, age)`), `KNOWS` relationships
/// (`relationships(id, src, dst)`), seeded with two people. Returns the
/// `Arc<Database>` alongside the session so callers can open further
/// connections onto the same graph (see [`second_connection`]).
pub fn social_graph_connection() -> (Arc<Database>, GraphConnection) {
    social_graph_connection_with_options(DatabaseOpts::default())
}

#[cfg(feature = "fts")]
#[allow(dead_code)] // This file is also compiled as its own integration-test crate.
pub fn social_graph_connection_with_fts() -> (Arc<Database>, GraphConnection) {
    social_graph_connection_with_options(DatabaseOpts::default().with_index_method(true))
}

fn social_graph_connection_with_options(opts: DatabaseOpts) -> (Arc<Database>, GraphConnection) {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open(
        io,
        ":memory:fixture-social",
        OpenOptions::new(Arc::new(SqliteDialect)).db_opts(opts),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
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
        connection,
        &registered,
        catalog,
        ParameterTypes::new(),
        shared_snapshots,
        BuildLimits::default(),
    )
    .expect("install graph session");
    // Seeded through Cypher CREATE (not a raw INSERT) so the label junction
    // table `SchemaCatalog` relies on is populated the same way production
    // writes populate it.
    session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada', age: 36}), \
             (:Person {id: 2, name: 'Grace', age: 85})",
            &Parameters::new(),
        )
        .expect("seed people");
    (database, session)
}

/// A second connection onto the same underlying database as `database`, for
/// exercising session setup (like [`GraphConnection::open`]) that must not
/// depend on the connection that performed the original registration.
#[allow(dead_code)] // Shared fixture; not every integration crate calls this.
pub fn second_connection(database: &Arc<Database>) -> Arc<Connection> {
    database.connect().expect("connect")
}
