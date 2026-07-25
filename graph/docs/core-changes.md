# Core changes introduced by the graph frontend branch

**Branch:** `feature/graph-frontend`  
**Compared to:** `origin/main`  
**Scope:** every path under `core/` that differs on this branch, why it
exists, and how it relates to the graph crates under `graph/*`.

This is branch-local documentation. It is not a substitute for
[`docs/multi-frontend.md`](../../docs/multi-frontend.md) or
[`docs/graph.md`](../../docs/graph.md); it inventories **what this branch
changed in the engine** so reviewers and agents can separate intentional
graph integration from incidental merge noise.

## Summary

| Area | Files (approx.) | Intent |
|---|---|---|
| Multi-frontend prepare | `frontend.rs` (new), `connection.rs`, `statement.rs`, `error.rs`, `lib.rs`, `vdbe/*` glue | Cypher (and Postgres) compile through a connection-local compiler registry; reprepare keeps original source |
| Graph catalog protection | `schema.rs` constants, `translate/{mod,index,trigger,update}.rs` | Reserve `__turso_internal_graph_*` names; allow generation triggers to write protected tables |
| Resumable graph expand | `vtab.rs`, `vdbe/execute.rs` | Internal vtab steps can `Yield` so long traversals cooperate with the VDBE IO loop |
| Shared type classification | `schema.rs` (`classify_column`), `statement.rs` | One column-type classifier for SQL result metadata and Cypher property binding |
| Observability | `pragma.rs`, `translate/pragma.rs`, `storage/pager.rs` | `PRAGMA memory_stats` for graph/perf measurement |
| Small glue / hygiene | `json/cache.rs`, incremental/blob paths, builder/explain | `PreparedSource` plumbing; lint stability |

**Design rule:** graph crates never emit VDBE opcodes. They register a
`FrontendCompiler`, lower Cypher to SQL (or use graph expand as an
internal virtual table), and rely on core for transactions, storage, and
bytecode. Core changes below exist only where that boundary needs an
engine hook.

At last inventory vs `origin/main`: **24 files**, roughly **+644 / −94**
lines under `core/`.

---

## 1. Multi-frontend preparation boundary

### 1.1 `core/frontend.rs` (new)

**What**

- `FrontendId` — non-empty stable name for a connection-local compiler
  (graph uses a fixed Cypher id; Postgres uses its own).
- `FrontendCompiler` trait — `compile(&str) -> FrontendCompilation`.
  Must be `Send + Sync + 'static` and **must not** store connection or
  statement state (reprepare-safe).
- `FrontendCompilation` — one-pass result:
  - `prerequisites: Vec<Stmt>` — run only on **initial**
    `prepare_frontend` (e.g. Postgres implicit `CREATE SEQUENCE` for
    `SERIAL`). Discarded on reprepare because recompile can run mid-step
    while pager locks are held.
  - `cmd: Option<Cmd>` — main Turso AST command.
  - `consumed: usize` — same byte-offset contract as dialect parse.
- `PreparedSource` enum — data-only recipe retained on the prepared
  program:
  - `Dialect { source }`
  - `Frontend { frontend, source }`
- `FrontendError` — typed registry/compile failures (empty id, not
  registered, already registered, no statement on reprepare, bad
  consumed offset).

**Why**

Without this, Cypher would have to be prepared as opaque SQL only, or
the engine would have to know Cypher. The branch keeps **language
frontends out of the bytecode interpreter** while still supporting:

1. `prepare_frontend` for Cypher text,
2. schema-change / cross-process **reprepare** that re-runs the same
   frontend compiler on the original source,
3. coexistence of SQLite dialect, Postgres frontend, and graph frontend
   on one connection without embedding any frontend crate into core.

Shared with the Postgres frontend on the same branch lineage; graph is a
consumer of the same registry.

### 1.2 `core/connection.rs`

**What**

