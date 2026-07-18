use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use turso_core::{
    Database, DatabaseOpts, FrontendCompilation, FrontendCompiler, FrontendError, FrontendId,
    LimboError, OpenFlags, PlatformIO, Result, SqliteDialect, StatementStatusCounter, Value, IO,
};
use turso_parser::ast::Cmd;
use turso_parser::parser::Parser;

use crate::common::TempDatabase;

fn parse_one(sql: &str) -> Result<Cmd> {
    Parser::new(sql.as_bytes())
        .next_cmd()?
        .ok_or_else(|| LimboError::InternalError("test compiler produced no command".to_string()))
}

fn compile_only(sql: &str, consumed: usize) -> Result<FrontendCompilation> {
    Ok(FrontendCompilation {
        prerequisites: Vec::new(),
        cmd: Some(parse_one(sql)?),
        consumed,
    })
}

#[derive(Debug)]
struct SyntheticCompiler {
    calls: AtomicUsize,
}

impl SyntheticCompiler {
    const fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl FrontendCompiler for SyntheticCompiler {
    fn compile(&self, source: &str) -> Result<FrontendCompilation> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let sql = match source {
            "SYNTHETIC RETURN $value" => "SELECT ?1",
            other => {
                return Err(LimboError::ParseError(format!(
                    "synthetic frontend does not understand {other:?}"
                )))
            }
        };
        compile_only(sql, source.len())
    }
}

/// Frontend compilation is a prepared-statement invariant: schema or prepare
/// context invalidation must use the original compiler and retain bindings.
#[turso_macros::test]
fn frontend_compiler_is_reused_for_reprepare_and_keeps_parameters(
    tmp_db: TempDatabase,
) -> Result<()> {
    let conn = tmp_db.connect_limbo();
    let frontend = FrontendId::new("synthetic")?;
    let compiler = Arc::new(SyntheticCompiler::new());
    conn.register_frontend_compiler(frontend.clone(), compiler.clone())?;

    let mut stmt = conn.prepare_frontend(&frontend, "SYNTHETIC RETURN $value")?;
    assert_eq!(compiler.calls(), 1, "initial prepare must compile once");
    stmt.bind_at(1.try_into().unwrap(), Value::from_i64(42))?;

    // This changes PrepareContext without changing the source. The retry must
    // dispatch through the frontend recipe rather than SQLite's parser.
    conn.set_full_column_names(true);
    let rows = stmt.run_collect_rows()?;

    assert_eq!(rows, vec![vec![Value::from_i64(42)]]);
    assert_eq!(compiler.calls(), 2, "reprepare must reuse the compiler");
    assert_eq!(
        stmt.stmt_status(StatementStatusCounter::Reprepare),
        1,
        "prepare-context invalidation must trigger exactly one reprepare"
    );
    assert_eq!(stmt.get_sql(), "SYNTHETIC RETURN $value");
    Ok(())
}

#[derive(Debug)]
struct PrereqCompiler;

impl FrontendCompiler for PrereqCompiler {
    fn compile(&self, source: &str) -> Result<FrontendCompilation> {
        // The prerequisite is a deliberately non-idempotent INSERT so a
        // replay is observable as a second `prereq_events` row.
        let (Cmd::Stmt(prereq) | Cmd::Explain(prereq) | Cmd::ExplainQueryPlan(prereq)) =
            parse_one("INSERT INTO prereq_events (id) VALUES (1)")?;
        Ok(FrontendCompilation {
            prerequisites: vec![prereq],
            cmd: Some(parse_one("SELECT count(*) FROM prereq_events")?),
            consumed: source.len(),
        })
    }
}

