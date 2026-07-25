# Graph dialect Core-alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the graph frontend (`graph/*`) into closer line with how Postgres uses Core: one clear dialect path, one compile pass for reads, unified function/catalog install, and a cleaner mutation path — without putting Cypher into Core.

**Architecture:** Keep the two-seam model (`GraphDialect` + `GraphCompiler`). Prefer graph-only changes that reuse existing Core APIs (`prepare_frontend`, `prepare_internal`, `prepare_translated_stmt`, internal virtual tables). Touch `turso_core` only when a shared multi-frontend primitive is required and justified. Follow the findings in [`docs/graph-frontend-core-alignment.md`](../../graph-frontend-core-alignment.md).

**Tech Stack:** Rust workspace; crates `turso_graph_frontend`, `turso_graph_temporal`, `turso_graph_testkit`; optional small `turso_core` change only in Task 8. Tests: `cargo test -p turso_graph_frontend`, `cargo test -p turso_graph_temporal`, selected testkit smoke.

## Global Constraints

- Never build with `--release` for normal work (`AGENTS.md` / `CLAUDE.md`).
- `cargo fmt` before every commit; `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` must pass for touched crates at minimum; workspace preferred at end of phase.
- Commits: signed (`git commit -S`); message style `[scope: ]imperative summary` per `AGENTS.md` (example: `graph/frontend: share one cypher compile pass for prepare`).
- Every change needs a test that fails without it.
- Do **not** reintroduce a Postgres↔graph crate dependency.
- Do **not** put Cypher grammar, Neo4j store, or graph catalog layout into Core.
- Do **not** emit VDBE / `ProgramBuilder` from `graph/`.
- Preserve attach mode: `GraphConnection::install` / `open` on a `SqliteDialect` database must keep working.
- Preserve mutation savepoint atomicity and snapshot clear-after-mutate.
- Do not rewrite complex multi-stage mutations “for purity”; only add a simple single-program path when tests prove a closed subset.
- Scope default is `graph/**` and docs. Core only in Task 8 if Tasks 2–3 prove insufficient.

## Spec map (alignment → tasks)

| Alignment path | Task(s) |
|----------------|---------|
| P0 Document contracts | Task 1 |
| P1 Single-pass compile + EXPLAIN | Tasks 2–3 |
| P3 Function / catalog unification | Tasks 4–5 |
| P2 Mutation convergence (hygiene + simple case) | Tasks 6–7 |
| Optional Core multi-frontend primitive | Task 8 (gate) |
| P5 Index-friendly lowering discipline | Task 9 (regression only) |
| P4 Product surfaces (async/CLI/Bolt) | **Out of scope** for this plan |

## File structure

| File | Role after this plan |
|------|----------------------|
| `docs/graph.md` | Documents dialect-pinned open vs attach; function/catalog guarantees |
| `graph/README.md` | Same contracts; points at alignment doc |
| `docs/graph-frontend-core-alignment.md` | Status line: which paths this plan implements |
| `graph/frontend/src/compiler.rs` | Shared compile cache / one Cypher pass; optional result types |
| `graph/frontend/src/session.rs` | Holds `Arc<GraphCompiler>`; uses cache; EXPLAIN via AST; install rules |
| `graph/frontend/src/dialect.rs` | Optional expand registration hook; tests for dialect-only temporal path |
| `graph/frontend/src/graph_expand.rs` | Safe dual install (dialect catalog + connection attach) |
| `graph/frontend/src/mutation.rs` | `prepare_internal` for helper SQL; optional simple single-program path |
| `graph/frontend/src/lowering.rs` | Simple-mutation AST lowerers if Task 7 lands |
| `graph/temporal/src/lib.rs` | Unchanged unless Task 4 needs a pure “dialect owns all names” helper export |
| `graph/frontend/tests/api_surface.rs` or new `tests/dialect_alignment.rs` | End-to-end alignment regressions |
| `core/frontend.rs` | **Only Task 8:** optional metadata on `FrontendCompilation` |

---

### Task 1: Document open modes and mutation debt

**Files:**
- Modify: `docs/graph.md`
- Modify: `graph/README.md`
- Modify: `docs/graph-frontend-core-alignment.md` (add “Implementation plan” pointer at top)
- Test: docs are the deliverable; add a short unit test only if a public constant is introduced (none required)

