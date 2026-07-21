use std::collections::HashMap;
use std::mem::size_of;
use std::sync::{Arc, RwLock, Weak};
use std::time::{Duration, Instant};

use thiserror::Error;
use turso_core::{Connection, Numeric, Value};
use turso_graph_ir::{GraphId, NodeId, RelationshipId, RelationshipTypeId, SourceTableId};
use turso_graph_runtime::{BuildLimits, Cancellation, EdgeInput, Graph, RuntimeError};

use crate::{load_registered_graph, CatalogError, GRAPH_CATALOG_VERSION};

const VISIBLE_SNAPSHOT_SAVEPOINT: &str = "__turso_graph_visible_snapshot";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SourceIdentity {
    Integer(i64),
    Real(u64),
    Text(String),
    Blob(Vec<u8>),
}

impl SourceIdentity {
    /// Total canonical bit encoding for REAL identity values used as hash
    /// keys: +0.0 and -0.0 collapse to one key, and every NaN payload
    /// collapses to the canonical quiet NaN so equal-comparing (or
    /// all-incomparable) floats cannot produce distinct identities.
    pub fn real(value: f64) -> Self {
        Self::Real(if value == 0.0 {
            0
        } else if value.is_nan() {
            f64::NAN.to_bits()
        } else {
            value.to_bits()
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    node_ids: HashMap<NodeCoordinate, NodeId>,
    relationships: Vec<RelationshipCoordinate>,
    build_elapsed: Duration,
    estimated_heap_bytes: u64,
    estimated_peak_build_bytes: u64,
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

    pub fn node_id(&self, source: SourceTableId, identity: &SourceIdentity) -> Option<NodeId> {
        self.node_ids
            .get(&NodeCoordinate {
                source,
                identity: identity.clone(),
            })
            .copied()
    }

    pub fn relationship(&self, id: RelationshipId) -> Option<&RelationshipCoordinate> {
        usize::try_from(id.get() - 1)
            .ok()
            .and_then(|index| self.relationships.get(index))
    }

    pub const fn build_elapsed(&self) -> Duration {
        self.build_elapsed
    }

    pub const fn estimated_heap_bytes(&self) -> u64 {
        self.estimated_heap_bytes
    }

    pub const fn estimated_peak_build_bytes(&self) -> u64 {
        self.estimated_peak_build_bytes
    }

    pub fn metadata(&self) -> SnapshotMetadata {
        SnapshotMetadata {
            graph_id: self.graph_id,
            catalog_version: self.catalog_version,
            source_generation: self.source_generation,
            node_count: self.graph.node_count(),
            relationship_count: self.graph.edge_count(),
            build_elapsed: self.build_elapsed,
            estimated_heap_bytes: self.estimated_heap_bytes,
            estimated_peak_build_bytes: self.estimated_peak_build_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotMetadata {
    pub graph_id: GraphId,
    pub catalog_version: u64,
    pub source_generation: u64,
    pub node_count: usize,
    pub relationship_count: usize,
    pub build_elapsed: Duration,
    pub estimated_heap_bytes: u64,
    pub estimated_peak_build_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotStatus {
    Missing,
    Current(SnapshotMetadata),
    Stale {
        snapshot: SnapshotMetadata,
        current_catalog_version: u64,
        current_generation: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPersistenceMode {
    InMemoryRebuildOnDemand,
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
    #[error("shared snapshot refresh requires autocommit; it publishes committed state only")]
    RefreshInsideTransaction,
}

#[derive(Default)]
pub struct SnapshotStore {
    snapshots: RwLock<HashMap<GraphId, Arc<TraversalSnapshot>>>,
    session_overlays: RwLock<Vec<(Weak<Connection>, Weak<SessionSnapshotStore>)>>,
}

impl SnapshotStore {
    pub const fn persistence_mode(&self) -> SnapshotPersistenceMode {
        SnapshotPersistenceMode::InMemoryRebuildOnDemand
    }

    pub fn get(&self, graph: GraphId) -> Result<Option<Arc<TraversalSnapshot>>, SnapshotError> {
        Ok(self
            .snapshots
            .read()
            .map_err(|_| SnapshotError::StorePoisoned)?
            .get(&graph)
            .cloned())
    }

    pub fn register_session(
        &self,
        connection: &Arc<Connection>,
        snapshots: &Arc<SessionSnapshotStore>,
    ) -> Result<(), SnapshotError> {
        let mut sessions = self
            .session_overlays
            .write()
            .map_err(|_| SnapshotError::StorePoisoned)?;
        sessions.retain(|(connection, snapshots)| {
            connection.strong_count() > 0 && snapshots.strong_count() > 0
        });
        if let Some((_, registered)) = sessions.iter_mut().find(|(registered, _)| {
            registered
                .upgrade()
                .is_some_and(|registered| Arc::ptr_eq(&registered, connection))
        }) {
            *registered = Arc::downgrade(snapshots);
        } else {
            sessions.push((Arc::downgrade(connection), Arc::downgrade(snapshots)));
        }
        Ok(())
    }

    pub(crate) fn get_for_connection(
        &self,
        connection: &Arc<Connection>,
        graph: GraphId,
    ) -> Result<Option<Arc<TraversalSnapshot>>, SnapshotError> {
        let session = self
            .session_overlays
            .read()
            .map_err(|_| SnapshotError::StorePoisoned)?
            .iter()
            .find_map(|(registered, snapshots)| {
                registered
                    .upgrade()
                    .filter(|registered| Arc::ptr_eq(registered, connection))
                    .and_then(|_| snapshots.upgrade())
            });
        let snapshot = match session {
            Some(session) => session.get(graph),
            None => self.get(graph),
        }?;
        snapshot
            .map(|snapshot| {
                is_snapshot_current(connection, &snapshot)
                    .map(|current| current.then_some(snapshot))
            })
            .transpose()
            .map(Option::flatten)
    }

    pub fn status(
        &self,
        connection: &Arc<Connection>,
        graph_name: &str,
    ) -> Result<SnapshotStatus, SnapshotError> {
        let registered = load_registered_graph(connection, graph_name)?;
        let Some(snapshot) = self.get(registered.id)? else {
            return Ok(SnapshotStatus::Missing);
        };
        Ok(classify_snapshot(&snapshot, registered.generation))
    }

    pub fn discard(&self, graph: GraphId) -> Result<bool, SnapshotError> {
        Ok(self
            .snapshots
            .write()
            .map_err(|_| SnapshotError::StorePoisoned)?
            .remove(&graph)
            .is_some())
    }

    pub fn get_current(
        &self,
        connection: &Arc<Connection>,
        graph_name: &str,
    ) -> Result<Option<Arc<TraversalSnapshot>>, SnapshotError> {
        let registered = load_registered_graph(connection, graph_name)?;
        Ok(self.get(registered.id)?.filter(|snapshot| {
            matches!(
                classify_snapshot(snapshot, registered.generation),
                SnapshotStatus::Current(_)
            )
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

/// Connection-local snapshot overlay used for transaction-visible traversal.
///
/// Local candidates are never published into the shared derived-state store.
/// Rebuilding before a graph read makes rollback, savepoint rollback, and
/// commit invalidation correctness independent of engine transaction hooks.
pub struct SessionSnapshotStore {
    shared: Arc<SnapshotStore>,
    local: RwLock<HashMap<GraphId, Arc<TraversalSnapshot>>>,
}

impl SessionSnapshotStore {
    pub fn new(shared: Arc<SnapshotStore>) -> Self {
        Self {
            shared,
            local: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, graph: GraphId) -> Result<Option<Arc<TraversalSnapshot>>, SnapshotError> {
        if let Some(snapshot) = self
            .local
            .read()
            .map_err(|_| SnapshotError::StorePoisoned)?
            .get(&graph)
            .cloned()
        {
            return Ok(Some(snapshot));
        }
        self.shared.get(graph)
    }

    pub fn refresh_visible(
        &self,
        connection: &Arc<Connection>,
        graph_name: &str,
        limits: BuildLimits,
        cancellation: &dyn Cancellation,
    ) -> Result<Arc<TraversalSnapshot>, SnapshotError> {
        let snapshot = Arc::new(build_visible_traversal_snapshot(
            connection,
            graph_name,
            limits,
            cancellation,
        )?);
        self.local
            .write()
            .map_err(|_| SnapshotError::StorePoisoned)?
            .insert(snapshot.graph_id(), snapshot.clone());
        Ok(snapshot)
    }

    pub fn refresh_visible_if_stale(
        &self,
        connection: &Arc<Connection>,
        graph_name: &str,
        limits: BuildLimits,
        cancellation: &dyn Cancellation,
    ) -> Result<(Arc<TraversalSnapshot>, bool), SnapshotError> {
        check_cancelled(cancellation)?;
        let registered = load_registered_graph(connection, graph_name)?;
        if let Some(snapshot) = self.get(registered.id)? {
            if matches!(
                classify_snapshot(&snapshot, registered.generation),
                SnapshotStatus::Current(_)
            ) {
                return Ok((snapshot, false));
            }
        }
        self.refresh_visible(connection, graph_name, limits, cancellation)
            .map(|snapshot| (snapshot, true))
    }

    pub fn clear(&self) -> Result<(), SnapshotError> {
        self.local
            .write()
            .map_err(|_| SnapshotError::StorePoisoned)?
            .clear();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn poison_for_test(&self) {
        let _ = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let _guard = self.local.write().unwrap();
                    panic!("poison session snapshot store");
                })
                .join()
        });
        assert!(self.local.read().is_err());
    }
}

/// Build a snapshot of committed state for shared publication.
///
/// Requires autocommit: inside an open transaction this connection could
/// observe (and publish) uncommitted rows, so refuse instead of opening a
/// nested transaction. Use [`build_visible_traversal_snapshot`] for
/// connection-local reads inside a transaction.
pub fn build_traversal_snapshot(
    connection: &Arc<Connection>,
    graph_name: &str,
    limits: BuildLimits,
    cancellation: &dyn Cancellation,
) -> Result<TraversalSnapshot, SnapshotError> {
    if !connection.get_auto_commit() {
        return Err(SnapshotError::RefreshInsideTransaction);
    }
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

/// Build from rows visible to this connection without publishing the result.
/// A nested savepoint supplies one consistent read boundary in both autocommit
/// mode and an existing explicit transaction.
pub fn build_visible_traversal_snapshot(
    connection: &Arc<Connection>,
    graph_name: &str,
    limits: BuildLimits,
    cancellation: &dyn Cancellation,
) -> Result<TraversalSnapshot, SnapshotError> {
    connection.execute(format!("SAVEPOINT {VISIBLE_SNAPSHOT_SAVEPOINT}"))?;
    let result = build_in_transaction(connection, graph_name, limits, cancellation);
    match result {
        Ok(snapshot) => {
            connection.execute(format!("RELEASE {VISIBLE_SNAPSHOT_SAVEPOINT}"))?;
            Ok(snapshot)
        }
        Err(cause) => {
            let rollback = connection
                .execute(format!("ROLLBACK TO {VISIBLE_SNAPSHOT_SAVEPOINT}"))
                .and_then(|()| connection.execute(format!("RELEASE {VISIBLE_SNAPSHOT_SAVEPOINT}")));
            match rollback {
                Ok(()) => Err(cause),
                Err(rollback) => Err(SnapshotError::RollbackFailed {
                    cause: Box::new(cause),
                    rollback,
                }),
            }
        }
    }
}

fn build_in_transaction(
    connection: &Arc<Connection>,
    graph_name: &str,
    limits: BuildLimits,
    cancellation: &dyn Cancellation,
) -> Result<TraversalSnapshot, SnapshotError> {
    let started = Instant::now();
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
        let default_relationship_type = next_relationship_type(type_index)?;
        // Resolve each relationship's Cypher type through the junction and
        // registry so traversal filters see the identities the binder uses;
        // rows without a recorded type keep the source-index identity.
        let rows = query_rows_cancellable(
            connection,
            &format!(
                "SELECT r.{}, r.{}, r.{}, reg.id FROM {} AS r \
                 LEFT JOIN \"{}\" AS jt ON jt.relationship_id = r.{} \
                 LEFT JOIN \"{}\" AS reg ON reg.name = jt.type \
                 ORDER BY r.{}",
                quote_identifier(&source.identity_column),
                quote_identifier(&source.start_column),
                quote_identifier(&source.end_column),
                quote_identifier(&source.table),
                crate::catalog::relationship_types_table_name(registered.id),
                quote_identifier(&source.identity_column),
                crate::catalog::relationship_type_registry_table_name(registered.id),
                quote_identifier(&source.identity_column)
            ),
            cancellation,
        )?;
        for row in rows {
            let relationship_type = match row.get(3) {
                Some(turso_core::Value::Numeric(turso_core::Numeric::Integer(id))) => {
                    u32::try_from(*id)
                        .ok()
                        .and_then(|id| RelationshipTypeId::new(id).ok())
                        .unwrap_or(default_relationship_type)
                }
                _ => default_relationship_type,
            };
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
    let node_lookup = node_coordinates
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, coordinate)| Ok((coordinate, next_node_id(index)?)))
        .collect::<Result<HashMap<_, _>, SnapshotError>>()?;
    let estimated_heap_bytes =
        estimated_snapshot_heap_bytes(&graph, &node_coordinates, &relationship_coordinates);
    let estimated_peak_build_bytes = estimated_peak_build_bytes(
        estimated_heap_bytes,
        node_coordinates.len(),
        relationship_coordinates.len(),
    );
    Ok(TraversalSnapshot {
        graph_id: registered.id,
        graph_name: registered.name,
        catalog_version: GRAPH_CATALOG_VERSION,
        source_generation: registered.generation,
        graph,
        nodes: node_coordinates,
        node_ids: node_lookup,
        relationships: relationship_coordinates,
        build_elapsed: started.elapsed(),
        estimated_heap_bytes,
        estimated_peak_build_bytes,
    })
}

fn classify_snapshot(snapshot: &TraversalSnapshot, current_generation: u64) -> SnapshotStatus {
    let metadata = snapshot.metadata();
    if snapshot.catalog_version == GRAPH_CATALOG_VERSION
        && snapshot.source_generation == current_generation
    {
        SnapshotStatus::Current(metadata)
    } else {
        SnapshotStatus::Stale {
            snapshot: metadata,
            current_catalog_version: GRAPH_CATALOG_VERSION,
            current_generation,
        }
    }
}

fn is_snapshot_current(
    connection: &Arc<Connection>,
    snapshot: &TraversalSnapshot,
) -> Result<bool, SnapshotError> {
    let registered = load_registered_graph(connection, snapshot.graph_name())?;
    Ok(registered.id == snapshot.graph_id
        && matches!(
            classify_snapshot(snapshot, registered.generation),
            SnapshotStatus::Current(_)
        ))
}

fn estimated_snapshot_heap_bytes(
    graph: &Graph,
    nodes: &[NodeCoordinate],
    relationships: &[RelationshipCoordinate],
) -> u64 {
    let coordinate_bytes = nodes
        .iter()
        .map(|node| size_of::<NodeCoordinate>() + source_identity_heap_bytes(&node.identity))
        .chain(relationships.iter().map(|relationship| {
            size_of::<RelationshipCoordinate>() + source_identity_heap_bytes(&relationship.identity)
        }))
        .sum::<usize>();
    graph
        .estimated_heap_bytes()
        .saturating_add(coordinate_bytes as u64)
}

fn estimated_peak_build_bytes(
    retained_bytes: u64,
    node_count: usize,
    relationship_count: usize,
) -> u64 {
    let transient_bytes = node_count
        .saturating_mul(size_of::<NodeId>() + size_of::<NodeCoordinate>())
        .saturating_add(relationship_count.saturating_mul(size_of::<EdgeInput>()));
    retained_bytes.saturating_add(transient_bytes as u64)
}

fn source_identity_heap_bytes(identity: &SourceIdentity) -> usize {
    match identity {
        SourceIdentity::Text(value) => value.capacity(),
        SourceIdentity::Blob(value) => value.capacity(),
        SourceIdentity::Integer(_) | SourceIdentity::Real(_) => 0,
    }
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
        Some(Value::Numeric(Numeric::Float(value))) => Ok(SourceIdentity::real(f64::from(*value))),
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
    fn real_identity_encoding_is_total_and_canonical() {
        // Hash-key encoding must not split one logical identity into many:
        // +0.0/-0.0 collapse, and every NaN payload maps to one canonical
        // key (duplicate NaN identities then fail loudly as duplicates
        // instead of silently splitting).
        assert_eq!(SourceIdentity::real(0.0), SourceIdentity::real(-0.0));
        let payload_nan = f64::from_bits(0x7ff8_0000_0000_1234);
        assert!(payload_nan.is_nan());
        assert_eq!(
            SourceIdentity::real(f64::NAN),
            SourceIdentity::real(payload_nan)
        );
        assert_ne!(SourceIdentity::real(1.5), SourceIdentity::real(2.5));
        assert_eq!(
            SourceIdentity::real(1.5),
            SourceIdentity::Real(1.5f64.to_bits())
        );
    }

    #[test]
    fn shared_refresh_refuses_an_open_transaction() {
        // The shared store publishes committed state only. A refresh inside
        // an open transaction would either fail on the nested BEGIN or
        // publish rows the caller has not committed, so it must refuse
        // loudly instead.
        let connection = connection(":memory:snapshot-open-txn");
        register(&connection);
        let store = SnapshotStore::default();
        connection.execute("BEGIN").unwrap();
        assert!(matches!(
            store.refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            ),
            Err(SnapshotError::RefreshInsideTransaction)
        ));
        connection.execute("ROLLBACK").unwrap();
        store
            .refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            )
            .expect("refresh works again in autocommit");
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
                error_at_max_hops: false,
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

    #[test]
    fn freshness_is_observable_and_stale_snapshots_cannot_be_opened() {
        let connection = connection(":memory:snapshot-freshness");
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

        let SnapshotStatus::Current(metadata) = store.status(&connection, "social").unwrap() else {
            panic!("published snapshot must be current");
        };
        assert_eq!(metadata.node_count, 1);
        assert_eq!(metadata.relationship_count, 0);
        assert!(metadata.estimated_heap_bytes > 0);
        assert!(metadata.estimated_peak_build_bytes >= metadata.estimated_heap_bytes);
        assert_eq!(
            store.persistence_mode(),
            SnapshotPersistenceMode::InMemoryRebuildOnDemand
        );

        connection
            .execute("INSERT INTO people VALUES (2, 'B')")
            .unwrap();
        assert!(matches!(
            store.status(&connection, "social").unwrap(),
            SnapshotStatus::Stale {
                snapshot: SnapshotMetadata {
                    source_generation: 1,
                    ..
                },
                current_generation: 2,
                ..
            }
        ));
        assert!(store
            .get_for_connection(&connection, registered.id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn discarded_or_process_lost_state_rebuilds_without_changing_canonical_rows() {
        let connection = connection(":memory:snapshot-discard");
        let registered = register(&connection);
        connection
            .execute(
                "INSERT INTO people VALUES (1, 'A'), (2, 'B'); \
                 INSERT INTO relationships VALUES (10, 1, 2)",
            )
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
        assert!(store.discard(registered.id).unwrap());
        assert_eq!(
            store.status(&connection, "social").unwrap(),
            SnapshotStatus::Missing
        );

        let after_process_loss = SnapshotStore::default();
        assert_eq!(
            after_process_loss.status(&connection, "social").unwrap(),
            SnapshotStatus::Missing
        );
        after_process_loss
            .refresh(
                &connection,
                "social",
                BuildLimits::default(),
                &turso_graph_runtime::NeverCancelled,
            )
            .unwrap();
        let rebuilt = after_process_loss
            .get_current(&connection, "social")
            .unwrap()
            .unwrap();
        assert_eq!(rebuilt.graph().node_count(), 2);
        assert_eq!(rebuilt.graph().edge_count(), 1);
        assert_eq!(
            query_rows_cancellable(
                &connection,
                "SELECT (SELECT count(*) FROM people), (SELECT count(*) FROM relationships)",
                &turso_graph_runtime::NeverCancelled,
            )
            .unwrap(),
            vec![vec![Value::from_i64(2), Value::from_i64(1)]]
        );
    }

    #[test]
    fn schema_damage_rejects_refresh_and_preserves_canonical_rows_and_old_state() {
        let connection = connection(":memory:snapshot-schema-damage");
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

        connection.execute("DROP TABLE relationships").unwrap();
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
        assert_eq!(
            query_rows_cancellable(
                &connection,
                "SELECT id, name FROM people ORDER BY id",
                &turso_graph_runtime::NeverCancelled,
            )
            .unwrap(),
            vec![vec![Value::from_i64(1), Value::build_text("A")]]
        );
    }
}
