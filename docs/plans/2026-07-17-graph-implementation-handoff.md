# Graph frontend implementation handoff

Date: 2026-07-17

Branch: `feature/graph-frontend`

Implementation checkpoint: `51f68e1f9`

Tracker: all 24 records closed: 1 root epic, 3 phase epics, and 20
implementation records

## Session outcome

This session took the graph frontend from feasibility analysis and detailed
planning through an experimental, end-to-end implementation. The branch is
clean, pushed to `origin/feature/graph-frontend`, and has no divergence from the
remote at this checkpoint.

The delivered architecture is:

```text
Uni-derived Cypher parser
        +
Turso-owned graph IR informed by Grafeo
        +
AGE-informed relational lowering
        +
pgGraph-derived portable CSR/traversal runtime
        +
Turso catalog, planner, VDBE, storage, and transactions
```

Turso code remains authoritative wherever donor functionality overlaps it.
There is no donor storage engine, transaction manager, planner, bytecode
program, PostgreSQL extension runtime, or sidecar in the result.

## What was delivered

### Multi-frontend preparation

Core now has a data-only prepared-source recipe, stable `FrontendId`, registered
frontend compilers, typed missing-compiler errors, and one dispatch path for
initial compile, schema retry, and statement reprepare. PostgreSQL was migrated
to this path before Cypher was added.

Important locations:

- `core/frontend.rs`
- `core/connection.rs`
- `core/statement.rs`
- `postgres/frontend/session.rs`

### Graph crate boundary

Four workspace crates isolate the graph language and runtime layers:

- `graph/ir`: Turso-owned identifiers, expressions, plans, scopes, and mutation
  requests;
- `graph/cypher`: Uni-derived grammar, source AST, spans, and diagnostics;
- `graph/runtime`: portable CSR, traversal, shortest-path algorithms, limits,
  cancellation, and typed errors; and
- `graph/frontend`: binding, relational lowering, catalog, snapshots,
  `GraphExpand`, mutation execution, and `GraphSession`.

Donor provenance, pinned revisions, source paths, adaptation type, and licenses
are recorded in `graph/PROVENANCE.md` and `NOTICE.md`.

### Canonical catalog and invalidation

Graph definitions, node sources, relationship sources, and generations are
stored in ordinary Turso tables. Registration validates tables, columns,
identity contracts, endpoints, and reserved names. Generated triggers advance
graph generations for graph-session and direct SQL writes, and those generation
changes follow transaction rollback.

### Query execution

Fixed patterns lower into ordinary Turso relational AST and execute through the
existing planner and VDBE. Variable-length traversal uses a constrained
internal `GraphExpand` virtual table over an immutable snapshot, then rejoins
canonical Turso tables for properties.

The pgGraph-derived runtime has no pgrx or PostgreSQL-server dependency. It
supports forward/reverse adjacency, bounded traversal, shortest-path runtime
operations, filtering, uniqueness modes, cancellation, and explicit resource
limits.

The virtual-table adapter was retained instead of adding graph opcodes. Cursor
work is sliced and resumable, so long traversals cooperate with the existing
execution model without making synchronous cursor methods perform database I/O.

### Mutations and transaction visibility

`CREATE`, `SET`, `REMOVE`, `DELETE`, `DETACH DELETE`, and `MERGE` are represented
as validated Turso-owned mutation requests and executed as ordinary DML inside
savepoints. This is intentionally separate from `FrontendCompiler`, whose
single-`Cmd` result cannot safely express arbitrary multi-entity mutation
batches.

After an in-transaction graph write, variable traversal rebuilds a private
snapshot from rows visible to that connection. It is never globally published.
WAL and supported MVCC tests cover read-your-writes, cross-connection isolation,
savepoints, commit, rollback, failure, cancellation, and abandonment.

### PostgreSQL surface

The narrow PostgreSQL API is:

```sql
SELECT * FROM graph.cypher(graph_name, cypher_query);
```

It resolves a stable Turso graph identity and delegates to the shared
`GraphSession`. It is not a pgrx extension and does not emulate OIDs, ACL/RLS,
GUCs, memory contexts, workers, callbacks, generic extension loading, or
compound extension result types. Unsupported forms fail precisely and are
documented in `postgres/COMPAT.md`.

### Derived-state policy

The selected mode is `InMemoryRebuildOnDemand`:

- canonical graph rows remain in Turso;
- CSR state is immutable, process-local, versioned by catalog and visible
  generation, and discardable;
- `GraphExpand` refuses stale state;
- the next graph read rebuilds after invalidation;
- failure leaves the last complete snapshot untouched; and
- restart or suspected damage is recovered by rebuilding from canonical rows.

No graph sidecar exists. Persistence inside the Turso file remains an option
only if future measurements exceed the accepted rebuild envelope.

### Conformance and operational evidence

The mixed-source corpus currently reports 12 supported, 0 failed, and 6
explicitly unsupported scenarios. Reproducible graph-shape benchmarks cover
sparse, dense, skewed, cyclic, and high-degree CSR construction. Detailed
commands, counts, measurements, caveats, and coverage are in
`docs/plans/2026-07-17-graph-performance-and-quality-report.md`.

## Key decisions that should remain stable

1. Turso owns catalog, planning, VDBE, storage, and transaction semantics.
2. Frontends produce Turso AST or a validated Turso-owned request; they do not
   construct bytecode.
