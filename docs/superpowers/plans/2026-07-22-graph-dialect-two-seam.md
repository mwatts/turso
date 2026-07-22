# GraphDialect Two-Seam Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the graph/Cypher frontend a `GraphDialect` implementing `turso_core::Dialect` — owning database identity, schema-row handling, catalog surface, and the runtime function surface — while statement compilation stays on the existing `FrontendCompiler`/`PreparedSource` path, mirroring exactly how the PostgreSQL frontend (`turso_pg`) already uses both seams.

**Architecture:** The postgres frontend is the vendor precedent: `PostgresDialect: Dialect` (`postgres/frontend/catalog.rs:21`) owns identity/persistence/catalog/functions, and `PostgresCompiler: FrontendCompiler` (`postgres/frontend/session.rs:31`) owns statement compilation, registered per-connection and driven through `Connection::prepare_frontend`. The graph frontend already has the second seam (`GraphCompiler`, `graph/frontend/src/compiler.rs:45`, id `"graph-cypher"`); this plan adds the first seam and switches the graph `open_database` helpers (which today pass `SqliteDialect`) to pass `GraphDialect`, byte-for-byte the `turso_pg::open_database` shape. A database file is 1:1 with its dialect (accepted tradeoff: no intermingled dialects per file; the process-wide registry enforces it by dialect `name()`).

**Tech Stack:** Rust workspace. Crates touched: `graph/frontend` (`turso_graph_frontend`), `graph/temporal` (`turso_graph_temporal`), `graph/testkit` (`turso_graph_testkit`). Core (`turso_core`) is NOT modified — both seams already exist in core.

## Global Constraints

- Never build with `--release` for dev/test (repo CLAUDE.md). Release builds only via `mise run cypherbench-*` for benchmark timing.
- `cargo fmt` before every commit; `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` must pass.
- Commits: signed (`git commit -S`), conventional style `type(scope): lowercase imperative`, body explains intent.
- Every change needs a test that fails without it (repo CLAUDE.md principle 3).
- Preserve unchanged (the "do not lose" list):
  - `GraphConnection` sessions, `ParameterTypes`, `Parameters` (`graph/frontend/src/session.rs`)
  - Snapshot machinery: `SnapshotStore`/`SessionSnapshotStore`, generation-trigger invalidation, `execute` → `snapshots.clear()` (`graph/frontend/src/snapshot.rs`)
  - Mutation orchestration: `execute_cypher_mutation` savepoint wrapping (`graph/frontend/src/mutation.rs:55`)
  - `register_graph` API and its persistent catalog tables (`__turso_internal_graph_*`, junction/registry tables) (`graph/frontend/src/catalog.rs:125`)
  - EXPLAIN handling in `prepare_cancellable` (`graph/frontend/src/session.rs:205`)
  - Dialect-agnostic attach mode: `GraphConnection::install`/`open` on a `SqliteDialect` database must keep working (existing consumers).
  - `DynamicCatalog` in testkit (`graph/testkit/src/dynamic_catalog.rs`)
- Non-goals (explicitly out of scope): Cypher text through `Dialect::parse` (compilation stays on `FrontendCompiler`); Cypher DDL (`CREATE GRAPH …`) and marked schema rows — schema methods delegate to SQLite until graph DDL exists; multi-source-per-kind support; any change to core seams.

## File Structure

- `graph/frontend/src/dialect.rs` — **new**: `GraphDialect` + `turso_graphs` catalog vtab + tests.
- `graph/temporal/src/lib.rs` — **modify**: extract `#[scalar]` bodies into safe `*_impl` fns; add `dispatch()` + `FUNCTION_NAMES`.
- `graph/frontend/src/session.rs` — **modify**: `open_database_with_io` passes `GraphDialect`.
- `graph/frontend/src/lib.rs` — **modify**: `mod dialect; pub use dialect::GraphDialect;`.
- `graph/testkit/src/runner.rs` — **modify**: fixture opens through `turso_graph_frontend::open_database_with_io` (worker-thread only; keep the same `DatabaseOpts`).
- `docs/multi-frontend.md`, `graph/README.md` — **modify**: document the two-seam alignment.