- `frontend_compilers: RwLock<HashMap<FrontendId, Arc<dyn FrontendCompiler>>>`
  on the connection.
- `register_frontend_compiler` / `unregister_frontend_compiler` —
  connection-local; double-register rejected so a prepared source cannot
  silently change meaning.
- `prepare_frontend(frontend, source)` — compile once, run
  prerequisites, prepare the main command, store
  `PreparedSource::Frontend { … }` on the program.
- `compile_prepared_source` / `compile_frontend_source` /
  `frontend_compiler` — reprepare path; does **not** re-run
  prerequisites; unlocks the registry before calling into the compiler
  so compilers cannot deadlock on the registry lock.
- `in_write_transaction()` — public probe for write txn state.

**Why (graph)**

- `GraphConnection::install` registers the Cypher compiler; `Drop`
  unregisters so sessions do not leak compilers on shared connections.
- Catalog registration and generation maintenance use **internal**
  nested statements that cannot upgrade a deferred read transaction to
  a write. Callers must check `in_write_transaction()` (or open an
  explicit write) before catalog DDL; the API makes that check honest.

### 1.3 `core/statement.rs`, `core/vdbe/mod.rs`, `core/vdbe/builder.rs`, `core/vdbe/explain.rs`

**What**

- Programs carry `prepared_source: PreparedSource` instead of a bare
  SQL string.
- Reprepare / “no statement” errors include the frontend id when the
  source was frontend-prepared.
- `get_column_type_info` delegates leaf classification to
  `Schema::classify_column` (see §3).
- Subprogram builds (FK actions, triggers) construct
  `PreparedSource::dialect(...)` explicitly.

**Why**

Reprepare after schema change must recompile **the same language** the
client prepared. Storing only rewritten SQL would lose Cypher (and
Postgres) provenance and force incorrect dialect reparse.

### 1.4 `core/error.rs`, `core/lib.rs`

**What**

- `FrontendError` integrated into the core error surface.
- `mod frontend` + re-exports of frontend types and
  `InternalVirtualTableStep`.
- Connection constructor initializes an empty frontend compiler map.

**Why**

Public API surface for bindings and graph/postgres crates without
reaching into private modules.

### 1.5 Incidental `PreparedSource` call-site updates

Files such as `core/incremental_blob.rs`,
`core/incremental/expr_compiler.rs`, `core/vdbe/blob_io_tests.rs`,
`core/translate/fkeys.rs`, and `core/translate/trigger_exec.rs` pass
`PreparedSource::dialect(...)` (or equivalent) where builders previously
took a plain string.

**Why**

Mechanical follow-through of the prepare API change—not graph-specific
logic.

---

## 2. Graph catalog: reserved names and generation triggers

### 2.1 `core/schema.rs` — constants

```text
TURSO_GRAPH_CATALOG_PREFIX            = "__turso_internal_graph_"
TURSO_GRAPH_GENERATIONS_TABLE_NAME    = "__turso_internal_graph_generations"
TURSO_GRAPH_GENERATION_TRIGGER_PREFIX = "__turso_internal_graph_gen_"
```

**Why**

Graph registration creates junction tables, registries, and generation
counters with stable internal names. User SQL must not create or drop
objects that collide with those names, and generation triggers must be
allowed to update the generations table even though it is otherwise a
protected system-style object.

### 2.2 `core/translate/index.rs`, `trigger.rs`

**What**

- `translate_drop_index(..., internal: bool)` — non-internal drops of
  system/reserved index names are rejected.
- `translate_create_trigger` / `translate_drop_trigger` take
  `internal: bool` and reject non-internal trigger names under
  `TURSO_GRAPH_CATALOG_PREFIX`.

**Why**

Graph catalog DDL runs as internal statements. Ordinary clients must not
hijack or delete graph catalog objects by name.

### 2.3 `core/translate/update.rs`

**What**

