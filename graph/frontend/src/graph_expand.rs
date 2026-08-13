use std::sync::Arc;

use parking_lot::RwLock;
use turso_core::{
    schema::Schema, Connection, InternalVirtualTable, InternalVirtualTableCursor,
    InternalVirtualTableStep, LimboError, Numeric, Value,
};
use turso_ext::{
    ConstraintInfo, ConstraintOp, ConstraintUsage, IndexInfo, OrderByInfo, ResultCode,
};
use turso_graph_ir::{GraphId, RelationshipTypeId, RoleId, SourceTableId};
use turso_graph_runtime::{
    Cancellation, Path, TraversalCursor, TraversalLimits, TraversalOrder, TraversalRequest,
    TraversalStep, Uniqueness,
};

use crate::{SnapshotStore, SourceIdentity, TraversalSnapshot};

pub const GRAPH_EXPAND_TABLE_NAME: &str = "__tdb_int_g_expand";

const OUTPUT_COLUMN_COUNT: usize = 11;
const INPUT_COLUMN_COUNT: usize = 16;
const COL_GRAPH_ID: usize = OUTPUT_COLUMN_COUNT;
const COL_MAX_MEMORY_BYTES: usize = COL_GRAPH_ID + 15;
const CURSOR_WORK_QUANTUM: u64 = 256;

/// Install graph expand into a schema under construction.
///
/// Prefer [`install_graph_catalog`] for production activation: expand needs a
/// connection-bound [`SnapshotStore`], which is not available when
/// [`crate::GraphDialect::register_catalog`] runs at schema build. This helper
/// remains for callers that already hold a store and own the schema lifecycle.
/// Safe to call more than once; later installs replace the earlier binding.
pub fn register_graph_catalog(
    schema: &mut Schema,
    snapshots: Arc<SnapshotStore>,
) -> turso_core::Result<String> {
    schema.register_internal_vtab(GraphExpandTable { snapshots })
}

/// Session-activate `__tdb_int_g_expand` on an open connection.
///
/// Variable-length path execution holds a [`SnapshotStore`] that is session-
/// (and optionally process-) local derived state, not durable catalog. That is
/// why expand is **not** installed from [`crate::GraphDialect::register_catalog`]
/// — dialect catalog registration has no connection snapshot to bind.
///
/// Called from [`crate::GraphConnection::install`] for both dialect-pinned and
/// attach opens. **Idempotent:** safe to call more than once; later installs
/// replace the earlier `SnapshotStore` binding (same contract as always-on
/// `install_temporal_extension` for InternalHelper / Root scalar symbols).
pub fn install_graph_catalog(
    connection: &Connection,
    snapshots: Arc<SnapshotStore>,
) -> turso_core::Result<String> {
    connection.register_internal_vtab(GraphExpandTable { snapshots })
}

struct GraphExpandTable {
    snapshots: Arc<SnapshotStore>,
}

impl std::fmt::Debug for GraphExpandTable {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GraphExpandTable")
            .finish_non_exhaustive()
    }
}

impl InternalVirtualTable for GraphExpandTable {
    fn name(&self) -> String {
        GRAPH_EXPAND_TABLE_NAME.to_owned()
    }

    fn open(
        &self,
        connection: Arc<Connection>,
    ) -> turso_core::Result<Arc<RwLock<dyn InternalVirtualTableCursor>>> {
        Ok(Arc::new(RwLock::new(GraphExpandCursor::new(
            self.snapshots.clone(),
            connection,
        ))))
    }

    fn best_index(
        &self,
        constraints: &[ConstraintInfo],
        _order_by: &[OrderByInfo],
    ) -> Result<IndexInfo, ResultCode> {
        let mut usages = vec![
            ConstraintUsage {
                argv_index: None,
                omit: false,
            };
            constraints.len()
        ];

        for (argument, column) in (COL_GRAPH_ID..=COL_MAX_MEMORY_BYTES).enumerate() {
            let matches = constraints
                .iter()
                .enumerate()
                .filter(|(_, constraint)| {
                    constraint.column_index as usize == column && constraint.op == ConstraintOp::Eq
                })
                .collect::<Vec<_>>();
            let Some((constraint_index, _constraint)) = matches
                .iter()
                .find(|(_, constraint)| constraint.usable)
                .copied()
            else {
                return if matches.is_empty() {
                    Err(ResultCode::InvalidArgs)
                } else {
                    Err(ResultCode::ConstraintViolation)
                };
            };
            usages[constraint_index] = ConstraintUsage {
                argv_index: Some((argument + 1) as u32),
                omit: true,
            };
        }

        let estimate = crate::expand_estimate::planner_expand_estimate();
        Ok(IndexInfo {
            idx_num: INPUT_COLUMN_COUNT as i32,
            idx_str: Some("graph-expand-v1".to_owned()),
            order_by_consumed: false,
            estimated_cost: estimate.estimated_cost,
            estimated_rows: estimate.estimated_rows.min(u32::MAX as u64) as u32,
            constraint_usages: usages,
        })
    }

