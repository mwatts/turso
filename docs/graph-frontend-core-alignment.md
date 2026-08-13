# Graph frontend and Core alignment

**Status:** findings + plan outcomes (2026-07-25); P0/P1/P3 hygiene and partial P2 shipped on this branch
**Implementation plan:** [`docs/superpowers/plans/2026-07-25-graph-dialect-core-alignment.md`](superpowers/plans/2026-07-25-graph-dialect-core-alignment.md)
**Branch:** `feature/graph-frontend`
**Readers:** Core and graph frontend maintainers who choose the next work
**Related docs:**
- [`docs/multi-frontend.md`](multi-frontend.md)
- [`docs/graph.md`](graph.md)
- [`postgres/COMPAT.md`](../postgres/COMPAT.md)
- [`graph/README.md`](../graph/README.md)
- [`graph/MAIN_MERGE_LEVERAGE.md`](../graph/MAIN_MERGE_LEVERAGE.md)

---

## 1. Purpose

Turso has one database engine (Core) and several language frontends.
People call this the "LLVM of databases."

This document does two jobs:
1. It shows how the three real frontends use Core today.
2. It shows how the graph (Cypher) frontend can follow the Postgres pattern more closely.

The document does **not** ask Core to become a graph engine or a Postgres engine.

### Frontends in scope

| Frontend | Language / protocol | Primary crates | Maturity |
|----------|---------------------|----------------|----------|
| **SQLite** | SQLite SQL and language bindings | `turso_core` dialect, `turso` (bindings/rust), `tursodb` | Primary; packages ship |
| **PostgreSQL** | PostgreSQL SQL and wire protocol v3 | `turso_pg`, `turso_pg_parser`, `turso_pg_server`, `tursopg` | Experimental on main |
| **Graph / Cypher** | openCypher subset | `turso_graph_*` | Experimental on this branch |

### Hard boundary

Core stays Core. Core owns:
- storage (B-tree, WAL, pager)
- VDBE
- translate and planner
- transactions
- schema cookie and reprepare
- async I/O
- shared extension points (`Dialect`, `FrontendCompiler`, virtual tables, values)

Frontends own:
- language parse and bind
- dialect surface for their language
- session and protocol

Frontends must not emit raw VDBE programs.

---

## 2. Shared model

All three frontends must share this flow:

```text
  Client / protocol / REPL
           │
           ▼
  Frontend session (optional wrapper)
           │  native source
           ▼
  ┌────────────────────────────────────────────┐
  │ FrontendCompiler  OR  Dialect::parse       │
  │   → turso_parser::ast::Cmd                 │
  │   → optional prerequisites (DDL once)      │
  └────────────────────┬───────────────────────┘
                       │ prepare_frontend / prepare / prepare_translated_stmt
                       ▼
  ┌────────────────────────────────────────────┐
  │ CORE                                       │
  │  translate → Program (Insn[])              │
  │  Statement::step → VDBE → storage          │
  │  PreparedSource keeps frontend id + source │
  └────────────────────────────────────────────┘
```

### Two official seams in Core

These seams are the template for every frontend that is not SQLite.

| Seam | Scope | Core types | Job |
|------|-------|------------|-----|
| **Database dialect** | One per open `Database` | `Dialect`, `SqliteDialect`, open registry | Identity (`name()`), schema row encode/decode, catalog virtual tables, function resolve/exec, custom-types flag |
| **Connection frontend compiler** | Many per `Connection` | `FrontendId`, `FrontendCompiler`, `FrontendCompilation`, `PreparedSource::Frontend` | Parse native statement text, lower it to engine AST, keep reprepare safe |

SQLite is special in one way only: the host dialect **is** the statement language.
There is no separate `SqliteCompiler`.
Postgres and Graph both need both seams.

### Composition rule (already shipped)

One core `Connection` may register more than one `FrontendCompiler` (for example Postgres and Graph).
One `Database` still has **one** `Dialect`.
This is controlled composition on one connection.
It is not full isolation of many languages on one file.

---

## 3. How each frontend uses Core

### 3.1 SQLite frontend

SQLite is not a thin side package.
It is the default and densest Core consumer.

