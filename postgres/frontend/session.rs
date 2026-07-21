use std::num::NonZero;
use std::str;
use std::sync::{Arc, OnceLock};

use crate::aliases;
use crate::catalog::{self, PostgresDialect};
use parking_lot::RwLock;
use turso_core::{
    Connection, FrontendCompilation, FrontendCompiler, FrontendId, LimboError, Result, Statement,
    Value,
};
use turso_graph_frontend::{
    GraphCompilationCatalog, GraphSession, MutationParameters, ParameterTypes, RegisteredGraph,
    SnapshotStore,
};
use turso_parser::ast;
use turso_pg_parser::translator::{
    is_comment_on, is_refresh_matview, try_extract_copy_from, try_extract_create_schema,
    try_extract_drop_schema, try_extract_graph_cypher, try_extract_set, try_extract_show,
    PgCopyFromStmt, PgCreateSchemaStmt, PgDropSchemaStmt, PostgreSQLTranslator,
};

use crate::copy::parse_copy_text_format;

#[derive(Clone)]
pub struct PgConnection {
    conn: Arc<Connection>,
    compiler_registration_error: Option<LimboError>,
    graph_session: Arc<RwLock<Option<Arc<GraphSession>>>>,
}

const POSTGRES_FRONTEND_NAME: &str = "postgres";

#[derive(Debug, Default)]
struct PostgresCompiler;

impl FrontendCompiler for PostgresCompiler {
    fn compile(&self, source: &str) -> Result<FrontendCompilation> {
        reject_sqlite_catalog_access(source)?;
        let parse_result =
            turso_pg_parser::parse(source).map_err(|e| LimboError::ParseError(e.to_string()))?;
        let translated = PostgreSQLTranslator::new()
            .translate_with_prereqs(&parse_result)
            .map_err(|e| LimboError::ParseError(e.to_string()))?;
        reject_catalog_dml(&translated.stmt)?;
        Ok(FrontendCompilation {
            prerequisites: translated.prereqs,
            cmd: Some(ast::Cmd::Stmt(translated.stmt)),
            consumed: source.len(),
        })
    }
}

fn postgres_frontend_id() -> FrontendId {
    static ID: OnceLock<FrontendId> = OnceLock::new();
    ID.get_or_init(|| {
        FrontendId::new(POSTGRES_FRONTEND_NAME)
            .expect("the static Postgres frontend id must be non-empty")
    })
    .clone()
}

fn postgres_compiler() -> Arc<dyn FrontendCompiler> {
    static COMPILER: OnceLock<Arc<PostgresCompiler>> = OnceLock::new();
    COMPILER.get_or_init(|| Arc::new(PostgresCompiler)).clone()
}

/// Open a database with the PostgreSQL schema dialect, resolving the IO
/// backend from `vfs` or the path like [`turso_core::Database::open_new`].
pub fn open_database(
    path: &str,
    vfs: Option<&str>,
    flags: turso_core::OpenFlags,
    opts: turso_core::DatabaseOpts,
) -> Result<(Arc<dyn turso_core::IO>, Arc<turso_core::Database>)> {
    let io = match vfs {
        Some(vfs) => turso_core::Database::io_for_vfs(vfs)?,
        None => turso_core::Database::io_for_path(path)?,
    };
    let db = open_database_with_io(io.clone(), path, flags, opts)?;
    Ok((io, db))
}

/// Open a database with the PostgreSQL schema dialect on an existing IO
/// backend.
pub fn open_database_with_io(
    io: Arc<dyn turso_core::IO>,
    path: &str,
    flags: turso_core::OpenFlags,
    opts: turso_core::DatabaseOpts,
) -> Result<Arc<turso_core::Database>> {
    let file = io.open_file(path, flags, true)?;
    let db_file = Arc::new(turso_core::storage::database::DatabaseFile::new(file));
    turso_core::Database::open(
        io,
        path,
        turso_core::OpenOptions::new(Arc::new(PostgresDialect))
            .storage(db_file)
            .flags(flags)
            .db_opts(opts),
    )
}