    fn sql(&self) -> String {
        format!(
            "CREATE TABLE {GRAPH_EXPAND_TABLE_NAME}(
                path_id INTEGER,
                path_position INTEGER,
                node_id INTEGER,
                node_source_id INTEGER,
                node_identity ANY,
                relationship_id INTEGER,
                relationship_source_id INTEGER,
                relationship_identity ANY,
                relationship_type INTEGER,
                depth INTEGER,
                is_terminal INTEGER,
                graph_id INTEGER HIDDEN,
                start_node_source_id INTEGER HIDDEN,
                start_node_identity ANY HIDDEN,
                from_role INTEGER HIDDEN,
                to_role INTEGER HIDDEN,
                symmetric INTEGER HIDDEN,
                relationship_types TEXT HIDDEN,
                min_hops INTEGER HIDDEN,
                max_hops INTEGER HIDDEN,
                error_at_max_hops INTEGER HIDDEN,
                uniqueness TEXT HIDDEN,
                max_node_visits INTEGER HIDDEN,
                max_edge_visits INTEGER HIDDEN,
                max_paths INTEGER HIDDEN,
                max_work INTEGER HIDDEN,
                max_memory_bytes INTEGER HIDDEN
            )"
        )
    }
}

struct GraphExpandCursor {
    snapshots: Arc<SnapshotStore>,
    connection: Arc<Connection>,
    snapshot: Option<Arc<TraversalSnapshot>>,
    traversal: Option<TraversalCursor>,
    current_path: Option<Path>,
    path_id: i64,
    path_position: usize,
    row_id: i64,
    inputs: Vec<Value>,
    filter_in_progress: bool,
}

impl GraphExpandCursor {
    fn new(snapshots: Arc<SnapshotStore>, connection: Arc<Connection>) -> Self {
        Self {
            snapshots,
            connection,
            snapshot: None,
            traversal: None,
            current_path: None,
            path_id: 0,
            path_position: 0,
            row_id: 0,
            inputs: Vec::new(),
            filter_in_progress: false,
        }
    }

    fn advance_path_step(&mut self) -> turso_core::Result<InternalVirtualTableStep> {
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            LimboError::InternalError("graph expansion has no snapshot".to_owned())
        })?;
        let traversal = self.traversal.as_mut().ok_or_else(|| {
            LimboError::InternalError("graph expansion has no traversal state".to_owned())
        })?;
        let cancellation = ConnectionCancellation(&self.connection);
        match traversal
            .step(snapshot.graph(), &cancellation, CURSOR_WORK_QUANTUM)
            .map_err(runtime_error)?
        {
            TraversalStep::Path(path) => {
                self.current_path = Some(path);
                self.path_position = 0;
                self.path_id = self.path_id.checked_add(1).ok_or_else(|| {
                    LimboError::ExtensionError("graph path identity overflow".to_owned())
                })?;
                Ok(InternalVirtualTableStep::Row)
            }
            TraversalStep::Pending => Ok(InternalVirtualTableStep::Yield),
            TraversalStep::Done => {
                self.current_path = None;
                Ok(InternalVirtualTableStep::Done)
            }
        }
    }

    fn initialize_filter(&mut self, args: &[Value], idx_num: i32) -> turso_core::Result<()> {
        if idx_num != INPUT_COLUMN_COUNT as i32 || args.len() != INPUT_COLUMN_COUNT {
            return Err(LimboError::InvalidArgument(format!(
                "{GRAPH_EXPAND_TABLE_NAME} requires {INPUT_COLUMN_COUNT} arguments"
            )));
        }
        let graph_id = graph_id(&args[0])?;
        let start_source = source_table_id(&args[1], "start_node_source_id")?;
        let start_identity = source_identity(&args[2], "start_node_identity")?;
        let from_role = role_id(&args[3], "from_role")?;
        let to_role = role_id(&args[4], "to_role")?;
        let symmetric = boolean(&args[5], "symmetric")?;
        let relationship_types = relationship_types(&args[6])?;
        let min_hops = nonnegative_u32(&args[7], "min_hops")?;
        let max_hops = nonnegative_u32(&args[8], "max_hops")?;
        let error_at_max_hops = nonnegative_u32(&args[9], "error_at_max_hops")? != 0;
        let uniqueness = uniqueness(&args[10])?;
        let limits = TraversalLimits {
            max_node_visits: nonnegative_u64(&args[11], "max_node_visits")?,
            max_edge_visits: nonnegative_u64(&args[12], "max_edge_visits")?,
            max_paths: nonnegative_u64(&args[13], "max_paths")?,
            max_hops,
            max_work: nonnegative_u64(&args[14], "max_work")?,
            max_memory_bytes: nonnegative_u64(&args[15], "max_memory_bytes")?,
        };
        let snapshot = self
            .snapshots
            .get_for_connection(&self.connection, graph_id)
            .map_err(|error| LimboError::ExtensionError(error.to_string()))?
            .ok_or_else(|| {
                LimboError::InvalidArgument(format!("graph snapshot {graph_id} is not built"))
            })?;
        let start = snapshot
            .node_id(start_source, &start_identity)
            .ok_or_else(|| {
                LimboError::InvalidArgument(format!(
                    "start node {start_source}:{start_identity:?} is not present in graph {graph_id}"
                ))
            })?;
        let request = TraversalRequest {
            start,
            from_role,
            to_role,
            symmetric,
            relationship_types,
            min_hops,
            max_hops,
            error_at_max_hops,
            uniqueness,
            order: TraversalOrder::BreadthFirst,
        };
        let traversal =
            TraversalCursor::new(snapshot.graph(), request, limits).map_err(runtime_error)?;

        self.snapshot = Some(snapshot);
        self.traversal = Some(traversal);
        self.current_path = None;
        self.path_id = 0;
        self.path_position = 0;
        self.row_id = 1;
        self.inputs = args.to_vec();
        Ok(())
    }

    fn current_path(&self) -> turso_core::Result<&Path> {
        self.current_path.as_ref().ok_or_else(|| {
            LimboError::InternalError("graph expansion cursor is not on a row".to_owned())
        })
    }
}

