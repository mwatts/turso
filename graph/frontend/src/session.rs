use std::{collections::HashMap, num::NonZero, sync::Arc};

use thiserror::Error;
use turso_core::{Connection, Value};
use turso_graph_ir::GraphId;
use turso_graph_runtime::{BuildLimits, Cancellation, NeverCancelled};

use crate::{
    execute_cypher_mutation, graph_frontend_id, install_graph_catalog, GraphCompilationCatalog,
    GraphCompiler, MutationError, MutationSummary, ParameterTypes, Parameters, RegisteredGraph,
    SessionSnapshotStore, SnapshotError, SnapshotStore,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Parse(#[from] turso_graph_cypher::ParseError),
    #[error(transparent)]
    Bind(#[from] crate::BindError),
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),
    #[error(transparent)]
    Mutation(#[from] MutationError),
    #[error(transparent)]
    Database(#[from] turso_core::LimboError),
    #[error("query parameter `${0}` was not declared for this graph session")]
    UndeclaredParameter(String),
    #[error("query parameter `${0}` has no bound value")]
    MissingParameter(String),
}

/// Open a database with the graph-cypher schema dialect, resolving the IO
/// backend from `vfs` or the path like [`turso_core::Database::open_new`].
///
/// This is the graph mirror of `turso_pg::open_database`. To attach the
/// graph layer to an existing SQLite-dialect database instead, open it
/// yourself and use [`GraphConnection::install`]/[`GraphConnection::open`];
/// in that mode call `turso_graph_temporal::install_temporal_extension`
/// per connection (GraphConnection::install already does).
pub fn open_database(
    path: &str,
    vfs: Option<&str>,
    flags: turso_core::OpenFlags,
    opts: turso_core::DatabaseOpts,
) -> turso_core::Result<(Arc<dyn turso_core::IO>, Arc<turso_core::Database>)> {
    let io = match vfs {
        Some(vfs) => turso_core::Database::io_for_vfs(vfs)?,
        None => turso_core::Database::io_for_path(path)?,
    };
    let db = open_database_with_io(io.clone(), path, flags, opts)?;
    Ok((io, db))
}

/// Open a database with the graph-cypher schema dialect on an existing IO
/// backend.
pub fn open_database_with_io(
    io: Arc<dyn turso_core::IO>,
    path: &str,
    flags: turso_core::OpenFlags,
    opts: turso_core::DatabaseOpts,
) -> turso_core::Result<Arc<turso_core::Database>> {
    let file = io.open_file(path, flags, true)?;
    let db_file = Arc::new(turso_core::storage::database::DatabaseFile::new(file));
    turso_core::Database::open(
        io,
        path,
        turso_core::OpenOptions::new(Arc::new(crate::GraphDialect))
            .storage(db_file)
            .flags(flags)
            .db_opts(opts),
    )
}

/// Connection-local graph service boundary.
///
/// Read compilation stays on `FrontendCompiler`. Variable traversal rebuilds
/// a private snapshot immediately before execution, so it observes the rows
/// visible to this connection without publishing uncommitted state globally.
pub struct GraphConnection {
    connection: Arc<Connection>,
    graph: GraphId,
    graph_name: String,
    catalog: Arc<dyn GraphCompilationCatalog>,
    parameters: ParameterTypes,
    snapshots: Arc<SessionSnapshotStore>,
    limits: BuildLimits,
}

impl Drop for GraphConnection {
    fn drop(&mut self) {
        self.connection
            .unregister_frontend_compiler(&graph_frontend_id());
    }
}

impl GraphConnection {
    pub fn install(
        connection: Arc<Connection>,
        graph: &RegisteredGraph,
        catalog: Arc<dyn GraphCompilationCatalog>,
        parameters: ParameterTypes,
        shared_snapshots: Arc<SnapshotStore>,
        limits: BuildLimits,
    ) -> Result<Self, Error> {
        let snapshots = Arc::new(SessionSnapshotStore::new(shared_snapshots.clone()));
        shared_snapshots.register_session(&connection, &snapshots)?;
        install_graph_catalog(connection.as_ref(), shared_snapshots)?;
        // Lowered SQL calls temporal/duration scalars; without them embedders
        // hit "runtime scalar function missing" on any temporal query.
        turso_graph_temporal::install_temporal_extension(connection.as_ref());
        connection.register_frontend_compiler(
            graph_frontend_id(),
            Arc::new(GraphCompiler::new(
                graph.id,
                catalog.clone(),
                parameters.clone(),
            )),
        )?;
        Ok(Self {
            connection,
            graph: graph.id,
            graph_name: graph.name.clone(),
            catalog,
            parameters,
            snapshots,
            limits,
        })
    }

    /// Attach to an already-registered graph by name with default limits and a
    /// private snapshot store. This is the one-call counterpart of
    /// [`GraphConnection::install`]; use `install` directly to share a
    /// [`SnapshotStore`] across connections or tune [`BuildLimits`].
    pub fn open(connection: Arc<Connection>, graph_name: &str) -> Result<Self, Error> {
        Self::open_with_parameters(connection, graph_name, ParameterTypes::new())
    }

    /// Like [`GraphConnection::open`], additionally declaring the `$parameter`
    /// names/types this session's queries may bind.
    pub fn open_with_parameters(
        connection: Arc<Connection>,
        graph_name: &str,
        parameters: ParameterTypes,
    ) -> Result<Self, Error> {
        let graph = crate::load_registered_graph(&connection, graph_name).map_err(|error| {
            Error::Database(turso_core::LimboError::ParseError(error.to_string()))
        })?;
        let catalog = Arc::new(crate::SchemaCatalog::new(connection.clone(), graph.clone()));
        Self::install(
            connection,
            &graph,
            catalog,
            parameters,
            Arc::new(SnapshotStore::default()),
            BuildLimits::default(),
        )
    }

    pub fn graph_id(&self) -> GraphId {
        self.graph
    }

    pub fn graph_name(&self) -> &str {
        &self.graph_name
    }

    pub fn query(&self, source: &str, parameters: &Parameters) -> Result<Vec<Vec<Value>>, Error> {
        self.query_cancellable(source, parameters, &NeverCancelled)
    }

    pub fn query_cancellable(
        &self,
        source: &str,
        parameters: &Parameters,
        cancellation: &dyn Cancellation,
    ) -> Result<Vec<Vec<Value>>, Error> {
        let mut statement = self.prepare_cancellable(source, parameters, cancellation)?;
        Ok(statement.run_collect_rows()?)
    }

    pub fn prepare(
        &self,
        source: &str,
        parameters: &Parameters,
    ) -> Result<crate::Statement, Error> {
        self.prepare_cancellable(source, parameters, &NeverCancelled)
    }

    /// Static result-column types of a read query, in projection order.
    /// Booleans reach storage as integers, so callers that need to render
    /// Cypher values faithfully must consult these types.
    fn result_types_for(
        &self,
        syntax: &turso_graph_cypher::Query,
    ) -> Result<Vec<turso_graph_ir::ValueType>, Error> {
        let bound = crate::bind(syntax, self.graph, self.catalog.as_ref(), &self.parameters)?;
        let scope = bound.plan.scope();
        Ok(bound
            .plan
            .result_shape()
            .iter()
            .map(|column| {
                scope
                    .get(column.binding())
                    .map(|binding| binding.value_type().clone())
                    .unwrap_or(turso_graph_ir::ValueType::Any)
            })
            .collect())
    }

    pub fn prepare_cancellable(
        &self,
        source: &str,
        parameters: &Parameters,
        cancellation: &dyn Cancellation,
    ) -> Result<crate::Statement, Error> {
        // EXPLAIN-prefixed queries (including postgres option lists like
        // EXPLAIN (VERBOSE, COSTS OFF)) compile the inner query and return
        // core's own plan via EXPLAIN QUERY PLAN over the lowered SQL.
        if let Some(inner) = strip_explain_prefix(source) {
            let syntax = turso_graph_cypher::parse(inner)?;
            if requires_traversal_snapshot(&syntax) {
                self.snapshots.refresh_visible_if_stale(
                    &self.connection,
                    &self.graph_name,
                    self.limits,
                    cancellation,
                )?;
            }
            let bound = crate::bind(&syntax, self.graph, self.catalog.as_ref(), &self.parameters)?;
            let statement =
                crate::lower_relational(&bound.plan, self.catalog.as_ref()).map_err(|error| {
                    Error::Database(turso_core::LimboError::ParseError(error.to_string()))
                })?;
            let mut statement = self
                .connection
                .prepare(format!("EXPLAIN QUERY PLAN {statement}"))?;
            bind_query_parameters(&mut statement, parameters)?;
            return Ok(crate::Statement::new(statement, Vec::new()));
        }
        let syntax = turso_graph_cypher::parse(source)?;
        if requires_traversal_snapshot(&syntax) {
            self.snapshots.refresh_visible_if_stale(
                &self.connection,
                &self.graph_name,
                self.limits,
                cancellation,
            )?;
        }
        let mut statement = self
            .connection
            .prepare_frontend(&graph_frontend_id(), source)?;
        bind_query_parameters(&mut statement, parameters)?;
        let result_types = self.result_types_for(&syntax)?;
        Ok(crate::Statement::new(statement, result_types))
    }

    pub fn execute(&self, source: &str, parameters: &Parameters) -> Result<MutationSummary, Error> {
        let result = execute_cypher_mutation(
            &self.connection,
            self.graph,
            self.catalog.clone(),
            source,
            parameters,
        );
        let cleared = self.snapshots.clear();
        if let Err(error) = &cleared {
            // The mutation outcome must not be masked by cache state: on
            // success a poisoned local snapshot cache is not an error, and
            // on failure the mutation's own error takes precedence. The
            // store surfaces its own failure on the next traversal read.
            tracing::warn!("clearing session snapshots after mutation failed: {error}");
        }
        Ok(result?)
    }
}

fn requires_traversal_snapshot(query: &turso_graph_cypher::Query) -> bool {
    let clause_needs =
        |clause: &turso_graph_cypher::Spanned<turso_graph_cypher::Clause>| match &clause.value {
            turso_graph_cypher::Clause::Match(value) => value.paths.iter().any(|path| {
                path.steps
                    .iter()
                    .any(|(relationship, _)| relationship.range.is_some())
            }),
            _ => false,
        };
    query.clauses.iter().any(clause_needs)
        || query
            .unions
            .iter()
            .any(|branch| branch.clauses.iter().any(clause_needs))
}

/// Strips a leading `EXPLAIN` (with an optional parenthesized postgres
/// option list and bare ANALYZE/VERBOSE keywords) and returns the inner
/// query, or None when the source is not an EXPLAIN form.
pub fn strip_explain_prefix(source: &str) -> Option<&str> {
    let trimmed = source.trim_start();
    let rest = trimmed
        .get(..7)
        .filter(|prefix| prefix.eq_ignore_ascii_case("explain"))
        .map(|_| &trimmed[7..])?;
    let mut rest = rest.trim_start();
    if let Some(options_start) = rest.strip_prefix('(') {
        let close = options_start.find(')')?;
        rest = options_start[close + 1..].trim_start();
    }
    while let Some(next) = ["analyze", "verbose"].iter().find(|keyword| {
        rest.get(..keyword.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(keyword))
    }) {
        rest = rest[next.len()..].trim_start();
    }
    (!rest.is_empty()).then_some(rest)
}

fn bind_query_parameters(
    statement: &mut turso_core::Statement,
    parameters: &HashMap<String, Value>,
) -> Result<(), Error> {
    for (name, value) in parameters {
        let parameter = format!("${name}");
        let index = statement
            .parameter_index(&parameter)
            .ok_or_else(|| Error::UndeclaredParameter(name.clone()))?;
        statement.bind_at(index, value.clone())?;
    }
    for raw_index in 1..=statement.parameters_count() {
        let Some(index) = NonZero::new(raw_index) else {
            continue;
        };
        let Some(name) = statement.parameters().name(index) else {
            continue;
        };
        let name = name.strip_prefix('$').unwrap_or(name.as_str()).to_owned();
        if !parameters.contains_key(&name) {
            return Err(Error::MissingParameter(name));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        register_graph, CatalogEntity, GraphCatalogSnapshot, GraphRegistration,
        NodeSourceRegistration, NodeTableLayout, RelationalCatalogSnapshot,
        RelationshipSourceRegistration, RelationshipTableLayout, ResolvedProperty,
    };
    use turso_core::{Database, MemoryIO, SqliteDialect};
    use turso_graph_ir as ir;

    struct Catalog {
        node_source: ir::SourceTableId,
        relationship_source: ir::SourceTableId,
    }

    impl GraphCatalogSnapshot for Catalog {
        fn node_source(&self, _graph: GraphId) -> Option<ir::SourceTableId> {
            Some(self.node_source)
        }

        fn relationship_source(&self, _graph: GraphId) -> Option<ir::SourceTableId> {
            Some(self.relationship_source)
        }

        fn label(&self, _graph: GraphId, name: &str) -> Option<ir::LabelId> {
            (name == "Person").then(|| ir::LabelId::new(1).unwrap())
        }

        fn relationship_type(&self, _graph: GraphId, name: &str) -> Option<ir::RelationshipTypeId> {
            (name == "KNOWS").then(|| ir::RelationshipTypeId::new(1).unwrap())
        }

        fn property(
            &self,
            _graph: GraphId,
            entity: CatalogEntity,
            name: &str,
        ) -> Option<ResolvedProperty> {
            let (id, value_type, nullability) = match (entity, name) {
                (CatalogEntity::Node, "id") => {
                    (1, ir::ValueType::Integer, ir::Nullability::NonNull)
                }
                (CatalogEntity::Node, "name") => {
                    (2, ir::ValueType::Text, ir::Nullability::Nullable)
                }
                _ => return None,
            };
            Some(ResolvedProperty {
                id: ir::PropertyId::new(id).unwrap(),
                value_type,
                nullability,
            })
        }
    }

    impl RelationalCatalogSnapshot for Catalog {
        fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout> {
            (source == self.node_source).then(|| NodeTableLayout {
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            })
        }

        fn relationship_layout(
            &self,
            source: ir::SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            (source == self.relationship_source).then(|| RelationshipTableLayout {
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
            })
        }

        fn property_column(
            &self,
            source: ir::SourceTableId,
            property: ir::PropertyId,
        ) -> Option<String> {
            match (source, property.get()) {
                (source, 1) if source == self.node_source => Some("id".to_owned()),
                (source, 2) if source == self.node_source => Some("name".to_owned()),
                _ => None,
            }
        }
    }

    struct Fixture {
        writer: Arc<Connection>,
        reader: Arc<Connection>,
        writer_session: GraphConnection,
        reader_session: GraphConnection,
    }

    fn fixture(name: &str) -> Fixture {
        fixture_with_mode(name, false)
    }

    fn fixture_with_mode(name: &str, mvcc: bool) -> Fixture {
        let io = Arc::new(MemoryIO::new());
        let database = Database::open_file(io, name, Arc::new(SqliteDialect)).unwrap();
        let writer = database.connect().unwrap();
        if mvcc {
            writer.execute("PRAGMA journal_mode = 'mvcc'").unwrap();
        }
        let reader = database.connect().unwrap();
        writer
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
                 CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER); \
                 INSERT INTO people VALUES (1, 'Ada'), (2, 'Grace');",
            )
            .unwrap();
        let registered = register_graph(
            &writer,
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
        .unwrap();
        let catalog = Arc::new(Catalog {
            node_source: registered.node_sources[0].id,
            relationship_source: registered.relationship_sources[0].id,
        });
        let shared = Arc::new(SnapshotStore::default());
        shared
            .refresh(&writer, "social", BuildLimits::default(), &NeverCancelled)
            .unwrap();
        let writer_session = GraphConnection::install(
            writer.clone(),
            &registered,
            catalog.clone(),
            ParameterTypes::new(),
            shared.clone(),
            BuildLimits::default(),
        )
        .unwrap();
        let reader_session = GraphConnection::install(
            reader.clone(),
            &registered,
            catalog,
            ParameterTypes::new(),
            shared,
            BuildLimits::default(),
        )
        .unwrap();
        Fixture {
            writer,
            reader,
            writer_session,
            reader_session,
        }
    }

    fn outgoing(session: &GraphConnection) -> Vec<Vec<Value>> {
        session
            .query(
                "MATCH (a:Person {id: 1})-[:KNOWS*1..1]->(b) RETURN b.name AS name ORDER BY b.name",
                &Parameters::new(),
            )
            .unwrap()
    }

    #[test]
    fn explicit_transaction_reads_its_writes_without_global_publication() {
        let fixture = fixture(":memory:graph-session-explicit");
        fixture.writer.execute("BEGIN").unwrap();
        fixture
            .writer_session
            .execute(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &Parameters::new(),
            )
            .unwrap();
        assert_eq!(
            outgoing(&fixture.writer_session),
            vec![vec![Value::build_text("C")]]
        );
        assert!(outgoing(&fixture.reader_session).is_empty());

        fixture.writer.execute("ROLLBACK").unwrap();
        assert!(outgoing(&fixture.writer_session).is_empty());
        assert!(outgoing(&fixture.reader_session).is_empty());
    }

    #[test]
    fn committed_and_autocommit_writes_become_visible_on_the_next_read() {
        let fixture = fixture(":memory:graph-session-commit");
        fixture.writer.execute("BEGIN").unwrap();
        fixture
            .writer_session
            .execute(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &Parameters::new(),
            )
            .unwrap();
        assert!(outgoing(&fixture.reader_session).is_empty());
        fixture.writer.execute("COMMIT").unwrap();
        assert_eq!(
            outgoing(&fixture.reader_session),
            vec![vec![Value::build_text("C")]]
        );

        fixture
            .writer_session
            .execute(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 4, name: 'D'})",
                &Parameters::new(),
            )
            .unwrap();
        assert_eq!(
            outgoing(&fixture.reader_session),
            vec![vec![Value::build_text("C")], vec![Value::build_text("D")]]
        );
    }

    #[test]
    fn savepoint_rollback_discards_the_next_transaction_local_snapshot() {
        let fixture = fixture(":memory:graph-session-savepoint");
        fixture
            .writer
            .execute("BEGIN; SAVEPOINT user_change")
            .unwrap();
        fixture
            .writer_session
            .execute(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &Parameters::new(),
            )
            .unwrap();
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM people WHERE id = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "same-connection SQL must see the created node"
        );
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM relationships WHERE src = 1 AND dst = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "same-connection SQL must see the created relationship"
        );
        assert_eq!(outgoing(&fixture.writer_session).len(), 1);
        fixture
            .writer
            .execute("ROLLBACK TO user_change; RELEASE user_change")
            .unwrap();
        assert!(outgoing(&fixture.writer_session).is_empty());
        fixture.writer.execute("ROLLBACK").unwrap();
    }

    #[test]
    fn failed_mutation_does_not_leak_partial_rows_into_traversal() {
        let fixture = fixture(":memory:graph-session-failure");
        let result = fixture.writer_session.execute(
            "CREATE (:Person {id: 3, name: 'C'}), (:Person {id: 1, name: 'duplicate'})",
            &Parameters::new(),
        );
        assert!(result.is_err());
        assert!(outgoing(&fixture.writer_session).is_empty());
        assert_eq!(
            fixture
                .reader
                .prepare("SELECT count(*) FROM people")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(2)]]
        );
    }

    #[test]
    fn compound_aggregate_projections_compute_after_grouping() {
        // Aggregates inside larger expressions must still introduce the
        // aggregate stage; lowering them as scalar SQL calls would collapse
        // all rows with no GROUP BY and return one mis-grouped row.
        let fixture = fixture(":memory:graph-session-compound-aggregates");

        let rows = fixture
            .reader_session
            .query(
                "MATCH (a:Person) RETURN count(*) + 1 AS c",
                &Parameters::new(),
            )
            .expect("count(*) + 1 must aggregate");
        assert_eq!(rows, vec![vec![Value::from_i64(3)]]);

        let rows = fixture
            .reader_session
            .query(
                "MATCH (a:Person) RETURN 2 * sum(a.id) AS s",
                &Parameters::new(),
            )
            .expect("2 * sum(x) must aggregate");
        assert_eq!(rows, vec![vec![Value::from_i64(6)]]);

        // With a grouping key the remainder computes per group, not once.
        let rows = fixture
            .reader_session
            .query(
                "MATCH (a:Person) RETURN a.name AS name, count(*) + 1 AS c ORDER BY name",
                &Parameters::new(),
            )
            .expect("grouped compound aggregate");
        assert_eq!(
            rows,
            vec![
                vec![Value::build_text("Ada"), Value::from_i64(2)],
                vec![Value::build_text("Grace"), Value::from_i64(2)],
            ]
        );
    }

    #[test]
    fn unbounded_expansion_past_the_hop_cap_fails_loudly() {
        // `[*]` has no semantic upper bound; the implicit 64-hop cap is a
        // resource limit. A graph with a longer real path must error rather
        // than silently return truncated results, while explicit bounds keep
        // truncation-as-semantics behavior.
        let fixture = fixture(":memory:graph-session-unbounded");
        let mut sql = String::new();
        for id in 3..=66 {
            sql.push_str(&format!("INSERT INTO people VALUES ({id}, 'P{id}'); "));
        }
        for from in 1..=65 {
            sql.push_str(&format!(
                "INSERT INTO relationships VALUES ({from}, {from}, {}); ",
                from + 1
            ));
        }
        fixture.writer.execute(sql).unwrap();

        let error = fixture
            .writer_session
            .query(
                "MATCH (a:Person {id: 1})-[:KNOWS*]->(b) RETURN b.id",
                &Parameters::new(),
            )
            .expect_err("65-hop chain must overflow the 64-hop implicit cap");
        assert!(
            error.to_string().contains("limit exceeded"),
            "unexpected error: {error}"
        );

        let rows = fixture
            .writer_session
            .query(
                "MATCH (a:Person {id: 1})-[:KNOWS*1..3]->(b) RETURN b.id",
                &Parameters::new(),
            )
            .expect("explicit bounds keep truncation semantics");
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn install_registers_runtime_scalar_functions() {
        // cypher_equals (and the temporal scalars) are registered by the
        // temporal extension. Lowered SQL calls them, so a session installed
        // without the testkit must not fail with a missing scalar function.
        let fixture = fixture(":memory:graph-session-scalars");
        let rows = fixture
            .reader_session
            .query(
                "MATCH (a:Person) WHERE a.name IN ['Ada'] RETURN a.id",
                &Parameters::new(),
            )
            .expect("IN lowering depends on session-installed cypher_equals");
        assert_eq!(rows, vec![vec![Value::from_i64(1)]]);
    }

    #[test]
    fn mutate_result_survives_snapshot_clear_failure() {
        // A durable write must never be reported as failed because the local
        // snapshot cache could not be cleared afterwards, and a failed
        // mutation must keep its own error instead of the clear failure.
        let fixture = fixture(":memory:graph-session-poisoned-clear");
        fixture.writer_session.snapshots.poison_for_test();

        let summary = fixture
            .writer_session
            .execute("CREATE (:Person {id: 3, name: 'C'})", &Parameters::new())
            .expect("clear failure must not flip a successful mutation to Err");
        assert_eq!(summary.operations_executed, 1);
        assert_eq!(
            fixture
                .reader
                .prepare("SELECT count(*) FROM people")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(3)]]
        );

        let error = fixture
            .writer_session
            .execute(
                "CREATE (:Person {id: 3, name: 'duplicate'})",
                &Parameters::new(),
            )
            .expect_err("duplicate identity must fail");
        assert!(
            matches!(error, Error::Mutation(_)),
            "mutation error must not be replaced by the clear failure: {error}"
        );
    }

    #[test]
    fn cancelled_local_rebuild_is_not_installed() {
        struct Cancelled;

        impl Cancellation for Cancelled {
            fn is_cancelled(&self) -> bool {
                true
            }
        }

        let fixture = fixture(":memory:graph-session-cancelled");
        let error = fixture
            .writer_session
            .query_cancellable(
                "MATCH (a:Person {id: 1})-[:KNOWS*1..1]->(b) RETURN b.name",
                &Parameters::new(),
                &Cancelled,
            )
            .expect_err("cancelled rebuild must fail");
        assert!(matches!(
            error,
            Error::Snapshot(SnapshotError::Runtime(
                turso_graph_runtime::RuntimeError::Cancelled
            ))
        ));
        assert!(outgoing(&fixture.writer_session).is_empty());
    }

    #[test]
    fn traversal_snapshot_rebuilds_once_per_visible_generation() {
        let fixture = fixture(":memory:graph-session-refresh-frequency");
        let (initial, rebuilt) = fixture
            .writer_session
            .snapshots
            .refresh_visible_if_stale(
                &fixture.writer,
                "social",
                BuildLimits::default(),
                &NeverCancelled,
            )
            .unwrap();
        assert!(!rebuilt, "the current shared snapshot should be reused");

        fixture
            .writer
            .execute("INSERT INTO relationships VALUES (10, 1, 2)")
            .unwrap();
        let (refreshed, rebuilt) = fixture
            .writer_session
            .snapshots
            .refresh_visible_if_stale(
                &fixture.writer,
                "social",
                BuildLimits::default(),
                &NeverCancelled,
            )
            .unwrap();
        assert!(rebuilt);
        assert!(!Arc::ptr_eq(&initial, &refreshed));

        let (reused, rebuilt) = fixture
            .writer_session
            .snapshots
            .refresh_visible_if_stale(
                &fixture.writer,
                "social",
                BuildLimits::default(),
                &NeverCancelled,
            )
            .unwrap();
        assert!(!rebuilt);
        assert!(Arc::ptr_eq(&refreshed, &reused));
    }

    #[test]
    fn mvcc_transaction_reads_its_writes_without_cross_connection_leakage() {
        let fixture = fixture_with_mode(":memory:graph-session-mvcc", true);
        fixture.writer.execute("BEGIN CONCURRENT").unwrap();
        fixture
            .writer_session
            .execute(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &Parameters::new(),
            )
            .unwrap();
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM people WHERE id = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "MVCC SQL must see the created node"
        );
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM relationships WHERE src = 1 AND dst = 3")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(1)]],
            "MVCC SQL must see the created relationship"
        );
        let visible = crate::build_visible_traversal_snapshot(
            &fixture.writer,
            "social",
            BuildLimits::default(),
            &NeverCancelled,
        )
        .unwrap();
        assert_eq!(visible.graph().node_count(), 3);
        assert_eq!(visible.graph().edge_count(), 1);
        assert_eq!(outgoing(&fixture.writer_session).len(), 1);
        assert!(outgoing(&fixture.reader_session).is_empty());
        fixture.writer.execute("ROLLBACK").unwrap();
        assert!(outgoing(&fixture.writer_session).is_empty());
    }

    fn social_registration() -> GraphRegistration {
        GraphRegistration {
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
        }
    }

    #[test]
    fn open_database_pins_the_graph_dialect() {
        let (_io, db) = crate::open_database(
            ":memory:graph-dialect-open",
            None,
            turso_core::OpenFlags::default(),
            turso_core::DatabaseOpts::new(),
        )
        .unwrap();
        let conn = db.connect().unwrap();
        // Dialect-owned surface proves which dialect is live: temporal
        // functions resolve with no extension install.
        let rows = conn
            .prepare("SELECT duration_parse('P1D')")
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(rows[0][0].to_string(), "P1D");
    }

    #[test]
    fn full_cycle_register_reopen_query() {
        // register + close + reopen the same file, then GraphConnection::open
        // by name — proves catalog persistence needs no re-registration.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("graph.db");
        let path = path.to_str().unwrap();
        {
            let (_io, db) = crate::open_database(
                path,
                None,
                turso_core::OpenFlags::default(),
                turso_core::DatabaseOpts::new(),
            )
            .unwrap();
            let conn = db.connect().unwrap();
            conn.execute("CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT)")
                .unwrap();
            conn.execute(
                "CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER)",
            )
            .unwrap();
            crate::register_graph(&conn, &social_registration()).unwrap();
            conn.close().unwrap();
        }
        let (_io, db) = crate::open_database(
            path,
            None,
            turso_core::OpenFlags::default(),
            turso_core::DatabaseOpts::new(),
        )
        .unwrap();
        let conn = db.connect().unwrap();
        let session = crate::GraphConnection::open(conn, "social").unwrap();
        let rows = session
            .query("MATCH (n:Person) RETURN n.name", &crate::Parameters::new())
            .unwrap();
        assert!(rows.is_empty());
    }
}