- `validate_update` gains `is_internal_graph_trigger`.
- An UPDATE is treated as an internal graph generation write when the
  target is `TURSO_GRAPH_GENERATIONS_TABLE_NAME` **and** the firing
  trigger name starts with `TURSO_GRAPH_GENERATION_TRIGGER_PREFIX`.

**Why**

Source-table DML bumps a generation counter via AFTER triggers so
traversal snapshots know when to rebuild. Those trigger bodies update a
table that would otherwise reject user DML. The exception is **narrow**
(table + trigger-name prefix), not a blanket unlock of system tables.

### 2.4 `core/translate/mod.rs`

**What**

- `translate` / program build take `PreparedSource` end-to-end.
- DDL helpers pass the `internal` flag into drop-index / drop-trigger
  (and related) paths so catalog maintenance works under internal
  prepare.

**Why**

Wires frontend-aware prepare and internal catalog DDL into one place.

### 2.5 Index drop error preservation (`translate/index.rs`)

**What**

- Related fix: constraint index drop errors are not swallowed when
  cleaning catalog indexes.

**Why**

Silent drop failure corrupts catalog teardown and confuses graph
registration rollback. Correctness for transactional graph catalog.

---

## 3. Shared column type classification

### 3.1 `Schema::classify_column` (`core/schema.rs`)

**What**

Extracts classification of a table column’s declared type into
`ColumnTypeInfo` (builtin / domain / struct / union / custom, base type,
array dimensions), including `CREATE TYPE` / `CREATE DOMAIN` chain
resolution. Unit tests cover builtin and strict custom struct columns.

**Why**

Two callers need the same rules:

1. SQL `Statement::get_column_type_info` (result metadata),
2. Graph `SchemaCatalog` / semantic binding (Cypher property types over
   SQLite columns).

Duplicating type-chain resolution in the graph crate would drift from
core. One implementation, two entry points.

### 3.2 `core/statement.rs`

**What**

Result-column typing calls `schema.classify_column` instead of an
inlined copy of the logic.

**Why**

Keeps SQL and graph paths consistent after the extract.

---

## 4. Resumable internal virtual tables (graph expand)

### 4.1 `core/vtab.rs`

**What**

- `InternalVirtualTableStep { Row, Done, Yield }`.
- `VirtualTableCursor::next` / `filter` return a step, not only `bool`.
- `InternalVirtualTableCursor::next_step` / `filter_step` with default
  adapters that map synchronous `bool` → `Row`/`Done` so existing
  internal tables stay simple.
- Merge with main: filter args remain `crate::alloc::Vec<Value>`
  (allocator-backed staging from main) while preserving the step enum
  (graph requirement).

**Why**

Variable-length path expansion (`__turso_graph_expand`) can do a lot of
CPU work per VDBE instruction. Cooperative **Yield** lets the expand
cursor return control to the VDBE IO / cancel path without blocking the
connection indefinitely. Non-graph internal tables are unchanged in
behavior via the default adapters.

### 4.2 `core/vdbe/execute.rs` (`op_vfilter`, `op_vnext`)

**What**

- On `Yield`, return `InsnFunctionStepResult::IO(IOCompletions(
  Completion::new_yield()))`.
- On `Done` / `Row`, preserve previous empty-vs-row PC semantics.
- After main’s `IOCompletions` unit-struct change, construction is
  `IOCompletions(completion)` (not a former `::Single` enum variant).

**Why**

Makes the step enum observable at the opcode layer. Without this, a
yielding expand cursor could not integrate with the existing IO
completion / yield machinery.

---

## 5. Memory observability

### 5.1 `core/storage/pager.rs`, `core/translate/pragma.rs`, `core/pragma.rs`

**What**

- `Pager::memory_stats()` — point-in-time pages / capacity / page size /
  WAL frames.
- `PRAGMA memory_stats` translation and registration.

**Why**

Graph benchmarks and memory-observability work on the branch need a
stable, engine-level snapshot of pager memory without instrumenting only
the graph crates. Used by branch docs/tools such as
[`graph/memory-observability.md`](../memory-observability.md).