impl InternalVirtualTableCursor for GraphExpandCursor {
    fn next(&mut self) -> turso_core::Result<bool> {
        loop {
            match self.next_step()? {
                InternalVirtualTableStep::Row => return Ok(true),
                InternalVirtualTableStep::Done => return Ok(false),
                InternalVirtualTableStep::Yield => {}
            }
        }
    }

    fn next_step(&mut self) -> turso_core::Result<InternalVirtualTableStep> {
        let path = self.current_path()?;
        if self.path_position + 1 < path.nodes.len() {
            self.path_position += 1;
            self.row_id = self.row_id.checked_add(1).ok_or_else(|| {
                LimboError::ExtensionError("graph expansion row identity overflow".to_owned())
            })?;
            return Ok(InternalVirtualTableStep::Row);
        }
        let step = self.advance_path_step()?;
        if step == InternalVirtualTableStep::Row {
            self.row_id = self.row_id.checked_add(1).ok_or_else(|| {
                LimboError::ExtensionError("graph expansion row identity overflow".to_owned())
            })?;
        }
        Ok(step)
    }

    fn rowid(&self) -> i64 {
        self.row_id
    }

    fn column(&self, column: usize) -> turso_core::Result<Value> {
        let path = self.current_path()?;
        match column {
            0 => Ok(Value::from_i64(self.path_id)),
            1 | 9 => integer_value(self.path_position, "path position"),
            2 => id_value(path.nodes[self.path_position].get(), "node"),
            3 => {
                let coordinate = self
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.node(path.nodes[self.path_position]));
                coordinate
                    .map(|coordinate| id_value(coordinate.source.get(), "node source"))
                    .transpose()?
                    .ok_or_else(|| {
                        LimboError::InternalError(
                            "graph snapshot is missing a node coordinate".to_owned(),
                        )
                    })
            }
            4 => {
                let coordinate = self
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| snapshot.node(path.nodes[self.path_position]));
                coordinate
                    .map(|coordinate| source_identity_value(&coordinate.identity))
                    .transpose()?
                    .ok_or_else(|| {
                        LimboError::InternalError(
                            "graph snapshot is missing a node coordinate".to_owned(),
                        )
                    })
            }
            5 if self.path_position == 0 => Ok(Value::Null),
            5 => id_value(
                path.relationships[self.path_position - 1].get(),
                "relationship",
            ),
            6 | 7 if self.path_position == 0 => Ok(Value::Null),
            6 => {
                let coordinate = self.snapshot.as_ref().and_then(|snapshot| {
                    snapshot.relationship(path.relationships[self.path_position - 1])
                });
                coordinate
                    .map(|coordinate| id_value(coordinate.source.get(), "relationship source"))
                    .transpose()?
                    .ok_or_else(|| {
                        LimboError::InternalError(
                            "graph snapshot is missing a relationship coordinate".to_owned(),
                        )
                    })
            }
            7 => {
                let coordinate = self.snapshot.as_ref().and_then(|snapshot| {
                    snapshot.relationship(path.relationships[self.path_position - 1])
                });
                coordinate
                    .map(|coordinate| source_identity_value(&coordinate.identity))
                    .transpose()?
                    .ok_or_else(|| {
                        LimboError::InternalError(
                            "graph snapshot is missing a relationship coordinate".to_owned(),
                        )
                    })
            }
            8 if self.path_position == 0 => Ok(Value::Null),
            8 => Ok(Value::from_i64(i64::from(
                path.relationship_types[self.path_position - 1].get(),
            ))),
            10 => Ok(Value::from_i64(i64::from(
                self.path_position + 1 == path.nodes.len(),
            ))),
            COL_GRAPH_ID..=COL_MAX_MEMORY_BYTES => Ok(self.inputs[column - COL_GRAPH_ID].clone()),
            _ => Err(LimboError::InternalError(format!(
                "graph expansion column {column} is out of range"
            ))),
        }
    }

    fn filter(
        &mut self,
        args: &[Value],
        _idx_str: Option<String>,
        idx_num: i32,
    ) -> turso_core::Result<bool> {
        loop {
            match self.filter_step(args, None, idx_num)? {
                InternalVirtualTableStep::Row => return Ok(true),
                InternalVirtualTableStep::Done => return Ok(false),
                InternalVirtualTableStep::Yield => {}
            }
        }
    }

    fn filter_step(
        &mut self,
        args: &[Value],
        _idx_str: Option<String>,
        idx_num: i32,
    ) -> turso_core::Result<InternalVirtualTableStep> {
        if !self.filter_in_progress {
            self.initialize_filter(args, idx_num)?;
            self.filter_in_progress = true;
        }
        let step = self.advance_path_step()?;
        if step != InternalVirtualTableStep::Yield {
            self.filter_in_progress = false;
        }
        Ok(step)
    }
}