---

### Task 1: GraphDialect skeleton (identity + SQLite delegation)

**Files:**
- Create: `graph/frontend/src/dialect.rs`
- Modify: `graph/frontend/src/lib.rs` (add `mod dialect;` + re-export)
- Test: inline `#[cfg(test)]` in `graph/frontend/src/dialect.rs`

**Interfaces:**
- Consumes: `turso_core::{Dialect, dialect::sqlite}`, `turso_graph_cypher::parse` (already a dependency of `turso_graph_frontend` — used by `compiler.rs`).
- Produces: `pub struct GraphDialect;` implementing `turso_core::Dialect` with `name() == "graph-cypher"`. `pub const GRAPH_DIALECT_NAME: &str = "graph-cypher";` Later tasks add functions/catalog to this same impl.

- [ ] **Step 1: Write the failing test**

In new file `graph/frontend/src/dialect.rs` (module body will be added in step 3; write tests first at the bottom):

```rust
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
        let io: Arc<dyn IO> = Arc::new(MemoryIO::new());
        let _db = open_graph_db(&io, ":memory:gd-registry");
        let err = Database::open_file_with_flags(
            io.clone(),
            ":memory:gd-registry",
            OpenFlags::default(),
            DatabaseOpts::new(),
            None,
            Arc::new(turso_core::SqliteDialect),
        )
        .unwrap_err();
        assert!(err.to_string().contains("already open with dialect"));
    }
}
```

- [ ] **Step 2: Wire the module and run tests to verify they fail**

In `graph/frontend/src/lib.rs`, next to the existing `mod compiler;`-style declarations add:

```rust
mod dialect;
pub use dialect::{GraphDialect, GRAPH_DIALECT_NAME};
```

Run: `cargo test -p turso_graph_frontend dialect -- --nocapture`
Expected: FAIL to compile — `GraphDialect` not defined.

- [ ] **Step 3: Implement `GraphDialect`**

Top of `graph/frontend/src/dialect.rs` (above the tests). Mirror `postgres/frontend/catalog.rs:21-161`; every schema method delegates to SQLite because graph has no frontend DDL of its own yet — the graph catalog persists as ordinary tables written by `register_graph`:

```rust
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

    fn resolve_function(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Result<Option<turso_core::Func>> {
        turso_core::dialect::sqlite::resolve_builtin_function(name, arg_count)
    }

    fn requires_custom_types(&self) -> bool {
        // Graph fixtures and consumers declare `CREATE TYPE duration`;
        // a graph database never opens with the machinery off (same
        // reasoning as PostgresDialect).
        true
    }
}
```

Note for the implementer: check the exact re-export names in `turso_core` before compiling — `Func` and `SqliteDialect` are re-exported at crate root (`core/lib.rs`); `dialect::sqlite::{parse, parse_table_sql_ast, table_sql_for_replay, register_builtin_catalog, resolve_builtin_function}` are the helpers `PostgresDialect` composes with. If `format_table_sql` returning bare `input` trips the round-trip (engine expects canonical text), delegate exactly as SQLite does instead — look at how `SqliteDialect::format_table_sql` renders from the AST (`core/dialect/sqlite.rs`) and copy that call.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend dialect -- --nocapture`
Expected: 3 passed.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-targets -- --deny=warnings
git add graph/frontend/src/dialect.rs graph/frontend/src/lib.rs
git commit -S -m "feat(graph): add GraphDialect implementing the core Dialect seam

Identity and per-database concerns move onto the same seam turso_pg
uses; statement compilation stays on the GraphCompiler FrontendCompiler
path. Schema methods delegate to SQLite because graph catalog state
persists as ordinary tables, not marked DDL."
```

---

### Task 2: Temporal function dispatch surface

**Files:**
- Modify: `graph/temporal/src/lib.rs`
- Test: inline `#[cfg(test)]` in `graph/temporal/src/lib.rs`

