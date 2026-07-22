# Archived: Cypher / graph frontend plan (superseded)

> **Status:** historical planning document. The functionality it describes
> was **delivered** on `feature/graph-frontend` as the `turso_graph_{cypher,
> ir,runtime,frontend,temporal}` crates, with a public `GraphConnection` API
> (`prepare`/`prepare_cancellable`, `query`/`query_cancellable`, `execute`,
> `install`, `open`/`open_with_parameters`). The M7 Postgres `graph.*`
> compatibility adapter described below **was built (commit `a7a22ff16`) and
> then deliberately removed (commit `178437223`)**: `postgres/` has no graph
> dependency, the frontends are separate crates, and an application that wants
> both composes them on one core connection via
> `Connection::register_frontend_compiler`. This document is kept for
> historical context on the design reasoning; it is not a current plan. For
> ground truth, see `graph/README.md`, `graph/DESIGN_DECISIONS.md`, and
> `graph/CONFORMANCE.md`, and for the still-live parts of the original
> planning corpus see `docs/archive/plans/`.
>
> Naming predates the shipped API: `GraphSession` below is now
> `GraphConnection`; `mutate` is now `execute`; `ReprepareRecipe` shipped as
> `PreparedSource::Frontend` + `FrontendCompiler`
> (`Connection::register_frontend_compiler` / `prepare_frontend` in
> `core/connection.rs`). Left as originally written except where annotated
> inline below.

---

## 6. Cypher / graph frontend (Neo4j-style)

### 6.1 Goal

Expose a **graph query language** (Cypher as the concrete example) while
storing and executing against Turso’s single backend. Clients might speak
Bolt (Neo4j) or a simpler HTTP/JSON API; the language + model matter more than
the first wire protocol.

### 6.2 Chosen implementation path

The graph frontend should proceed as a mixed-source implementation with one
Turso-owned boundary:

```text
Uni Cypher parser
    +
Grafeo-informed shared graph IR
    +
AGE relational lowering rules
    +
pgGraph-derived CSR/traversal runtime
    +
Turso catalog, planner, VDBE, storage, and transactions
```

This makes the project materially more feasible, but **not** because the
existing Postgres frontend can load or host pgGraph unchanged. pgGraph is a
PostgreSQL `pgrx` extension and depends on PostgreSQL ABI and lifecycle
facilities that Turso's Postgres-compatible parser, catalog, and wire server do
not implement. The useful asset is its portable Rust graph runtime: CSR
construction, forward/reverse adjacency, traversal algorithms, filtering,
bounded-path safety, and the `CatalogSnapshot`-style adapter seam.

The architecture therefore extracts and adapts pgGraph algorithms behind
Turso-owned interfaces. It does not emulate SPI, background workers, GUCs,
PostgreSQL transaction callbacks, OIDs, `regclass`, ACL/RLS, or the `pgrx`
extension ABI.

### 6.3 Scope and success criteria

The first deliverable is a graph frontend over ordinary Turso tables with:

- Cypher parsing, binding, diagnostics, parameters, and prepared-statement
  reprepare that retain frontend identity.
- Fixed-length reads lowered into the existing relational planner and VDBE.
- Bounded variable-length and shortest-path reads backed by a derived CSR
  runtime without bypassing Turso transactions or storage ownership.
- A graph catalog with stable Turso identifiers and explicit graph-to-table
  mappings.
- One source of truth: node and relationship rows committed through Turso.
  CSR data is derived, rebuildable state.
- A thin Postgres-facing `graph.*` compatibility surface that calls the same
  graph services as Cypher and future HTTP/Bolt adapters.

The initial plan does **not** promise the pgGraph extension ABI, Neo4j's disk
format, the complete openCypher language, arbitrary graph algorithms, Bolt,
background index maintenance, or PostgreSQL ACL/RLS behavior.

### 6.4 Ownership boundary

```text
Cypher text                  Postgres graph.* SQL
    |                               |
    v                               v
Uni-derived source AST       thin Postgres adapter
    |                               |
    +---------------+---------------+
                    v
             Turso graph binder
                    |
                    v
          Turso-owned bound graph IR
              |                 |
      fixed/relational      path expansion
              |                 |
              v                 v
       AGE-informed SQL      GraphExpand
          lowering          logical operator
              |                 |
              v                 v
       Turso planner/VDBE  pgGraph-derived runtime
              |                 |
              +--------+--------+
                       v
       Turso catalog, rows, storage, WAL/MVCC,
             transactions, and async I/O
```