**Interfaces:**
- Consumes: open/install behaviour already in `session.rs` / `dialect.rs`
- Produces: written contract for Task 4–5 install rules

- [ ] **Step 1: Write the contract section into `docs/graph.md`**

Add a section titled `## Open modes and Core seams` after the quickstart. Required facts (use STE-style short sentences):

1. **Dialect-pinned open** — `open_database` / `open_database_with_io` open with `GraphDialect` (`name() == "graph-cypher"`). Temporal functions resolve on the dialect. `turso_graphs` is registered on schema build.
2. **Attach mode** — `GraphConnection::open` / `install` on an existing connection (often `SqliteDialect`). Guarantees come from `install` (compiler registration, temporal extension, expand vtab). File dialect name may stay `"sqlite"`.
3. **Reads** — go through `prepare_frontend("graph-cypher")`.
4. **Mutations** — multi-statement orchestration under a savepoint today; not a single `PreparedSource`. This is known debt, not an accident.
5. **Composition** — Postgres and Graph stay separate crates; apps register both compilers on one core connection if needed.

- [ ] **Step 2: Mirror a shorter version in `graph/README.md`**

Under “Opening a graph database”, add a bullet list that matches the five facts above. Link to `docs/graph-frontend-core-alignment.md`.

- [ ] **Step 3: Point the alignment doc at this plan**

At the top of `docs/graph-frontend-core-alignment.md`, after Status, add:

```markdown
**Implementation plan:** [`docs/superpowers/plans/2026-07-25-graph-dialect-core-alignment.md`](superpowers/plans/2026-07-25-graph-dialect-core-alignment.md)
```

- [ ] **Step 4: Commit**

```bash
git add docs/graph.md graph/README.md docs/graph-frontend-core-alignment.md docs/superpowers/plans/2026-07-25-graph-dialect-core-alignment.md
git commit -S -m "$(cat <<'EOF'
docs(graph): record dialect open modes and mutation debt

Align consumer docs with the Core-alignment findings so attach mode
and multi-prepare mutations are explicit contracts.
EOF
)"
```

---

### Task 2: One Cypher compile pass for read prepare

**Files:**
- Modify: `graph/frontend/src/compiler.rs`
- Modify: `graph/frontend/src/session.rs`
- Test: `graph/frontend/src/session.rs` (inline) and/or `graph/frontend/tests/dialect_alignment.rs`

**Interfaces:**
- Consumes: `bind`, `lower_relational`, `FrontendCompiler::compile`
- Produces:
  - `GraphCompiler` kept as `Arc<GraphCompiler>` in `GraphConnection`
  - `GraphCompiler::peek_last_compile()` or equivalent that returns `result_types` and `needs_snapshot` for the source just prepared
  - Session `prepare_cancellable` no longer calls `result_types_for` as a second full bind when the cache hits

**Design (graph-only, no Core change):**

```rust
// compiler.rs — sketch
struct CompileOutcome {
    source: String,
    cmd: turso_parser::ast::Cmd,
    result_types: Vec<turso_graph_ir::ValueType>,
    needs_snapshot: bool,
}

pub struct GraphCompiler {
    graph: ir::GraphId,
    catalog: SharedGraphCatalog,
    parameters: ParameterTypes,
    last: parking_lot::Mutex<Option<CompileOutcome>>,
}

impl GraphCompiler {
    /// Parse, bind, lower once. Cache by exact source string.
    pub(crate) fn compile_outcome(&self, source: &str) -> Result<CompileOutcome, LimboError> { /* ... */ }

    pub(crate) fn take_result_types_for(&self, source: &str) -> Option<Vec<ir::ValueType>> { /* ... */ }
}

impl FrontendCompiler for GraphCompiler {
    fn compile(&self, source: &str) -> Result<FrontendCompilation> {
        let outcome = self.compile_outcome(source)?;
        // store in last
        Ok(FrontendCompilation {
            prerequisites: Vec::new(),
            cmd: Some(outcome.cmd),
            consumed: source.len(),
        })
    }
}
```

Session keeps `compiler: Arc<GraphCompiler>` and registers **the same** `Arc` with Core:

```rust
connection.register_frontend_compiler(graph_frontend_id(), compiler.clone())?;
```