**Interfaces:**
- Consumes: existing `#[scalar]` functions (`graph/temporal/src/lib.rs:407` onward, 23 registrations listed at lines 36-59).
- Produces:
  - `pub const FUNCTION_NAMES: &[&str]` — exactly the 23 registered names: `duration_make, duration_parse, duration_get, duration_add, duration_neg, duration_between, temporal_make, temporal_truncate, temporal_parse, temporal_get, temporal_now, datetime_add_duration, datetime_sub_duration, jsonb_get, jsonb_get_text, jsonb_get_path, jsonb_exists, jsonb_exists_any, jsonb_exists_all, jsonb_contains, cypher_raise, cypher_equals, cypher_add, cypher_div`
  - `pub fn dispatch(name: &str, args: &[turso_ext::Value]) -> Option<turso_ext::Value>` — `None` for unknown names.

- [ ] **Step 1: Write the failing test**

At the bottom of `graph/temporal/src/lib.rs`:

```rust
#[cfg(test)]
mod dispatch_tests {
    use super::*;

    #[test]
    fn dispatch_covers_every_registered_name() {
        for name in FUNCTION_NAMES {
            assert!(
                dispatch(name, &[]).is_some(),
                "{name} must dispatch (even if the empty-args result is an error value)"
            );
        }
        assert!(dispatch("no_such_function", &[]).is_none());
    }

    #[test]
    fn dispatch_matches_scalar_behavior() {
        let args = vec![ExtValue::from_text("P1DT25H".to_string())];
        let out = dispatch("duration_parse", &args).expect("known name");
        // duration_parse normalizes but must not carry fields across:
        // P1DT25H keeps 25 hours (module doc, lib.rs:8).
        assert_eq!(out.to_text(), Some("P1DT25H"));
    }
}
```

Note for the implementer: `ExtValue` constructor/accessor names above are illustrative — copy the exact ones from existing tests or usages in this file (search `ExtValue::` in `graph/temporal/src/lib.rs`); assert on whatever accessor the crate actually provides. The invariant being encoded: dispatch reaches the same body as the registered scalar, so Cypher duration semantics (fields never cross) hold through both entries.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_temporal dispatch -- --nocapture`
Expected: FAIL to compile — `FUNCTION_NAMES`/`dispatch` not defined.

- [ ] **Step 3: Implement — mechanical extraction**

For each of the 23 `#[scalar]` functions, rename the body into a private safe impl and keep the macro wrapper one line. Pattern (repeat for all 23):

```rust
#[scalar(name = "duration_parse")]
fn duration_parse(args: &[ExtValue]) -> ExtValue {
    duration_parse_impl(args)
}

fn duration_parse_impl(args: &[ExtValue]) -> ExtValue {
    // ... the exact body previously inside the #[scalar] fn, unchanged ...
}
```

Then add, near `install_temporal_extension`:

```rust
/// Every scalar name `install_temporal_extension` registers, in the same
/// order. `GraphDialect::resolve_function` treats this list as the
/// dialect-owned function surface.
pub const FUNCTION_NAMES: &[&str] = &[
    "duration_make", "duration_parse", "duration_get", "duration_add",
    "duration_neg", "duration_between", "temporal_make", "temporal_truncate",
    "temporal_parse", "temporal_get", "temporal_now", "datetime_add_duration",
    "datetime_sub_duration", "jsonb_get", "jsonb_get_text", "jsonb_get_path",
    "jsonb_exists", "jsonb_exists_any", "jsonb_exists_all", "jsonb_contains",
    "cypher_raise", "cypher_equals", "cypher_add", "cypher_div",
];

/// Execute a temporal/cypher scalar by name outside the extension ABI.
/// Returns `None` for names this crate does not own.
pub fn dispatch(name: &str, args: &[ExtValue]) -> Option<ExtValue> {
    Some(match name {
        "duration_make" => duration_make_impl(args),
        "duration_parse" => duration_parse_impl(args),
        "duration_get" => duration_get_impl(args),
        "duration_add" => duration_add_impl(args),
        "duration_neg" => duration_neg_impl(args),
        "duration_between" => duration_between_impl(args),
        "temporal_make" => temporal_make_impl(args),
        "temporal_truncate" => temporal_truncate_impl(args),
        "temporal_parse" => temporal_parse_impl(args),
        "temporal_get" => temporal_get_impl(args),
        "temporal_now" => temporal_now_impl(args),
        "datetime_add_duration" => datetime_add_duration_impl(args),
        "datetime_sub_duration" => datetime_sub_duration_impl(args),
        "jsonb_get" => jsonb_get_impl(args),
        "jsonb_get_text" => jsonb_get_text_impl(args),
        "jsonb_get_path" => jsonb_get_path_impl(args),
        "jsonb_exists" => jsonb_exists_impl(args),
        "jsonb_exists_any" => jsonb_exists_any_impl(args),
        "jsonb_exists_all" => jsonb_exists_all_impl(args),
        "jsonb_contains" => jsonb_contains_impl(args),
        "cypher_raise" => cypher_raise_impl(args),
        "cypher_equals" => cypher_equals_impl(args),
        "cypher_add" => cypher_add_impl(args),
        "cypher_div" => cypher_div_impl(args),
        _ => return None,
    })
}
```