The following rules keep the seam stable:

- Donor AST, value, catalog, physical-plan, record-id, and storage types do not
  cross into core.
- The binder and bound graph IR are Turso code. Resolved identifiers use
  stable newtypes such as `GraphId`, `GraphTableId`, `NodeId`, and
  `RelationshipTypeId`, not PostgreSQL OIDs.
- Fixed patterns, filtering, projection, aggregation, `WITH`, and optional
  joins use the existing relational planner wherever their semantics can be
  represented correctly.
- `GraphExpand` is a small logical/core contract for topology operations. The
  frontend and pgGraph-derived runtime never assemble VDBE instructions.
- The traversal runtime returns graph identities/path state; row hydration and
  expressions remain Turso planner/VDBE work unless a proven, typed pushdown is
  added.
- All database I/O remains resumable and yield-safe. CSR code may be
  synchronous over an immutable in-memory snapshot, but building, loading, or
  refreshing that snapshot must use Turso's async I/O state-machine model.
- The Postgres adapter translates SQL functions/table functions into graph IR
  or graph service requests. It is not a second graph engine.

### 6.5 Proposed module layout and contracts

Use repository-level graph crates/modules parallel to `postgres/`, rather than
placing reusable graph code under `postgres/frontend`:

```text
graph/
  cypher/       Uni-adapted lexer, parser, source AST, diagnostics
  ir/           binder, bound IR, catalog traits, semantic errors
  runtime/      pgGraph-derived CSR, traversal, overlays, resource limits
  frontend/     session, lowering orchestration, prepared input
postgres/
  frontend/     thin graph.* compatibility adapter
core/
  translate/    GraphExpand planning/lowering integration only
  vdbe/         resumable execution support only where required
```

Final crate names should follow the workspace's existing naming convention,
but the dependency direction is mandatory:

```text
cypher -> ir <- postgres adapter
             |
             +-> relational lowerer -> existing planner
             +-> runtime port/trait  -> pgGraph-derived runtime

core depends only on Turso-owned graph contracts, never parser or pgrx types.
```

Required contracts:

| Contract | Responsibility |
|----------|----------------|
| `FrontendId` + `ReprepareRecipe` | Preserve source language, original text, parameters, and compiler across schema-triggered reprepare |
| `GraphCatalogSnapshot` | Resolve graph names, mapped tables/columns, labels, relationship types, properties, and stable Turso ids without SPI |
| `BoundGraphPlan` | Express scans, expands, filters, projections, joins/applies, aggregates, ordering, limits, and later mutations |
| Relational lowerer | Convert relationally expressible graph IR into Turso AST while preserving Cypher null/scope semantics |
| `TraversalSnapshot` | Immutable, versioned CSR view with forward/reverse adjacency and optional typed filter indexes |
| `TraversalRequest` / result cursor | Direction, type set, hop bounds, uniqueness mode, limits, cancellation, and deterministic output |
| Transaction overlay | Make in-transaction graph changes visible to the same transaction without publishing them before commit |
| Graph maintenance service | Build, mark stale, refresh, publish, invalidate, and account for derived snapshots |

### 6.6 Data and derived-state model

Canonical graph data remains relational. A graph registration maps logical
node and relationship types to ordinary Turso tables and indexed endpoint
columns. Catalog metadata is stored in reserved internal tables in the same
database file.

The CSR is a **derived index**, not a second authoritative database:

1. Read a consistent Turso snapshot through normal statements.
2. Build forward and reverse adjacency plus any selected filter indexes.
3. Tag the snapshot with graph/catalog and data-version information.
4. Publish it atomically only after a complete build.
5. Reject, rebuild, or fall back when its freshness contract is not met.

The MVP should use an explicit build/refresh command and an immutable in-memory
snapshot. This removes PostgreSQL background-worker and `$PGDATA` assumptions
while the semantics are proven. Persistence is a later decision gate:

- **Preferred one-file path:** internal tables or opaque chunks managed by
  Turso and rebuilt transactionally.
- **Simplest correctness fallback:** rebuild on open or on demand.
- **Optional product choice:** a derived sidecar, only if the one-file promise
  is explicitly relaxed. A `.pggraph` sidecar must not appear accidentally.

Writes require a base snapshot plus a transaction-local overlay. Commit may
publish or schedule a new derived version only after the Turso transaction
commits; rollback drops the overlay. Savepoint behavior must be specified
before graph mutations are enabled.