`prepare_cancellable` flow:

1. `refresh_catalog_if_stale`
2. If EXPLAIN prefix → Task 3 path (leave a stub that still works with old logic until Task 3)
3. `let outcome = self.compiler.compile_outcome(source)?` **or** rely on `prepare_frontend` then `take_result_types_for`
4. If `needs_snapshot`, refresh snapshots
5. `prepare_frontend` (hits cache / same compile)
6. Build `Statement` with cached `result_types`

**Important:** Cache must key on exact source text and invalidate when catalog generation changes (clear `last` in `refresh_catalog_if_stale` when generation bumps).

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/tests/dialect_alignment.rs` (or add to existing integration module). Use the social fixture pattern from `tests/fixture.rs` / `native_capabilities.rs`.

```rust
//! Alignment regressions for GraphDialect + compile seams.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Count bind calls only if you add a test hook; otherwise assert behaviour:
// prepare returns correct types and rows without a public bind counter.
// Minimal behavioural test:

#[test]
fn prepare_returns_cypher_result_types_for_boolean_projection() {
    let graph = /* open GraphConnection on a registered graph with a boolean property or
                   use RETURN true AS flag via Cypher literal if supported */;
    let stmt = graph
        .prepare("MATCH (n:Person) RETURN n.name AS name", &Default::default())
        .expect("prepare");
    let types = stmt.result_types();
    assert!(
        !types.is_empty(),
        "result_types must come from the single compile path"
    );
    // Types length matches column count
    assert_eq!(types.len(), 1);
}
```

Also add a unit test in `compiler.rs` that two calls to `compile_outcome` with the same source reuse the same outcome identity (for example equal `result_types` and equal lowered `cmd` string), and that a catalog generation clear drops the cache.

If you need a bind counter for a hard fail-without-fix, add:

```rust
#[cfg(test)]
pub(crate) static BIND_COUNT: AtomicUsize = AtomicUsize::new(0);
// increment once inside bind path used by GraphCompiler::compile_outcome only
```

Then the test asserts `BIND_COUNT` increases by 1 across `prepare` of one query (not 2+).

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p turso_graph_frontend --test dialect_alignment prepare_returns_cypher
```

Expected: FAIL (missing file, missing types, or bind count ≥ 2 if hook exists).

- [ ] **Step 3: Implement `CompileOutcome` cache on `GraphCompiler`**

Implement `compile_outcome`, store `last`, clear on catalog swap. Change `FrontendCompiler::compile` to use `compile_outcome`. Keep public API of `GraphCompiler::new` / `with_shared` working for tests that construct compilers directly.

- [ ] **Step 4: Hold `Arc<GraphCompiler>` on `GraphConnection` and wire prepare**

In `install`:

```rust
let compiler = Arc::new(GraphCompiler::with_shared(
    graph.id,
    catalog.clone(),
    parameters.clone(),
));
connection.register_frontend_compiler(graph_frontend_id(), compiler.clone())?;
// store compiler on Self
```

In `prepare_cancellable` (non-EXPLAIN):

```rust
// Optional pre-pass for snapshot: use compile_outcome needs_snapshot
let outcome = self.compiler.compile_outcome(source).map_err(/* to Error */)?;
if outcome.needs_snapshot {
    self.snapshots.refresh_visible_if_stale(/* ... */)?;
}
let mut statement = self
    .connection
    .prepare_frontend(&graph_frontend_id(), source)?;
bind_query_parameters(&mut statement, parameters)?;
let result_types = self
    .compiler
    .take_result_types_for(source)
    .unwrap_or_default();
Ok(crate::Statement::new(statement, result_types))
```

Remove the separate `result_types_for` call from the hot path (keep helper for EXPLAIN/debug if still useful, or delete if unused).

- [ ] **Step 5: Run tests**

```bash
cargo test -p turso_graph_frontend --test dialect_alignment
cargo test -p turso_graph_frontend --lib
cargo test -p turso_graph_frontend --test api_surface
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add graph/frontend/src/compiler.rs graph/frontend/src/session.rs graph/frontend/tests/dialect_alignment.rs
git commit -S -m "$(cat <<'EOF'
graph/frontend: share one cypher compile pass for prepare

Cache GraphCompiler bind/lower outcome so result types and prepare_frontend
reuse one parse, matching Postgres's single compiler pass for reads.
EOF
)"
```

