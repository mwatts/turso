use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use thiserror::Error;
use turso_core::{Connection, Numeric, Value};
use turso_graph_ir::{GraphId, NodeId, RelationshipId, RelationshipTypeId, SourceTableId};
use turso_graph_runtime::{BuildLimits, Cancellation, EdgeInput, Graph, RuntimeError};

use crate::{load_registered_graph, CatalogError, GRAPH_CATALOG_VERSION};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SourceIdentity {
    Integer(i64),
    Real(u64),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeCoordinate {
    pub source: SourceTableId,
    pub identity: SourceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipCoordinate {
    pub source: SourceTableId,
    pub identity: SourceIdentity,
    pub relationship_type: RelationshipTypeId,
}

pub struct TraversalSnapshot {
    graph_id: GraphId,
    graph_name: String,
    catalog_version: u64,
    source_generation: u64,
    graph: Graph,
    nodes: Vec<NodeCoordinate>,
    relationships: Vec<RelationshipCoordinate>,
}

impl TraversalSnapshot {
    pub const fn graph_id(&self) -> GraphId {
        self.graph_id
    }

    pub fn graph_name(&self) -> &str {
        &self.graph_name
    }

    pub const fn catalog_version(&self) -> u64 {
        self.catalog_version
    }

    pub const fn source_generation(&self) -> u64 {
        self.source_generation
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn node(&self, id: NodeId) -> Option<&NodeCoordinate> {
        usize::try_from(id.get() - 1)
            .ok()
            .and_then(|index| self.nodes.get(index))
    }

    pub fn relationship(&self, id: RelationshipId) -> Option<&RelationshipCoordinate> {
        usize::try_from(id.get() - 1)
            .ok()
            .and_then(|index| self.relationships.get(index))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishOutcome {
    Published {
        replaced: bool,
        generation: u64,
    },
    Stale {
        built_generation: u64,
        current_generation: u64,
    },
}

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error("{kind} identity in source {source_id} has unsupported or null SQL value")]
    InvalidSourceIdentity {
        kind: &'static str,
        source_id: SourceTableId,
    },
    #[error("duplicate {kind} identity {identity:?} in source {source_id}")]
    DuplicateSourceIdentity {
        kind: &'static str,
        source_id: SourceTableId,
        identity: SourceIdentity,
    },
    #[error(
        "relationship {relationship:?} in source {relationship_source} references missing {role} identity {identity:?} in node source {node_source}"
    )]
    MissingEndpoint {
        relationship_source: SourceTableId,
        relationship: SourceIdentity,
        role: &'static str,
        node_source: SourceTableId,
        identity: SourceIdentity,
    },
    #[error("snapshot contains too many {0} identities")]
    TooManyIdentities(&'static str),
    #[error("snapshot build database operation failed: {0}")]
    Database(#[from] turso_core::LimboError),
    #[error("snapshot build failed and rollback also failed: {cause}; rollback: {rollback}")]
    RollbackFailed {
        cause: Box<SnapshotError>,
        rollback: turso_core::LimboError,
    },
    #[error("snapshot store lock is poisoned")]
    StorePoisoned,
}

#[derive(Default)]
pub struct SnapshotStore {
    snapshots: RwLock<HashMap<GraphId, Arc<TraversalSnapshot>>>,
}

impl SnapshotStore {
    pub fn get(&self, graph: GraphId) -> Result<Option<Arc<TraversalSnapshot>>, SnapshotError> {
        Ok(self
            .snapshots
            .read()
            .map_err(|_| SnapshotError::StorePoisoned)?
            .get(&graph)
            .cloned())
    }

    pub fn get_current(
        &self,
        connection: &Arc<Connection>,
        graph_name: &str,
    ) -> Result<Option<Arc<TraversalSnapshot>>, SnapshotError> {
        let registered = load_registered_graph(connection, graph_name)?;
        Ok(self.get(registered.id)?.filter(|snapshot| {
            snapshot.catalog_version == GRAPH_CATALOG_VERSION
                && snapshot.source_generation == registered.generation
        }))
    }

    pub fn publish_if_current(
        &self,
        connection: &Arc<Connection>,
        snapshot: TraversalSnapshot,
    ) -> Result<PublishOutcome, SnapshotError> {
        let current = load_registered_graph(connection, snapshot.graph_name())?;
        if current.id != snapshot.graph_id
            || snapshot.catalog_version != GRAPH_CATALOG_VERSION
            || current.generation != snapshot.source_generation
        {
            return Ok(PublishOutcome::Stale {
                built_generation: snapshot.source_generation,
                current_generation: current.generation,
            });
        }
        let generation = snapshot.source_generation;
        let mut snapshots = self
            .snapshots
            .write()
            .map_err(|_| SnapshotError::StorePoisoned)?;
        let replaced = snapshots
            .insert(snapshot.graph_id, Arc::new(snapshot))
            .is_some();
        Ok(PublishOutcome::Published {
            replaced,
            generation,
        })
    }

    pub fn refresh(
        &self,
        connection: &Arc<Connection>,
        graph_name: &str,
        limits: BuildLimits,
        cancellation: &dyn Cancellation,
    ) -> Result<PublishOutcome, SnapshotError> {
        let snapshot = build_traversal_snapshot(connection, graph_name, limits, cancellation)?;
        self.publish_if_current(connection, snapshot)
    }
}

pub fn build_traversal_snapshot(
    connection: &Arc<Connection>,
    graph_name: &str,
    limits: BuildLimits,
    cancellation: &dyn Cancellation,
) -> Result<TraversalSnapshot, SnapshotError> {
    connection.execute("BEGIN")?;
    let result = build_in_transaction(connection, graph_name, limits, cancellation);
    match result {
        Ok(snapshot) => match connection.execute("COMMIT") {
            Ok(()) => Ok(snapshot),
            Err(cause) => match connection.execute("ROLLBACK") {
                Ok(()) => Err(SnapshotError::Database(cause)),
                Err(rollback) => Err(SnapshotError::RollbackFailed {
                    cause: Box::new(SnapshotError::Database(cause)),
                    rollback,
                }),
            },
        },
        Err(cause) => match connection.execute("ROLLBACK") {
            Ok(()) => Err(cause),
            Err(rollback) => Err(SnapshotError::RollbackFailed {
                cause: Box::new(cause),
                rollback,
            }),
        },
    }
}

fn build_in_transaction(
    connection: &Arc<Connection>,
    graph_name: &str,
    limits: BuildLimits,
    cancellation: &dyn Cancellation,
) -> Result<TraversalSnapshot, SnapshotError> {
    check_cancelled(cancellation)?;
    let registered = load_registered_graph(connection, graph_name)?;
    let mut node_coordinates = Vec::new();
    let mut node_ids = HashMap::new();

    for source in &registered.node_sources {
        check_cancelled(cancellation)?;
        let rows = query_rows_cancellable(
            connection,
            &format!(
                "SELECT {} FROM {} ORDER BY {}",
                quote_identifier(&source.identity_column),
                quote_identifier(&source.table),
                quote_identifier(&source.identity_column)
            ),
            cancellation,
        )?;
        for row in rows {
            let identity = source_identity(row.first(), "node", source.id)?;
            let id = next_node_id(node_coordinates.len())?;
            if node_ids.insert((source.id, identity.clone()), id).is_some() {
                return Err(SnapshotError::DuplicateSourceIdentity {
                    kind: "node",
                    source_id: source.id,
                    identity,
                });
            }
            node_coordinates.push(NodeCoordinate {
                source: source.id,
                identity,
            });
        }
    }

    let mut relationship_coordinates = Vec::new();
    let mut relationship_ids = HashMap::new();
    let mut edges = Vec::new();
    for (type_index, source) in registered.relationship_sources.iter().enumerate() {
        check_cancelled(cancellation)?;
        let relationship_type = next_relationship_type(type_index)?;
        let rows = query_rows_cancellable(
            connection,
            &format!(
                "SELECT {}, {}, {} FROM {} ORDER BY {}",
                quote_identifier(&source.identity_column),
                quote_identifier(&source.start_column),
                quote_identifier(&source.end_column),
                quote_identifier(&source.table),
                quote_identifier(&source.identity_column)
            ),
            cancellation,
        )?;
        for row in rows {
            let identity = source_identity(row.first(), "relationship", source.id)?;
            let relationship = next_relationship_id(relationship_coordinates.len())?;
            if relationship_ids
                .insert((source.id, identity.clone()), relationship)
                .is_some()
            {
                return Err(SnapshotError::DuplicateSourceIdentity {
                    kind: "relationship",
                    source_id: source.id,
                    identity,
                });
            }
            let start_identity = source_identity(row.get(1), "endpoint", source.start_node_source)?;
            let end_identity = source_identity(row.get(2), "endpoint", source.end_node_source)?;
            let start = node_ids
                .get(&(source.start_node_source, start_identity.clone()))
                .copied()
                .ok_or_else(|| SnapshotError::MissingEndpoint {
                    relationship_source: source.id,
                    relationship: identity.clone(),
                    role: "start",
                    node_source: source.start_node_source,
                    identity: start_identity,
                })?;
            let end = node_ids
                .get(&(source.end_node_source, end_identity.clone()))
                .copied()
                .ok_or_else(|| SnapshotError::MissingEndpoint {
                    relationship_source: source.id,
                    relationship: identity.clone(),
                    role: "end",
                    node_source: source.end_node_source,
                    identity: end_identity,
                })?;
            relationship_coordinates.push(RelationshipCoordinate {
                source: source.id,
                identity,
                relationship_type,
            });
            edges.push(EdgeInput {
                relationship,
                source: start,
                target: end,
                relationship_type,
                weight: None,
            });
        }
    }

    check_cancelled(cancellation)?;
    let graph_node_ids = (0..node_coordinates.len())
        .map(next_node_id)
        .collect::<Result<Vec<_>, _>>()?;
    let graph = Graph::build_cancellable(graph_node_ids, edges, limits, cancellation)?;
    Ok(TraversalSnapshot {
        graph_id: registered.id,
        graph_name: registered.name,
        catalog_version: GRAPH_CATALOG_VERSION,
        source_generation: registered.generation,
        graph,
        nodes: node_coordinates,
        relationships: relationship_coordinates,
    })
}

fn query_rows_cancellable(
    connection: &Arc<Connection>,
    sql: &str,
    cancellation: &dyn Cancellation,
) -> Result<Vec<Vec<Value>>, SnapshotError> {
    let mut rows = Vec::new();
    let result = connection.prepare(sql)?.run_with_row_callback(|row| {
        if cancellation.is_cancelled() {
            return Err(turso_core::LimboError::Interrupt);
        }
        rows.push(row.get_values().cloned().collect());
        Ok(())
    });
    match result {
        Err(turso_core::LimboError::Interrupt) if cancellation.is_cancelled() => {
            Err(RuntimeError::Cancelled.into())
        }
        Err(error) => Err(error.into()),
        Ok(()) => Ok(rows),
    }
}

fn source_identity(
    value: Option<&Value>,
    kind: &'static str,
    source: SourceTableId,
) -> Result<SourceIdentity, SnapshotError> {
    match value {
        Some(Value::Numeric(Numeric::Integer(value))) => Ok(SourceIdentity::Integer(*value)),
        Some(Value::Numeric(Numeric::Float(value))) => {
            let value = f64::from(*value);
            Ok(SourceIdentity::Real(if value == 0.0 {
                0
            } else {
                value.to_bits()
            }))
        }
        Some(Value::Text(value)) => Ok(SourceIdentity::Text(value.as_str().to_owned())),
        Some(Value::Blob(value)) => Ok(SourceIdentity::Blob(value.to_vec())),
        Some(Value::Null) | None => Err(SnapshotError::InvalidSourceIdentity {
            kind,
            source_id: source,
        }),
    }
}

fn next_node_id(index: usize) -> Result<NodeId, SnapshotError> {
    u64::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(1))
        .and_then(|value| NodeId::new(value).ok())
        .ok_or(SnapshotError::TooManyIdentities("node"))
}

fn next_relationship_id(index: usize) -> Result<RelationshipId, SnapshotError> {
    u64::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(1))
        .and_then(|value| RelationshipId::new(value).ok())
        .ok_or(SnapshotError::TooManyIdentities("relationship"))
}

fn next_relationship_type(index: usize) -> Result<RelationshipTypeId, SnapshotError> {
    u32::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(1))
        .and_then(|value| RelationshipTypeId::new(value).ok())
        .ok_or(SnapshotError::TooManyIdentities("relationship type"))
}

