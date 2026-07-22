# Graph Frontend API Alignment Implementation Plan

> **Status: fully executed on 2026-07-21.** Do not re-execute this plan.
> The checkboxes below were tracked externally and are retained unticked
> as written. See "Final state" directly below for what landed.

## Final state (recorded 2026-07-21, post-execution)

**Delivered API** (documented in `graph/README.md` and `docs/graph.md`):
`GraphConnection` (root alias `Connection`) with `prepare`/
`prepare_cancellable` returning the `Statement` wrapper
(`result_types()`, `into_inner()`, `Deref` to `turso_core::Statement`),
`query`/`query_cancellable`, `execute` → `MutationSummary`, `install`,
and the new one-call setup `open_database`/`open_database_with_io` +
`GraphConnection::open`/`open_with_parameters`; types `Error`,
`Result<T>`, `Parameters`; `pub use turso_core as core` plus root
re-exports. All old names (`GraphSession`, `GraphSessionError`,
`MutationParameters`, `prepare_query`, `query_result_types`, `mutate`)
are gone — hard renames, no compat aliases, since nothing outside
`graph/` consumes the crate after the decoupling below.

**Commit trail** on `feature/graph-frontend`:

| Commit | What |
|---|---|
| `178437223` | revert(postgres): removed the `graph.cypher` adapter coupling — precondition that made alias-free renames possible |
| `5a1014c79` | checkpoint of pre-existing branch WIP (kept task commits surgical) |
| `0c5a4ceb6` | this plan document |
| `88cc63ce7` | Task 1 — `Error`/`Result` + core re-exports (see caveat below) |
| `4c147e5c3` | Task 2 — `GraphConnection`/`Connection`/`Parameters` |
| `f966111bd` | Task 3 — `Statement` wrapper, `prepare` rename, `query_result_types` removed |
| `26d26664c` | Task 4 — `mutate` → `execute` |
| `b16750389` | Task 5 — `open_database` + `GraphConnection::open` |
| `ca3947ca4` | repair of pre-existing clippy breakage in `tests/integration/` (gate prerequisite, user-approved scope exception) |
| `44dca7653` | Task 6 — README quickstart + separation/roadmap notes |
| `440a4195f` | final-review follow-up: double-failure snapshot-clear logging + README snippet fix |

**Review outcome:** each task passed an independent spec+quality review;
the final whole-branch review returned "with fixes / explicit sign-off"
and both items were resolved (follow-up commit above; history caveat
accepted). **Known caveat:** `88cc63ce7` is labeled a pure rename but
also carries the pre-existing `session.rs` WIP (temporal-extension
install, four tests, `strip_explain_prefix` rewrite, mutate clear-failure
handling) that the checkpoint split failed to separate for that one
file — accepted as-is by the author; relevant to anyone bisecting.

**Verification at completion:** `cargo clippy --workspace --all-features
--all-targets -- --deny=warnings` exit 0; full workspace `cargo test`
green; `make test` (all 13 targets, incl. 1,843 sqltest conformance
cases) green; `turso_graph_frontend` 117 tests including the new
`api_surface` suite.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align `turso_graph_frontend`'s consumer API (names, types, entry points) with the `turso` (bindings/rust) and `turso_pg` baselines, and close the open-helper / statement-metadata / re-export gaps found in the API review.

**Architecture:** Every change lives under `graph/` — nothing else in the repo may be edited. `turso` (bindings/rust) and `turso_pg` are read-only *reference shapes*. The pg→graph coupling was removed from the branch (commit "revert(postgres): remove graph.cypher adapter coupling from pg frontend"), so no crate outside `graph/` consumes this crate — renames are clean hard renames with no compat aliases. No `turso_core` changes.

**Tech Stack:** Rust; editable crates are `turso_graph_frontend` (`graph/frontend`) and `turso_graph_testkit` (`graph/testkit`) only.

## Global Constraints