If the `#[scalar]` macro rejects a one-line delegating body (check `turso_macros`' expansion by compiling), instead move the macro attribute onto the impl-calling wrapper unchanged and verify `install_temporal_extension` still compiles — the registration list at lines 36-59 must not change.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_temporal`
Expected: dispatch tests pass, all pre-existing temporal tests still pass.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p turso_graph_temporal --all-targets -- --deny=warnings
git add graph/temporal/src/lib.rs
git commit -S -m "refactor(graph): expose temporal scalars via a safe dispatch surface

Extract each #[scalar] body into a plain fn and add dispatch()/
FUNCTION_NAMES so the same implementations can back both the
per-connection extension registration and a dialect function surface."
```

---

### Task 3: Dialect-owned function surface

**Files:**
- Modify: `graph/frontend/src/dialect.rs` (extend `resolve_function`, add `exec_scalar_function`)
- Modify: `graph/frontend/Cargo.toml` only if `turso_graph_temporal` is not already a dependency (check first — `session.rs` calls `turso_graph_temporal::install_temporal_extension`, so it almost certainly is).
- Test: inline in `graph/frontend/src/dialect.rs`

**Interfaces:**
- Consumes: `turso_graph_temporal::{dispatch, FUNCTION_NAMES}` (Task 2), `turso_core::Value::{to_ffi, from_ffi}` (`core/types.rs:604,657`), `turso_core::Func::Dialect`.
- Produces: any connection to a `GraphDialect` database resolves the 24 temporal/cypher scalars with **no** `install_temporal_extension` call. Per-connection registration keeps working under `SqliteDialect` (dialect-agnostic mode) and is shadowed-but-consistent under `GraphDialect` (dialect resolution runs before extension functions — `core/dialect/mod.rs:130-132`); both paths share one implementation via `dispatch`.

- [ ] **Step 1: Write the failing test**

Add to `tests` module in `graph/frontend/src/dialect.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend dialect::tests::temporal_functions_resolve_without_extension_install`
Expected: FAIL — "no such function: duration_parse".

- [ ] **Step 3: Implement**

Replace `resolve_function` and add `exec_scalar_function` in the `impl Dialect for GraphDialect`:

```rust
    fn resolve_function(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Result<Option<turso_core::Func>> {
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
        let ext_args: Vec<turso_ext::Value> =
            args.iter().map(turso_core::Value::to_ffi).collect();
        let out = turso_graph_temporal::dispatch(name, &ext_args).ok_or_else(|| {
            turso_core::LimboError::ParseError(format!("no such function: {name}"))
        })?;
        turso_core::Value::from_ffi(out)
    }
```

Add `turso_ext` to `graph/frontend/Cargo.toml` dependencies if not present (workspace dep, same version key the temporal crate uses). Check whether `Value::to_ffi` leaks (FFI values often own allocations freed by the callee) — read how core frees `ExtValue` after extension calls (`core/types.rs` around `from_ffi`, and the extension call site in `core/vdbe/execute.rs`, search `to_ffi`); mirror that exact free/ownership pattern here. If core uses an explicit `ExtValue::__free` or similar on args after the call, do the same in a `defer`-style block.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend dialect`
Expected: all dialect tests pass, including the new one.

- [ ] **Step 5: Verify both entry paths agree**

Add and run this test (same file):

```rust
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
```

Run: `cargo test -p turso_graph_frontend dialect`
Expected: PASS.

- [ ] **Step 6: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-targets -- --deny=warnings
git add graph/frontend/src/dialect.rs graph/frontend/Cargo.toml
git commit -S -m "feat(graph): resolve temporal scalars through GraphDialect

Any connection to a graph-dialect database gets the cypher/temporal
function surface without install_temporal_extension; both the dialect
and extension entry points share one implementation via dispatch()."
```

---

### Task 4: `turso_graphs` catalog virtual table

**Files:**
- Modify: `graph/frontend/src/dialect.rs` (vtab types + `register_catalog`)
- Test: inline in `graph/frontend/src/dialect.rs`

**Interfaces:**
- Consumes: `turso_core::{VirtualTable, InternalVirtualTable, InternalVirtualTableCursor}` (pattern: `TestCatalogTable` in `core/dialect/mod.rs:333-404` and pg's connection-reading vtabs in `postgres/frontend/catalog.rs`); graph catalog table names from `graph/frontend/src/catalog.rs` (`__turso_internal_graph_graphs`, `__turso_internal_graph_sources`, `__turso_internal_graph_node_sources`, `__turso_internal_graph_relationship_sources`, and `turso_core::schema::TURSO_GRAPH_GENERATIONS_TABLE_NAME` — confirm the exact constant names exported by `catalog.rs` and reuse them; do not re-string them).
- Produces: `SELECT * FROM turso_graphs` on any graph-dialect connection returns one row per registered source with columns `(graph_id INTEGER, graph_name TEXT, generation INTEGER, kind TEXT, source_name TEXT, table_name TEXT, identity_column TEXT, start_column TEXT, end_column TEXT)`; empty when no graph is registered.

- [ ] **Step 1: Write the failing test**

```rust
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
```

(Adjust the registration struct field spellings to the actual `GraphRegistration`/`NodeSourceRegistration`/`RelationshipSourceRegistration` definitions in `graph/frontend/src/catalog.rs:30-68` — the fixture in `graph/testkit/src/runner.rs:200-218` shows a working literal to copy.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend dialect::tests::turso_graphs_vtab`
Expected: FAIL — "no such table: turso_graphs".

- [ ] **Step 3: Implement the vtab**

In `graph/frontend/src/dialect.rs`:

```rust
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
        conn: std::sync::Arc<turso_core::Connection>,
    ) -> Result<
        std::sync::Arc<turso_core::sync::RwLock<dyn turso_core::InternalVirtualTableCursor>>,
    > {
        Ok(std::sync::Arc::new(turso_core::sync::RwLock::new(
            TursoGraphsCursor { conn, rows: Vec::new(), row: usize::MAX },
        )))
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
                .map(|_| turso_ext::ConstraintUsage { argv_index: None, omit: false })
                .collect(),
        })
    }
}

struct TursoGraphsCursor {
    conn: std::sync::Arc<turso_core::Connection>,
    rows: Vec<Vec<turso_core::Value>>,
    row: usize,
}

impl TursoGraphsCursor {
    fn load(&mut self) -> Result<()> {
        // The catalog tables only exist once register_graph has run.
        let mut probe = self.conn.prepare_internal(
            "SELECT 1 FROM sqlite_schema WHERE type = 'table' \
             AND name = '__turso_internal_graph_graphs'",
        )?;
        if probe.run_collect_rows()?.is_empty() {
            self.rows = Vec::new();
            return Ok(());
        }
        let sql = format!(
            "SELECT g.id, g.name, COALESCE(gen.generation, 0), s.kind, s.name, \
                    COALESCE(ns.table_name, rs.table_name), \
                    COALESCE(ns.identity_column, rs.identity_column), \
                    rs.start_column, rs.end_column \
             FROM __turso_internal_graph_graphs g \
             LEFT JOIN \"{generations}\" gen ON gen.graph_id = g.id \
             JOIN __turso_internal_graph_sources s ON s.graph_id = g.id \
             LEFT JOIN __turso_internal_graph_node_sources ns ON ns.source_id = s.id \
             LEFT JOIN __turso_internal_graph_relationship_sources rs ON rs.source_id = s.id \
             ORDER BY g.id, s.id",
            generations = turso_core::schema::TURSO_GRAPH_GENERATIONS_TABLE_NAME,
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
```

And extend `register_catalog`:

```rust
    fn register_catalog(
        &self,
        schema: &mut turso_core::schema::Schema,
        enable_custom_types: bool,
    ) -> Result<()> {
        turso_core::dialect::sqlite::register_builtin_catalog(schema, enable_custom_types)?;
        let vtab = turso_core::VirtualTable::new_internal(
            "turso_graphs".to_string(),
            TURSO_GRAPHS_VTAB_SQL.to_string(),
            turso_ext::VTabKind::VirtualTable,
            std::sync::Arc::new(turso_core::sync::RwLock::new(TursoGraphsTable)),
        )?;
        schema.add_virtual_table(std::sync::Arc::new(vtab))
    }
```

Match the exact trait-method signatures against `core/lib.rs`'s `InternalVirtualTable`/`InternalVirtualTableCursor` definitions (the core-test `TestCatalogTable` at `core/dialect/mod.rs:333` is the authoritative shape) — copy discrepancies from there, not from this plan. If `catalog.rs` exports constants for the internal table names (`GRAPHS_TABLE` etc., `catalog.rs:17-18`), make them `pub(crate)` and use them in `load()` instead of string literals.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend dialect`
Expected: all pass, including empty-before-registration and two-rows-after.

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-targets -- --deny=warnings
git add graph/frontend/src/dialect.rs graph/frontend/src/catalog.rs
git commit -S -m "feat(graph): expose registered graphs through a turso_graphs catalog vtab

register_catalog installs it on every schema build and rebuild, the
same lifecycle pg_catalog tables get, so graph registration state is
inspectable from any connection without the frontend API."
```

---

### Task 5: Switch the graph open path to GraphDialect

**Files:**
- Modify: `graph/frontend/src/session.rs:61` (`open_database_with_io`: `Arc::new(turso_core::SqliteDialect)` → `Arc::new(crate::GraphDialect)`) and the doc comments at `session.rs:33-34`
- Test: `graph/frontend/tests/fixture.rs` or inline in `session.rs` tests (follow where `open_database` is currently tested — check `graph/frontend/tests/api_surface.rs`)

**Interfaces:**
- Consumes: `GraphDialect` (Task 1), existing `open_database`/`open_database_with_io` (`session.rs:35,50`).
- Produces: `turso_graph_frontend::open_database(path, vfs, flags, opts)` yields a database whose dialect is `"graph-cypher"`, mirroring `turso_pg::open_database` exactly. `GraphConnection::install`/`open` signatures unchanged. Attach-mode (caller opens with `SqliteDialect` themselves and calls `GraphConnection::install`) still supported and documented.

- [ ] **Step 1: Write the failing test**

In the file that already tests `open_database` (or new `#[cfg(test)]` block in `session.rs`):

```rust
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
```

(`social_registration()` = the same `GraphRegistration` literal as Task 4's test; extract it as a test helper if both live in one crate. `tempfile` is already a workspace dev-dependency — confirm in `graph/frontend/Cargo.toml`, add under `[dev-dependencies]` if missing.)

- [ ] **Step 2: Run tests to verify current failure mode**

Run: `cargo test -p turso_graph_frontend open_database_pins`
Expected: FAIL — "no such function: duration_parse" (because today's open path passes `SqliteDialect`).

- [ ] **Step 3: Implement the switch**

In `session.rs`, in `open_database_with_io`, replace the dialect argument:

```rust
        turso_core::OpenOptions::new(Arc::new(crate::GraphDialect))
```

Update the function doc comments: `open_database` opens with the graph dialect (the `turso_pg::open_database` mirror is now exact); note attach-mode explicitly:

```rust
/// Open a database with the graph-cypher schema dialect, resolving the IO
/// backend from `vfs` or the path like [`turso_core::Database::open_new`].
///
/// This is the graph mirror of `turso_pg::open_database`. To attach the
/// graph layer to an existing SQLite-dialect database instead, open it
/// yourself and use [`GraphConnection::install`]/[`GraphConnection::open`];
/// in that mode call `turso_graph_temporal::install_temporal_extension`
/// per connection (GraphConnection::install already does).
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p turso_graph_frontend`
Expected: new tests pass; the whole frontend suite stays green (fixtures that open via `SqliteDialect` + `install` are unaffected — attach mode).

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt
cargo clippy -p turso_graph_frontend --all-targets -- --deny=warnings
git add graph/frontend/src/session.rs graph/frontend/tests/ graph/frontend/Cargo.toml
git commit -S -m "feat(graph): open_database pins GraphDialect

The graph open helpers now mirror turso_pg::open_database exactly:
dialect baked into the crate-level open path, one dialect per database
file enforced by the process registry. Attach mode on SQLite-dialect
databases remains supported via GraphConnection::install."
```

---

### Task 6: Migrate the testkit fixture and validate the full suite

**Files:**
- Modify: `graph/testkit/src/runner.rs:171-186` (`build_fixture_with_io`)
- Test: the entire existing graph test surface is the test.

**Interfaces:**
- Consumes: `turso_graph_frontend::open_database_with_io` (Task 5 behavior).
- Produces: every testkit fixture (TCK, corpus, age, grafeo, cypherbench) runs on a `GraphDialect` database — the whole graph conformance surface becomes the dialect's regression suite.

- [ ] **Step 1: Make the change**

In `build_fixture_with_io`, replace the `Database::open_file_with_flags(..., Arc::new(SqliteDialect))` call (runner.rs:179-186) with:

```rust
    let database = turso_graph_frontend::open_database_with_io(
        io,
        path,
        OpenFlags::default(),
        DatabaseOpts::new().with_custom_types(true),
    )
    .map_err(|error| RunnerError::Fixture(error.to_string()))?;
```

Remove the now-unused `SqliteDialect` import if nothing else uses it. Keep the explicit `turso_graph_temporal::install_temporal_extension(&connection)` call — it is harmless (dialect resolution shadows it) and still exercised by attach-mode consumers; deleting it here would silently stop covering the extension path.

- [ ] **Step 2: Run the graph unit/integration suites**

Run: `cargo test -p turso_graph_testkit -p turso_graph_frontend -p turso_graph_temporal`
Expected: PASS. Failure triage rule: a failure here is a real behavioral difference between the dialects (most likely function-resolution precedence or custom-types enablement) — debug it, do not revert the fixture.

- [ ] **Step 3: Run the conformance suites through the testkit binary**

Run:
```bash
cargo run -q -p turso_graph_testkit -- run smoke --no-record
cargo run -q -p turso_graph_testkit -- corpus --no-record
```
Expected: same pass counts as the last recorded runs in `graph/test-results/runs.jsonl` (compare the most recent record for each suite; zero new failures).

- [ ] **Step 4: Benchmark sanity (release, per the benchmark rule)**

Run: `mise run cypherbench-sample`
Expected: nba/company-scale load times in the ~150-200ms band and match counts equal to the latest release rows in `graph/test-results/benchmarks.jsonl` (2026-07-22 rows: nba 25q sample matched 25, company matched 13/25). This also appends a fresh record — keep it, it documents the dialect switch.

- [ ] **Step 5: Workspace-wide lint and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
git add graph/testkit/src/runner.rs graph/test-results/benchmarks.jsonl
git commit -S -m "test(graph): run all testkit fixtures on GraphDialect

The conformance surface (TCK, corpus, donor suites, cypherbench) now
regression-tests the dialect seam on every run. Temporal extension
install stays in the fixture so the attach-mode path keeps coverage."
```

---

### Task 7: Documentation

**Files:**
- Modify: `docs/multi-frontend.md` (the frontend comparison — it already documents the pg/graph seam difference; update it to the two-seam convergence)
- Modify: `graph/README.md` (consumer-facing: when to use `open_database` vs attach mode)
- Test: `cargo test -p turso_graph_frontend --doc` if doc examples are added; otherwise `rg` sanity that no doc still claims graph "does not use Dialect".

- [ ] **Step 1: Update `docs/multi-frontend.md`**

Find the section describing how each frontend attaches to core (search for `FrontendCompiler` and `Dialect` mentions). Replace the graph-vs-pg contrast with the aligned two-seam table:

```markdown
Both non-SQLite frontends now use the same two seams:

| Seam | Scope | PostgreSQL | Graph/Cypher |
|---|---|---|---|
| `Dialect` (per-database) | identity, schema rows, catalog vtabs, function surface | `PostgresDialect` (`postgres/frontend/catalog.rs`) | `GraphDialect` (`graph/frontend/src/dialect.rs`) |
| `FrontendCompiler` (per-connection) | statement compilation, prerequisites, reprepare | `PostgresCompiler` via `prepare_frontend("postgres")` | `GraphCompiler` via `prepare_frontend("graph-cypher")` |

Differences that remain by design: graph schema rows are unmarked SQLite
DDL (graph catalog state lives in `__turso_internal_graph_*` tables, not
in marked `sqlite_schema` text), and Cypher never enters `Dialect::parse`
— a Cypher statement on a raw connection gets an error pointing at
`GraphConnection`. The graph layer additionally supports attach mode on a
SQLite-dialect database, which pg does not.
```

- [ ] **Step 2: Update `graph/README.md`**

In the consumer/usage section, document the two open modes:

```markdown
## Opening a graph database

Preferred — dialect-pinned database (mirrors `turso_pg::open_database`):

    let (_io, db) = turso_graph_frontend::open_database(path, None, flags, opts)?;
    let conn = db.connect()?;
    turso_graph_frontend::register_graph(&conn, &registration)?;   // first time only
    let graph = turso_graph_frontend::GraphConnection::open(conn, "social")?;

The dialect gives you: `"graph-cypher"` database identity (mismatched
reopens rejected), the temporal/cypher function surface on every
connection, custom types always on, and the `turso_graphs` catalog
virtual table.

Attach mode — graph layer on an existing SQLite-dialect database:

    let session = GraphConnection::open(existing_conn, "social")?;

`GraphConnection::install` registers the per-connection compiler and the
temporal extension; nothing about the database file changes.
```

- [ ] **Step 3: Sweep stale claims**

Run: `rg -n "dialect-agnostic|does not use Dialect|SqliteDialect" docs/ graph/ --glob '!target' -g '!*.jsonl'`
Update any doc line (including the `session.rs:33-34` doc comment if Task 5 missed it, and `graph/memory-observability.md` if it mentions the open path) that still describes the graph frontend as SQLite-dialect-only.

- [ ] **Step 4: Commit**

```bash
git add docs/multi-frontend.md graph/README.md
git commit -S -m "docs(graph): document the two-seam dialect alignment with turso_pg"
```

---

## Verification (whole-plan)

- `cargo test -p turso_graph_frontend -p turso_graph_temporal -p turso_graph_testkit` — green.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` — clean.
- `cargo run -q -p turso_graph_testkit -- run smoke --no-record` and `corpus --no-record` — pass counts match `graph/test-results/runs.jsonl` latest records.
- `mise run cypherbench-sample` — match counts and load times consistent with the 2026-07-22 release rows in `benchmarks.jsonl`.
- Manual: `tursodb`-independent check that a graph DB file opened with plain `SqliteDialect` in a *fresh process* still works (attach mode, file has no on-disk dialect stamp — the registry constraint is per-process only). This is intentional; note it in the Task 7 docs if surprising.