---

### Task 3: EXPLAIN uses one lower path without dialect reparse of Cypher

**Files:**
- Modify: `graph/frontend/src/session.rs` (`prepare_cancellable` EXPLAIN branch)
- Test: `graph/frontend/tests/dialect_alignment.rs`, reuse patterns from `native_capabilities.rs`

**Interfaces:**
- Consumes: `compile_outcome` / `lower_relational` result as `ast::Stmt`
- Produces: EXPLAIN statements prepared via Core on **SQL** `EXPLAIN QUERY PLAN …` text produced from the lowered AST only (never re-parse Cypher through the dialect as if it were Cypher)

Current code lowers then:

```rust
self.connection.prepare(format!("EXPLAIN QUERY PLAN {statement}"))?;
```

That is acceptable if `{statement}` is the engine AST `Display` of pure SQL. Task 3 makes this explicit and routes through the same `compile_outcome` as Task 2 so Cypher is not parsed three times.

Target flow:

```rust
if let Some(inner) = strip_explain_prefix(source) {
    let outcome = self.compiler.compile_outcome(inner)?;
    if outcome.needs_snapshot { /* refresh */ }
    let sql = match outcome.cmd {
        turso_parser::ast::Cmd::Stmt(stmt) => format!("EXPLAIN QUERY PLAN {stmt}"),
        other => format!("EXPLAIN QUERY PLAN {other}"),
    };
    let mut statement = self.connection.prepare(sql)?;
    // Prefer prepare_translated_stmt only if Core accepts EXPLAIN-wrapped forms;
    // if not, keep prepare(sql) with pure SQL text (dialect is SQLite-compatible).
    bind_query_parameters(&mut statement, parameters)?;
    return Ok(crate::Statement::new(statement, Vec::new()));
}
```

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn explain_match_returns_core_eqp_rows() {
    let graph = /* social GraphConnection */;
    let rows = graph
        .query(
            "EXPLAIN MATCH (n:Person) RETURN n.name",
            &Default::default(),
        )
        .expect("explain");
    assert!(
        !rows.is_empty(),
        "EXPLAIN must return EQP rows from core"
    );
    // Optional: assert at least one cell contains SCAN or SEARCH or similar
}
```

If EXPLAIN already works, change the test to assert compile cache behaviour (bind count +1 for EXPLAIN only) so the task still has a fail-without-fix.

- [ ] **Step 2: Run test**

```bash
cargo test -p turso_graph_frontend --test dialect_alignment explain_match
```

- [ ] **Step 3: Implement EXPLAIN via `compile_outcome`**

Remove duplicate parse/bind/lower in the EXPLAIN branch.

- [ ] **Step 4: Run tests**

```bash
cargo test -p turso_graph_frontend --test dialect_alignment
cargo test -p turso_graph_frontend --test native_capabilities
```

- [ ] **Step 5: Commit**

```bash
git commit -S -m "$(cat <<'EOF'
graph/frontend: route EXPLAIN through shared compile outcome

Reuse one Cypher lower for EXPLAIN QUERY PLAN and avoid a second
session-side bind path.
EOF
)"
```

---

### Task 4: Temporal install only when the host dialect needs it

**Files:**
- Modify: `graph/frontend/src/session.rs` (`GraphConnection::install`)
- Modify: `graph/frontend/src/dialect.rs` tests if needed
- Test: `graph/frontend/tests/dialect_alignment.rs`, existing dialect tests in `dialect.rs`

**Interfaces:**
- Consumes: `connection.db` dialect name if available, or `GraphDialect` / `SqliteDialect` detection
- Produces (superseded by final review fix): dialect is source of truth for Root under GraphDialect; **always** call `install_temporal_extension` on install for InternalHelper mutation SQL safety (not “zero installs on dialect open”)

**How to detect host dialect:**

Prefer a public Core API if one exists (`connection` → database dialect `name()`). If not exposed, add a small graph-side helper:

```rust
fn host_dialect_name(connection: &Connection) -> &str {
    // Use whatever public accessor Core already exposes.
    // As of this plan, check Connection / Database for dialect().name().
    // If none is public, keep install_temporal_extension always on attach
    // but skip when open_database was used by tracking a flag on GraphConnection.
}
```

**Pragmatic approach (no Core API change):**

Track mode on the session:

```rust
pub enum GraphHostMode {
    /// open_database* with GraphDialect — dialect owns Func::Dialect temporal path
    DialectPinned,
    /// install/open on foreign dialect — need static extension
    Attach,
}
```

- `open_database` + `GraphConnection::open` on that db → `DialectPinned`
- `GraphConnection::install` on arbitrary connection → `Attach` (call `install_temporal_extension`)

Wire `open` path: after `open_database`, `open` uses `DialectPinned` and **still installs** the temporal extension (InternalHelper mutation symbols; Root still dialect-owned).
`install` keeps extension install for attach.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn dialect_pinned_open_runs_duration_without_second_extension_install() {
    // open via turso_graph_frontend::open_database_with_io + register_graph + GraphConnection::open
    // Query that lowers to duration_* / temporal_* must succeed.
    // Assert GraphHostMode is DialectPinned if exported under cfg(test), or
    // simply assert the cypher/SQL path works and document that install was skipped
    // via a test-only counter on install_temporal_extension (optional hook).
}
```

