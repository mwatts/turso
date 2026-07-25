//! The graph frontend's [`Dialect`]: database identity and the shared-core
//! seams that are per-database rather than per-connection. Statement
//! compilation deliberately stays on the [`crate::GraphCompiler`]
//! `FrontendCompiler` path (`Connection::prepare_frontend`), which is
//! connection-aware and already owns reprepare; this mirrors how
//! `turso_pg` splits `PostgresDialect` (schema dialect) from
//! `PostgresCompiler` (statement compilation).

use parking_lot::RwLock;
use std::sync::Arc;
use turso_core::{schema::BTreeTable, Dialect, Result};

/// Shared with [`crate::graph_frontend_id`] so the dialect name and the
/// frontend-compiler id stay one identity, like `"postgres"` does for pg.
pub const GRAPH_DIALECT_NAME: &str = "graph-cypher";

/// `turso_graphs`: the graph analogue of a `pg_catalog` table, listing every
/// source (node or relationship) of every graph registered on this
/// database via [`crate::register_graph`]. Installed by
/// [`GraphDialect::register_catalog`] on every schema build/rebuild, the
/// same lifecycle catalog tables get, so registration state is inspectable
/// from any connection without going through the frontend API.
const TURSO_GRAPHS_VTAB_SQL: &str = "CREATE TABLE turso_graphs (\
    graph_id INTEGER, graph_name TEXT, generation INTEGER, kind TEXT, \
    source_name TEXT, table_name TEXT, identity_column TEXT, \
    start_column TEXT, end_column TEXT)";

#[derive(Debug)]
struct TursoGraphsTable;

impl turso_core::InternalVirtualTable for TursoGraphsTable {
    fn name(&self) -> String {
        "turso_graphs".to_string()
    }

    fn sql(&self) -> String {
        TURSO_GRAPHS_VTAB_SQL.to_string()
    }

    fn open(
        &self,
        conn: Arc<turso_core::Connection>,
    ) -> Result<Arc<RwLock<dyn turso_core::InternalVirtualTableCursor>>> {
        Ok(Arc::new(RwLock::new(TursoGraphsCursor {
            conn,
            rows: Vec::new(),
            row: 0,
        })))
    }

    fn best_index(
        &self,
        constraints: &[turso_ext::ConstraintInfo],
        _order_by: &[turso_ext::OrderByInfo],
    ) -> std::result::Result<turso_ext::IndexInfo, turso_ext::ResultCode> {
        Ok(turso_ext::IndexInfo {
            idx_num: 0,
            idx_str: None,
            order_by_consumed: false,
            estimated_cost: 1.0,
            estimated_rows: 32,
            constraint_usages: constraints
                .iter()
                .map(|_| turso_ext::ConstraintUsage {
                    argv_index: None,
                    omit: false,
                })
                .collect(),
        })
    }
}

struct TursoGraphsCursor {
    conn: Arc<turso_core::Connection>,
    rows: Vec<Vec<turso_core::Value>>,
    row: usize,
}

impl TursoGraphsCursor {
    fn load(&mut self) -> Result<()> {
        // The catalog tables only exist once register_graph has run.
        let mut probe = self.conn.prepare_internal(format!(
            "SELECT 1 FROM sqlite_schema WHERE type = 'table' AND name = '{}'",
            crate::catalog::GRAPHS_TABLE
        ))?;
        if probe.run_collect_rows()?.is_empty() {
            self.rows = Vec::new();
            return Ok(());
        }
        let sql = format!(
            "SELECT g.id, g.name, COALESCE(gen.generation, 0), s.kind, s.name, \
                    COALESCE(ns.table_name, rs.table_name), \
                    COALESCE(ns.identity_column, rs.identity_column), \
                    rs.start_column, rs.end_column \
             FROM {graphs} g \
             LEFT JOIN {generations} gen ON gen.graph_id = g.id \
             JOIN {sources} s ON s.graph_id = g.id \
             LEFT JOIN {node_sources} ns ON ns.source_id = s.id \
             LEFT JOIN {relationship_sources} rs ON rs.source_id = s.id \
             ORDER BY g.id, s.id",
            graphs = crate::catalog::GRAPHS_TABLE,
            generations = crate::catalog::GENERATIONS_TABLE,
            sources = crate::catalog::SOURCES_TABLE,
            node_sources = crate::catalog::NODE_SOURCES_TABLE,
            relationship_sources = crate::catalog::RELATIONSHIP_SOURCES_TABLE,
        );
        let mut stmt = self.conn.prepare_internal(&sql)?;
        self.rows = stmt.run_collect_rows()?;
        Ok(())
    }
}

impl turso_core::InternalVirtualTableCursor for TursoGraphsCursor {
    fn filter(
        &mut self,
        _args: &[turso_core::Value],
        _idx_str: Option<String>,
        _idx_num: i32,
    ) -> Result<bool> {
        self.load()?;
        self.row = 0;
        Ok(!self.rows.is_empty())
    }