3. Fixed patterns stay relational. Only traversal-specific work uses CSR.
4. Properties remain in canonical Turso rows and are rejoined after traversal.
5. Derived state is versioned, immutable, resource-bounded, and disposable.
6. PostgreSQL is an adapter over `GraphSession`, not a second graph stack.
7. Optional HTTP/JSON and Bolt surfaces require separate approval and remain
   outside the implementation.
8. New donor code requires pinned provenance and a compatible license entry.

## Current known gaps

The checked conformance report makes the remaining language gaps explicit:

- `CALL` subqueries;
- `shortestPath()` and `SHORTEST` source syntax;
- all-shortest-path multiplicity and memory semantics;
- Cypher weight expressions; and
- independent multi-pattern enumeration and join planning.

The runtime already has shortest-path machinery, but public Cypher
`shortestPath()` syntax is not yet bound into it. Do not describe shortest path
as end-to-end Cypher support until that frontend gap is closed.

Performance evidence is an MVP baseline. It does not yet cover million-node
graphs, end-to-end query latency, p95/p99 behavior, concurrent refresh load,
external allocation/RSS profiling, fuzzing, or long-running simulator/stress
tests.

## Recommended next work

Start a new compatibility-and-hardening phase rather than reopening the closed
delivery epic. The first bounded milestone should be **single shortest-path
Cypher syntax end to end**.

### First milestone: `shortestPath()` end to end

1. Add the smallest `shortestPath(path_pattern)` source-AST and grammar slice,
   retaining source spans and explicit hop bounds.
2. Bind it to the existing Turso-owned shortest-path IR and define direction,
   relationship filters, endpoint binding, nullability, path uniqueness, and
   zero-length semantics.
3. Lower it through the existing snapshot and traversal runtime. Do not add a
   second executor or graph-specific storage path.
4. Add direct `GraphSession` and PostgreSQL `graph.cypher` tests with identical
   rows and errors.
5. Promote the applicable `sparrow-shortest-function` conformance scenario from
   unsupported to supported. Keep `allShortestPaths`, weighted expressions, and
   Ladybug's distinct `SHORTEST` syntax separate until their contracts are
   designed.
6. Add an end-to-end latency benchmark and retain cancellation/resource-limit
   assertions for the new public path.

This is the best next slice because the runtime and resource contracts already
exist; the work closes a visible frontend gap without destabilizing storage,
transactions, or the relational lowering boundary.

### Follow-on order

After single shortest path is green:

1. define all-shortest-path result multiplicity and memory limits;
2. add independent multi-pattern binding and join enumeration, informed by
   Samyama but executed by Turso;
3. define and bind weighted Cypher expressions;
4. expand the normalized TCK/donor corpus and add parser/binder fuzz targets;
5. add release-profile end-to-end benchmarks at 100,000 and 1,000,000 nodes,
   allocation/RSS profiling, cancellation distributions, and mixed
   read/write/refresh workloads;
6. add deterministic simulator coverage for interruption and concurrency paths;
   and
7. revisit same-file derived chunks only if measurements breach the accepted
   in-memory rebuild envelope.

HTTP/JSON and Bolt should remain separate product decisions. Bolt in particular
needs its own compatibility matrix, transaction/session model, authentication
boundary, cancellation behavior, and protocol test plan.

## Restart checklist

1. Check out `feature/graph-frontend` and verify it matches
   `origin/feature/graph-frontend` at or after `51f68e1f9`.
2. Read, in order:
   - `docs/multi-frontend.md`;
   - `docs/plans/2026-07-17-multi-frontend-graph-roadmap.md`;
   - `docs/plans/2026-07-17-graph-query-runtime.md`;
   - `docs/plans/2026-07-17-graph-delivery.md`;
   - `graph/PROVENANCE.md`;
   - `graph/CONFORMANCE.md`; and
   - the performance and quality report adjacent to this handoff.
3. Inspect `graph/cypher`, `graph/ir`, `graph/frontend/src/session.rs`,
   `graph/runtime/src/shortest.rs`, and `postgres/frontend/session.rs` before
   changing the boundary.
4. Create a new Beads epic and written plan for compatibility/hardening; do not
   reopen `turso-graph` merely to add optional scope.
5. Preserve the repository's RTK, Rust, testing, async-I/O, transaction, MVCC,
   and conventional-commit requirements.

One testing trap is worth preserving: filtering the core integration harness by
`graph` selected zero tests. Use the actual `frontend_` and `multi_frontend`
filters plus the graph crate, dialect, virtual-table, PostgreSQL, and SQL-suite
targets recorded in the performance report.

## Milestone commits

The branch history is intentionally incremental. Useful architecture
checkpoints are:

- `20bdf5ab8` — frontend-aware prepared source and reprepare;
- `8b05324d8` — PostgreSQL migration to frontend preparation;
- `d1947bcfd` — graph IR;
- `3aee871aa` — Cypher parser and binder;
- `5c015911f` — transactional graph catalog;
- `ce924adaf` — fixed-pattern relational lowering;
- `56797f8b5` — portable pgGraph-derived runtime;
- `c89e719c3` — versioned traversal snapshots;
- `0d70b5700` — `GraphExpand` virtual-table execution;
- `8a401d2df` — cooperative resumable traversal cursor;
- `a821b7d7c` — atomic Cypher mutations;
- `9fa239e78` — transaction-local traversal snapshots;
- `a7a22ff16` — PostgreSQL `graph.cypher` adapter;
- `e2ba3c4af` — enforced snapshot freshness and rebuild policy; and
- `428812b47` — mixed-source conformance and graph-shape benchmarks.