struct ConnectionCancellation<'a>(&'a Connection);

impl Cancellation for ConnectionCancellation<'_> {
    fn is_cancelled(&self) -> bool {
        self.0.is_interrupted()
    }
}

fn graph_id(value: &Value) -> turso_core::Result<GraphId> {
    GraphId::new(positive_u64(value, "graph_id")?)
        .map_err(|error| LimboError::InvalidArgument(error.to_string()))
}

fn source_table_id(value: &Value, name: &str) -> turso_core::Result<SourceTableId> {
    SourceTableId::new(positive_u64(value, name)?)
        .map_err(|error| LimboError::InvalidArgument(error.to_string()))
}

fn source_identity(value: &Value, name: &str) -> turso_core::Result<SourceIdentity> {
    match value {
        Value::Numeric(Numeric::Integer(value)) => Ok(SourceIdentity::Integer(*value)),
        Value::Numeric(Numeric::Float(value)) => Ok(SourceIdentity::real(f64::from(*value))),
        Value::Text(value) => Ok(SourceIdentity::Text(value.as_str().to_owned())),
        Value::Blob(value) => Ok(SourceIdentity::Blob(value.to_vec())),
        Value::Null => Err(LimboError::InvalidArgument(format!(
            "{name} must not be NULL"
        ))),
    }
}

fn role_id(value: &Value, name: &str) -> turso_core::Result<RoleId> {
    RoleId::new(nonnegative_u32(value, name)?)
        .map_err(|error| LimboError::InvalidArgument(error.to_string()))
}

fn boolean(value: &Value, name: &str) -> turso_core::Result<bool> {
    Ok(nonnegative_u32(value, name)? != 0)
}

fn uniqueness(value: &Value) -> turso_core::Result<Uniqueness> {
    match text(value, "uniqueness")?.to_ascii_lowercase().as_str() {
        "walk" => Ok(Uniqueness::Walk),
        "trail" => Ok(Uniqueness::Trail),
        "path" => Ok(Uniqueness::Path),
        value => Err(LimboError::InvalidArgument(format!(
            "invalid graph uniqueness `{value}`"
        ))),
    }
}

