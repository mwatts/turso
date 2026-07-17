use std::sync::Arc;
use std::time::Instant;

use turso_core::{Database, MemoryIO, SqliteDialect};
use turso_graph_frontend::{
    register_graph, GraphRegistration, NodeSourceRegistration, RelationshipSourceRegistration,
    SnapshotStore,
};
use turso_graph_runtime::{BuildLimits, NeverCancelled};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sizes = std::env::args()
        .skip(1)
        .map(|value| value.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()?;
    let sizes = if sizes.is_empty() {
        vec![1_000, 10_000, 100_000]
    } else {
        sizes
    };

    println!(
        "nodes,relationships,startup_ns,build_ns,refresh_ns,retained_bytes,peak_build_bytes,durable_write_bytes"
    );
    for nodes in sizes {
        profile(nodes)?;
    }
    Ok(())
}

fn profile(nodes: u64) -> Result<(), Box<dyn std::error::Error>> {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        &format!(":memory:graph-snapshot-profile-{nodes}"),
        Arc::new(SqliteDialect),
    )?;
    let connection = database.connect()?;
    connection.execute(
        "CREATE TABLE nodes(id INTEGER PRIMARY KEY, payload TEXT); \
         CREATE TABLE edges(id INTEGER PRIMARY KEY, src INTEGER NOT NULL, dst INTEGER NOT NULL)",
    )?;
    insert_fixture(&connection, nodes)?;
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "profile".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Node".to_owned(),
                table: "nodes".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "NEXT".to_owned(),
                table: "edges".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
                start_node_source: "Node".to_owned(),
                end_node_source: "Node".to_owned(),
            }],
        },
    )?;

    let startup = Instant::now();
    let store = SnapshotStore::default();
    let startup_ns = startup.elapsed().as_nanos();
    store.refresh(
        &connection,
        "profile",
        BuildLimits::default(),
        &NeverCancelled,
    )?;
    let snapshot = store
        .get(registered.id)?
        .expect("successful refresh publishes a snapshot");

    connection.execute("UPDATE nodes SET payload = 'changed' WHERE id = 1")?;
    let refresh = Instant::now();
    store.refresh(
        &connection,
        "profile",
        BuildLimits::default(),
        &NeverCancelled,
    )?;
    let refresh_ns = refresh.elapsed().as_nanos();

    println!(
        "{},{},{},{},{},{},{},0",
        snapshot.graph().node_count(),
        snapshot.graph().edge_count(),
        startup_ns,
        snapshot.build_elapsed().as_nanos(),
        refresh_ns,
        snapshot.estimated_heap_bytes(),
        snapshot.estimated_peak_build_bytes(),
    );
    Ok(())
}

fn insert_fixture(
    connection: &Arc<turso_core::Connection>,
    nodes: u64,
) -> Result<(), turso_core::LimboError> {
    const CHUNK_SIZE: u64 = 500;
    for start in (1..=nodes).step_by(CHUNK_SIZE as usize) {
        let end = nodes.min(start + CHUNK_SIZE - 1);
        let node_values = (start..=end)
            .map(|id| format!("({id}, 'node-{id}')"))
            .collect::<Vec<_>>()
            .join(",");
        connection.execute(format!("INSERT INTO nodes VALUES {node_values}"))?;

        let edge_values = (start.max(2)..=end)
            .map(|id| format!("({}, {}, {})", id - 1, id - 1, id))
            .collect::<Vec<_>>();
        if !edge_values.is_empty() {
            connection.execute(format!(
                "INSERT INTO edges VALUES {}",
                edge_values.join(",")
            ))?;
        }
    }
    Ok(())
}