### 6.7 Detailed implementation plan

Each milestone has an exit gate. Later milestones may prototype in parallel,
but no public surface should depend on a milestone whose gate has not passed.

#### M0 — freeze semantics, provenance, and baselines

- Pin the exact Uni, Grafeo, AGE, and pgGraph revisions used for adaptation.
- Record every adapted or translated file's source path, license, revision,
  and whether it is copied, structurally adapted, or behaviorally rewritten.
- Select the first openCypher feature slice and convert a small set of AGE,
  pgGraph, Grafeo, and Ladybug cases into frontend-neutral scenarios.
- Decide graph identity, path uniqueness modes, deterministic ordering rules,
  maximum hop/path/memory limits, and error categories.

**Exit:** architecture decision record, provenance manifest, compatibility
matrix, and tests that fail because the frontend is not implemented—not
because fixtures were silently skipped.

#### M1 — frontend-aware prepare and reprepare

- Add `FrontendId` and a reprepare recipe to prepared input/program state.
- Ensure initial prepare and schema-triggered reprepare invoke the same
  frontend compiler.
- Define collision-free internal function names or a prepare-scoped resolver.
- Add focused tests for schema changes, parameters, errors, and abandoned
  statements.

**Exit:** a synthetic non-SQL frontend statement can be prepared, invalidated,
and correctly reprepared without entering the SQLite parser.

#### M2 — parser, binder, catalog, and shared graph IR

- Extract only Uni's Cypher grammar, source AST, spans, and diagnostics needed
  for the selected language slice.
- Implement a Turso binder over `GraphCatalogSnapshot`; do not reuse Uni's
  Arrow values, catalog, physical plans, or executor.
- Define the minimal graph IR using Grafeo's operator taxonomy as a design
  reference. Start with scan, fixed expand, filter, project, aggregate, sort,
  skip/limit, distinct, optional/left apply, union, and unwind.
- Create reserved catalog tables and stable identifier allocation.

**Exit:** parser and binder suites produce stable bound plans and typed errors
without touching donor execution or storage code.

#### M3 — fixed-length relational reads

- Lower node scans and fixed one/multi-hop patterns to Turso AST.
- Implement property/label/type predicates, parameters, projection,
  aggregation, `WITH`, `OPTIONAL MATCH`, `UNWIND`, sort, skip, and limit.
- Follow AGE's proven clause-placement rules—for example, optional-match
  predicates belong in join conditions rather than post-join filters.
- Add endpoint, label, and relationship-type indexes through ordinary Turso
  schema operations.

**Exit:** the selected read-only TCK slice and AGE/Grafeo fixed-pattern
regressions pass through the normal Turso planner and VDBE.

#### M4 — extract the pgGraph traversal runtime

- Create a pgrx-free runtime crate from pgGraph's CSR, forward/reverse
  adjacency, BFS/DFS, bounded traversal, shortest/weighted path, filter, and
  resource-limit code.
- Replace SPI/catalog/OID access with `GraphCatalogSnapshot` and Turso row-scan
  inputs. Replace PostgreSQL errors and memory contexts with typed Rust errors
  and explicit resource accounting.
- Preserve applicable pgrx-free unit tests and add equivalence fixtures against
  the pinned pgGraph behavior.
- Build immutable snapshots explicitly from a consistent Turso read.

**Exit:** the runtime builds and queries a CSR from Turso-provided rows with no
`pgrx`, PostgreSQL server, `$PGDATA`, background worker, GUC, SPI, or OID
dependency.

#### M5 — integrate `GraphExpand`

- Define the core logical operator with direction, allowed relationship types,
  lower/upper hop bounds, uniqueness mode, predicates, and output shape.
- Let the optimizer combine relational scans/filters with expansion while
  retaining a clear cost and cardinality boundary.
- Execute expansion through a resumable cursor. Hydrate node/edge properties
  through Turso; push filters into CSR only when semantics and invalidation are
  proven equivalent.
- Add cancellation, deterministic memory/path caps, and yield/failure tests.

**Exit:** bounded variable-length and shortest-path scenarios pass under
normal, injected-yield, cancellation, and resource-exhaustion execution.

##### Graph cursor decision record (2026-07-17)

