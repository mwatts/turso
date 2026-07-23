//! Structural + runtime checks for the multi-frontend architecture guide.
//!
//! Proves:
//! 1. `docs/multi-frontend.md` exists with the required architecture sections.
//! 2. Paths the guide cites as extension points exist on disk.
//! 3. The live frontend→backend boundary used by Postgres-style frontends
//!    (`prepare_translated_stmt` after producing engine AST) still works.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect, IO};
use turso_graph_frontend::{
    register_graph, GraphConnection, GraphRegistration, NodeSourceRegistration,
    RelationshipSourceRegistration,
};
use turso_parser::ast;
use turso_parser::parser::Parser;

fn workspace_root() -> PathBuf {
    // tests/integration/multi_frontend_doc.rs → repo root is ../..
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn guide_path() -> PathBuf {
    workspace_root().join("docs/multi-frontend.md")
}

fn read_guide() -> String {
    let path = guide_path();
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("multi-frontend guide missing at {}: {e}", path.display()))
}

#[test]
fn multi_frontend_guide_has_required_sections() {
    let text = read_guide();
    let required_substrings = [
        // (i) bytecode / VDBE
        "VDBE",
        "bytecode",
        "enum Insn",
        "core/vdbe/",
        // (ii) SQLite frontend ↔ backend
        "SQLite frontend",
        "SqliteDialect",
        "sqlite/parser/",
        // (iii) Postgres frontend ↔ backend
        "Postgres frontend",
        "PostgresDialect",
        "prepare_translated_stmt",
        "postgres/parser/translator.rs",
        "postgres/frontend/",
        "postgres/server/",
        // (iv) Cypher / graph
        "Cypher",
        "graph",
        "Neo4j",
        // (v) Kafka-style topics
        "Kafka",
        "append-only",
        "topic",
        // (vi) shared-backend consolidation
        "shared-backend",
        "Consolidat",
        "Dialect",
        "core/dialect/",
        "core/translate/",
        // (vii) same-file multi-frontend runtime complexity (namespace-isolated model)
        "Same file, multiple frontends",
        "namespace-isolated",
        "check_registry_dialect",
        "Dialect-owned namespaces",
        "Cross-dialect",
        "turso_frontend:postgres",
        "Host Dialect",
    ];
    for needle in required_substrings {
        assert!(
            text.contains(needle),
            "docs/multi-frontend.md must contain {needle:?}"
        );
    }
}

#[test]
fn multi_frontend_guide_cites_real_extension_points() {
    let root = workspace_root();
    let required_paths = [
        "core/vdbe/insn.rs",
        "core/vdbe/execute.rs",
        "core/vdbe/mod.rs",
        "core/translate/mod.rs",
        "core/dialect/mod.rs",
        "core/dialect/sqlite.rs",
        "core/connection.rs",
        "core/statement.rs",
        "sqlite/parser/src/lib.rs",
        "postgres/parser/translator.rs",
        "postgres/frontend/session.rs",
        "postgres/frontend/catalog.rs",
        "postgres/server/lib.rs",
        "postgres/COMPAT.md",
        "docs/manual.md",
        "docs/multi-frontend.md",
    ];
    for rel in required_paths {
        let p = root.join(rel);
        assert!(
            Path::new(&p).exists(),
            "guide extension point must exist on disk: {}",
            p.display()
        );
    }
}