---

## 6. Minor / hygiene

| File | Change | Why |
|---|---|---|
| `core/json/cache.rs` | Stable lint allowance | Keep clippy clean under workspace deny-warnings |
| `core/vdbe/builder.rs`, `explain.rs` | `PreparedSource` plumbing | Compile/explain paths match prepare API |
| Merge-only touch points | Auto-merged with main recycling / covering-index work | Keep graph hooks while accepting main VDBE/types work; see [`MAIN_MERGE_LEVERAGE.md`](../MAIN_MERGE_LEVERAGE.md) |

---

## 7. What core does **not** do (boundary reminder)

- Core does **not** parse Cypher, own graph IR, or implement openCypher
  semantics beyond hooks listed above.
- Core does **not** own CSR / snapshot layout; that lives in
  `graph/runtime` and session overlays in `graph/frontend`.
- Graph still lowers most reads/writes to SQL; expand is the main
  non-SQL internal vtab path.
- Frontend compilers must remain connection-stateless for reprepare
  safety (catalog handles are outside the compiler object, or are
  shared read-only snapshots).

---

## 8. File checklist vs `origin/main`

| Path | Role |
|---|---|
| `core/frontend.rs` | New frontend compiler API |
| `core/connection.rs` | Registry, `prepare_frontend`, write-txn probe |
| `core/error.rs` | `FrontendError` wiring |
| `core/lib.rs` | Module exports, connection init |
| `core/statement.rs` | Frontend reprepare errors; `classify_column` use |
| `core/schema.rs` | Graph name constants; `classify_column` |
| `core/translate/mod.rs` | `PreparedSource` + internal DDL flags |
| `core/translate/index.rs` | Internal drop index / reserved names |
| `core/translate/trigger.rs` | Reserved graph trigger names |
| `core/translate/trigger_exec.rs` | `PreparedSource` on subprograms |
| `core/translate/update.rs` | Generation-trigger UPDATE exception |
| `core/translate/fkeys.rs` | `PreparedSource` on FK subprograms |
| `core/translate/pragma.rs` | `memory_stats` |
| `core/pragma.rs` | Pragma table entry |
| `core/storage/pager.rs` | `memory_stats()` |
| `core/vtab.rs` | `InternalVirtualTableStep`, step APIs |
| `core/vdbe/execute.rs` | Yield on VFilter/VNext |
| `core/vdbe/mod.rs` | `prepared_source` on program |
| `core/vdbe/builder.rs` | Build with `PreparedSource` |
| `core/vdbe/explain.rs` | Explain with `PreparedSource` |
| `core/vdbe/blob_io_tests.rs` | Test prepare API update |
| `core/incremental_blob.rs` | Prepare API update |
| `core/incremental/expr_compiler.rs` | Prepare API update |
| `core/json/cache.rs` | Lint hygiene |

---

## 9. Related branch docs

- [`MAIN_MERGE_LEVERAGE.md`](../MAIN_MERGE_LEVERAGE.md) — how main’s
  covering-index / recycling / collation work is used from graph
  lowering (mostly `graph/`, not more core surface).
- [`BRANCH_QUALITY_REVIEW.md`](../BRANCH_QUALITY_REVIEW.md) — quality
  bar and known follow-ups.
- [`DESIGN_DECISIONS.md`](../DESIGN_DECISIONS.md) — product/semantics
  choices for the graph layer.
- [`docs/multi-frontend.md`](../../docs/multi-frontend.md) — product
  documentation for the multi-frontend model.
- [`docs/graph.md`](../../docs/graph.md) — consumer-facing graph API.

## 10. How to refresh this inventory

```sh
git diff --name-only origin/main...HEAD -- core/
git diff --stat origin/main...HEAD -- core/
git log --oneline origin/main..HEAD -- core/
```

Update this file when the branch adds or removes core touch points so
reviewers can trust the checklist.