| Concern | Mechanism |
|---------|-----------|
| Open | `Database::open*` with `SqliteDialect` |
| Prepare | `Connection::prepare` → `Dialect::parse` → `translate` |
| Schema rows | Canonical SQLite `CREATE TABLE` text |
| Catalog | `register_builtin_catalog` (pragma TVFs, `json_each`, and more) |
| Functions | Built-in resolver in `SqliteDialect` |
| Session surface | `turso` async API, C API, other language bindings, `tursodb` |
| Wire protocol | None in the tree (library and CLI only) |

Core gives SQLite the full SQL surface, EXPLAIN, indexes, MVCC and WAL modes, extensions, FTS index methods, and CDC.
Each other frontend reuses this pipeline when it lowers to engine AST.

### 3.2 PostgreSQL frontend (reference second frontend)

Postgres is the live template for a second language on Core.

| Concern | Mechanism | Location |
|---------|-----------|----------|
| Open | `open_database` / `open_database_with_io` with `PostgresDialect` | `postgres/frontend/session.rs` |
| Session | `PgConnection` wraps `Arc<Connection>` | same |
| Compiler registration | `register_frontend_compiler("postgres", PostgresCompiler)` on construct | same |
| Statement path | Special cases (COPY, SCHEMA, SET/SHOW, and more), then `prepare_frontend` | same |
| Compile | `pg_query` parse → `PostgreSQLTranslator` → engine `ast::Stmt` plus prereqs | `postgres/parser/`, `PostgresCompiler` |
| Reads and DML | Same path: one lowered statement → one Core `Statement` | translator |
| Dialect `parse` | PG translate with **SQLite fallback** for helpers and PRAGMA | `PostgresDialect` |
| Schema store | Marker `/* turso_frontend:postgres */` plus original PG DDL | `catalog.rs` |
| Catalog | SQLite builtins plus `pg_*` internal virtual tables | `register_catalog` |
| Functions | `resolve_function` / `exec_scalar_function` on the dialect | `functions.rs` |
| Protocol / CLI | `postgres/server` (pgwire), `tursopg` | separate crates |
| Custom types | `requires_custom_types() == true` | dialect |

Postgres does **not**:
- generate SQLite text as the primary compile path
- emit VDBE
- own storage

The translator targets Turso AST only.

### 3.3 Graph / Cypher frontend (current)

Graph copies the two-seam Postgres layout for reads.
It also adds graph IR, a traversal runtime, and a separate mutation path.

| Concern | Mechanism | Location |
|---------|-----------|----------|
| Open (preferred) | `open_database*` with `GraphDialect` (`"graph-cypher"`) | `graph/frontend/src/session.rs` |
| Open (attach) | `GraphConnection::install` / `open` on an existing SQLite-dialect DB | same — Postgres has no attach mode |
| Session | `GraphConnection` (catalog, parameters, snapshots, limits, graph id) | same |
| Compiler registration | `register_frontend_compiler("graph-cypher", GraphCompiler)` on install; unregister on Drop | same |
| Read path | Refresh snapshot if needed → `prepare_frontend` | `prepare_cancellable` |
| Compile (reads) | Cypher parse → bind IR → `lower_relational` → engine AST | `compiler.rs`, `binder.rs`, `lowering.rs` |
| Mutation path | **Does not use** `FrontendCompiler`: autocommit `BEGIN IMMEDIATE` or write-txn savepoint plus many `prepare_internal` SQL statements in Rust | `mutation.rs` |
| Variable-length paths | `GraphExpand` IR → `__tdb_int_g_expand` internal virtual table | `graph_expand.rs`, runtime CSR/snapshot |
| Dialect `parse` | **SQLite only**; Cypher text returns an error that points to `GraphConnection` | `dialect.rs` |
| Schema store | Unmarked SQLite DDL for user tables; graph meta in `__tdb_int_g_*` | catalog |
| Catalog surface | `turso_graphs` virtual table via dialect plus registration tables | dialect + catalog |
| Functions | Dialect temporal names for Root; every `install` also calls `install_temporal_extension` for InternalHelper mutation SQL | dialect + `graph/temporal` |
| Protocol / CLI | **None** | — |
| Custom types | `requires_custom_types() == true` (for duration fixtures) | dialect |

Pipeline:

```text
  Cypher source
       │
       ├─ READ ──► GraphCompiler ──► engine AST ──► Core prepare/step
       │              │
       │              └─ GraphExpand fragment → __turso_graph_expand vtab
       │
       └─ WRITE ─► bind_mutation ──► many lowered SQL statements via
                      prepare_internal (InternalHelper); autocommit uses
                      BEGIN IMMEDIATE, write txn uses SAVEPOINT
                      (not prepare_frontend)
```