Add to `graph/temporal` under `cfg(test)`:

```rust
#[cfg(test)]
pub static INSTALL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn install_temporal_extension(connection: &Connection) {
    #[cfg(test)]
    INSTALL_COUNT.fetch_add(1, Ordering::SeqCst);
    // existing body
}
```

Dialect-pinned `GraphConnection::open` after this task: `INSTALL_COUNT` stays 0 for that path. Attach path increments.

- [ ] **Step 2: Run test (expect fail if open still always installs)**

```bash
cargo test -p turso_graph_frontend --test dialect_alignment dialect_pinned_open
```

- [ ] **Step 3: Implement host mode + conditional install**

- [ ] **Step 4: Run dialect unit tests that require no extension on GraphDialect**

```bash
cargo test -p turso_graph_frontend --lib temporal_functions_resolve
cargo test -p turso_graph_frontend --test dialect_alignment
```

- [ ] **Step 5: Commit**

```bash
git commit -S -m "$(cat <<'EOF'
graph/frontend: skip temporal extension on GraphDialect open

Dialect-pinned opens already resolve temporal scalars via GraphDialect;
install the static extension only in attach mode.
EOF
)"
```

---

### Task 5: Register expand vtab with the dialect when dialect-pinned

**Files:**
- Modify: `graph/frontend/src/dialect.rs`
- Modify: `graph/frontend/src/graph_expand.rs`
- Modify: `graph/frontend/src/session.rs` (`install_graph_catalog` call sites)
- Test: `graph/frontend/tests/dialect_alignment.rs`

**Interfaces:**
- Consumes: `register_graph_catalog(schema, snapshots)` (already in `graph_expand.rs`)
- Produces: `GraphDialect::register_catalog` installs `__turso_graph_expand` when a process-global or dialect-held `SnapshotStore` is available **or** documents that expand still requires session install because snapshots are connection-scoped

**Reality check:** Expand needs a `SnapshotStore`. Today that is per session/process. Dialect `register_catalog` runs at schema build **without** a connection snapshot.

**Chosen design for this plan (safe, incremental):**

1. Keep `install_graph_catalog(connection, snapshots)` for attach and for all sessions (source of truth for expand).
2. Make install **idempotent** and documented.
3. Add `GraphDialect::register_catalog` only if you can install a **placeholder** expand table that fails cleanly until session install binds snapshots — **or** skip dialect registration and instead document why expand stays session-scoped.

**Preferred deliverable if snapshot cannot be dialect-global:**

- Do **not** force expand into `register_catalog`.
- Instead: document in `docs/graph.md` that expand is session-activated (like attach temporal install).
- Strengthen `install_graph_catalog` idempotency tests.

If a shared `SnapshotStore` already exists at process level and sessions register into it (see `SessionSnapshotStore` / `SnapshotStore::register_session`), then:

```rust
// dialect.rs register_catalog — only if SnapshotStore can be empty-default and session later binds
schema.register_internal_vtab(GraphExpandTable { snapshots: SnapshotStore::global() })?;
```

Only take this path if existing snapshot design already has a global default. Otherwise choose the document + idempotent install path and mark Task 5 complete with tests for idempotent install.