fn check_cancelled(cancellation: &dyn Cancellation) -> Result<(), SnapshotError> {
    if cancellation.is_cancelled() {
        Err(RuntimeError::Cancelled.into())
    } else {
        Ok(())
    }
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        register_graph, GraphRegistration, NodeSourceRegistration, RegisteredGraph,
        RelationshipSourceRegistration,
    };
    use turso_core::{Database, MemoryIO, SqliteDialect};
    use turso_graph_ir::Direction;
    use turso_graph_runtime::{
        traverse, LimitKind, TraversalLimits, TraversalOrder, TraversalRequest, Uniqueness,
    };

    fn connection(name: &str) -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file(io, name, Arc::new(SqliteDialect))
            .unwrap()
            .connect()
            .unwrap()
    }

    fn register(connection: &Arc<Connection>) -> RegisteredGraph {
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
                 CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
            )
            .unwrap();
        register_graph(
            connection,
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
        .unwrap()
    }

    #[test]
    fn empty_snapshot_publishes_and_replacement_is_atomic() {
        let connection = connection(":memory:snapshot-empty");
        let registered = register(&connection);
        let store = SnapshotStore::default();
        let outcome = store
            .refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            )
            .unwrap();
        assert_eq!(
            outcome,
            PublishOutcome::Published {
                replaced: false,
                generation: 0
            }
        );
        assert_eq!(
            store
                .get(registered.id)
                .unwrap()
                .unwrap()
                .graph()
                .node_count(),
            0
        );

        connection
            .execute("INSERT INTO people VALUES (2, 'B')")
            .unwrap();
        let outcome = store
            .refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            )
            .unwrap();
        assert_eq!(
            outcome,
            PublishOutcome::Published {
                replaced: true,
                generation: 1
            }
        );
        let snapshot = store.get_current(&connection, "social").unwrap().unwrap();
        assert_eq!(snapshot.graph().node_count(), 1);
        assert_eq!(
            snapshot.node(NodeId::new(1).unwrap()),
            Some(&NodeCoordinate {
                source: registered.node_sources[0].id,
                identity: SourceIdentity::Integer(2)
            })
        );
    }

    #[test]
    fn populated_snapshot_preserves_coordinates_and_supports_traversal() {
        let connection = connection(":memory:snapshot-populated");
        let registered = register(&connection);
        connection
            .execute(
                "INSERT INTO people VALUES (30, 'C'), (10, 'A'), (20, 'B'); \
                 INSERT INTO relationships VALUES (200, 20, 30), (100, 10, 20)",
            )
            .unwrap();

        let snapshot = build_traversal_snapshot(
            &connection,
            "social",
            BuildLimits::default(),
            &turso_graph_runtime::NeverCancelled,
        )
        .unwrap();

        assert_eq!(snapshot.graph().node_count(), 3);
        assert_eq!(snapshot.graph().edge_count(), 2);
        assert_eq!(
            snapshot.node(NodeId::new(1).unwrap()),
            Some(&NodeCoordinate {
                source: registered.node_sources[0].id,
                identity: SourceIdentity::Integer(10),
            })
        );
        assert_eq!(
            snapshot.relationship(RelationshipId::new(2).unwrap()),
            Some(&RelationshipCoordinate {
                source: registered.relationship_sources[0].id,
                identity: SourceIdentity::Integer(200),
                relationship_type: RelationshipTypeId::new(1).unwrap(),
            })
        );

        let paths = traverse(
            snapshot.graph(),
            &TraversalRequest {
                start: NodeId::new(1).unwrap(),
                direction: Direction::Outgoing,
                relationship_types: vec![RelationshipTypeId::new(1).unwrap()],
                min_hops: 2,
                max_hops: 2,
                uniqueness: Uniqueness::Trail,
                order: TraversalOrder::BreadthFirst,
            },
            TraversalLimits::default(),
            &turso_graph_runtime::NeverCancelled,
        )
        .unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths[0].nodes,
            vec![
                NodeId::new(1).unwrap(),
                NodeId::new(2).unwrap(),
                NodeId::new(3).unwrap(),
            ]
        );
        assert_eq!(
            paths[0].relationships,
            vec![
                RelationshipId::new(1).unwrap(),
                RelationshipId::new(2).unwrap(),
            ]
        );
    }

    #[test]
    fn stale_candidate_is_discarded_after_concurrent_invalidation() {
        let connection = connection(":memory:snapshot-stale");
        let registered = register(&connection);
        let candidate = build_traversal_snapshot(
            &connection,
            "social",
            BuildLimits::default(),
            &turso_graph_runtime::NeverCancelled,
        )
        .unwrap();
        connection
            .execute("INSERT INTO people VALUES (1, 'A')")
            .unwrap();
        let store = SnapshotStore::default();
        assert_eq!(
            store.publish_if_current(&connection, candidate).unwrap(),
            PublishOutcome::Stale {
                built_generation: 0,
                current_generation: 1
            }
        );
        assert!(store.get(registered.id).unwrap().is_none());
    }

    #[test]
    fn missing_endpoint_and_duplicate_identity_fail_without_publication() {
        let connection = connection(":memory:snapshot-invalid");
        let registered = register(&connection);
        connection
            .execute("INSERT INTO people VALUES (1, 'A')")
            .unwrap();
        connection
            .execute("INSERT INTO relationships VALUES (10, 1, 999)")
            .unwrap();
        assert!(matches!(
            build_traversal_snapshot(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            ),
            Err(SnapshotError::MissingEndpoint { role: "end", .. })
        ));

        connection.execute("DELETE FROM relationships").unwrap();
        connection
            .execute("CREATE TABLE aliases(id TEXT NOT NULL); CREATE UNIQUE INDEX aliases_id ON aliases(id)")
            .unwrap();
        register_graph(
            &connection,
            &GraphRegistration {
                name: "duplicates".to_owned(),
                node_sources: vec![NodeSourceRegistration {
                    name: "Alias".to_owned(),
                    table: "aliases".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![],
            },
        )
        .unwrap();
        connection.execute("DROP INDEX aliases_id").unwrap();
        connection
            .execute("INSERT INTO aliases VALUES ('x'), ('x')")
            .unwrap();
        assert!(matches!(
            build_traversal_snapshot(
                &connection,
                "duplicates",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            ),
            Err(SnapshotError::DuplicateSourceIdentity { kind: "node", .. })
        ));
        assert!(SnapshotStore::default()
            .get(registered.id)
            .unwrap()
            .is_none());
    }

    struct Cancelled;

    impl Cancellation for Cancelled {
        fn is_cancelled(&self) -> bool {
            true
        }
    }

    #[test]
    fn cancelled_and_failed_refreshes_leave_the_previous_snapshot_unchanged() {
        let connection = connection(":memory:snapshot-cancel");
        let registered = register(&connection);
        connection
            .execute("INSERT INTO people VALUES (1, 'A')")
            .unwrap();
        let store = SnapshotStore::default();
        store
            .refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            )
            .unwrap();
        let original = store.get(registered.id).unwrap().unwrap();

        assert!(matches!(
            store.refresh(&connection, "social", BuildLimits::default(), &Cancelled),
            Err(SnapshotError::Runtime(RuntimeError::Cancelled))
        ));
        assert!(Arc::ptr_eq(
            &original,
            &store.get(registered.id).unwrap().unwrap()
        ));

        connection
            .execute("INSERT INTO relationships VALUES (10, 1, 999)")
            .unwrap();
        assert!(store
            .refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            )
            .is_err());
        assert!(Arc::ptr_eq(
            &original,
            &store.get(registered.id).unwrap().unwrap()
        ));
        connection
            .execute("INSERT INTO people VALUES (2, 'B')")
            .unwrap();
    }

    #[test]
    fn resource_limited_refresh_leaves_previous_snapshot_unpublished() {
        let connection = connection(":memory:snapshot-limited");
        let registered = register(&connection);
        connection
            .execute("INSERT INTO people VALUES (1, 'A')")
            .unwrap();
        let store = SnapshotStore::default();
        store
            .refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            )
            .unwrap();
        let original = store.get(registered.id).unwrap().unwrap();

        connection
            .execute("INSERT INTO people VALUES (2, 'B')")
            .unwrap();
        let limits = BuildLimits {
            max_nodes: 1,
            ..BuildLimits::default()
        };
        assert!(matches!(
            store.refresh(
                &connection,
                "social",
                limits,
                &turso_graph_runtime::NeverCancelled,
            ),
            Err(SnapshotError::Runtime(RuntimeError::LimitExceeded {
                kind: LimitKind::Nodes,
                limit: 1,
            }))
        ));
        assert!(Arc::ptr_eq(
            &original,
            &store.get(registered.id).unwrap().unwrap()
        ));
        assert!(store.get_current(&connection, "social").unwrap().is_none());
    }
}