---

## 4. Comparison matrix

Legend:
- **Full** = uses the Core seam as designed
- **Partial** = works, but incomplete or dual-path
- **Bypass** = works around Core
- **N/A** = not applicable yet

| Capability / seam | SQLite | Postgres | Graph | Notes |
|-------------------|--------|----------|-------|-------|
| Host `Dialect` at open | Full | Full | Full (preferred) / Partial (attach) | Attach keeps SQLite as host dialect |
| `FrontendCompiler` + reprepare | N/A (dialect is the language) | Full | Full (reads) | Mutations are not frontend sources |
| All statements → one engine AST | Full | Full (normal path) | Partial | Graph mutations use many programs |
| Prerequisites on prepare | N/A | Full (for example SERIAL sequences) | Unused | Compiler always returns empty prereqs |
| Schema marker / native DDL round-trip | SQLite text | Full | N/A (by design for now) | Graph uses meta tables, not marked Cypher DDL |
| `register_catalog` virtual tables | Builtins | Builtins + `pg_*` | Builtins + `turso_graphs` | Graph expand installs per connection, not in dialect catalog |
| Dialect function surface | Builtins | PG scalars | Temporal names + exec | Graph also installs a static extension on each connection |
| Internal helper SQL (`prepare_internal`) | Engine uses it | Schema / COPY use it | Catalog / FTS use it | Correct pattern for all |
| Virtual tables for non-SQL ops | Many | Catalog | `__tdb_int_g_expand` | Expand is the main graph-specific Core hook |
| FTS / index methods | Core FTS | Via SQL | Graph wrappers → Core FTS | Good use (see `native_capabilities` tests) |
| Covering index / sorter / recycle wins | Automatic | Automatic | Automatic when lowering shape is good | See `MAIN_MERGE_LEVERAGE.md` |
| EXPLAIN | Full | Full | Full (session): one `compile_outcome` then SQL `EXPLAIN QUERY PLAN` over lowered AST | Empty Cypher `result_types` for EQP columns; not re-parsing Cypher as dialect |
| Transaction API | Core | Core (BEGIN via SQL) | Mutations: autocommit `BEGIN IMMEDIATE`, write-txn savepoint, reject bare deferred `BEGIN` | Composition test: PG + Graph share rollback |
| Wire / CLI product surface | CLI + bindings | Wire + CLI | Embed only | Product gap, not Core gap |
| Async wrapper | `turso` crate | None | None | Shared product gap for PG and Graph |
| Result type metadata | Column affinity / custom types | Wire types from SQL | Cypher `result_types()` from shared `CompileOutcome` cache (recompile on miss) | Graph-side cache; Core `FrontendCompilation` types still deferred |

---

## 5. Alignment already done (keep these)

These choices match Postgres on purpose. Keep them.

1. **Two-seam model** — `GraphDialect` + `GraphCompiler` match `PostgresDialect` + `PostgresCompiler` (plan `docs/superpowers/plans/2026-07-22-graph-dialect-two-seam.md` is done).
2. **`open_database` / `open_database_with_io` shape** — same open helpers as `turso_pg`.
3. **Session names** — `GraphConnection` / `prepare` / `query` / `execute` match `PgConnection` (API alignment plan done 2026-07-21).
4. **Reads through `prepare_frontend`** — reprepare keeps the `"graph-cypher"` source; schema change does not reparse Cypher as SQLite.
5. **No direct VDBE emission** — lowering produces engine AST or SQL that Core prepares.
6. **Crate separation from Postgres** — apps compose frontends only through Core registration; no `graph.cypher` adapter inside PG.
7. **Core automatic benefits** — VDBE recycling, covering counts (after lowering fixes), and collation/sorter behavior apply when SQL shapes cooperate.
8. **Internal virtual table for hard ops** — `__tdb_int_g_expand` is the right pattern: Core-backed operator without public `ProgramBuilder`.

---

## 6. Where graph diverges or fails to use Core well

### 6.1 Mutation path is a second execution engine (largest structure gap)

Postgres DML is **one compile → one VDBE program**.

Graph mutations:
- bind a mutation IR pipeline in Rust
- wrap with **autocommit `BEGIN IMMEDIATE`** or **write-txn savepoint** (reject deferred bare `BEGIN`)
- run many `Connection::prepare_internal` statements of **generated SQLite SQL**
- apply ORDER BY / DISTINCT / SKIP / LIMIT for RETURN in Rust
- clear snapshot caches after success or failure