fn relationship_types(value: &Value) -> turso_core::Result<Vec<RelationshipTypeId>> {
    let Value::Text(types) = value else {
        if matches!(value, Value::Null) {
            return Ok(Vec::new());
        }
        return Err(LimboError::InvalidArgument(
            "relationship_types must be comma-separated text or NULL".to_owned(),
        ));
    };
    types
        .as_str()
        .split(',')
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            let parsed = value.trim().parse::<u32>().map_err(|_| {
                LimboError::InvalidArgument(format!("invalid relationship type identity `{value}`"))
            })?;
            RelationshipTypeId::new(parsed)
                .map_err(|error| LimboError::InvalidArgument(error.to_string()))
        })
        .collect()
}

fn text<'a>(value: &'a Value, name: &str) -> turso_core::Result<&'a str> {
    match value {
        Value::Text(value) => Ok(value.as_str()),
        _ => Err(LimboError::InvalidArgument(format!("{name} must be text"))),
    }
}

fn positive_u64(value: &Value, name: &str) -> turso_core::Result<u64> {
    let value = integer(value, name)?;
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| LimboError::InvalidArgument(format!("{name} must be positive")))
}

fn nonnegative_u64(value: &Value, name: &str) -> turso_core::Result<u64> {
    u64::try_from(integer(value, name)?)
        .map_err(|_| LimboError::InvalidArgument(format!("{name} must be non-negative")))
}

fn nonnegative_u32(value: &Value, name: &str) -> turso_core::Result<u32> {
    u32::try_from(nonnegative_u64(value, name)?)
        .map_err(|_| LimboError::InvalidArgument(format!("{name} exceeds u32")))
}

fn integer(value: &Value, name: &str) -> turso_core::Result<i64> {
    match value {
        Value::Numeric(Numeric::Integer(value)) => Ok(*value),
        _ => Err(LimboError::InvalidArgument(format!(
            "{name} must be an integer"
        ))),
    }
}

fn integer_value(value: usize, name: &str) -> turso_core::Result<Value> {
    i64::try_from(value)
        .map(Value::from_i64)
        .map_err(|_| LimboError::ExtensionError(format!("{name} exceeds SQL integer range")))
}

fn id_value(value: u64, name: &str) -> turso_core::Result<Value> {
    i64::try_from(value)
        .map(Value::from_i64)
        .map_err(|_| LimboError::ExtensionError(format!("{name} id exceeds SQL integer range")))
}

fn source_identity_value(identity: &SourceIdentity) -> turso_core::Result<Value> {
    match identity {
        SourceIdentity::Integer(value) => Ok(Value::from_i64(*value)),
        SourceIdentity::Real(bits) => Ok(Value::from_f64(f64::from_bits(*bits))),
        SourceIdentity::Text(value) => Ok(Value::build_text(value.clone())),
        SourceIdentity::Blob(value) => Value::from_slice(value).map_err(|error| {
            LimboError::ExtensionError(format!("failed to copy graph source identity: {error}"))
        }),
    }
}

