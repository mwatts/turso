//! The graph frontend's [`Dialect`]: database identity and the shared-core
//! seams that are per-database rather than per-connection. Statement
//! compilation deliberately stays on the [`crate::GraphCompiler`]
//! `FrontendCompiler` path (`Connection::prepare_frontend`), which is
//! connection-aware and already owns reprepare; this mirrors how
//! `turso_pg` splits `PostgresDialect` (schema dialect) from
//! `PostgresCompiler` (statement compilation).

use turso_core::{schema::BTreeTable, Dialect, Result};

/// Shared with [`crate::graph_frontend_id`] so the dialect name and the
/// frontend-compiler id stay one identity, like `"postgres"` does for pg.
pub const GRAPH_DIALECT_NAME: &str = "graph-cypher";

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
        turso_core::dialect::sqlite::register_builtin_catalog(schema, enable_custom_types)
    }

    fn resolve_function(&self, name: &str, arg_count: usize) -> Result<Option<turso_core::Func>> {
        turso_core::dialect::sqlite::resolve_builtin_function(name, arg_count)
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
}