| Effect | Detail |
|--------|--------|
| No single `PreparedSource` for mutations | Reprepare, EXPLAIN, statement-journal batching, and statement-level cancel differ from reads |
| Uses InternalHelper prepare | Mutation SQL uses `prepare_internal` (SQLite function resolve), not `prepare_frontend`. Needs session temporal extension even under GraphDialect |
| Harder atomicity and re-entry | Correctness depends on txn/savepoint rules and per-step prepares. Async yield across the whole mutation spans many statements |
| Blocks "mutation as Statement" API | The API alignment plan left this out on purpose. It is still the main Core-alignment gap |

**Direction (not a rewrite order for tomorrow):** let Core own more of the mutation pipeline.
Options:
- multi-statement programs in Core
- staged plans with one outer transaction/savepoint helper in Core

Do not force Neo4j semantics into VDBE opcodes.

### 6.2 Double parse and double bind on reads — **largely closed on the graph side**

`prepare_cancellable` now shares one `GraphCompiler::compile_outcome` for:
1. the traversal-snapshot decision
2. Cypher `result_types` (from the same `CompileOutcome`; recompile on cache miss, never silent empty types)
3. `prepare_frontend` recompile through the same compiler Arc / last-outcome cache

EXPLAIN strips the prefix, uses the same `compile_outcome` on the inner Cypher, then prepares pure SQL `EXPLAIN QUERY PLAN …` text (no dialect reparse of Cypher).

**Residual:** Core `FrontendCompilation` still does not carry frontend result-type metadata. Graph compensates with a session-side cache. A richer Core compile result remains a multi-frontend opportunity (§7.1), not a live double-bind bug.

### 6.3 Compiler contract is AST-only and connection-blind

`FrontendCompiler::compile(&self, source: &str)` returns prerequisites and one `Cmd`.
That fits PG translation and graph **fixed** reads, but it cannot:
- return frontend-specific result-type vectors
- attach plan diagnostics or IR for EXPLAIN without a second path
- express multi-command mutation scripts as one prepared source

Graph puts the catalog in an `Arc` inside the compiler object.
That is correct for connection-local state.
Core still has no formal compile context (schema generation, frontend options, parameter types).

### 6.4 Graph expand is strong, but not a first-class Core operator

`__tdb_int_g_expand` is an **internal virtual table** with process-local snapshot state.
It reuses VDBE virtual-table machinery and yield-safe cursors. That is good.

Gaps:
- It is not part of the public logical plan set in `core/translate` (unlike ordinary joins and aggregates).
- Snapshot CSR build and refresh are fully graph-owned. Core has no reusable service for "derived read-optimized structures."
- Variable-length path values and uniqueness modes stay in the frontend runtime. The planner sees little cost, covering, or pushdown data.

This is acceptable while the feature is experimental.
A future Core **bounded expand / table-valued traversal** primitive would make GraphExpand less of a side path and more of a planner citizen.
Any frontend that walks tables in steps could reuse it.

### 6.5 Dialect seam is thinner than Postgres

| Postgres | Graph |
|----------|-------|
| Native DDL is marked and reloaded | User tables are plain SQLite; graph registry is side tables |
| `parse` accepts PG SQL | `parse` rejects Cypher (by design). Cypher never enters the dialect |
| Rich `pg_*` catalog | One `turso_graphs` listing plus private tables |
| Function surface is mainly dialect-owned | Root: dialect resolve/exec; **every** install also registers the static extension |

Reject Cypher in `Dialect::parse` is correct. The compiler owns statements.
Dual resolution is intentional, not waste:
- Root dialect-pinned prepares use `GraphDialect` for temporal/`cypher_*` names
- Mutation helpers use `prepare_internal` → InternalHelper → **SQLite** symbol table only, so they need `install_temporal_extension` even under DialectPinned
- Attach mode relies on the extension for both Root and InternalHelper

One Core mechanism (frontend-scoped function resolve, §7.1) could collapse this later.

### 6.6 Attach mode vs dialect-pinned open

Graph can attach Cypher to a **SQLite-host** database.
Postgres does not treat "PG compiler on a SQLite-dialect file" as a first-class path.
You open Postgres with `PostgresDialect`.