- **HARD SCOPE: only files under `graph/` may be created or modified.** `postgres/`, `bindings/`, `core/`, `docs/` are read-only. If a change seems to require touching them, stop and surface it — do not touch them.
- **Hard renames** — no `#[deprecated]` shims, no aliases. All consumers of `turso_graph_frontend` live under `graph/` (verify before Task 2 with `rg -l "turso_graph_frontend" --type rust | grep -v '^graph/'` → must print nothing outside `graph/`).
- Renames (baseline model in parens):
  | Current | New |
  |---|---|
  | `GraphSession` | `GraphConnection`, root alias `Connection` (model: `PgConnection` / `turso_pg::Connection`) |
  | `GraphSessionError` | `Error` + `pub type Result<T>` (model: `turso::Error` / `turso::Result`) |
  | `MutationParameters` | `Parameters` |
  | `prepare_query[_cancellable]` | `prepare[_cancellable]` → returns new `Statement` wrapper |
  | `mutate` | `execute` (still returns `MutationSummary`) |
  | `query_result_types` | **removed**; `Statement::result_types()` |
  | *(new)* | `open_database`, `open_database_with_io` (model: `turso_pg::open_database`, postgres/frontend/session.rs — read-only reference) |
  | *(new)* | `GraphConnection::open`, `::open_with_parameters` |
  | *(new)* | `pub use turso_core as core;` + root re-exports (model: `turso`'s `core` re-export; `turso_pg`'s root re-exports) |
- Every commit: `cargo fmt` first, sign with `git commit -S`, message `type(graph): lowercase imperative`.
- **Scope guard after every task:** `git status --porcelain postgres/ bindings/ core/ docs/` prints nothing.
- Final gate: `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` and `cargo test -p turso_graph_frontend -p turso_graph_testkit` green.
- Never build with `--release`.