/// Frontend pattern from postgres/frontend/session.rs: translate offline to
/// engine AST, then `prepare_translated_stmt` so the original frontend text is
/// preserved while compile still goes through `core/translate` → VDBE.
#[test]
fn prepare_translated_stmt_frontend_boundary_executes() {
    let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
    let file = io
        .open_file("multi-frontend-boundary.db", OpenFlags::Create, true)
        .unwrap();
    let db_file = Arc::new(turso_core::storage::database::DatabaseFile::new(file));
    let db = Database::open(
        io,
        "multi-frontend-boundary.db",
        turso_core::OpenOptions::new(Arc::new(SqliteDialect))
            .storage(db_file)
            .flags(OpenFlags::default())
            .db_opts(DatabaseOpts::new()),
    )
    .unwrap();
    let conn = db.connect().unwrap();

    // Simulate a frontend that already produced engine AST (Postgres translator
    // does this from pg_query; Cypher/Kafka frontends would do the same).
    let frontend_text = "CREATE TABLE events(id INTEGER PRIMARY KEY, payload TEXT)";
    let stmt = match Parser::new(frontend_text.as_bytes())
        .next_cmd()
        .unwrap()
        .unwrap()
    {
        ast::Cmd::Stmt(s) => s,
        other => panic!("expected Stmt, got {other:?}"),
    };
    conn.prepare_translated_stmt(stmt, frontend_text)
        .unwrap()
        .run_ignore_rows()
        .unwrap();

    let insert_text = "INSERT INTO events VALUES (1, 'hello')";
    let insert_stmt = match Parser::new(insert_text.as_bytes())
        .next_cmd()
        .unwrap()
        .unwrap()
    {
        ast::Cmd::Stmt(s) => s,
        other => panic!("expected Stmt, got {other:?}"),
    };
    conn.prepare_translated_stmt(insert_stmt, insert_text)
        .unwrap()
        .run_ignore_rows()
        .unwrap();

    let select_text = "SELECT id, payload FROM events";
    let select_stmt = match Parser::new(select_text.as_bytes())
        .next_cmd()
        .unwrap()
        .unwrap()
    {
        ast::Cmd::Stmt(s) => s,
        other => panic!("expected Stmt, got {other:?}"),
    };
    let rows = conn
        .prepare_translated_stmt(select_stmt, select_text)
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], turso_core::Value::from_i64(1));
    assert_eq!(rows[0][1].to_string().trim_matches('\''), "hello");

    // Dialect parse path still works on the same connection (SQLite frontend).
    let via_prepare = conn
        .prepare("SELECT payload FROM events WHERE id = 1")
        .unwrap()
        .run_collect_rows()
        .unwrap();
    assert_eq!(via_prepare.len(), 1);
    conn.close().unwrap();
}

/// PostgreSQL and Cypher remain separate frontend crates, but their session
/// wrappers can register both compilers on one core connection. Keeping one
/// connection is what makes transaction ownership unambiguous.
#[test]
fn postgres_and_graph_frontends_share_one_connection_transaction() {
    let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
    let file = io
        .open_file("multi-frontend-transaction.db", OpenFlags::Create, true)
        .unwrap();
    let db_file = Arc::new(turso_core::storage::database::DatabaseFile::new(file));
    let db = Database::open(
        io,
        "multi-frontend-transaction.db",
        turso_core::OpenOptions::new(Arc::new(SqliteDialect))
            .storage(db_file)
            .flags(OpenFlags::default())
            .db_opts(DatabaseOpts::new()),
    )
    .unwrap();
    let conn = db.connect().unwrap();
    conn.execute(
        "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT NOT NULL); \
         CREATE TABLE knows(id INTEGER PRIMARY KEY, src INTEGER NOT NULL, dst INTEGER NOT NULL);",
    )
    .unwrap();
    register_graph(
        &conn,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "KNOWS".to_owned(),
                table: "knows".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
                start_node_source: "Person".to_owned(),
                end_node_source: "Person".to_owned(),
            }],
        },
    )
    .unwrap();

    let postgres = turso_pg::PgConnection::new(conn.clone());
    let graph = GraphConnection::open(conn.clone(), "social").unwrap();

    conn.execute("BEGIN IMMEDIATE").unwrap();
    graph
        .execute("CREATE (:Person {name: 'Ada'})", &Default::default())
        .unwrap();
    let mut postgres_rows = postgres.prepare("SELECT name FROM people").unwrap();
    let postgres_rows = postgres_rows.run_collect_rows().unwrap();
    assert_eq!(
        postgres_rows,
        vec![vec![turso_core::Value::build_text("Ada")]]
    );

    postgres
        .execute("UPDATE people SET name = 'Grace'")
        .unwrap();
    let graph_rows = graph
        .query("MATCH (n:Person) RETURN n.name", &Default::default())
        .unwrap();
    assert_eq!(
        graph_rows,
        vec![vec![turso_core::Value::build_text("Grace")]]
    );

    conn.execute("ROLLBACK").unwrap();
    let rows_after_rollback = graph
        .query("MATCH (n:Person) RETURN n.name", &Default::default())
        .unwrap();
    assert!(rows_after_rollback.is_empty());
}