Attach mode helps multi-language apps, but:
- graph function and catalog guarantees depend on `GraphConnection::install`, not on the open dialect
- the process registry identity may still be `"sqlite"` while Cypher runs
- composition is easier; namespace ownership is weaker

Document both modes on purpose.
Do not claim attach mode has the same dialect guarantees as `open_database`.

### 6.7 Product surface (not Core, but frontend completeness)

| Surface | SQLite | Postgres | Graph |
|---------|--------|----------|-------|
| REPL | `tursodb` | `tursopg` | — |
| Wire server | — | pgwire | — (Bolt later?) |
| Async Rust wrapper | `turso` | — | — |
| Language bindings | Many | — | — |

Core does not need Bolt to be "aligned."
A shared thin session trait (prepare / query / execute / transactions) would make CLI and bindings cheaper for both PG and Graph.

### 6.8 Core capabilities graph already uses — and ones to use more

**Already used well:**
- internal virtual tables and yield-safe step (`InternalVirtualTableStep`)
- FTS index methods via Core
- custom types and STRICT tables
- savepoints and explicit transactions
- EXPLAIN QUERY PLAN over lowered SQL
- covering indexes when lowering avoids wrapped subqueries (`MAIN_MERGE_LEVERAGE.md`)
- `prepare_internal` for catalog maintenance

**Used little or only in part:**

| Core capability | Graph opportunity |
|-----------------|-------------------|
| `FrontendCompilation::prerequisites` | Idempotent graph bootstrap DDL (if any) or semantic-constraint helpers at prepare time |
| `prepare_translated_stmt` with retained original text | EXPLAIN and hybrid paths that already have AST (avoid stringify then reparse) |
| `get_column_type_info` / custom types | Map Cypher `ValueType` onto Core column type info instead of a parallel type vector only |
| Statement origin / internal helpers | Use `prepare_internal` for all mutation and catalog SQL that must not hit user function resolution |
| CDC / triggers | Optional integrity and outbox for graph mutations (product; not required) |
| Sequences | Prefer Core sequences for node/rel ids when registration allows (if not already uniform) |
| MVCC concurrent writes | Document and test graph mutations under MVCC; do not invent graph-level locking |
| `ProgramBuilder` / new `Insn` | **Do not use from the frontend.** If traversal needs more, extend Core operators |

---

## 7. Core opportunities (value for more than one frontend)

These changes strengthen Core as a multi-frontend platform.
Graph forces several of them.
SQLite and Postgres benefit too.

### 7.1 High value — extend existing seams (small blast radius)

| Opportunity | Why | Who uses it |
|-------------|-----|-------------|
| **Richer `FrontendCompilation`** | Optional metadata: result types, debug IR hash, warnings; optional multi-`Cmd` or script handle | Graph (remove double bind); PG (wire describe); future frontends |
| **Prepare-context / frontend-scoped function resolve** | Today function resolve is database-dialect-wide. Cypher vs PG name collisions force renames | All multi-compiler connections |
| **Formal compile context** | Pass schema cookie / generation, parameter declarations, and frontend options into `compile` without ad hoc `Arc` capture | Graph catalog freshness; PG GUC-like options later |
| **Generic schema ownership helpers** | Shared marker encode/decode and ownership registry helpers (prefix + meta table) | PG (partial today), Graph, future Kafka |
| **`prepare_frontend` diagnostics** | Stable "compiler not registered" errors already exist; add "wrong dialect for this source" helpers | All |

### 7.2 Medium value — new Core services for several frontends

| Opportunity | Why | Who uses it |
|-------------|-----|-------------|
| **Multi-statement transactional program API** | Run N prepared steps under one savepoint/transaction with a shared parameter map and one cancel | Graph mutations; PG multi-command scripts; batch APIs |
| **Bounded iterative expand / recursive walk TVF contract** | Standard resumable, yield-safe adjacency walk with limits (hops, memory, uniqueness) | GraphExpand today; future recursive analytics; optional alternative to `WITH RECURSIVE` |
| **Derived structure / snapshot invalidation hooks** | Generation counters and connection-visible rebuild without global uncommitted publish | Graph CSR; materialised side indexes; cache layers |
| **Statement result-type channel** | Compilers attach logical types to `Statement` when storage affinity is not enough (bool as int) | Graph Cypher types; PG bool/array describe |
| **Namespace / authorization hooks at resolve** | Policy callbacks when resolve sees table names for DDL/DML | Same-file multi-frontend isolation (all) |