impl PgConnection {
    pub fn new(conn: Arc<Connection>) -> Self {
        aliases::install(&conn);
        let compiler_registration_error = conn
            .register_frontend_compiler(postgres_frontend_id(), postgres_compiler())
            .err();
        Self {
            conn,
            compiler_registration_error,
            graph_session: Arc::new(RwLock::new(None)),
        }
    }

    pub fn inner(&self) -> &Arc<Connection> {
        &self.conn
    }

    pub fn prepare(&self, sql: impl AsRef<str>) -> Result<Statement> {
        if let Some(err) = &self.compiler_registration_error {
            return Err(err.clone());
        }
        prepare_connection_statement(&self.conn, &self.graph_session, sql.as_ref())
    }

    pub fn install_graph(
        &self,
        graph: &RegisteredGraph,
        catalog: Arc<dyn GraphCompilationCatalog>,
        parameters: ParameterTypes,
        shared_snapshots: Arc<SnapshotStore>,
    ) -> Result<()> {
        let mut installed = self.graph_session.write();
        if let Some(existing) = installed.as_ref() {
            if existing.graph_id() == graph.id {
                return Ok(());
            }
            return Err(LimboError::ParseError(format!(
                "Postgres graph adapter already targets graph `{}`; multiple graph compilers on one connection are not yet supported",
                existing.graph_name()
            )));
        }
        let session = GraphSession::install(
            self.conn.clone(),
            graph,
            catalog,
            parameters,
            shared_snapshots,
            Default::default(),
        )
        .map_err(|error| LimboError::ParseError(error.to_string()))?;
        *installed = Some(Arc::new(session));
        Ok(())
    }

    pub fn query(&self, sql: impl AsRef<str>) -> Result<Option<Statement>> {
        let sql = sql.as_ref().trim();
        if sql.is_empty() {
            return Ok(None);
        }
        self.prepare(sql).map(Some)
    }

    pub fn execute(&self, sql: impl AsRef<str>) -> Result<()> {
        for stmt in self.query_runner(sql.as_ref().as_bytes()) {
            if let Some(mut stmt) = stmt? {
                stmt.run_ignore_rows()?;
            }
        }
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        self.conn.close()
    }

    pub fn pragma_update(&self, name: &str, value: impl std::fmt::Display) -> Result<()> {
        let sql = format!("PRAGMA {name} = {value}");
        let mut stmt = self.conn.prepare_internal(sql)?;
        stmt.run_ignore_rows()
    }

    pub fn query_runner<'a>(&'a self, sql: &'a [u8]) -> PgQueryRunner<'a> {
        PgQueryRunner::new(
            &self.conn,
            &self.graph_session,
            sql,
            self.compiler_registration_error.clone(),
        )
    }
}

pub struct PgQueryRunner<'a> {
    conn: &'a Arc<Connection>,
    graph_session: &'a Arc<RwLock<Option<Arc<GraphSession>>>>,
    stmts: Vec<String>,
    index: usize,
    compiler_registration_error: Option<LimboError>,
}

impl<'a> PgQueryRunner<'a> {
    fn new(
        conn: &'a Arc<Connection>,
        graph_session: &'a Arc<RwLock<Option<Arc<GraphSession>>>>,
        sql: &'a [u8],
        compiler_registration_error: Option<LimboError>,
    ) -> Self {
        let sql = str::from_utf8(sql).unwrap_or("");
        Self {
            conn,
            graph_session,
            stmts: split_statements(sql)
                .unwrap_or_else(|_| vec![sql.trim().to_string()])
                .into_iter()
                .filter(|stmt| !stmt.trim().is_empty())
                .collect(),
            index: 0,
            compiler_registration_error,
        }
    }
}

impl Iterator for PgQueryRunner<'_> {
    type Item = Result<Option<Statement>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.compiler_registration_error.take() {
            return Some(Err(err));
        }
        if self.index >= self.stmts.len() {
            return None;
        }

        let sql = &self.stmts[self.index];
        self.index += 1;
        Some(prepare_connection_statement(self.conn, self.graph_session, sql).map(Some))
    }
}