The synchronous virtual-table spike failed the fairness gate: a single
`VFilter` or `VNext` call could consume the complete traversal work budget
before producing a row. The selected implementation retains the virtual-table
planning boundary but makes only the internal graph cursor resumable. Turso's
internal-vtable adapter now accepts `Row`, `Done`, or `Yield`; existing internal
tables keep their synchronous default, while `GraphExpand` advances at most 256
state/adjacency operations and returns an explicit cooperative yield when more
work remains. No graph opcode and no general asynchronous virtual-table API are
needed.

The reproducible debug-build gate in
`graph/runtime/examples/cursor_gate.rs` uses a 100,000-edge star whose requested
two-hop result set is empty, an adversarial case for work before first output.
Three local runs produced 1,172 cursor calls and 1,171 yields, took
49.2–49.7 ms total, and had a maximum individual call of 0.547–0.625 ms. Wall
times are machine-specific; the enforced contract is the deterministic
256-operation quantum. Tests additionally prove that:

- a 300-edge high-fanout `VFilter` yields before completing, resumes to the
  exact three rows of the only two-hop path, and may be abandoned at the yield;
- connection interruption is observed on the next quantum;
- node, edge, path, hop, work, and retained-memory limits fail explicitly; and
- adjacency filtering itself advances one raw CSR edge at a time, so a single
  high-degree node cannot bypass the quantum.

**Decision:** retain `GraphExpand` as an internal table-valued scan with its
specialized resumable cursor. Revisit a dedicated opcode only if later
predicate pushdown or weighted-shortest-path measurements cannot satisfy the
same bounded-call contract.

#### M6 — transaction visibility and mutations

- Add `CREATE`, `SET`, `REMOVE`, `DELETE`/`DETACH DELETE`, then `MERGE` to IR
  and relational lowering.
- Implement transaction-local adjacency overlays or a correctness-preserving
  relational fallback so a transaction reads its own writes.
- Define commit publication, rollback, savepoint, schema-change, and stale
  snapshot behavior.
- Never update derived graph state ahead of the authoritative row commit.

**Exit:** mutation TCK cases and multi-connection rollback/visibility tests
pass in supported transaction modes, including injected failures.

**D1 implementation checkpoint (2026-07-17):** the Cypher source AST, Turso-owned
mutation IR, binder, and relational executor now cover `CREATE`, property
`SET`/`REMOVE`, `DELETE`, `DETACH DELETE`, and property/end-point based `MERGE`.
The read frontend remains a single-statement `FrontendCompiler`; mutations use
a separate validated request boundary because a multi-entity mutation cannot
be represented atomically by that interface. The executor materializes the
matched binding identities, applies ordinary Turso DML once per match row, and
wraps the complete Cypher statement in a savepoint. This preserves an outer
transaction while rolling back all earlier row changes on a later constraint,
detach, or statement error. Focused tests cover per-match creation, parameters,
missing matches, uniqueness failure, detach behavior, merge idempotence, and
outer rollback.

This checkpoint deliberately does not publish or patch the shared CSR snapshot.
Transaction-local read-your-writes and snapshot invalidation remain the next M6
task. Mutation queries currently require `MATCH` clauses to precede writes and
reject `WITH`, `UNWIND`, `RETURN`, named paths, variable-length creation, and
whole-map updates rather than approximating their semantics.

**D2 implementation checkpoint (2026-07-17, historical):** at the time this
checkpoint was written, `GraphSession` (renamed `GraphConnection` in the
shipped API) owned the connection-local query/mutation lifecycle. Immediately
before each variable-length query it rebuilds a private CSR from rows visible
to that connection inside a nested savepoint. The completed candidate replaces
only that session's overlay; it is never published to the shared snapshot
store. Rebuilding on every variable traversal is the correctness fallback from
the delivery plan: commit, rollback, savepoint rollback, mutation failure, and
statement cancellation cannot leave a transaction-generation cache stale.
Fixed-pattern reads continue through ordinary relational lowering without an
unnecessary CSR rebuild.

Internal virtual tables are database-schema scoped rather than connection
scoped. Session overlays therefore live in the shared snapshot registry as
weak `(Connection, SessionSnapshotStore)` registrations, and the graph cursor
selects its overlay using the `Arc<Connection>` supplied by `open`. This is
required for MVCC, where schema refresh can replace a connection's prior table
object. WAL and MVCC tests prove same-transaction visibility, cross-connection
isolation, explicit commit and rollback, nested savepoint rollback, autocommit,
failed mutation cleanup, and cancelled rebuild behavior. The existing
resumable-cursor abandonment tests cover dropping execution after a local
snapshot has been selected. A transaction/generation delta cache remains a
measurement-gated optimization, not part of the correctness contract.