### 7.3 Explicit non-goals for Core

| Do not put in Core | Why |
|--------------------|-----|
| Cypher grammar or Neo4j store format | Language- and storage-specific |
| Property-graph catalog as mandatory system tables | Graph registration is a frontend product |
| Full recursive CTE as the only traversal answer | GraphExpand already exists; recursive CTE is a separate SQL feature |
| Kafka broker, Bolt protocol, pgwire | Protocol servers stay outside Core |
| Public stable `ProgramBuilder` frontend API | Transaction, async, and reprepare rules stay engine-owned |

### 7.4 What "easier frontends" means

A fourth frontend (for example a log/topic API) must be able to:
1. Implement `Dialect`, **or** attach to a SQLite host dialect on purpose.
2. Implement `FrontendCompiler` → engine AST (or register internal virtual tables for non-relational ops).
3. Use `register_frontend_compiler` + `prepare_frontend` for every user statement that needs reprepare.
4. Use Core transactions and savepoints; avoid a private interpreter when SQL/AST is enough.
5. Optionally add `server/` and CLI without Core changes.

Graph is closest on (1)–(3) for reads.
Graph is farthest on (4) for mutations.

---

## 8. Graph-side alignment roadmap (ordered)

Priority is **alignment with how Postgres uses Core** and multi-frontend hygiene.
Priority is **not** raw TCK pass rate.

Plan: [`docs/superpowers/plans/2026-07-25-graph-dialect-core-alignment.md`](superpowers/plans/2026-07-25-graph-dialect-core-alignment.md) (2026-07-25). Status below reflects that plan’s outcomes on `feature/graph-frontend`.

### P0 — Document and freeze contracts (cheap) — **done** (2026-07-25, plan Task 1)

- Keep the two-seam model and the read path through `prepare_frontend`.
- Document attach mode vs dialect-pinned open (guarantees, function install) — shipped in `docs/graph.md` and `graph/README.md`.
- Keep mutation savepoint semantics tested. Treat the multi-prepare path as known debt, not as an accident.

### P1 — Cut double work without a full mutation redesign — **done** (2026-07-25, plan Tasks 2–3)

- **Shipped:** session-owned shared `GraphCompiler` / `CompileOutcome` cache — one Cypher parse+bind pass for prepare and types (Task 2).
- **Shipped:** EXPLAIN routes through the same `compile_outcome` path (Task 3).
- Avoid dialect `prepare` of stringified SQL when you already have AST.
- **Deferred (Task 8 cancelled):** Core `FrontendCompilation` result-type metadata. Graph-side cache is reprepare-safe for current use; no gate evidence that Core metadata is required yet.

### P2 — Mutation path convergence (largest graph effort) — **partial** (2026-07-25, plan Tasks 6–7)

Choose by measured pain:

1. **Lower simple mutations** (single CREATE/SET/DELETE without multi-stage WITH) to **one** engine AST and one `prepare_frontend` statement — match Postgres.
   - **Shipped (partial):** mutation helpers use `prepare_internal` (Task 6); **closed CREATE** (single node, no multi-stage WITH) takes a fast path whose **node INSERT is one** `prepare_internal` program (Task 7).
   - **Honest limit:** labeled CREATE still issues **extra prepares** for label-junction membership rows when the catalog has a labels table — a fast-path hit is not “one VDBE program for the whole mutation.”
   - **Still open:** true one-program path for labeled CREATE; SET/DELETE single-program; full multi-stage mutation as one VDBE program.
2. Keep complex pipelines in Rust, but drive them with a **Core multi-step transaction helper** (shared with batch SQL) — **open** (needs Core multi-cmd / multi-step prepare).
3. Long term: multi-command `PreparedSource` with frontend id for the whole script — **open**.

Do not rewrite mutation orchestration only for purity.
Gate rewrites on test failures, performance, or cancel/reprepare bugs.

### P3 — Function and catalog unification — **hygiene done** (2026-07-25, plan Tasks 4–5; Task 4 success redefined 2026-07-25 final fix)

- Dialect is source of truth for **Root** temporal/`cypher_*` under `GraphDialect` — **shipped** (Task 4).
- **Always** call `install_temporal_extension` on every `GraphConnection::install` (including DialectPinned) so InternalHelper mutation SQL can resolve the same names — **shipped** (final review fix; overrides earlier “zero installs on dialect open” claim).
- Expand install: `__tdb_int_g_expand` catalog install is idempotent; session install for both modes (Task 5).
- Keep `turso_graphs` as the public listing.
- Document private `__turso_internal_*` tables as engine-adjacent metadata (like `sqlite_sequence`).