    fn next(&mut self) -> Result<bool> {
        self.row += 1;
        Ok(self.row < self.rows.len())
    }

    fn rowid(&self) -> i64 {
        self.row as i64
    }

    fn column(&self, column: usize) -> Result<turso_core::Value> {
        Ok(self
            .rows
            .get(self.row)
            .and_then(|row| row.get(column))
            .cloned()
            .unwrap_or(turso_core::Value::Null))
    }
}

#[derive(Debug)]
pub struct GraphDialect;

impl Dialect for GraphDialect {
    fn name(&self) -> &'static str {
        GRAPH_DIALECT_NAME
    }

    fn parse(&self, sql: &str) -> Result<(Option<turso_parser::ast::Cmd>, usize)> {
        // The engine and graph lowering both speak SQL here; Cypher enters
        // only through GraphConnection / prepare_frontend. When SQL parsing
        // fails but the text is valid Cypher, point at the right door
        // instead of surfacing a SQLite syntax error.
        match turso_core::dialect::sqlite::parse(sql) {
            Ok(parsed) => Ok(parsed),
            Err(sql_error) => {
                if turso_graph_cypher::parse(sql).is_ok() {
                    return Err(turso_core::LimboError::ParseError(
                        "Cypher statements must be prepared through \
                         GraphConnection (the graph-cypher frontend), not \
                         the SQL connection"
                            .to_string(),
                    ));
                }
                Err(sql_error)
            }
        }
    }

    fn parse_table_sql(&self, sql: &str, root_page: i64) -> Result<BTreeTable> {
        // Graph schema rows are plain SQLite DDL written by register_graph;
        // there is no marked graph DDL (yet), so this is pure delegation.
        BTreeTable::from_sql(sql, root_page)
    }

    fn parse_table_sql_ast(&self, sql: &str) -> Result<turso_parser::ast::Stmt> {
        turso_core::dialect::sqlite::parse_table_sql_ast(sql)
    }

    fn table_sql_for_replay(&self, sql: &str) -> Result<String> {
        turso_core::dialect::sqlite::table_sql_for_replay(sql)
    }

    fn format_table_sql(
        &self,
        input: &str,
        _tbl_name: &turso_parser::ast::QualifiedName,
        _body: &turso_parser::ast::CreateTableBody,
    ) -> Result<String> {
        Ok(input.to_string())
    }

    fn register_catalog(
        &self,
        schema: &mut turso_core::schema::Schema,
        enable_custom_types: bool,
    ) -> Result<()> {
        turso_core::dialect::sqlite::register_builtin_catalog(schema, enable_custom_types)?;
        // Durable graph metadata only. `__turso_graph_expand` stays
        // session-activated via `install_graph_catalog` because it holds a
        // SnapshotStore that is not available at schema build (no process-
        // global snapshot default). See docs/graph.md § expand activation.
        let vtab = turso_core::VirtualTable::new_internal(
            "turso_graphs".to_string(),
            TURSO_GRAPHS_VTAB_SQL.to_string(),
            turso_ext::VTabKind::VirtualTable,
            Arc::new(RwLock::new(TursoGraphsTable)),
        )?;
        schema.add_virtual_table(Arc::new(vtab))
    }

    fn resolve_function(&self, name: &str, arg_count: usize) -> Result<Option<turso_core::Func>> {
        if turso_graph_temporal::FUNCTION_NAMES
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(name))
        {
            return Ok(Some(turso_core::Func::Dialect(name.to_ascii_lowercase())));
        }
        turso_core::dialect::sqlite::resolve_builtin_function(name, arg_count)
    }

    fn exec_scalar_function(
        &self,
        _conn: &turso_core::Connection,
        name: &str,
        args: &[turso_core::Value],
    ) -> Result<turso_core::Value> {
        // Mirrors the ownership pattern core uses for `Func::External` scalar
        // calls (core/vdbe/execute.rs, the `ExtFunc::Scalar` arm): `to_ffi`
        // allocates an owned `ExtValue` per argument, `dispatch` only
        // borrows `&[ExtValue]`, and the caller stays responsible for
        // freeing every argument after the call — on both the success and
        // the "no such function" path. `Value::from_ffi` is core's safe
        // wrapper around that free (`core/types.rs:657`); this crate
        // forbids `unsafe_code`, so we reuse it purely for the free side
        // effect and discard its round-tripped `Value`.
        let ext_args: Vec<turso_ext::Value> = args.iter().map(turso_core::Value::to_ffi).collect();
        let out = turso_graph_temporal::dispatch(name, &ext_args);
        let result = match out {
            Some(v) => turso_core::Value::from_ffi(v),
            None => Err(turso_core::LimboError::ParseError(format!(
                "no such function: {name}"
            ))),
        };
        for ext_arg in ext_args {
            let _ = turso_core::Value::from_ffi(ext_arg);
        }
        result
    }

    fn requires_custom_types(&self) -> bool {
        // Graph fixtures and consumers declare `CREATE TYPE duration`;
        // a graph database never opens with the machinery off (same
        // reasoning as PostgresDialect).
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, IO};

    fn open_graph_db(io: &Arc<dyn turso_core::IO>, path: &str) -> Arc<Database> {
        Database::open_file_with_flags(
            io.clone(),
            path,
            OpenFlags::default(),
            DatabaseOpts::new().with_custom_types(true),
            None,
            Arc::new(GraphDialect),
        )
        .expect("open with GraphDialect")
    }

    #[test]
    fn sql_round_trips_under_graph_dialect() {
        let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let db = open_graph_db(&io, ":memory:gd-sql");
        let conn = db.connect().unwrap();
        conn.execute("CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT)")
            .unwrap();
        conn.execute("INSERT INTO people VALUES (1, 'a')").unwrap();
        let rows = conn
            .prepare("SELECT name FROM people")
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(rows, vec![vec![turso_core::Value::build_text("a")]]);
    }

    #[test]
    fn direct_cypher_prepare_gets_a_targeted_error() {
        let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let db = open_graph_db(&io, ":memory:gd-cypher");
        let conn = db.connect().unwrap();
        let err = conn.prepare("MATCH (n:Person) RETURN n.name").unwrap_err();
        assert!(
            err.to_string().contains("GraphConnection"),
            "want a pointer to the frontend path, got: {err}"
        );
    }

    #[test]
    fn temporal_functions_resolve_without_extension_install() {
        let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let db = open_graph_db(&io, ":memory:gd-funcs");
        let conn = db.connect().unwrap();
        // No install_temporal_extension call anywhere on this connection.
        let rows = conn
            .prepare("SELECT duration_parse('P1DT25H')")
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(rows[0][0].to_string(), "P1DT25H");

        // SQLite builtins still resolve through the fallback.
        let rows = conn
            .prepare("SELECT abs(-7)")
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(rows[0][0], turso_core::Value::from_i64(7));
    }

    #[test]
    fn dialect_and_extension_paths_agree() {
        // Dialect path (GraphDialect database, no install):
        let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let db = open_graph_db(&io, ":memory:gd-agree");
        let conn = db.connect().unwrap();
        let via_dialect = conn
            .prepare("SELECT duration_add('P1D', 'PT25H')")
            .unwrap()
            .run_collect_rows()
            .unwrap();

        // Extension path (SqliteDialect database + install), the
        // dialect-agnostic mode existing consumers use:
        let io2: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let db2 = Database::open_file_with_flags(
            io2.clone(),
            ":memory:gd-agree-ext",
            OpenFlags::default(),
            DatabaseOpts::new(),
            None,
            Arc::new(turso_core::SqliteDialect),
        )
        .unwrap();
        let conn2 = db2.connect().unwrap();
        turso_graph_temporal::install_temporal_extension(&conn2);
        let via_extension = conn2
            .prepare("SELECT duration_add('P1D', 'PT25H')")
            .unwrap()
            .run_collect_rows()
            .unwrap();

        assert_eq!(via_dialect, via_extension);
    }

    #[test]
    fn registry_rejects_reopen_with_other_dialect() {
        // `:memory:`-prefixed paths deliberately bypass the process-wide
        // database registry (see `is_memory_like` in core/lib.rs), so this
        // needs a real (if MemoryIO-backed) path to exercise the registry's
        // dialect check at all.
        let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let _db = open_graph_db(&io, "gd-registry.db");
        let err = Database::open_file_with_flags(
            io.clone(),
            "gd-registry.db",
            OpenFlags::default(),
            DatabaseOpts::new(),
            None,
            Arc::new(turso_core::SqliteDialect),
        )
        .unwrap_err();
        assert!(err.to_string().contains("already open with dialect"));
    }

    #[test]
    fn turso_graphs_vtab_lists_registered_sources() {
        let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let db = open_graph_db(&io, ":memory:gd-vtab");
        let conn = db.connect().unwrap();

        // Empty before any registration.
        let rows = conn
            .prepare("SELECT count(*) FROM turso_graphs")
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(rows[0][0], turso_core::Value::from_i64(0));

        conn.execute("CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT)")
            .unwrap();
        conn.execute(
            "CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER)",
        )
        .unwrap();
        crate::register_graph(
            &conn,
            &crate::GraphRegistration {
                name: "social".to_owned(),
                node_sources: vec![crate::NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![crate::RelationshipSourceRegistration {
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

        let rows = conn
            .prepare(
                "SELECT graph_name, kind, source_name, table_name \
                 FROM turso_graphs ORDER BY kind, source_name",
            )
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][1].to_string(), "node");
        assert_eq!(rows[0][2].to_string(), "Person");
        assert_eq!(rows[1][1].to_string(), "relationship");
        assert_eq!(rows[1][3].to_string(), "relationships");
    }
}