#### M7 — Postgres graph compatibility adapter

> **Historical note:** this milestone was implemented (commit `a7a22ff16`,
> `graph.cypher()` table function + `PgConnection::install_graph`) and then
> **deliberately reverted** (commit `178437223`) to keep the Postgres and
> graph frontends decoupled. It is neither pending nor currently delivered as
> a built-in surface; an application that wants both composes them itself on
> one core connection via `Connection::register_frontend_compiler`. The
> milestone body below is left as originally written for historical context.

- Expose a deliberately scoped `graph.*` SQL API through
  `postgres/frontend`; translate calls to the shared graph services.
- Rewrite schema-qualified range/table functions to collision-free internal
  functions where the existing translator cannot preserve qualification.
- Replace pgGraph `regclass`/OID arguments with stable name/id resolution.
- Document unsupported named arguments, arrays/compound results, ACL/RLS,
  triggers, and background maintenance rather than approximating them.
- Optionally accept `CREATE EXTENSION graph` only as activation syntax for the
  built-in adapter; it must not claim general pgrx extension loading.

**Exit:** Postgres clients can create/register a graph, build it, and execute
the supported graph query/functions through the shared runtime, with an exact
compatibility matrix.

#### M8 — derived-state maintenance and persistence

- Choose rebuild-on-demand versus one-file persisted chunks using measured
  build time, memory, startup, and write-amplification data.
- Add stale/version detection and an atomic publish protocol.
- Integrate post-commit refresh scheduling without PostgreSQL background-worker
  assumptions; a host service may schedule work through normal Turso
  connections.
- Test crashes during build/publish, abandoned refreshes, schema changes, and
  recovery. Derived corruption must be recoverable by discard/rebuild and must
  not corrupt canonical rows.

**Exit:** freshness is observable and enforced, recovery is deterministic, and
the chosen persistence mode satisfies the one-file product contract.

**D4 implementation checkpoint (2026-07-17):** the selected MVP mode is
explicit in-memory rebuild on demand. `SnapshotStore` exposes the mode and a
`Missing`/`Current`/`Stale` status with catalog version, source generation,
build duration, graph size, retained-memory estimate, and conservative
peak-build estimate. Snapshot consumers validate freshness against the rows
visible to their connection; sessions reuse a current generation and lazily
rebuild after the next committed or transaction-local generation change.

The checked-in `snapshot_profile` example measures startup, build, refresh,
memory, and durable write amplification. On the recorded development run, a
100,000-node/99,999-edge sparse chain rebuilt in about 0.54 seconds, retained
about 18.3 MiB, conservatively peaked at 27.5 MiB, and wrote zero derived bytes.
This meets the recorded experimental envelope, so same-file chunks would add a
publish/recovery format without evidence that the MVP needs it. Restart,
discard, stale publish, cancellation, resource failure, schema damage, and
rebuild tests prove the derived state can disappear without changing canonical
rows. D5 benchmark shapes remain the gate that can reopen same-file persistence;
a sidecar remains outside the one-file product contract.

#### M9 — conformance, optimization, and protocol surfaces

- Expand TCK coverage and keep unsupported scenarios visible.
- Profile before importing Samyama/Grafeo optimizer ideas; add only rules with
  measured benefit and equivalent semantics.
- Add HTTP/JSON first. Add Bolt only if client demand justifies its protocol,
  transaction, and type-system surface.
- Publish compatibility, resource-limit, operational, and migration docs.

**Exit:** the compatibility target, performance envelope, and operational
failure modes are reproducible in CI and documented for users.

**D5 implementation checkpoint (2026-07-17, historical):** an executable,
provenance-complete 18-scenario corpus at the time covered the TCK-via-Uni,
AGE, Grafeo, pgGraph, Ladybug, SparrowDB, CQLite, and Samyama. The checked
report had 12 supported, zero failed, and six unsupported scenarios; CI
separates all three outcomes, rejects zero discovery, preserves ordered
results, and treats unordered results as multisets. The corpus also confirmed
that AGE-style omitted traversal bounds are supported under Turso's finite
runtime caps, replacing the earlier deferred classification. (The corpus has
since grown substantially further; see `graph/test-results/REPORT.md` and
`graph/CONFORMANCE.md` for current numbers — 10,242 identities, 8,800
passing as of the latest run.)