**Non-goals (explicitly out of scope):**
- Making mutations preparable/steppable through a single `Statement`. The mutation executor interprets the bound IR pipeline in Rust (mutation.rs:55-130); routing it through one VDBE program is an architecture change, not an API alignment. The rename to `execute` closes the *naming* asymmetry only.
- Building the `bindings/rust`-style async wrapper (Task 6 documents it as a roadmap gap in `graph/README.md`).
- Eliminating the internal double-parse (session parses for the traversal check + types; `prepare_frontend`'s compiler parses again). Removing it requires threading bound queries through core's `FrontendCompilation` — a `turso_core` change, out of scope. This plan removes the *consumer-facing* extra call only.
- Re-introducing any Postgres↔graph adapter. The frontends stay separate; apps compose them on one core `Connection` via `register_frontend_compiler` themselves.

**Caller files under `graph/` that must compile after each rename task** (from `rg` sweep):
`graph/frontend/tests/type_system_fixtures.rs`, `graph/testkit/src/{runner,performance,tck,grafeo,age,cypherbench,rust_donor}.rs`.

---

### Task 1: `Error`/`Result` rename + core re-exports

**Files:**
- Modify: `graph/frontend/src/session.rs` (rename `GraphSessionError` → `Error` at session.rs:15, all in-file uses)
- Modify: `graph/frontend/src/lib.rs` (exports)
- Modify: files under `graph/` matching `rg -n "GraphSessionError" graph --type rust`
- Test: `graph/frontend/tests/api_surface.rs` (create)

**Interfaces:**
- Produces: `turso_graph_frontend::Error` (enum, same variants as old `GraphSessionError`), `turso_graph_frontend::Result<T>`, `turso_graph_frontend::core` (= `turso_core`), root re-exports `Database, DatabaseOpts, LimboError, Numeric, OpenFlags, Row, StepResult, Value`.

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/tests/api_surface.rs`:

```rust
//! Compile-time surface check: the crate exposes baseline-aligned names so
//! consumers do not need a direct `turso_core` dependency for common types.

#[test]
fn baseline_aligned_reexports_are_usable() {
    // Value/Row/StepResult come from the crate root, mirroring turso_pg.
    let v: turso_graph_frontend::Value = turso_graph_frontend::Value::Null;
    assert!(matches!(v, turso_graph_frontend::Value::Null));

    // Full core access via the `core` module, mirroring `turso::core`.
    fn _takes_core_stmt(_s: &turso_graph_frontend::core::Statement) {}
    fn _takes_flags(_f: turso_graph_frontend::OpenFlags) {}

    // Error/Result aliases exist.
    fn _returns_result() -> turso_graph_frontend::Result<()> {
        Ok(())
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend --test api_surface`
Expected: FAIL to compile — `Value`, `core`, `Result` not found in `turso_graph_frontend`.

- [ ] **Step 3: Rename the error enum and add exports**

In `graph/frontend/src/session.rs`: rename `pub enum GraphSessionError` → `pub enum Error` (session.rs:15) and update every in-file mention (`Result<Self, GraphSessionError>`, `GraphSessionError::Database(...)`, `GraphSessionError::UndeclaredParameter/MissingParameter` in `bind_query_parameters`, test module). Keep all variants and `#[from]` impls identical.

In `graph/frontend/src/lib.rs` replace line 38 and append re-exports:

```rust
pub use session::{strip_explain_prefix, Error, GraphSession};

/// Full access to the underlying engine, mirroring `turso`'s `core` re-export.
pub use turso_core as core;
pub use turso_core::{
    Database, DatabaseOpts, LimboError, Numeric, OpenFlags, Row, StepResult, Value,
};

pub type Result<T> = std::result::Result<T, Error>;
```

(`GraphSession` still exported under its old name — renamed in Task 2.)

Update every `GraphSessionError` mention under `graph/` (testkit, tests) to `turso_graph_frontend::Error` — alias in `use` statements as `GraphError` where a local `Error` already exists.

- [ ] **Step 4: Run tests and scope guard**

Run: `cargo test -p turso_graph_frontend --test api_surface && cargo build -p turso_graph_testkit && git status --porcelain postgres/ bindings/ core/ docs/`
Expected: PASS, clean build, empty scope-guard output.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt
git add -A graph
git commit -S -m "refactor(graph): rename GraphSessionError to Error and re-export core types"
```

---

### Task 2: `GraphConnection` + `Connection` alias, `Parameters` rename

**Files:**
- Modify: `graph/frontend/src/session.rs` (struct rename, session.rs:37)
- Modify: `graph/frontend/src/mutation.rs` (type alias rename, mutation.rs:20)
- Modify: `graph/frontend/src/lib.rs` (exports)
- Modify: caller files under `graph/` (see Global Constraints list)
- Test: extend `graph/frontend/tests/api_surface.rs`

**Interfaces:**
- Consumes: `Error`/`Result` from Task 1.
- Produces: `turso_graph_frontend::GraphConnection` (was `GraphSession`, methods unchanged in this task), root alias `turso_graph_frontend::Connection`, `turso_graph_frontend::Parameters` (= `HashMap<String, turso_core::Value>`, was `MutationParameters`).

- [ ] **Step 1: Confirm no out-of-tree consumers**

Run: `rg -l "turso_graph_frontend" --type rust | grep -v '^graph/'`
Expected: no output. If anything prints, stop and surface it — the hard-rename assumption is broken.

- [ ] **Step 2: Write the failing test**

Append to `graph/frontend/tests/api_surface.rs`:

```rust
#[test]
fn session_type_names_match_baseline_convention() {
    // GraphConnection is aliased to Connection at the root, mirroring
    // `pub use session::PgConnection as Connection` in turso_pg.
    fn _takes_conn(_c: &turso_graph_frontend::Connection) {}
    fn _takes_graph_conn(_c: &turso_graph_frontend::GraphConnection) {}

    let params: turso_graph_frontend::Parameters = Default::default();
    assert!(params.is_empty());
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend --test api_surface`
Expected: FAIL to compile — `GraphConnection`, `Connection`, `Parameters` not found.

- [ ] **Step 4: Apply hard renames**

- `graph/frontend/src/session.rs`: `pub struct GraphSession` → `pub struct GraphConnection`; update `impl Drop`, `impl` block, doc comment, module tests. **Naming collision note:** session.rs:4 imports `turso_core::Connection`; keep that import — the root-level `Connection` alias lives only in `lib.rs`, so in-crate code stays unambiguous.
- `graph/frontend/src/mutation.rs`: `pub type MutationParameters = HashMap<String, Value>` → `pub type Parameters = HashMap<String, Value>`; update in-file uses (`execute_cypher_mutation` signature and internals).
- `graph/frontend/src/lib.rs`:

```rust
pub use mutation::{execute_cypher_mutation, MutationError, MutationSummary, Parameters};
pub use session::{strip_explain_prefix, Error, GraphConnection, GraphConnection as Connection};
```

- Callers: sweep `rg -n "GraphSession\b|MutationParameters" graph --type rust` and rename every use (testkit, frontend tests). Old names must no longer exist anywhere: `rg -n "GraphSession\b|MutationParameters" graph --type rust` returns nothing when done.

- [ ] **Step 5: Run tests and scope guard**

Run: `cargo test -p turso_graph_frontend && cargo build -p turso_graph_testkit && git status --porcelain postgres/ bindings/ core/ docs/`
Expected: PASS, clean build, empty scope-guard output.

- [ ] **Step 6: Format and commit**

```bash
cargo fmt
git add -A graph
git commit -S -m "refactor(graph): rename GraphSession to GraphConnection and MutationParameters to Parameters"
```

---

### Task 3: `Statement` wrapper, `prepare` rename, remove `query_result_types`

**Files:**
- Create: `graph/frontend/src/statement.rs`
- Modify: `graph/frontend/src/session.rs` (rename `prepare_query[_cancellable]` → `prepare[_cancellable]` at session.rs:114/145, delete `query_result_types` at session.rs:125-143, thread result types)
- Modify: `graph/frontend/src/lib.rs` (export `Statement`)
- Modify: caller files under `graph/` using `prepare_query` or `query_result_types` (`graph/frontend/tests/type_system_fixtures.rs`, `graph/testkit/src/*`)
- Test: `graph/frontend/tests/api_surface.rs` + existing session tests

**Interfaces:**
- Consumes: `GraphConnection`, `Parameters`, `Error` from Tasks 1–2.
- Produces:
  - `turso_graph_frontend::Statement` — `pub fn result_types(&self) -> &[turso_graph_ir::ValueType]`, `pub fn into_inner(self) -> turso_core::Statement`, `Deref`/`DerefMut` to `turso_core::Statement`.
  - `GraphConnection::prepare(&self, source: &str, parameters: &Parameters) -> Result<Statement>`
  - `GraphConnection::prepare_cancellable(&self, source: &str, parameters: &Parameters, cancellation: &dyn Cancellation) -> Result<Statement>`
  - `GraphConnection::query`/`query_cancellable` keep signatures, now built on the wrapper.

- [ ] **Step 1: Write the failing test**

Append to `graph/frontend/tests/api_surface.rs` (uses an in-memory fixture — copy the `register_graph` + `SchemaCatalog` + `GraphConnection::install` setup pattern from `session.rs`'s `#[cfg(test)]` module into `graph/frontend/tests/fixture.rs`; lift it verbatim rather than inventing a new schema, and do not make private items public just for the test):

```rust
mod fixture;

#[test]
fn prepare_exposes_result_types_on_the_statement() {
    let (connection, session) = fixture::social_graph_connection();
    let stmt = session
        .prepare("MATCH (n:Person) RETURN n.name, n.age", &Default::default())
        .expect("prepare");
    // Metadata rides on the statement — no second parse call.
    assert_eq!(stmt.result_types().len(), 2);
    // Deref gives the full core statement surface.
    assert_eq!(stmt.num_columns(), 2);
    drop(stmt);
    drop(session);
    drop(connection);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend --test api_surface`
Expected: FAIL to compile — no `prepare` method / `result_types` on return value.

- [ ] **Step 3: Add the wrapper and rewire prepare**

Create `graph/frontend/src/statement.rs`:

```rust
use std::ops::{Deref, DerefMut};

use turso_graph_ir::ValueType;

/// Prepared Cypher read statement: the core prepared statement plus the
/// query's static result-column types in projection order.
///
/// Booleans reach storage as integers, so callers that need to render Cypher
/// values faithfully must consult [`Statement::result_types`]. EXPLAIN forms
/// report an empty slice: their output shape belongs to core's
/// `EXPLAIN QUERY PLAN`, not the Cypher projection.
pub struct Statement {
    inner: turso_core::Statement,
    result_types: Vec<ValueType>,
}

impl Statement {
    pub(crate) fn new(inner: turso_core::Statement, result_types: Vec<ValueType>) -> Self {
        Self {
            inner,
            result_types,
        }
    }

    pub fn result_types(&self) -> &[ValueType] {
        &self.result_types
    }

    pub fn into_inner(self) -> turso_core::Statement {
        self.inner
    }
}

impl Deref for Statement {
    type Target = turso_core::Statement;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Statement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
```

In `graph/frontend/src/session.rs`:

1. Change the import at session.rs:4 to `use turso_core::{Connection, Value};` and refer to the core statement as `turso_core::Statement` inside `bind_query_parameters` (its parameter type stays `&mut turso_core::Statement`).
2. Extract the type-computation body of the old `query_result_types` (session.rs:129-142) into a private helper taking the already-parsed syntax — this is what kills the consumer-facing re-parse:

```rust
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
```

3. Delete `pub fn query_result_types` entirely.
4. Rename `prepare_query` → `prepare`, `prepare_query_cancellable` → `prepare_cancellable`; both now return `Result<crate::Statement, Error>`. In `prepare_cancellable`, both exit paths already hold a parsed `syntax`:
   - EXPLAIN path (session.rs:154-175): return `Ok(crate::Statement::new(statement, Vec::new()))`.
   - Normal path (session.rs:177-190): compute `let result_types = self.result_types_for(&syntax)?;` after the traversal-snapshot refresh, then `Ok(crate::Statement::new(statement, result_types))`.
5. `query_cancellable` (session.rs:110) keeps its body — `run_collect_rows` resolves through `DerefMut`.

In `graph/frontend/src/lib.rs`: add `mod statement;` and `pub use statement::Statement;`.

Callers: sweep `rg -n "prepare_query|query_result_types" graph --type rust`; move them to `prepare(..)` / `.result_types()`. `query_result_types(src)` call sites become `prepare(src, &params)?.result_types().to_vec()` using the same parameter map the site already executes with (fixtures always execute the query they type-check; a site with no values constructs the same empty map it would have run with).

- [ ] **Step 4: Run tests and scope guard**

Run: `cargo test -p turso_graph_frontend && cargo test -p turso_graph_frontend --test type_system_fixtures && cargo build -p turso_graph_testkit && git status --porcelain postgres/ bindings/ core/ docs/`
Expected: PASS; empty scope-guard output. The session test module's existing prepare/EXPLAIN/transaction tests must pass with renamed calls only.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt
git add -A graph
git commit -S -m "feat(graph): return statement wrapper with result types from prepare"
```

---

### Task 4: `mutate` → `execute`

**Files:**
- Modify: `graph/frontend/src/session.rs` (session.rs:193)
- Modify: caller files under `graph/` matching `rg -n "\.mutate\(" graph --type rust`
- Test: existing session/mutation tests (renamed calls only)

**Interfaces:**
- Consumes: `GraphConnection`, `Parameters`.
- Produces: `GraphConnection::execute(&self, source: &str, parameters: &Parameters) -> Result<MutationSummary>` — body identical to today's `mutate` (savepoint-wrapped `execute_cypher_mutation` + snapshot clear). `MutationSummary` name and fields unchanged (richer than baseline's `u64` by design — carries Cypher `RETURN` rows).

- [ ] **Step 1: Rename method and graph-tree callers**

In `graph/frontend/src/session.rs` rename `pub fn mutate` → `pub fn execute` (keep body verbatim, including the snapshot-clear warning path). Sweep `rg -n "\.mutate\(" graph --type rust` and rename each call, including the in-crate test module.

- [ ] **Step 2: Run tests and scope guard**

Run: `cargo test -p turso_graph_frontend && cargo build -p turso_graph_testkit && git status --porcelain postgres/ bindings/ core/ docs/`
Expected: PASS — pure rename; empty scope-guard output.

- [ ] **Step 3: Format and commit**

```bash
cargo fmt
git add -A graph
git commit -S -m "refactor(graph): rename GraphConnection::mutate to execute for baseline parity"
```

---

### Task 5: open helpers — `open_database` + `GraphConnection::open`

**Files:**
- Modify: `graph/frontend/src/session.rs` (new free fns + ctors)
- Modify: `graph/frontend/src/lib.rs` (exports)
- Modify: `graph/frontend/Cargo.toml` (only if the build in Step 3 demands core features — mirror the feature list `turso_pg` enables on `turso_core` in `postgres/frontend/Cargo.toml:18-20` (`default-features = true, features = ["conn_raw_api"]`), minus features that turn out unneeded; reading `postgres/` for reference is fine, editing it is not)
- Test: `graph/frontend/tests/api_surface.rs`

**Interfaces:**
- Consumes: `register_graph`/`load_registered_graph` (catalog.rs:125/180), `SchemaCatalog::new` (schema_catalog.rs:22), `SnapshotStore` (snapshot.rs:216), `GraphConnection::install`, `BuildLimits::default()` (turso_graph_runtime), `ParameterTypes` (binder.rs:37).
- Produces:
  - `open_database(path: &str, vfs: Option<&str>, flags: turso_core::OpenFlags, opts: turso_core::DatabaseOpts) -> turso_core::Result<(Arc<dyn turso_core::IO>, Arc<turso_core::Database>)>`
  - `open_database_with_io(io, path, flags, opts) -> turso_core::Result<Arc<turso_core::Database>>`
  - `GraphConnection::open(connection: Arc<turso_core::Connection>, graph_name: &str) -> Result<Self>`
  - `GraphConnection::open_with_parameters(connection, graph_name, parameters: ParameterTypes) -> Result<Self>`

- [ ] **Step 1: Write the failing test**

Append to `graph/frontend/tests/api_surface.rs`:

```rust
#[test]
fn open_replaces_the_install_ceremony() {
    // The fixture registers a graph the long way; a consumer then attaches
    // with one call instead of load + catalog + store + limits + install.
    let (connection, _existing) = fixture::social_graph_connection();
    let conn2 = fixture::second_connection(&connection);
    let session = turso_graph_frontend::GraphConnection::open(conn2, "social")
        .expect("open by graph name");
    let rows = session
        .query("MATCH (n:Person) RETURN n.name", &Default::default())
        .expect("query through opened session");
    assert!(!rows.is_empty());
}
```

(`fixture::second_connection` calls `database.connect()` on the fixture's `Arc<Database>` — extend `graph/frontend/tests/fixture.rs` to also return/hold the `Arc<Database>` so a second connection can be made. `GraphConnection::open` must work on a fresh connection where nothing was installed.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p turso_graph_frontend --test api_surface`
Expected: FAIL to compile — no `open` associated fn.

- [ ] **Step 3: Implement helpers**

In `graph/frontend/src/session.rs` add (above `impl GraphConnection`), mirroring the shape of `turso_pg::open_database` (postgres/frontend/session.rs:70-102, read-only reference) with the SQLite dialect:

```rust
/// Open a database with the default SQLite dialect, resolving the IO backend
/// from `vfs` or the path. Mirrors `turso_pg::open_database`; the graph layer
/// itself is dialect-agnostic and attaches to any core connection.
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

/// Open a database with the default SQLite dialect on an existing IO backend.
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
        turso_core::OpenOptions::new(Arc::new(turso_core::SqliteDialect))
            .storage(db_file)
            .flags(flags)
            .db_opts(opts),
    )
}
```

(Check `core/lib.rs:140-195` for whether `SqliteDialect` is re-exported from the crate root; if not, use its actual public path from `core/dialect/mod.rs:15`. If `storage::database::DatabaseFile` is feature-gated, copy the exact feature list `turso_pg` enables on `turso_core` into `graph/frontend/Cargo.toml`.)

In `impl GraphConnection` add:

```rust
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
    let graph = crate::load_registered_graph(&connection, graph_name)
        .map_err(|error| Error::Database(turso_core::LimboError::ParseError(error.to_string())))?;
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
```

(If `CatalogError` already converts into `Error` via a `#[from]` variant, use `?` instead of the `map_err`; check `Error`'s variants first — do not add a new variant unless the conversion is missing, and if one is needed add `#[error(transparent)] Catalog(#[from] crate::CatalogError)`.)

`ParameterTypes` is already imported at session.rs:10. Export from `graph/frontend/src/lib.rs`:

```rust
pub use session::{
    open_database, open_database_with_io, strip_explain_prefix, Error, GraphConnection,
    GraphConnection as Connection,
};
```

- [ ] **Step 4: Run tests and scope guard**

Run: `cargo test -p turso_graph_frontend --test api_surface && cargo test -p turso_graph_frontend && git status --porcelain postgres/ bindings/ core/ docs/`
Expected: PASS, including a query through the `open`-created session on a second connection; empty scope-guard output.

- [ ] **Step 5: Format and commit**

```bash
cargo fmt
git add -A graph
git commit -S -m "feat(graph): add open_database and GraphConnection::open one-call setup"
```

---

### Task 6: Docs — README quickstart + roadmap gap note (graph tree only)

**Files:**
- Modify: `graph/README.md` (consumer quickstart uses new names; roadmap gap note)

**Interfaces:**
- Consumes: final API from Tasks 1–5.

- [ ] **Step 1: Update `graph/README.md`**

Replace any `GraphSession`/`prepare_query`/`mutate`/`query_result_types` mentions with the new names and add a short quickstart block showing the aligned shape:

```rust
let (io, db) = turso_graph_frontend::open_database("app.db", None, OpenFlags::default(), DatabaseOpts::default())?;
let conn = db.connect()?;
let graph = turso_graph_frontend::GraphConnection::open(conn, "social")?;
let stmt = graph.prepare("MATCH (n:Person) RETURN n.name", &Default::default())?;
let types = stmt.result_types();
let summary = graph.execute("CREATE (:Person {name: $name})", &params)?;
```

Add two short notes in the same README section:

```markdown
- Frontend separation: this crate never depends on, and is never depended on
  by, the Postgres frontend. An app that wants Cypher and Postgres SQL on one
  connection installs both compilers itself via core's
  `Connection::register_frontend_compiler`.
- Roadmap gap: no `bindings/rust`-level ergonomic/async wrapper exists for the
  graph frontend (nor for `turso_pg`); only core SQL has the `turso` crate's
  async `Rows`/`Transaction` surface. Consumers embed this crate synchronously.
```

- [ ] **Step 2: Full verification gate**

Run:
```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend -p turso_graph_testkit
git status --porcelain postgres/ bindings/ core/ docs/
```
Expected: all green; the final `git status` line prints nothing (proves scope held). Report any failure verbatim — do not skip.

- [ ] **Step 3: Commit**

```bash
git add graph/README.md
git commit -S -m "docs(graph): document aligned consumer API and frontend separation"
```