### P4 — Product surfaces (shared gap with Postgres) — **open** (out of scope for 2026-07-25 plan)

- Thin async wrapper pattern shared with `turso_pg` (not graph-specific Core work).
- Optional Cypher REPL / HTTP later.
- Add Bolt only if client demand justifies it.

### P5 — Keep using Core performance primitives — **discipline locked** (2026-07-25, plan Task 9)

- Keep lowering shapes index-friendly (junction covering-count pattern) — covering-count regressions already present and green; no new code required.
- Re-run merge leverage notes when main lands planner/VDBE wins.
- Profile CSR build vs pure SQL joins before you add more Core opcodes.

---

## 9. Findings summary

### What works

- Three frontends on one storage and VDBE core validate the multi-frontend design.
- Postgres is the correct template for a language frontend: dialect + compiler + session + optional protocol.
- Graph already implements the two Core seams for reads and open identity.
- Graph correctly uses an internal virtual table for variable-length traversal.
- Frontend crate separation (no PG↔graph dependency) forces healthy composition through Core registration APIs.

### What is missing or uneven

1. **Uniform statement lifecycle** — Graph writes are a multi-prepare Rust interpreter; SQLite/PG writes are single programs.
2. **Compiler richness** — Core `FrontendCompiler` has no first-class result types, multi-cmd, or compile context.
3. **Function resolution scope** — still database-dialect-global under multi-compiler connections.
4. **Polyglot host dialect** — still one `Dialect` per file open; attach mode and composition work around this; they do not solve namespace isolation.
5. **Product wrappers** — only SQLite has the polished async and bindings layer.
6. **Traversal as planner citizen** — expand works as a virtual table; it is not yet a shared Core logical operator with statistics and cost.

### Recommendation

Invest in Core primitives that **more than one frontend** needs:
- richer frontend compilation
- multi-step transactional prepare
- prepare-scoped functions
- optional expand TVF contract

Do not special-case Cypher inside Core.

On the graph side, the highest-value alignment work is:
- single-program lowering for common mutations
- single-pass compile metadata

Keep graph IR, CSR snapshots, and Cypher semantics in `graph/`.

Core remains the relational engine and bytecode machine.
Frontends remain languages and sessions.
Alignment means **the same prepare/reprepare, transaction, catalog, and extension hooks** — not one dialect for three languages.

---

## 10. Appendix: code map

| Topic | Path |
|-------|------|
| Frontend registry / `prepare_frontend` | `core/frontend.rs`, `core/connection.rs` |
| Dialect trait + SQLite | `core/dialect/mod.rs`, `core/dialect/sqlite.rs` |
| Internal virtual tables / yield step | `core/vtab.rs` |
| PG session + compiler | `postgres/frontend/session.rs` |
| PG dialect + catalog | `postgres/frontend/catalog.rs` |
| PG translator | `postgres/parser/` |
| Graph session | `graph/frontend/src/session.rs` |
| Graph dialect | `graph/frontend/src/dialect.rs` |
| Graph compiler | `graph/frontend/src/compiler.rs` |
| Graph lowering | `graph/frontend/src/lowering.rs` |
| Graph mutations | `graph/frontend/src/mutation.rs` |
| Graph expand virtual table | `graph/frontend/src/graph_expand.rs` |
| Temporal functions | `graph/temporal/` |
| Multi-frontend architecture (broad) | `docs/multi-frontend.md` |
| Main merge leverage notes | `graph/MAIN_MERGE_LEVERAGE.md` |

---

## 11. Appendix: checklist for new Core PRs

When you propose a Core change "for the graph frontend," answer:

1. Do SQLite or Postgres benefit without graph crates?
2. Does the change strengthen `Dialect` / `FrontendCompiler` / virtual table / transaction seams, rather than add graph types to Core?
3. Does it reduce frontend-private interpreters (mutation loops, double parse) without forcing frontends to emit bytecode?
4. Is the durable file format still SQLite-compatible, with only optional marked or prefixed metadata?
5. Do reprepare, schema cookie, and async yield stay Core-owned?

If the answer is yes to 1–5, the change is multi-frontend value.
If it only serves Cypher catalog layout or openCypher semantics, put it in `graph/`.