- [ ] **Step 1: Write failing test for idempotent expand install**

```rust
#[test]
fn install_graph_catalog_is_idempotent() {
    let conn = /* connection */;
    let store = Arc::new(SnapshotStore::default());
    install_graph_catalog(&conn, store.clone()).unwrap();
    install_graph_catalog(&conn, store).unwrap(); // must not error
}
```

- [ ] **Step 2: Implement idempotency + docs**

- [ ] **Step 3: Run tests**

```bash
cargo test -p turso_graph_frontend --test dialect_alignment install_graph_catalog
cargo test -p turso_graph_frontend --lib
```

- [ ] **Step 4: Commit**

```bash
git commit -S -m "$(cat <<'EOF'
graph/frontend: make graph expand catalog install idempotent

Clarify session-scoped expand activation versus GraphDialect catalog
ownership so attach and dialect-pinned opens stay consistent.
EOF
)"
```

---

### Task 6: Mutation helper SQL uses `prepare_internal`

**Files:**
- Modify: `graph/frontend/src/mutation.rs` (`run_ignore`, `run_rows`, and any direct `connection.prepare` / `connection.execute` for engine-generated SQL)
- Test: existing mutation tests must stay green; add one focused test that mutation still works under GraphDialect

**Interfaces:**
- Consumes: `Connection::prepare_internal`
- Produces: internal mutation SQL prepared with `StatementOrigin::InternalHelper` semantics (SQLite function resolution, no user-dialect surprises)

Replace:

```rust
let mut statement = connection.prepare(sql)?;
```

with:

```rust
let mut statement = connection.prepare_internal(sql)?;
```

in `run_ignore` and `run_rows`.

Keep user-facing transaction/savepoint control via `connection.execute` only if that is the public API today for `SAVEPOINT` / `RELEASE` / `ROLLBACK TO`. If `execute` routes through dialect parse, that is fine for `SAVEPOINT` text (SQLite-compatible).

- [ ] **Step 1: Write / identify regression test**

```rust
#[test]
fn simple_create_mutation_commits_under_graph_dialect() {
    // open GraphDialect db, register graph, GraphConnection::open
    // execute CREATE (:Person {id: 1, name: 'Ada'})
    // query MATCH and assert name
}
```

- [ ] **Step 2: Change `run_ignore` / `run_rows` to `prepare_internal`**

- [ ] **Step 3: Run mutation-heavy tests**

```bash
cargo test -p turso_graph_frontend --lib
cargo test -p turso_graph_frontend --test fixed_pattern_fixtures
cargo test -p turso_graph_frontend --test dialect_alignment
```

- [ ] **Step 4: Commit**

```bash
git commit -S -m "$(cat <<'EOF'
graph/frontend: prepare mutation helper sql as internal

Use prepare_internal for generated mutation SQL so helper statements
follow engine/SQLite function resolution like other frontends.
EOF
)"
```

---

### Task 7: Single-program path for a closed simple-mutation subset

**Files:**
- Modify: `graph/frontend/src/mutation.rs`
- Modify: `graph/frontend/src/lowering.rs` (if a single AST lowerer is needed)
- Modify: `graph/frontend/src/compiler.rs` only if mutations gain a frontend compile entry (optional)
- Test: `graph/frontend/tests/dialect_alignment.rs` + unit tests in `mutation.rs`

**Interfaces:**
- Consumes: `bind_mutation`, existing CreateNode/Set/Delete lowerers
- Produces: `try_execute_as_single_statement(...) -> Option<MutationSummary>` that returns `None` for unsupported shapes and falls back to the multi-prepare path

**Closed subset (v1 — do not expand mid-task):**

Supported **only** when all of these hold:
1. No multi-stage `WITH` after mutation start
2. No `MERGE`
3. No `FOREACH`
4. No variable-length patterns
5. Single operation kind in one of: one `CREATE` node, one `SET` property, one `DELETE` node/rel **or** a small fixed list you implement fully in this task
6. Optional simple `RETURN` of literals/properties that lower to one `INSERT…RETURNING`-style or `SELECT` after DML **only if** Core supports the shape; otherwise require no RETURN for v1

**Algorithm:**