/// Prerequisites are initial-prepare side effects: they must execute before
/// the main statement prepares, and recompiles must discard them because a
/// reprepare can run mid-step while the statement holds pager locks.
#[turso_macros::test]
fn frontend_prerequisites_run_once_before_compile_and_never_on_reprepare(
    tmp_db: TempDatabase,
) -> Result<()> {
    let conn = tmp_db.connect_limbo();
    conn.prepare("CREATE TABLE prereq_events (id INTEGER)")?
        .run_ignore_rows()?;
    let frontend = FrontendId::new("prereq-synthetic")?;
    conn.register_frontend_compiler(frontend.clone(), Arc::new(PrereqCompiler))?;

    let mut stmt = conn.prepare_frontend(&frontend, "PREREQ QUERY")?;
    let count_events = || {
        conn.prepare("SELECT count(*) FROM prereq_events")?
            .run_collect_rows()
    };
    assert_eq!(
        count_events()?,
        vec![vec![Value::from_i64(1)]],
        "initial prepare must execute the prerequisite exactly once"
    );

    // Invalidate the prepare context so the next run reprepares.
    conn.set_full_column_names(true);
    let rows = stmt.run_collect_rows()?;

    assert_eq!(rows, vec![vec![Value::from_i64(1)]]);
    assert_eq!(
        stmt.stmt_status(StatementStatusCounter::Reprepare),
        1,
        "prepare-context invalidation must trigger exactly one reprepare"
    );
    assert_eq!(
        count_events()?,
        vec![vec![Value::from_i64(1)]],
        "reprepare must not replay prerequisites"
    );
    Ok(())
}

#[derive(Debug)]
struct RetryCompiler {
    calls: AtomicUsize,
}

impl RetryCompiler {
    const fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }
}

impl FrontendCompiler for RetryCompiler {
    fn compile(&self, source: &str) -> Result<FrontendCompilation> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let sql = if call == 0 {
            // Force compile_cmd's cross-process schema-lookup retry. The retry
            // compiler returns a valid command so this test distinguishes
            // recipe dispatch from reparsing the non-SQL source as SQLite.
            "SELECT value FROM table_that_is_not_registered"
        } else {
            "SELECT 7"
        };
        compile_only(sql, source.len())
    }
}

/// The cold schema-lookup retry inside `Connection::compile_cmd` is separate
/// from `Statement::reprepare`; both must dispatch the same prepared source.
#[test]
fn compile_cmd_schema_retry_uses_frontend_recipe() -> Result<()> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("frontend-schema-retry.db");
    let io: Arc<dyn IO> = Arc::new(PlatformIO::new()?);
    let db = Database::open_file_with_flags(
        io,
        path.to_str().unwrap(),
        OpenFlags::default(),
        DatabaseOpts::new().with_multiprocess_wal(true),
        None,
        Arc::new(SqliteDialect),
    )?;
    let conn = db.connect()?;
    let frontend = FrontendId::new("retry-synthetic")?;
    let compiler = Arc::new(RetryCompiler::new());
    conn.register_frontend_compiler(frontend.clone(), compiler.clone())?;

    let rows = conn
        .prepare_frontend(&frontend, "SYNTHETIC RETRY")?
        .run_collect_rows()?;

    assert_eq!(rows, vec![vec![Value::from_i64(7)]]);
    assert_eq!(
        compiler.calls.load(Ordering::SeqCst),
        2,
        "schema retry must invoke the frontend compiler again"
    );
    Ok(())
}

/// An absent compiler is a registry/configuration error. Sending source text
/// to SQLite instead would hide the lost frontend identity behind a parse
/// error and make schema-triggered failures nondeterministic.
#[turso_macros::test]
fn missing_frontend_compiler_is_typed_and_never_falls_through_to_sqlite(tmp_db: TempDatabase) {
    let conn = tmp_db.connect_limbo();
    let frontend = FrontendId::new("not-registered").unwrap();

    let err = conn
        .prepare_frontend(&frontend, "MATCH (n) RETURN n")
        .expect_err("an unregistered compiler must be rejected");

    let LimboError::Frontend(FrontendError::CompilerNotRegistered { frontend: actual }) = err
    else {
        panic!("expected typed missing-compiler error, got {err:?}");
    };
    assert_eq!(actual, frontend);
}

/// Compiler identity is stable for a connection. Replacing a registration
/// would let an existing prepared recipe acquire different semantics.
#[turso_macros::test]
fn frontend_compiler_registration_cannot_be_replaced(tmp_db: TempDatabase) -> Result<()> {
    let conn = tmp_db.connect_limbo();
    let frontend = FrontendId::new("synthetic")?;
    conn.register_frontend_compiler(frontend.clone(), Arc::new(SyntheticCompiler::new()))?;

    let err = conn
        .register_frontend_compiler(frontend.clone(), Arc::new(SyntheticCompiler::new()))
        .expect_err("replacing a compiler must be rejected");
    let LimboError::Frontend(FrontendError::CompilerAlreadyRegistered { frontend: actual }) = err
    else {
        panic!("expected typed duplicate-compiler error, got {err:?}");
    };
    assert_eq!(actual, frontend);
    Ok(())
}