fn prepare_connection_statement(
    conn: &Arc<Connection>,
    graph_session: &Arc<RwLock<Option<Arc<GraphSession>>>>,
    sql: &str,
) -> Result<Statement> {
    let parse_result =
        turso_pg_parser::parse(sql).map_err(|error| LimboError::ParseError(error.to_string()))?;
    let graph_call = try_extract_graph_cypher(&parse_result)
        .map_err(|error| LimboError::ParseError(error.to_string()))?;
    let Some(graph_call) = graph_call else {
        return prepare_statement(conn, sql);
    };
    let session = graph_session.read().clone().ok_or_else(|| {
        LimboError::ParseError(
            "graph.cypher is not active on this Postgres connection; install a registered Turso graph first"
                .to_owned(),
        )
    })?;
    if !session
        .graph_name()
        .eq_ignore_ascii_case(&graph_call.graph_name)
    {
        return Err(LimboError::ParseError(format!(
            "graph `{}` is not installed on this Postgres connection",
            graph_call.graph_name
        )));
    }
    session
        .prepare_query(&graph_call.cypher, &MutationParameters::new())
        .map_err(|error| LimboError::ParseError(error.to_string()))
}

pub fn split_statements(sql: &str) -> Result<Vec<String>> {
    match turso_pg_parser::split_statements(sql) {
        Ok(stmts) if stmts.is_empty() && !sql.trim().is_empty() => Ok(vec![sql.trim().to_string()]),
        Ok(stmts) => Ok(stmts),
        Err(_) => Ok(vec![sql.trim().to_string()]),
    }
}

fn prepare_statement(conn: &Arc<Connection>, sql: &str) -> Result<Statement> {
    let sql = sql.trim();
    if sql.is_empty() {
        return Err(LimboError::InvalidArgument(
            "The supplied SQL string contains no statements".to_string(),
        ));
    }

    reject_sqlite_catalog_access(sql)?;

    if let Some(stmt) = try_prepare_special(conn, sql)? {
        return Ok(stmt);
    }

    // Parsing, translation, catalog-DML rejection, and serial-sequence
    // prerequisites all run through the registered PostgresCompiler so the
    // initial prepare and every recompile share one compilation path.
    conn.prepare_frontend(&postgres_frontend_id(), sql)
}

fn reject_catalog_dml(stmt: &ast::Stmt) -> Result<()> {
    let table_name = match stmt {
        ast::Stmt::Insert { tbl_name, .. } => Some(tbl_name.name.as_str()),
        ast::Stmt::Delete { tbl_name, .. } => Some(tbl_name.name.as_str()),
        ast::Stmt::Update(update) => Some(update.tbl_name.name.as_str()),
        _ => None,
    };

    let Some(table_name) = table_name else {
        return Ok(());
    };

    if !catalog::is_catalog_table_name(table_name) {
        return Ok(());
    }

    let verb = match stmt {
        ast::Stmt::Insert { .. } => "insert into",
        ast::Stmt::Delete { .. } => "delete from",
        ast::Stmt::Update { .. } => "update",
        _ => unreachable!(),
    };
    Err(LimboError::ParseError(format!(
        "cannot {verb} pg_catalog table \"{table_name}\""
    )))
}

fn reject_sqlite_catalog_access(sql: &str) -> Result<()> {
    let lower = sql.to_ascii_lowercase();
    for table_name in ["sqlite_master", "sqlite_schema"] {
        if lower.contains(table_name) {
            return Err(LimboError::ParseError(format!(
                "no such table: {table_name}"
            )));
        }
    }
    Ok(())
}

fn try_prepare_special(conn: &Arc<Connection>, sql: &str) -> Result<Option<Statement>> {
    let parse_result = match turso_pg_parser::parse(sql) {
        Ok(result) => result,
        Err(_) => return Ok(None),
    };

    if let Some(set_stmt) = try_extract_set(&parse_result) {
        let pragma_sql = format!("PRAGMA {} = {}", set_stmt.name, set_stmt.value);
        return Ok(Some(conn.prepare(&pragma_sql)?));
    }

    if let Some(show_stmt) = try_extract_show(&parse_result) {
        let pragma_sql = format!("PRAGMA {}", show_stmt.name);
        return Ok(Some(conn.prepare(&pragma_sql)?));
    }

    if let Some(stmt) = try_extract_create_schema(&parse_result) {
        handle_pg_create_schema(conn, &stmt)?;
        return Ok(Some(noop_statement(conn)?));
    }

    if let Some(stmt) = try_extract_drop_schema(&parse_result) {
        handle_pg_drop_schema(conn, &stmt)?;
        return Ok(Some(noop_statement(conn)?));
    }

    if is_refresh_matview(&parse_result) {
        return Ok(Some(noop_statement(conn)?));
    }

    if is_comment_on(&parse_result) {
        return Ok(Some(noop_statement(conn)?));
    }

    if let Some(stmt) = try_extract_copy_from(&parse_result) {
        let rows_inserted = handle_pg_copy_from(conn, &stmt)?;
        let stmt = noop_statement(conn)?;
        stmt.set_n_change(rows_inserted as i64);
        return Ok(Some(stmt));
    }

    Ok(None)
}