```rust
pub fn execute_cypher_mutation(...) -> Result<MutationSummary, MutationError> {
    let syntax = parse(source)?;
    let bound = bind_mutation(...)?;
    if let Some(summary) = try_single_program_mutation(connection, catalog, &bound, parameters)? {
        return Ok(summary);
    }
    // existing savepoint multi-prepare path
}
```

`try_single_program_mutation`:
1. Pattern-match bound IR for supported shape
2. Build one `ast::Stmt` (or one SQL string prepared via `prepare_internal`)
3. Run once; map row changes into `MutationSummary`
4. On any unsupported feature, return `Ok(None)`

- [ ] **Step 1: Write failing tests for the subset**

```rust
#[test]
fn single_create_node_uses_single_program_path() {
    // Prefer a cfg(test) counter SINGLE_PROGRAM_HITS on the success path.
    let graph = /* ... */;
    graph.execute(
        "CREATE (:Person {id: 42, name: 'Grace'})",
        &Default::default(),
    ).unwrap();
    assert_eq!(SINGLE_PROGRAM_HITS.load(Ordering::SeqCst), 1);
    let rows = graph.query(
        "MATCH (n:Person {id: 42}) RETURN n.name",
        &Default::default(),
    ).unwrap();
    assert_eq!(rows[0][0], Value::build_text("Grace"));
}

#[test]
fn multi_stage_mutation_still_uses_savepoint_path() {
    // A mutation that includes WITH stages must not increment SINGLE_PROGRAM_HITS
    // (or increments 0) and must still succeed if previously supported.
}
```

- [ ] **Step 2: Implement detector + one CREATE node lowerer end-to-end**

Start with **CREATE single node** only if that is the smallest green path. Add SET/DELETE only if CREATE lands cleanly in the same task; otherwise split a follow-up commit still under Task 7 checkboxes.

- [ ] **Step 3: Run tests**

```bash
cargo test -p turso_graph_frontend --test dialect_alignment single_create
cargo test -p turso_graph_frontend --lib
# Optional deeper gate:
cargo run -q -p turso_graph_testkit -- run smoke --no-record
```

- [ ] **Step 4: Commit**

```bash
git commit -S -m "$(cat <<'EOF'
graph/frontend: lower simple create mutations to one program

Route a closed CREATE-node subset through a single prepare_internal
statement while keeping multi-stage mutations on the savepoint path.
EOF
)"
```

---

### Task 8 (gate): Core `FrontendCompilation` metadata — only if needed

**Do this task only if** Tasks 2–3 still force a fragile compiler-side cache that breaks reprepare or multi-threaded prepare.

**Files:**
- Modify: `core/frontend.rs` (`FrontendCompilation`)
- Modify: `core/connection.rs` if metadata must attach to `Statement`
- Modify: `graph/frontend/src/compiler.rs`, `session.rs`
- Test: `core` unit tests + graph dialect_alignment

**Interfaces (proposed):**

```rust
// core/frontend.rs
pub struct FrontendCompilation {
    pub prerequisites: Vec<Stmt>,
    pub cmd: Option<Cmd>,
    pub consumed: usize,
    /// Opaque bytes for the frontend; Core does not interpret them.
    pub frontend_payload: Vec<u8>,
}
```

Or, cleaner multi-frontend design:

```rust
pub struct FrontendCompilation {
    pub prerequisites: Vec<Stmt>,
    pub cmd: Option<Cmd>,
    pub consumed: usize,
    pub result_type_names: Vec<String>, // optional logical names
}
```

Graph encodes Cypher `ValueType` as stable strings; session maps back.

**Gate criteria (must document in commit body):**
- Reprepare loses types with Task 2 cache, **or**
- Concurrent prepare on one connection corrupts `last` outcome, **or**
- Review rejects Arc cache as non-reprepare-safe

If gate fails (cache is fine), mark Task 8 cancelled in the plan checkboxes and skip.

- [x] **CANCELLED (gate not met, 2026-07-25):** Task 2–3 shared `Arc<GraphCompiler>` `CompileOutcome` cache is reprepare-safe; reviews APPROVED; no proof that reprepare loses types or concurrent prepare corrupts `last`. No Core metadata. See `.superpowers/sdd/2026-07-25-graph-dialect-core-alignment/task-8-report.md`.
- [ ] ~~Step 1: Prove the gate with a failing test that only Core metadata fixes~~
- [ ] ~~Step 2: Minimal Core API + graph consumer~~
- [ ] ~~Step 3: `cargo test -p turso_core --lib` and graph tests~~
- [ ] ~~Step 4: Commit Core and graph separately if possible~~