Divan CSR-build benchmarks cover manifest-defined sparse, dense, skewed, cyclic,
and high-degree graphs, while tests force node, relationship, and memory-limit
failures on every shape. The recorded medians range from about 1.0 to 4.6 ms for
the selected inputs. No optimizer transplant is justified by this baseline;
Samyama and Grafeo remain evidence-triggered donors rather than dependencies.

Optional HTTP/JSON and Bolt surfaces are explicitly deferred. They were not
approved as part of this implementation goal; at the time this was written,
`GraphSession` (now `GraphConnection`) was the shared service seam and the
Postgres `graph.cypher` adapter was the delivered external surface — that
adapter was subsequently removed (see the M7 note above); Cypher access today
is through `GraphConnection` directly, composed with other frontends
app-side. Any future protocol must remain a thin adapter with a separately
approved authentication, namespace, cancellation, timeout, and transaction
lifecycle contract.

### 6.8 Verification strategy

| Layer | Primary verification |
|-------|----------------------|
| Parser | Uni/openCypher golden cases, source-span/error tests, parser fuzzing |
| Binder/IR | Scope/type/error tests and bound-plan snapshots |
| Relational lowering | `.sqltest` coverage where SQL-visible semantics fit; AGE-derived clause regressions |
| Runtime extraction | Ported pgrx-free pgGraph unit tests plus differential fixtures |
| Session/API | Rust integration tests for prepare/reprepare, parameters, multi-connection visibility, and rollback |
| Core traversal | Deterministic simulator and yield/failure injection for cursor state, cancellation, and abandonment |
| Conformance | Upstream openCypher TCK with explicit feature tags and a hard failure on zero discovered scenarios |
| Persistence | Crash/reopen, stale-version, interrupted-build, corruption-discard, and rebuild tests |

Tests should encode graph semantics, not donor storage behavior. Storage, WAL,
and general transaction cases remain Turso tests; donor suites contribute only
frontend-observable behavior not already covered.

### 6.9 Blocking decisions and removal paths

| Decision/blocker | Default or removal path |
|------------------|-------------------------|
| Frontend identity is lost on reprepare | Complete M1 before exposing Cypher prepared statements |
| pgGraph assumes PostgreSQL server services | Extract algorithms behind Turso traits; do not emulate pgrx/SPI/bg workers |
| CSR freshness after writes | Explicit versioning plus transaction overlay; start with explicit rebuild |
| Same-file requirement conflicts with `.pggraph` | Default to memory/rebuild, then internal Turso persistence; sidecar requires an explicit product change |
| Path uniqueness/termination | Make mode, bounds, visited state, and limits explicit in IR and tests |
| Recursive CTEs are unavailable | Use `GraphExpand`; recursive CTE support is not a prerequisite |
| Postgres OIDs/`regclass` are unstable or absent | Resolve stable Turso catalog ids/names at the adapter boundary |
| Postgres API shapes exceed current translator/types | Publish a narrow compatibility API; add named args/compound values only as independent frontend features |
| CSR memory or build cost is unbounded | Resource accounting, cancellation, graph size limits, and benchmark gates before persistence |
| Read-your-writes across derived state | Transaction-local overlay or relational fallback; never serve a known-stale snapshot silently |
| Namespace isolation is only cooperative | Keep raw connections private for MVP; complete §8 ownership/authorization hooks before claiming a security boundary |

### 6.10 Recommended delivery slices

1. **Feasibility spike:** M0–M2 plus a pgrx-free CSR build and one bounded BFS
   over Turso rows. This validates both critical seams before broad language
   work.
2. **Read-only MVP:** M3–M5 with explicit graph build, fixed patterns,
   bounded variable paths, and shortest path through the Cypher session.
3. **Postgres-accessible MVP:** the narrow M7 adapter over the same catalog,
   IR, and runtime—not a separate pgGraph port. (Built, then removed; see the
   M7 note above.)
4. **Transactional graph:** M6 with overlays/read-your-writes and defined
   rollback/savepoint behavior.
5. **Operational graph:** M8–M9 persistence, maintenance, broader conformance,
   optimization, and optional protocols.

The first go/no-go gate is the feasibility spike. It succeeds only if the Uni
AST can bind into Turso-owned IR and the extracted pgGraph runtime can build
and traverse from Turso row snapshots without PostgreSQL facilities. Passing
only one of those proofs is insufficient.