fn runtime_error(error: turso_graph_runtime::RuntimeError) -> LimboError {
    if matches!(error, turso_graph_runtime::RuntimeError::Cancelled) {
        LimboError::Interrupt
    } else {
        LimboError::ExtensionError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph_frontend_id, load_registered_graph, register_graph, CatalogEntity,
        GraphCatalogSnapshot, GraphCompiler, GraphRegistration, NodeSourceRegistration,
        NodeTableLayout, ParameterTypes, PublishOutcome, RelationalCatalogSnapshot,
        RelationshipRoleLayout, RelationshipSourceRegistration, RelationshipTableLayout,
        ResolvedProperty,
    };
    use turso_core::{Database, MemoryIO, SqliteDialect};
    use turso_graph_ir::{LabelId, Nullability, PropertyId, RoleCardinality, ValueType};
    use turso_graph_runtime::{BuildLimits, NeverCancelled};

    fn setup() -> (Arc<Connection>, Arc<SnapshotStore>, GraphId) {
        setup_with_fanout(0)
    }

    fn setup_with_fanout(fanout: u32) -> (Arc<Connection>, Arc<SnapshotStore>, GraphId) {
        let io = Arc::new(MemoryIO::new());
        let connection = Database::open_file(io, ":memory:graph-expand", Arc::new(SqliteDialect))
            .unwrap()
            .connect()
            .unwrap();
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
                 CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER); \
                 INSERT INTO people VALUES (10, 'A'), (20, 'B'), (30, 'C'); \
                 INSERT INTO relationships VALUES (100, 10, 20), (200, 20, 30)",
            )
            .unwrap();
        if fanout > 0 {
            let people = (0..fanout)
                .map(|index| format!("({}, 'F{index}')", 1_000 + index))
                .collect::<Vec<_>>()
                .join(",");
            let relationships = (0..fanout)
                .map(|index| format!("({}, 10, {})", 1_000 + index, 1_000 + index))
                .collect::<Vec<_>>()
                .join(",");
            connection
                .execute(format!(
                    "INSERT INTO people VALUES {people}; \
                     INSERT INTO relationships VALUES {relationships}"
                ))
                .unwrap();
        }
        let registered = register_graph(
            &connection,
            &GraphRegistration {
                name: "social".to_owned(),
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
        .unwrap();
        let snapshots = Arc::new(SnapshotStore::default());
        assert!(matches!(
            snapshots
                .refresh(
                    &connection,
                    "social",
                    BuildLimits::default(),
                    &NeverCancelled,
                )
                .unwrap(),
            PublishOutcome::Published {
                replaced: false,
                ..
            }
        ));
        install_graph_catalog(&connection, snapshots.clone()).unwrap();
        (connection, snapshots, registered.id)
    }

    fn invocation(graph_id: GraphId) -> String {
        format!(
            "{GRAPH_EXPAND_TABLE_NAME}({}, 1, 10, 1, 2, 0, '1', 1, 2, 0, 'trail', \
             100, 100, 100, 1000, 1048576)",
            graph_id.get()
        )
    }

    #[test]
    fn table_valued_scan_resumes_paths_and_exposes_hydration_coordinates() {
        let (connection, _snapshots, graph_id) = setup();
        let mut statement = connection
            .prepare(format!(
                "SELECT e.path_id, e.path_position, e.node_id, e.node_identity, p.name, \
                        e.relationship_identity, e.relationship_type, e.depth \
                 FROM {} AS e \
                 JOIN people AS p ON p.id = e.node_identity \
                 ORDER BY e.path_id, e.path_position",
                invocation(graph_id)
            ))
            .unwrap();
        let rows = statement.run_collect_rows().unwrap();

        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0][0], Value::from_i64(1));
        assert_eq!(rows[0][1], Value::from_i64(0));
        assert_eq!(rows[0][2], Value::from_i64(1));
        assert_eq!(rows[0][3], Value::from_i64(10));
        assert_eq!(rows[0][4], Value::build_text("A"));
        assert_eq!(rows[0][5], Value::Null);
        assert_eq!(rows[1][3], Value::from_i64(20));
        assert_eq!(rows[1][5], Value::from_i64(100));
        assert_eq!(rows[1][6], Value::from_i64(1));
        assert_eq!(rows[4][0], Value::from_i64(2));
        assert_eq!(rows[4][1], Value::from_i64(2));
        assert_eq!(rows[4][3], Value::from_i64(30));
        assert_eq!(rows[4][4], Value::build_text("C"));
        assert_eq!(rows[4][5], Value::from_i64(200));
        assert_eq!(rows[4][7], Value::from_i64(2));
    }

    #[test]
    fn breadth_first_terminal_rows_support_unweighted_shortest_path_lowering() {
        let (connection, _snapshots, graph_id) = setup();
        let rows = connection
            .prepare(format!(
                "SELECT e.path_id, e.depth, e.node_identity \
                 FROM {} AS e \
                 WHERE e.is_terminal = 1 AND e.node_identity = 30 \
                 ORDER BY e.depth LIMIT 1",
                invocation(graph_id)
            ))
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(
            rows,
            vec![vec![
                Value::from_i64(2),
                Value::from_i64(2),
                Value::from_i64(30),
            ]]
        );
    }

    struct CompilerCatalog {
        node_source: SourceTableId,
        relationship_source: SourceTableId,
    }

    impl GraphCatalogSnapshot for CompilerCatalog {
        fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
            Some(self.node_source)
        }

        fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
            Some(self.relationship_source)
        }

        fn label(&self, _graph: GraphId, name: &str) -> Option<LabelId> {
            (name == "Person").then(|| LabelId::new(1).unwrap())
        }

        fn relationship_type(&self, _graph: GraphId, name: &str) -> Option<RelationshipTypeId> {
            (name == "KNOWS").then(|| RelationshipTypeId::new(1).unwrap())
        }

        fn property(
            &self,
            _graph: GraphId,
            entity: CatalogEntity,
            name: &str,
        ) -> Option<ResolvedProperty> {
            (entity == CatalogEntity::Node && name == "name").then(|| ResolvedProperty {
                id: PropertyId::new(1).unwrap(),
                value_type: ValueType::Text,
                nullability: Nullability::Nullable,
            })
        }

        fn relationship_source_roles(
            &self,
            source: SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            self.relationship_layout(source)
        }
    }

    impl RelationalCatalogSnapshot for CompilerCatalog {
        fn node_layout(&self, source: SourceTableId) -> Option<NodeTableLayout> {
            (source == self.node_source).then(|| NodeTableLayout {
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            })
        }

        fn relationship_layout(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
            (source == self.relationship_source).then(|| RelationshipTableLayout {
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                roles: vec![
                    RelationshipRoleLayout {
                        role: RoleId::new(1).unwrap(),
                        name: "start".to_owned(),
                        column: "src".to_owned(),
                        cardinality: RoleCardinality::One,
                        spill_table: None,
                    },
                    RelationshipRoleLayout {
                        role: RoleId::new(2).unwrap(),
                        name: "end".to_owned(),
                        column: "dst".to_owned(),
                        cardinality: RoleCardinality::One,
                        spill_table: None,
                    },
                ],
            })
        }

        fn property_column(&self, source: SourceTableId, property: PropertyId) -> Option<String> {
            (source == self.node_source && property.get() == 1).then(|| "name".to_owned())
        }
    }

    #[test]
    fn best_index_uses_hop_based_estimates_not_fixed_hundred() {
        use turso_ext::{ConstraintInfo, ConstraintOp, ConstraintUsage};
        let table = GraphExpandTable {
            snapshots: Arc::new(SnapshotStore::default()),
        };
        let constraints: Vec<ConstraintInfo> = (0..INPUT_COLUMN_COUNT)
            .map(|column| ConstraintInfo {
                column_index: (COL_GRAPH_ID + column) as u32,
                op: ConstraintOp::Eq,
                usable: true,
                index: column,
            })
            .collect();

        crate::expand_estimate::begin_lower_estimates();
        crate::expand_estimate::record_expand_estimate(crate::expand_estimate::estimate_expand(
            1, 1, 1, "trail", 100_000,
        ));
        let short = table.best_index(&constraints, &[]).unwrap();

        crate::expand_estimate::begin_lower_estimates();
        crate::expand_estimate::record_expand_estimate(crate::expand_estimate::estimate_expand(
            1, 4, 1, "trail", 100_000,
        ));
        let long = table.best_index(&constraints, &[]).unwrap();

        assert!(
            long.estimated_rows > short.estimated_rows,
            "1..4 hops ({}) should estimate more rows than 1..1 ({})",
            long.estimated_rows,
            short.estimated_rows
        );
        assert!(
            long.estimated_cost > short.estimated_cost,
            "1..4 hops ({}) should cost more than 1..1 ({})",
            long.estimated_cost,
            short.estimated_cost
        );
        // Not the old fixed constants.
        assert!(short.estimated_rows != 100 || short.estimated_cost != 100.0);
        assert_eq!(long.constraint_usages.len(), INPUT_COLUMN_COUNT);
        assert!(long
            .constraint_usages
            .iter()
            .all(|usage: &ConstraintUsage| usage.argv_index.is_some()));
    }

    #[test]
    fn bounded_cypher_expansion_lowers_through_the_normal_planner_and_vdbe() {
        let (connection, _snapshots, graph_id) = setup();
        let registered = load_registered_graph(&connection, "social").unwrap();
        connection
            .register_frontend_compiler(
                graph_frontend_id(),
                Arc::new(GraphCompiler::new(
                    graph_id,
                    Arc::new(CompilerCatalog {
                        node_source: registered.node_sources[0].id,
                        relationship_source: registered.relationship_sources[0].id,
                    }),
                    ParameterTypes::new(),
                )),
            )
            .unwrap();

        let rows = connection
            .prepare_frontend(
                &graph_frontend_id(),
                "MATCH (p:Person {name: 'A'})-[:KNOWS*1..2]->(friend) \
                 RETURN friend.name AS name ORDER BY friend.name",
            )
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(
            rows,
            vec![vec![Value::build_text("B")], vec![Value::build_text("C")],]
        );

        let zero_hop = connection
            .prepare_frontend(
                &graph_frontend_id(),
                "MATCH (p:Person {name: 'A'})-[:KNOWS*0..0]->(friend) \
                 RETURN friend.name AS name",
            )
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(zero_hop, vec![vec![Value::build_text("A")]]);
    }

    #[test]
    fn scan_requires_a_built_snapshot_and_all_resource_arguments() {
        let (connection, snapshots, graph_id) = setup();
        let empty_store = Arc::new(SnapshotStore::default());
        install_graph_catalog(&connection, empty_store).unwrap();
        assert!(connection
            .prepare(format!("SELECT * FROM {}", invocation(graph_id)))
            .unwrap()
            .run_collect_rows()
            .is_err());

        install_graph_catalog(&connection, snapshots).unwrap();
        assert!(connection
            .prepare(format!(
                "SELECT * FROM {GRAPH_EXPAND_TABLE_NAME}({}, 1, 10, 1)",
                graph_id.get()
            ))
            .is_err());
    }

    #[test]
    fn scan_propagates_resource_exhaustion() {
        let (connection, _snapshots, graph_id) = setup();
        let sql = format!(
            "SELECT * FROM {GRAPH_EXPAND_TABLE_NAME}({}, 1, 10, 1, 2, 0, '1', 1, 2, 0, \
             'trail', 100, 100, 100, 1, 1048576)",
            graph_id.get()
        );
        let error = connection
            .prepare(sql)
            .unwrap()
            .run_collect_rows()
            .unwrap_err();
        assert!(error.to_string().contains("Work resource limit exceeded"));
    }

    #[test]
    fn high_fanout_filter_yields_resumes_and_can_be_abandoned() {
        let (connection, _snapshots, graph_id) = setup_with_fanout(300);
        let sql = format!(
            "SELECT path_id, node_identity FROM {GRAPH_EXPAND_TABLE_NAME}(
                {}, 1, 10, 1, 2, 0, '1', 2, 2, 0, 'trail',
                10000, 10000, 10000, 100000, 16777216
            )",
            graph_id.get()
        );

        let mut abandoned = connection.prepare(&sql).unwrap();
        assert!(matches!(
            abandoned.step().unwrap(),
            turso_core::StepResult::Yield
        ));
        drop(abandoned);

        let mut statement = connection.prepare(sql).unwrap();
        let mut yields = 0;
        let mut rows = 0;
        loop {
            match statement.step().unwrap() {
                turso_core::StepResult::Yield => yields += 1,
                turso_core::StepResult::Row => rows += 1,
                turso_core::StepResult::Done => break,
                turso_core::StepResult::IO => panic!("in-memory graph scan performed I/O"),
                turso_core::StepResult::Interrupt | turso_core::StepResult::Busy => {
                    panic!("unexpected graph scan interruption")
                }
            }
        }
        assert!(yields >= 1);
        assert_eq!(rows, 3);
    }

    #[test]
    fn connection_interrupt_is_observed_at_the_next_graph_quantum() {
        let (connection, _snapshots, graph_id) = setup_with_fanout(300);
        let mut statement = connection
            .prepare(format!(
                "SELECT path_id FROM {GRAPH_EXPAND_TABLE_NAME}(
                    {}, 1, 10, 1, 2, 0, '1', 2, 2, 0, 'trail',
                    10000, 10000, 10000, 100000, 16777216
                )",
                graph_id.get()
            ))
            .unwrap();
        assert!(matches!(
            statement.step().unwrap(),
            turso_core::StepResult::Yield
        ));
        connection.interrupt();
        assert!(matches!(
            statement.step().unwrap(),
            turso_core::StepResult::Interrupt
        ));
    }

    /// Regression net for the `from_role`/`to_role`/`symmetric` argument
    /// shift: replacing the single `direction` argument with three
    /// arguments moves every later argument (`relationship_types` onward)
    /// two slots to the right. A wrong index here is silent at compile
    /// time -- it either misparses a value (caught below by `.unwrap()`)
    /// or, worse, parses successfully into the wrong meaning. Requesting
    /// the *reversed* role pair (role `2`, role `1`) from the traversal
    /// runtime's terminal node C, over an exact 2-hop expansion (so only
    /// one path length is ever terminal, keeping the row unambiguous), must
    /// walk the KNOWS edges backward to A, the opposite of the fixture's
    /// forward A->B->C edges. That only happens if `from_role`/`to_role`/
    /// `symmetric` land in the vtab's role/symmetric columns (not, say,
    /// `min_hops`/`max_hops`) and `min_hops`/`max_hops` still land on their
    /// own (shifted) slots -- an index slip either misparses a value
    /// (caught by `.unwrap()`) or silently produces the forward path's
    /// terminal node (20 at depth 1) or no rows at all instead.
    #[test]
    fn variable_length_expand_reads_role_and_hop_arguments_at_their_shifted_index() {
        let (connection, _snapshots, graph_id) = setup();
        let rows = connection
            .prepare(format!(
                "SELECT e.depth, e.node_identity FROM {GRAPH_EXPAND_TABLE_NAME}(
                    {}, 1, 30, 2, 1, 0, '1', 2, 2, 0, 'trail',
                    100, 100, 100, 1000, 1048576
                ) AS e WHERE e.is_terminal = 1 ORDER BY e.depth LIMIT 1",
                graph_id.get()
            ))
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(rows, vec![vec![Value::from_i64(2), Value::from_i64(10)]]);
    }
}