---

### Task 9: Index-friendly lowering regression lock

**Files:**
- Test only: extend `graph/frontend/tests/native_capabilities.rs` or `dialect_alignment.rs`
- Reference: `graph/MAIN_MERGE_LEVERAGE.md`

**Interfaces:** None new — lock existing pure `count(*)` covering behaviour.

- [ ] **Step 1: Ensure these tests exist and pass**

```bash
cargo test -p turso_graph_frontend --test native_capabilities pure_count_star_uses_junction
cargo test -p turso_graph_frontend --test native_capabilities relationship_table_count
```

- [ ] **Step 2: If missing, re-add the assertions from `MAIN_MERGE_LEVERAGE.md`**

- [ ] **Step 3: Commit only if tests were added**

```bash
git commit -S -m "$(cat <<'EOF'
graph/frontend: lock covering-count lowering regressions

Keep labeled and relationship pure count(*) shapes eligible for core
covering indexes after dialect alignment work.
EOF
)"
```

---

### Task 10: Phase verification and status update

**Files:**
- Modify: `docs/graph-frontend-core-alignment.md` (mark implemented paths)
- Modify: this plan file checkboxes as done during execution

- [x] **Step 1: Run the verification suite** (2026-07-25)

```bash
cargo fmt
cargo test -p turso_graph_frontend
cargo test -p turso_graph_temporal
cargo test -p turso_graph_testkit
# If Core changed:
cargo test -p turso_core --lib
cargo clippy -p turso_graph_frontend -p turso_graph_temporal --all-targets --all-features -- --deny=warnings
```

Graph package tests green. Clippy on graph packages blocked by pre-existing `turso_core` unused-import denials (not introduced by this plan).

- [x] **Step 2: Update alignment doc §8 roadmap** (2026-07-25)

Mark P0, P1, P3 hygiene, and partial P2 as done with date and plan link. Leave full multi-stage mutation convergence and P4 product surfaces open.

- [x] **Step 3: Final commit**

```bash
git commit -S -m "$(cat <<'EOF'
docs(graph): record completed core-alignment plan outcomes

Update the alignment report with paths shipped by the graph dialect
alignment plan.
EOF
)"
```

---

## Out of scope (explicit)

| Item | Why |
|------|-----|
| Async `turso`-style wrapper for Graph/PG | Product surface; separate plan |
| Bolt / Cypher REPL / wire server | Product surface; separate plan |
| Full mutation multi-stage single VDBE program | Too large; Task 7 is the wedge |
| `WITH RECURSIVE` in Core | GraphExpand already covers traversal |
| Frontend-scoped function resolve in Core | Multi-frontend Core project; not required for dialect-pinned Graph |
| Namespace isolation / polyglot host dialect | Separate multi-frontend Core project |
| Cypher DDL / marked schema rows | Deferred until graph owns DDL language |

---

## Self-review (plan quality)

| Alignment requirement | Task coverage |
|----------------------|---------------|
| Document attach vs dialect open | Task 1 |
| Single-pass read compile | Task 2 |
| EXPLAIN one path | Task 3 |
| Temporal dual-path cleanup | Task 4 |
| Expand/catalog install clarity | Task 5 |
| Mutation `prepare_internal` | Task 6 |
| Simple mutation single program | Task 7 |
| Core metadata if cache unsafe | Task 8 (gated) |
| Covering index discipline | Task 9 |
| Verify + report status | Task 10 |

No TBD placeholders in steps. Types and entry points named from current tree (`GraphCompiler`, `prepare_frontend`, `prepare_internal`, `install_temporal_extension`, `install_graph_catalog`).

---

## Execution notes

- Prefer **one task per commit** as written.
- If Task 7 CREATE-only lands but SET/DELETE do not, leave follow-ups as new plan items rather than blocking Task 10.
- When in doubt, re-read [`docs/graph-frontend-core-alignment.md`](../../graph-frontend-core-alignment.md) §7.3 non-goals before proposing Core changes.