fn noop_statement(conn: &Arc<Connection>) -> Result<Statement> {
    conn.prepare("SELECT 0 WHERE 0")
}

fn execute_sqlite_internal(conn: &Arc<Connection>, sql: impl AsRef<str>) -> Result<()> {
    let mut stmt = conn.prepare_internal(sql)?;
    stmt.run_ignore_rows()
}

fn handle_pg_create_schema(conn: &Arc<Connection>, stmt: &PgCreateSchemaStmt) -> Result<()> {
    let name = stmt.name.to_lowercase();
    if name == "public" {
        if stmt.if_not_exists {
            return Ok(());
        }
        return Err(LimboError::ParseError(format!(
            "schema \"{name}\" already exists"
        )));
    }

    if schema_exists(conn, &name)? {
        if stmt.if_not_exists {
            return Ok(());
        }
        return Err(LimboError::ParseError(format!(
            "schema \"{name}\" already exists"
        )));
    }

    let path = schema_file_path(conn, &name);
    execute_sqlite_internal(
        conn,
        format!("ATTACH '{}' AS \"{}\"", path.replace('\'', "''"), name),
    )?;
    Ok(())
}

fn schema_file_path(conn: &Connection, schema_name: &str) -> String {
    let main_path = conn.db_file_path();
    let filename = format!("turso-postgres-schema-{schema_name}.db");
    if main_path == ":memory:" {
        filename
    } else {
        let parent = std::path::Path::new(&main_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        parent.join(&filename).to_string_lossy().to_string()
    }
}

fn handle_pg_drop_schema(conn: &Arc<Connection>, stmt: &PgDropSchemaStmt) -> Result<()> {
    let name = stmt.name.to_lowercase();
    if name == "public" {
        return handle_pg_drop_schema_public(conn, stmt.cascade);
    }

    if !schema_exists(conn, &name)? {
        if stmt.if_exists {
            return Ok(());
        }
        return Err(LimboError::ParseError(format!(
            "schema \"{name}\" does not exist"
        )));
    }

    if stmt.cascade {
        drop_all_tables_in_schema(conn, &name)?;
    }

    execute_sqlite_internal(conn, format!("DETACH \"{name}\""))?;
    Ok(())
}

fn handle_pg_drop_schema_public(conn: &Arc<Connection>, cascade: bool) -> Result<()> {
    let table_names = list_user_tables(conn, None)?;
    if !cascade && !table_names.is_empty() {
        return Err(LimboError::ParseError(
            "cannot drop schema \"public\" because other objects depend on it".to_string(),
        ));
    }

    for table_name in table_names {
        let mut stmt = prepare_statement(conn, &format!("DROP TABLE \"{table_name}\""))?;
        stmt.run_ignore_rows()?;
    }
    Ok(())
}

fn drop_all_tables_in_schema(conn: &Arc<Connection>, schema_name: &str) -> Result<()> {
    for table_name in list_user_tables(conn, Some(schema_name))? {
        let mut stmt = prepare_statement(
            conn,
            &format!("DROP TABLE \"{schema_name}\".\"{table_name}\""),
        )?;
        stmt.run_ignore_rows()?;
    }
    Ok(())
}

fn handle_pg_copy_from(conn: &Arc<Connection>, stmt: &PgCopyFromStmt) -> Result<usize> {
    let data = std::fs::read_to_string(&stmt.filename).map_err(|e| {
        LimboError::ParseError(format!("COPY FROM: cannot read '{}': {}", stmt.filename, e))
    })?;

    let table_name = match &stmt.schema_name {
        Some(schema) => format!("\"{schema}\".\"{}\"", stmt.table_name),
        None => format!("\"{}\"", stmt.table_name),
    };
    let column_names = get_table_columns(conn, &stmt.table_name, stmt.schema_name.as_deref())?;
    if column_names.is_empty() {
        return Err(LimboError::ParseError(format!(
            "COPY FROM: table '{}' not found or has no columns",
            stmt.table_name
        )));
    }

    let (insert_cols, num_columns) = match &stmt.columns {
        Some(cols) => {
            let col_list = cols
                .iter()
                .map(|c| format!("\"{c}\""))
                .collect::<Vec<_>>()
                .join(", ");
            (format!(" ({col_list})"), cols.len())
        }
        None => (String::new(), column_names.len()),
    };

    let placeholders = (0..num_columns).map(|_| "?").collect::<Vec<_>>().join(", ");
    let insert_sql = format!("INSERT INTO {table_name}{insert_cols} VALUES ({placeholders})");

    let delimiter = stmt
        .delimiter
        .as_ref()
        .and_then(|d| d.chars().next())
        .unwrap_or('\t');
    let null_string = stmt.null_string.as_deref().unwrap_or("\\N");

    let mut rows = parse_copy_text_format(&data, delimiter, null_string, num_columns)?;
    if stmt.header && !rows.is_empty() {
        rows.remove(0);
    }

    let rows_inserted = rows.len();
    let mut begin = conn.prepare_sqlite("BEGIN")?;
    begin.run_ignore_rows()?;

    let result = (|| {
        let mut insert_stmt = conn.prepare_sqlite(&insert_sql)?;
        for row in &rows {
            for (i, val) in row.iter().enumerate() {
                let index = NonZero::new(i + 1).unwrap();
                match val {
                    Some(s) => insert_stmt.bind_at(index, Value::build_text(s.clone()))?,
                    None => insert_stmt.bind_at(index, Value::Null)?,
                }
            }
            insert_stmt.run_ignore_rows()?;
            insert_stmt.reset()?;
            insert_stmt.clear_bindings();
        }

        let mut commit = conn.prepare_sqlite("COMMIT")?;
        commit.run_ignore_rows()?;
        Ok(rows_inserted)
    })();

    if result.is_err() {
        if let Ok(mut rollback) = conn.prepare_sqlite("ROLLBACK") {
            let _ = rollback.run_ignore_rows();
        }
    }

    result
}

fn get_table_columns(
    conn: &Arc<Connection>,
    table_name: &str,
    schema_name: Option<&str>,
) -> Result<Vec<String>> {
    let sql = match schema_name {
        Some(schema) => format!("PRAGMA \"{schema}\".table_info('{table_name}')"),
        None => format!("PRAGMA table_info('{table_name}')"),
    };
    let mut stmt = conn.prepare_internal(&sql)?;
    let rows = stmt.run_collect_rows()?;
    Ok(rows
        .into_iter()
        .filter_map(|row| match row.get(1) {
            Some(Value::Text(t)) => Some(t.as_str().to_string()),
            _ => None,
        })
        .collect())
}

fn list_user_tables(conn: &Arc<Connection>, schema_name: Option<&str>) -> Result<Vec<String>> {
    let filter = "type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '__turso_internal_%'";
    let sql = match schema_name {
        Some(name) => format!("SELECT name FROM \"{name}\".sqlite_schema WHERE {filter}"),
        None => format!("SELECT name FROM sqlite_schema WHERE {filter}"),
    };
    let mut stmt = conn.prepare_internal(&sql)?;
    let rows = stmt.run_collect_rows()?;
    Ok(rows
        .into_iter()
        .filter_map(|row| match row.first() {
            Some(Value::Text(t)) => Some(t.as_str().to_string()),
            _ => None,
        })
        .collect())
}

fn schema_exists(conn: &Arc<Connection>, schema_name: &str) -> Result<bool> {
    let sql = format!(
        "SELECT 1 FROM pragma_database_list WHERE name = '{}'",
        schema_name.replace('\'', "''")
    );
    let mut stmt = conn.prepare_internal(&sql)?;
    let rows = stmt.run_collect_rows()?;
    Ok(!rows.is_empty())
}

#[cfg(test)]
mod graph_tests {
    use super::*;
    use turso_core::{DatabaseOpts, MemoryIO, OpenFlags};
    use turso_graph_frontend::{
        register_graph, CatalogEntity, GraphCatalogSnapshot, GraphRegistration,
        NodeSourceRegistration, NodeTableLayout, RelationalCatalogSnapshot,
        RelationshipSourceRegistration, RelationshipTableLayout, ResolvedProperty,
    };
    use turso_graph_ir as ir;

    struct Catalog {
        node_source: ir::SourceTableId,
        relationship_source: ir::SourceTableId,
    }

    impl GraphCatalogSnapshot for Catalog {
        fn node_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
            Some(self.node_source)
        }

        fn relationship_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
            Some(self.relationship_source)
        }

        fn label(&self, _graph: ir::GraphId, name: &str) -> Option<ir::LabelId> {
            (name == "Person").then(|| ir::LabelId::new(1).unwrap())
        }

        fn relationship_type(
            &self,
            _graph: ir::GraphId,
            name: &str,
        ) -> Option<ir::RelationshipTypeId> {
            (name == "KNOWS").then(|| ir::RelationshipTypeId::new(1).unwrap())
        }

        fn property(
            &self,
            _graph: ir::GraphId,
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

    fn setup() -> PgConnection {
        let database = open_database_with_io(
            Arc::new(MemoryIO::new()),
            ":memory:postgres-graph-adapter",
            OpenFlags::default(),
            DatabaseOpts::new(),
        )
        .unwrap();
        let connection = PgConnection::new(database.connect().unwrap());
        connection
            .execute(
                "CREATE TABLE people(id BIGINT PRIMARY KEY, name TEXT); \
                 CREATE TABLE relationships(id BIGINT PRIMARY KEY, src BIGINT, dst BIGINT); \
                 INSERT INTO people VALUES (1, 'Ada'), (2, 'Grace'), (3, 'Linus'); \
                 INSERT INTO relationships VALUES (10, 1, 2), (20, 2, 3)",
            )
            .unwrap();
        let registered = register_graph(
            connection.inner(),
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
        connection
            .install_graph(
                &registered,
                catalog,
                ParameterTypes::new(),
                Arc::new(SnapshotStore::default()),
            )
            .unwrap();
        connection
    }

    #[test]
    fn postgres_graph_call_delegates_to_the_shared_cypher_session() {
        let connection = setup();
        let cypher = "MATCH (a:Person {id: 1})-[:KNOWS*1..2]->(friend) RETURN friend.name AS name ORDER BY friend.name";
        let direct = connection
            .graph_session
            .read()
            .as_ref()
            .unwrap()
            .query(cypher, &MutationParameters::new())
            .unwrap();
        let postgres = connection
            .prepare(format!(
                "SELECT * FROM graph.cypher('social', '{}')",
                cypher.replace('\'', "''")
            ))
            .unwrap()
            .run_collect_rows()
            .unwrap();
        assert_eq!(postgres, direct);
        assert_eq!(
            postgres,
            vec![
                vec![Value::build_text("Grace")],
                vec![Value::build_text("Linus")]
            ]
        );
    }

    #[test]
    fn postgres_graph_call_rejects_missing_or_mismatched_registration() {
        let connection = setup();
        let error = connection
            .prepare("SELECT * FROM graph.cypher('other', 'MATCH (n) RETURN n')")
            .expect_err("unknown graph must fail");
        assert!(error.to_string().contains("not installed"));

        let database = open_database_with_io(
            Arc::new(MemoryIO::new()),
            ":memory:postgres-graph-inactive",
            OpenFlags::default(),
            DatabaseOpts::new(),
        )
        .unwrap();
        let inactive = PgConnection::new(database.connect().unwrap());
        let error = inactive
            .prepare("SELECT * FROM graph.cypher('social', 'MATCH (n) RETURN n')")
            .expect_err("inactive graph API must fail");
        assert!(error.to_string().contains("not active"));
    }
}
