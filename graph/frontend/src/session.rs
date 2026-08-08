use std::{collections::HashMap, num::NonZero, sync::Arc};

use parking_lot::{Mutex, RwLock};
use thiserror::Error;
use turso_core::{Connection, Value};
use turso_graph_ir::GraphId;
use turso_graph_runtime::{BuildLimits, Cancellation, NeverCancelled};

use crate::{
    compiler::SharedGraphCatalog, graph_frontend_id, install_graph_catalog,
    mutation::execute_cypher_mutation, statement_cache::StatementCache, GraphCompilationCatalog,
    GraphCompiler, GraphDiagnostics, MutationError, MutationSummary, ParameterTypes, Parameters,
    RegisteredGraph, SessionSnapshotStore, SnapshotError, SnapshotStore, GRAPH_DIALECT_NAME,
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
    Semantic(#[from] crate::SemanticCatalogError),
    #[error(transparent)]
    Catalog(#[from] crate::CatalogError),
    #[error(transparent)]
    Database(#[from] turso_core::LimboError),
    #[cfg(feature = "fts")]
    #[error(transparent)]
    Fts(#[from] crate::GraphFtsError),
    #[error("query parameter `${0}` was not declared for this graph session")]
    UndeclaredParameter(String),
    #[error("query parameter `${0}` has no bound value")]
    MissingParameter(String),
    #[error("this graph connection is read-only and cannot run a {kind:?} statement")]
    ReadOnlyConnection { kind: crate::StatementKind },
}

/// How the graph layer was attached to the host connection's dialect.
///
/// Dialect-pinned databases (`open_database*`) own temporal scalars for
/// **Root** prepares via [`GraphDialect`]. Both modes still call
/// [`turso_graph_temporal::install_temporal_extension`] so mutation helpers
/// prepared with `prepare_internal` (SQLite symbol table only) can resolve
/// `cypher_*` / `duration_*` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphHostMode {
    /// Database opened with [`GraphDialect`] — dialect owns `Func::Dialect`
    /// temporal execution for Root statements; extension still installed for
    /// InternalHelper mutation SQL.
    DialectPinned,
    /// `install` / `open` on a foreign dialect (typically SQLite) — Root and
    /// InternalHelper both rely on
    /// [`turso_graph_temporal::install_temporal_extension`].
    Attach,
}

fn host_mode_for(connection: &Connection) -> GraphHostMode {
    if connection.dialect().name() == GRAPH_DIALECT_NAME {
        GraphHostMode::DialectPinned
    } else {
        GraphHostMode::Attach
    }
}

/// Open a database with the graph-cypher schema dialect, resolving the IO
/// backend from `vfs` or the path like [`turso_core::Database::open_new`].
///
/// This is the graph mirror of `turso_pg::open_database`. To attach the
/// graph layer to an existing SQLite-dialect database instead, open it
/// yourself and use [`GraphConnection::install`]/[`GraphConnection::open`].
/// Every install registers the temporal extension so InternalHelper mutation
/// SQL can resolve `cypher_*` / `duration_*`; dialect-pinned Root reads also
/// resolve the same names via [`GraphDialect`].
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

/// How a session decides its cached `SchemaCatalog` needs reloading.
enum CatalogFreshness {
    /// The session was built through [`GraphConnection::install`], so its
    /// caller owns the catalog and no reload happens here.
    CallerOwned,
    /// The database records a catalog-only generation. Reloads happen only
    /// when the catalog actually changed, so ordinary row writes cost one
    /// primary-key lookup instead of a full catalog rebuild.
    SchemaGeneration(Mutex<u64>),
    /// The database predates `schema_generation`. Fall back to the data
    /// generation, which advances on every write to a mapped table — correct,
    /// but it reloads the catalog after every mutation.
    DataGeneration(Mutex<u64>),
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
    catalog: SharedGraphCatalog,
    catalog_freshness: CatalogFreshness,
    /// Same `Arc` registered with Core for `prepare_frontend` recompile safety.
    /// Declared query parameters live on the compiler (shared bind path).
    compiler: Arc<GraphCompiler>,
    snapshots: Arc<SessionSnapshotStore>,
    /// The SQL a mutation runs around its writes — the freshness probe and the
    /// constraint checks — is the same text every time, so the session keeps
    /// those statements compiled instead of paying for them per mutation.
    statements: StatementCache,
    limits: BuildLimits,
    host_mode: GraphHostMode,
    /// When set, the session refuses any statement the binder classifies as a
    /// write. Enforcement is syntactic and happens before the statement runs,
    /// so a write that would have changed nothing is still refused.
    read_only: bool,
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
        let host_mode = host_mode_for(connection.as_ref());
        let catalog = Arc::new(RwLock::new(catalog));
        let snapshots = Arc::new(SessionSnapshotStore::new(shared_snapshots.clone()));
        shared_snapshots.register_session(&connection, &snapshots)?;
        // Expand is session-activated for both host modes (not dialect catalog).
        // install_graph_catalog is idempotent if install runs again on the same connection.
        install_graph_catalog(connection.as_ref(), shared_snapshots)?;
        // Always install temporal/cypher scalars. Root dialect-pinned prepares
        // resolve via GraphDialect, but mutation helpers use prepare_internal
        // (InternalHelper → SQLite symbol table only) and need the extension.
        turso_graph_temporal::install_temporal_extension(connection.as_ref());
        let compiler = Arc::new(GraphCompiler::with_shared(
            graph.id,
            catalog.clone(),
            parameters,
        ));
        // Register the same Arc Core will recompile through; session prepare
        // reuses its compile cache for result types.
        connection.register_frontend_compiler(graph_frontend_id(), compiler.clone())?;
        Ok(Self {
            connection,
            graph: graph.id,
            graph_name: graph.name.clone(),
            catalog,
            catalog_freshness: CatalogFreshness::CallerOwned,
            compiler,
            snapshots,
            statements: StatementCache::default(),
            limits,
            host_mode,
            read_only: false,
        })
    }

    /// Refuse every statement this connection classifies as a write.
    ///
    /// The check is syntactic and runs before the statement does, so a write
    /// that would have changed nothing is refused too.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Read/write character of `source`, without binding or running it.
    pub fn classify(&self, source: &str) -> Result<crate::StatementKind, Error> {
        Ok(crate::classify_statement(&turso_graph_cypher::parse(
            source,
        )?))
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
        let semantic = crate::load_semantic_snapshot(&connection, &graph)?.map(Arc::new);
        let catalog = Arc::new(crate::SchemaCatalog::with_semantic(
            connection.clone(),
            graph.clone(),
            semantic,
        ));
        let mut session = Self::install(
            connection,
            &graph,
            catalog,
            parameters,
            Arc::new(SnapshotStore::default()),
            BuildLimits::default(),
        )?;
        session.catalog_freshness = match graph.schema_generation {
            Some(schema_generation) => {
                CatalogFreshness::SchemaGeneration(Mutex::new(schema_generation))
            }
            None => CatalogFreshness::DataGeneration(Mutex::new(graph.generation)),
        };
        Ok(session)
    }

    pub fn graph_id(&self) -> GraphId {
        self.graph
    }

    pub fn graph_name(&self) -> &str {
        &self.graph_name
    }

    /// Whether this session is dialect-pinned or attach-mode.
    pub fn host_mode(&self) -> GraphHostMode {
        self.host_mode
    }

    /// Returns metadata for the snapshot visible to this graph session.
    ///
    /// Diagnostics are observational: this method does not refresh, publish,
    /// or otherwise mutate snapshot or catalog state.
    pub fn diagnostics(&self) -> Result<GraphDiagnostics, Error> {
        Ok(GraphDiagnostics {
            graph_id: self.graph,
            graph_name: self.graph_name.clone(),
            persistence_mode: self.snapshots.persistence_mode(),
            status: self.snapshots.status(&self.connection, &self.graph_name)?,
        })
    }

    /// Returns a read-only inventory of graph sources and their semantic names.
    pub fn inspect_schema(&self) -> Result<crate::GraphSchemaInspection, Error> {
        crate::inspection::inspect(&self.connection, &self.graph_name)
    }

    #[cfg(feature = "fts")]
    pub fn create_fts_index(
        &self,
        spec: &crate::GraphFtsIndexSpec,
    ) -> Result<crate::GraphFtsIndex, Error> {
        let catalog = self.catalog.read().clone();
        Ok(crate::fts::create(
            &self.connection,
            &self.graph_name,
            catalog.as_ref(),
            spec,
        )?)
    }

    #[cfg(feature = "fts")]
    pub fn list_fts_indexes(&self) -> Result<Vec<crate::GraphFtsIndex>, Error> {
        Ok(crate::fts::list(&self.connection, &self.graph_name)?)
    }

    #[cfg(feature = "fts")]
    pub fn drop_fts_index(&self, name: &str) -> Result<bool, Error> {
        Ok(crate::fts::drop(&self.connection, &self.graph_name, name)?)
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

    pub fn prepare_cancellable(
        &self,
        source: &str,
        parameters: &Parameters,
        cancellation: &dyn Cancellation,
    ) -> Result<crate::Statement, Error> {
        self.refresh_catalog_if_stale()?;
        // EXPLAIN-prefixed queries (including postgres option lists like
        // EXPLAIN (VERBOSE, COSTS OFF)) share the same compile_outcome as
        // ordinary reads, then prepare pure SQL EXPLAIN QUERY PLAN text so
        // Core never re-parses the original Cypher through the dialect.
        if let Some(inner) = strip_explain_prefix(source) {
            let outcome = self.compiler.compile_outcome(inner)?;
            if outcome.needs_snapshot {
                self.snapshots.refresh_visible_if_stale(
                    &self.connection,
                    &self.graph_name,
                    self.limits,
                    cancellation,
                )?;
            }
            let sql = match outcome.cmd {
                turso_parser::ast::Cmd::Stmt(stmt) => format!("EXPLAIN QUERY PLAN {stmt}"),
                other => format!("EXPLAIN QUERY PLAN {other}"),
            };
            let mut statement = self.connection.prepare(sql)?;
            bind_query_parameters(&mut statement, parameters)?;
            return Ok(crate::Statement::new(statement, Vec::new()));
        }
        // One parse/bind/lower: cache feeds prepare_frontend recompile and
        // result_types without a second bind.
        let outcome = self.compiler.compile_outcome(source)?;
        if outcome.needs_snapshot {
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
        // Prefer the compile cache from the shared outcome; on miss recompile
        // rather than silently returning empty types (bool-as-int fidelity).
        let result_types = match self.compiler.take_result_types_for(source) {
            Some(types) => types,
            None => self.compiler.compile_outcome(source)?.result_types,
        };
        Ok(crate::Statement::new(statement, result_types))
    }

    pub fn execute(&self, source: &str, parameters: &Parameters) -> Result<MutationSummary, Error> {
        if self.read_only {
            let kind = self.classify(source)?;
            if kind.writes() {
                return Err(Error::ReadOnlyConnection { kind });
            }
        }
        self.refresh_catalog_if_stale()?;
        // A source the parser rejects is left to the mutation path below, which
        // owns the error message callers already match on.
        let syntax = turso_graph_cypher::parse(source).ok();
        // A mutation is the only entry point that does not compile through
        // GraphCompiler, so nothing else refreshes the snapshot `graph_expand`
        // reads. Without this, a variable-length pattern in a mutation works
        // only when some earlier read happened to leave a snapshot behind.
        if syntax
            .as_ref()
            .is_some_and(crate::compiler::query_needs_traversal_snapshot)
        {
            self.snapshots.refresh_visible_if_stale(
                &self.connection,
                &self.graph_name,
                self.limits,
                &NeverCancelled,
            )?;
        }
        let catalog = self.catalog.read().clone();
        let result = execute_cypher_mutation(
            &self.connection,
            &self.statements,
            self.graph,
            catalog,
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

    fn refresh_catalog_if_stale(&self) -> Result<(), Error> {
        // The probe runs before every statement, so it must stay cheap. On the
        // tracked path it is one primary-key lookup; only a genuine catalog
        // change pays for the full reload below.
        let known = match &self.catalog_freshness {
            CatalogFreshness::CallerOwned => return Ok(()),
            CatalogFreshness::SchemaGeneration(known) => {
                let known = known.lock();
                let current = crate::catalog::load_schema_generation(
                    &self.connection,
                    &self.statements,
                    self.graph,
                )
                .map_err(|error| {
                    Error::Database(turso_core::LimboError::ParseError(error.to_string()))
                })?;
                if current == Some(*known) {
                    return Ok(());
                }
                known
            }
            CatalogFreshness::DataGeneration(known) => known.lock(),
        };
        self.reload_catalog(known)
    }

    fn reload_catalog(&self, mut known: parking_lot::MutexGuard<'_, u64>) -> Result<(), Error> {
        let graph =
            crate::load_registered_graph(&self.connection, &self.graph_name).map_err(|error| {
                Error::Database(turso_core::LimboError::ParseError(error.to_string()))
            })?;
        let current = match &self.catalog_freshness {
            CatalogFreshness::SchemaGeneration(_) => graph.schema_generation.unwrap_or(*known),
            _ => graph.generation,
        };
        if current == *known {
            return Ok(());
        }
        let semantic = crate::load_semantic_snapshot(&self.connection, &graph)?.map(Arc::new);
        *self.catalog.write() = Arc::new(crate::SchemaCatalog::with_semantic(
            self.connection.clone(),
            graph,
            semantic,
        ));
        *known = current;
        // Catalog shapes affect bind/lower; drop the shared compile cache.
        self.compiler.clear_last_compile();
        Ok(())
    }
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
        NodeSourceRegistration, NodeTableLayout, RelationalCatalogSnapshot, RelationshipRoleLayout,
        RelationshipSourceRegistration, RelationshipTableLayout, ResolvedProperty, SnapshotStatus,
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

        fn relationship_source_roles(
            &self,
            source: ir::SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            self.relationship_layout(source)
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
                roles: vec![
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(1).unwrap(),
                        name: "start".to_owned(),
                        column: "src".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(2).unwrap(),
                        name: "end".to_owned(),
                        column: "dst".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                ],
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
        fixture.writer.execute("BEGIN IMMEDIATE").unwrap();
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
        fixture.writer.execute("BEGIN IMMEDIATE").unwrap();
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
            .execute("BEGIN IMMEDIATE; SAVEPOINT user_change")
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
    fn aggregates_in_loop_inputs_compute_before_iteration() {
        // Quantifier and comprehension loop variables create nested scopes,
        // but their input lists are evaluated by the outer projection. An
        // aggregate there must run before the loop starts.
        let fixture = fixture(":memory:graph-session-loop-input-aggregates");

        let rows = fixture
            .reader_session
            .query(
                "MATCH (a:Person) RETURN ALL(x IN collect(a.id) WHERE x > 0) AS valid",
                &Parameters::new(),
            )
            .expect("aggregate quantifier input");
        assert_eq!(rows, vec![vec![Value::from_i64(1)]]);

        let rows = fixture
            .reader_session
            .query(
                "MATCH (a:Person) RETURN size([x IN collect(a.id) WHERE x > 0]) AS count",
                &Parameters::new(),
            )
            .expect("aggregate comprehension input");
        assert_eq!(rows, vec![vec![Value::from_i64(2)]]);
    }

    #[test]
    fn split_follows_cypher_string_and_null_semantics() {
        let fixture = fixture(":memory:graph-session-split");

        let rows = fixture
            .reader_session
            .query(
                "UNWIND split('one1two', '1') AS item RETURN item",
                &Parameters::new(),
            )
            .expect("split result should be a list");
        assert_eq!(
            rows,
            vec![
                vec![Value::build_text("one")],
                vec![Value::build_text("two")]
            ]
        );

        let rows = fixture
            .reader_session
            .query(
                "RETURN split('a,b', '') AS characters, split('a  b', ' ') AS repeated, \
                 split(null, ',') AS null_text, split('a,b', null) AS null_delimiter",
                &Parameters::new(),
            )
            .expect("split edge cases");
        assert_eq!(rows[0][0].to_string(), r#"["a",",","b"]"#);
        assert_eq!(rows[0][1].to_string(), r#"["a","","b"]"#);
        assert_eq!(rows[0][2], Value::Null);
        assert_eq!(rows[0][3], Value::Null);

        let error = fixture
            .reader_session
            .query("RETURN split(1, ',')", &Parameters::new())
            .expect_err("non-text split input must fail");
        assert!(error
            .to_string()
            .contains("split() over non-text arguments"));
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
    fn prepare_binds_cypher_once_for_result_types_and_frontend() {
        // Read prepare must not re-bind solely to recover result types after
        // FrontendCompiler::compile already bound the same source.
        let fixture = fixture(":memory:graph-session-single-bind");
        let before = crate::binder::BIND_COUNT.with(|count| count.get());
        let compile_before = fixture.reader_session.compiler.compile_misses();
        let stmt = fixture
            .reader_session
            .prepare("MATCH (n:Person) RETURN n.name AS name", &Parameters::new())
            .expect("prepare");
        let after = crate::binder::BIND_COUNT.with(|count| count.get());
        let compile_after = fixture.reader_session.compiler.compile_misses();
        assert_eq!(
            after - before,
            1,
            "prepare must bind once (compile_outcome shared with prepare_frontend), got {}",
            after - before
        );
        assert_eq!(
            compile_after - compile_before,
            1,
            "prepare must miss the compile cache once, got {}",
            compile_after - compile_before
        );
        assert_eq!(stmt.result_types().len(), 1);
        assert_eq!(stmt.result_types()[0], ir::ValueType::Text);
    }

    #[test]
    fn explain_binds_cypher_once_via_compile_outcome() {
        // EXPLAIN must reuse the shared compile_outcome path (one bind / one
        // cache miss on the inner Cypher) and not session-side re-parse.
        let fixture = fixture(":memory:graph-session-explain-single-bind");
        let before = crate::binder::BIND_COUNT.with(|count| count.get());
        let compile_before = fixture.reader_session.compiler.compile_misses();
        let rows = fixture
            .reader_session
            .query("EXPLAIN MATCH (n:Person) RETURN n.name", &Parameters::new())
            .expect("explain");
        let after = crate::binder::BIND_COUNT.with(|count| count.get());
        let compile_after = fixture.reader_session.compiler.compile_misses();
        assert!(!rows.is_empty(), "EXPLAIN must return core EQP rows");
        assert_eq!(
            after - before,
            1,
            "EXPLAIN must bind the inner Cypher once, got {}",
            after - before
        );
        assert_eq!(
            compile_after - compile_before,
            1,
            "EXPLAIN must go through compile_outcome (one cache miss), got {}",
            compile_after - compile_before
        );
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
    fn diagnostics_prefer_transaction_visible_overlay_and_observe_rollback() {
        let fixture = fixture(":memory:graph-session-diagnostics-overlay");
        fixture.writer.execute("BEGIN IMMEDIATE").unwrap();
        fixture
            .writer_session
            .execute(
                "MATCH (a:Person {id: 1}) CREATE (a)-[:KNOWS]->(:Person {id: 3, name: 'C'})",
                &Parameters::new(),
            )
            .unwrap();
        assert_eq!(outgoing(&fixture.writer_session).len(), 1);

        let SnapshotStatus::Current(writer) = fixture.writer_session.diagnostics().unwrap().status
        else {
            panic!("writer overlay must be current")
        };
        let SnapshotStatus::Current(reader) = fixture.reader_session.diagnostics().unwrap().status
        else {
            panic!("reader shared snapshot must remain current")
        };
        assert_eq!((writer.node_count, writer.relationship_count), (3, 1));
        assert_eq!((reader.node_count, reader.relationship_count), (2, 0));

        fixture.writer.execute("ROLLBACK").unwrap();
        assert!(matches!(
            fixture.writer_session.diagnostics().unwrap().status,
            // The overlay was built from rows the rollback threw away, so the
            // signal it recorded must no longer match. The values are opaque
            // tokens, so the assertion is inequality, not an ordering.
            SnapshotStatus::Stale { snapshot, current_generation, .. }
                if snapshot.node_count == 3
                    && snapshot.relationship_count == 1
                    && current_generation != snapshot.source_generation
        ));
        assert!(outgoing(&fixture.writer_session).is_empty());
        let SnapshotStatus::Current(rebuilt) = fixture.writer_session.diagnostics().unwrap().status
        else {
            panic!("next traversal must replace rolled-back overlay")
        };
        assert_eq!((rebuilt.node_count, rebuilt.relationship_count), (2, 0));
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
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        }
    }

    #[test]
    fn a_mutation_over_a_variable_length_pattern_builds_its_own_snapshot() {
        // graph_expand reads the session snapshot, and a mutation is the only
        // entry point that never prepares through the compiler. Before this,
        // the snapshot existed only because a failed read attempt happened to
        // refresh it before its bind error; a caller that goes straight to
        // execute got "graph snapshot 1 is not built".
        let fixture = fixture(":memory:graph-mutation-snapshot");
        fixture
            .writer
            .execute("INSERT INTO relationships VALUES (1, 1, 2)")
            .unwrap();
        fixture
            .writer_session
            .execute(
                "MATCH (a:Person)-[*1..2]->(b:Person) DETACH DELETE a, b",
                &Parameters::new(),
            )
            .expect("a variable-length mutation must not depend on a prior read");
        assert_eq!(
            fixture
                .writer
                .prepare("SELECT count(*) FROM people")
                .unwrap()
                .run_collect_rows()
                .unwrap(),
            vec![vec![Value::from_i64(0)]],
            "both endpoints of the only relationship must be gone"
        );
    }

    #[test]
    fn a_read_only_connection_runs_reads() {
        let mut fixture = fixture(":memory:graph-read-only-reads");
        fixture.writer_session.set_read_only(true);
        assert!(fixture.writer_session.is_read_only());
        fixture
            .writer_session
            .query("MATCH (n:Person) RETURN n.name", &Parameters::new())
            .expect("a read-only connection serves reads");
    }

    #[test]
    fn a_read_only_connection_refuses_a_write_before_running_it() {
        let mut fixture = fixture(":memory:graph-read-only-writes");
        fixture.writer_session.set_read_only(true);
        let error = fixture
            .writer_session
            .execute("CREATE (n:Person {id: 3, name: 'a'})", &Parameters::new())
            .expect_err("a read-only connection refuses writes");
        assert!(
            matches!(error, Error::ReadOnlyConnection { .. }),
            "unexpected error: {error}"
        );
        // Refused before running: the graph is untouched.
        assert_eq!(
            fixture
                .writer_session
                .query("MATCH (n:Person) RETURN n.name", &Parameters::new())
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn a_read_only_connection_refuses_a_delete_that_would_match_nothing() {
        // The refusal is decided by syntax, so it does not depend on the graph
        // containing a matching row.
        let mut fixture = fixture(":memory:graph-read-only-empty-delete");
        fixture.writer_session.set_read_only(true);
        let error = fixture
            .writer_session
            .execute("MATCH (n:Absent) DELETE n", &Parameters::new())
            .expect_err("an empty DELETE is still a write");
        assert!(
            matches!(
                error,
                Error::ReadOnlyConnection {
                    kind: crate::StatementKind::WriteWithoutRows
                }
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn classify_routes_without_trying_and_failing() {
        let fixture = fixture(":memory:graph-classify-route");
        assert_eq!(
            fixture
                .writer_session
                .classify("MATCH (n) RETURN n")
                .expect("parses"),
            crate::StatementKind::ReadOnly
        );
        assert_eq!(
            fixture
                .writer_session
                .classify("CREATE (n:Person) RETURN n")
                .expect("parses"),
            crate::StatementKind::WriteReturningRows
        );
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
            let registered = crate::register_graph(&conn, &social_registration()).unwrap();
            conn.execute("INSERT INTO people(id, name) VALUES (1, 'Alice')")
                .unwrap();
            conn.execute(format!(
                "INSERT INTO \"{}\"(source_id, node_id, label) VALUES ({}, 1, 'Person')",
                crate::labels_table_name(registered.id),
                registered.node_sources[0].id.get(),
            ))
            .unwrap();
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
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0].to_string(), "Alice");
    }
}
