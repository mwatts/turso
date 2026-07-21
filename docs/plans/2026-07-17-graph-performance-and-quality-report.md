# Graph frontend performance and quality report

> **Status as of 2026-07-21:** this document predates the graph frontend
> delivery on `feature/graph-frontend` and is retained as an archival plan.
> Since it was written: Ladybug/Kuzu was removed from the corpus, the deep
> corpus grew to ~10k identities, `PreparedSource` + `FrontendCompiler`
> replaced the `ReprepareRecipe` naming, and `__turso_graph_expand`
> (GraphExpand) shipped. Where this text and the code disagree, the code and
> `graph/test-results/REPORT.md` are authoritative.

Date: 2026-07-17

Branch: `feature/graph-frontend`

Implementation checkpoint: `51f68e1f9`

## Purpose

This report summarizes the performance measurements and quality checks run for
the first Turso graph frontend delivery. It separates reproducible measurements
from compile checks and correctness tests so that the current evidence is not
overstated.

The implementation is an experimental frontend. The results establish an MVP
baseline for the selected architecture; they are not a production capacity or
full openCypher compatibility claim.

## Executive summary

- The complete graph crate suite passed: 82 tests across 12 suites.
- PostgreSQL integration passed: 367 `turso_pg_tests` tests, plus 185 parser and
  frontend tests across 6 suites.
- Focused core frontend, reprepare, and virtual-table tests passed.
- The Rust SQL test runner completed successfully. The final command output was
  truncated, so this report does not claim an exact count for that run; the Q6
  implementation checkpoint recorded 9,589 SQL tests.
- Formatting and strict workspace Clippy commands completed successfully.
- The mixed-source conformance corpus reports 12 supported, 0 failed, and 6
  explicitly unsupported scenarios.
- A 100,000-node sparse snapshot rebuilt in about 0.54 seconds with a 27.47 MiB
  conservative peak estimate, within the accepted experimental envelope of one
  second and 64 MiB.
- Reproducible CSR-build fixtures cover sparse, dense, skewed, cyclic, and
  high-degree shapes. Sample medians ranged from 1.04 ms to 4.64 ms for the
  configured fixture sizes.
- Resource-limit tests prove node, edge, and memory caps for every benchmark
  shape. Traversal tests additionally cover hop, path, work, cancellation, and
  resumable cursor behavior.

Test counts below overlap and must not be summed into a single total.

## Final verification matrix

| Area | Command or check | Result |
|---|---|---|
| Formatting | `rtk cargo fmt --all -- --check` | Passed |
| Workspace lint | `rtk cargo clippy --workspace --all-features --all-targets -- --deny=warnings` | Exited successfully |
| Graph crates | `rtk cargo test -p turso_graph_ir -p turso_graph_cypher -p turso_graph_runtime -p turso_graph_frontend` | 82 passed, 12 suites |
| PostgreSQL integration | `rtk cargo test -p turso_pg_tests` | 367 passed |
| PostgreSQL parser/frontend | `rtk cargo test -p turso_pg_parser -p turso_pg` | 185 passed, 6 suites |
| Core frontend preparation | `rtk cargo test -p core_tester --test integration_tests frontend_` | 7 passed |
| Multi-frontend integration | `rtk cargo test -p core_tester --test integration_tests multi_frontend` | 3 passed |
| Dialect-aware reprepare | `rtk cargo test -p turso_core dialect_parser_is_used_for_reprepare` | 1 passed |
| Virtual-table core tests | `rtk cargo test -p turso_core vtab::tests` | 2 passed |
| SQL suite | `rtk make -C testing/sqltests run-rust ARGS='--snapshot-filter __never__'` | Exited successfully; exact final count not retained because output was truncated |
| Runtime benchmark target | `rtk cargo bench -p turso_graph_runtime --bench graph_shapes --no-run` | Compiled successfully |
| Beads integrity | `rtk bd dep cycles`; `rtk bd lint` | No dependency cycles; no warnings |
| Patch hygiene | `rtk git diff --check` | Passed |

The planned command `cargo test -p core_tester --test integration_tests graph`
selected zero tests because `graph` is not a useful filter in that harness. It
was not treated as evidence. The focused `frontend_`, `multi_frontend`, dialect,
and virtual-table targets above replaced it.

## Performance measurements

### Snapshot build and refresh

The snapshot profile builds a sparse directed chain from canonical Turso rows,
publishes an immutable traversal snapshot, invalidates it by generation, and
refreshes it. The recorded command is:

```sh
cargo run -q -p turso_graph_frontend --example snapshot_profile -- 1000 10000 100000
```

The 2026-07-17 development-host debug-build sample produced:

| Nodes | Relationships | Initial build | Refresh | Retained estimate | Conservative build peak | Durable derived writes |
|---:|---:|---:|---:|---:|---:|---:|
| 1,000 | 999 | 6.37 ms | 6.96 ms | 0.18 MiB | 0.27 MiB | 0 bytes |
| 10,000 | 9,999 | 55.69 ms | 55.79 ms | 1.83 MiB | 2.75 MiB | 0 bytes |
| 100,000 | 99,999 | 542.51 ms | 540.80 ms | 18.31 MiB | 27.47 MiB | 0 bytes |

Snapshot-store startup remained below 0.003 ms at every size. The selected MVP
mode is therefore `InMemoryRebuildOnDemand`: derived CSR state is process-local,
rebuildable, and never written to a sidecar or the canonical database.

The accepted experimental envelope was a sparse 100,000-node/relationship
graph rebuilt in at most one second with at most 64 MiB conservative peak
memory. The recorded sample met that envelope.

### CSR construction by graph shape

The Divan benchmark fixtures are declared in
`graph/testdata/benchmarks/manifest.toml` and built by
`graph/runtime/benches/graph_shapes.rs`.

| Shape | Nodes | Relationships | Definition | Sample median CSR build |
|---|---:|---:|---|---:|
| Sparse | 10,000 | 9,999 | Directed chain | 1.17 ms |
| Dense | 250 | 62,250 | Complete directed graph without self-loops | 4.64 ms |
| Skewed | 10,000 | 19,998 | Hub edge plus chain edge per non-root node | 1.82 ms |
| Cyclic | 10,000 | 10,000 | Directed ring | 1.57 ms |
| High-degree | 10,000 | 9,999 | Outgoing star from node 1 | 1.04 ms |

These are development-host samples, not cross-machine baselines. The committed
manifest makes the inputs reproducible; it does not normalize hardware,
compiler, background load, or allocator effects.

### Resource and cancellation behavior

The quality suite verifies that every benchmark shape fails with the expected
typed `LimitExceeded` error when configured below its required:

- node count;
- relationship count; or
- estimated heap requirement.

Runtime and cursor tests also cover bounded hops, paths, work units, memory,
relationship filtering, uniqueness modes, cancellation, resume after a bounded
work slice, and statement abandonment. The retained virtual-table approach was
accepted because cursor calls perform bounded incremental work and preserve
state between calls.

## Correctness and quality coverage

### Frontend and reprepare boundary

Tests prove that:

- initial preparation invokes the registered frontend compiler;
- schema invalidation reparses the original frontend source through the same
  compiler rather than falling through to SQLite;
- parameters survive reprepare;
- missing compiler registration produces a typed error; and
- PostgreSQL uses the same frontend-aware preparation path.

### Parser, binder, and graph IR

The suite covers source spans and diagnostics, graph identity types, scope,
duplicate variables, unresolved names, parameters, nullability, labels,
relationship types, properties, fixed patterns, projections, aggregation,
`WITH`, `OPTIONAL MATCH`, `UNWIND`, ordering, skip, and limit.

Mutation coverage includes `CREATE`, `SET`, `REMOVE`, `DELETE`, `DETACH DELETE`,
and `MERGE`, including parameters, missing matches, per-match creation,
uniqueness failure, detach semantics, idempotence, statement rollback, and
outer-transaction rollback.

### Relational lowering and execution

Fixed graph patterns are tested through Turso AST lowering, relational planning,
VDBE execution, and result comparison. Coverage includes directed and
undirected edges, multi-hop fixed patterns, labels/types/properties, optional
null extension, aggregation, ordering, and multiplicity.

### Traversal runtime and snapshots

Runtime tests cover forward and reverse CSR construction, bounded BFS/DFS,
shortest-path runtime behavior, relationship filters, uniqueness modes, invalid
endpoints, duplicate identities, and normalized equivalence fixtures derived
from the pinned pgGraph source.

Snapshot lifecycle tests cover:

- empty and invalid graphs;
- atomic replacement of the last complete snapshot;
- catalog-version and source-generation freshness;
- stale publication rejection;
- cancellation and resource exhaustion;
- schema damage and missing source tables;
- explicit discard and process-loss rebuild; and
- preservation of canonical rows across every derived-state failure.

`GraphExpand` refuses stale snapshots. A session reuses a current snapshot and
rebuilds at most once per transaction-visible generation.

### Transaction behavior

Tests cover WAL and supported MVCC paths for:

- read-your-writes traversal;
- isolation from other connections;
- autocommit and explicit transactions;
- commit and rollback;
- named savepoint rollback;
- failed statement cleanup;
- cancellation and abandonment; and
- transaction-local snapshots that are never globally published.

### PostgreSQL graph surface

The PostgreSQL tests prove that `graph.cypher(name, query)` resolves a stable
Turso graph identity and delegates to the same `GraphSession` used by direct
Cypher. Tests also require precise rejection of unsupported names, argument
forms, surrounding SQL shapes, graph functions, and pgrx/extension assumptions.

### Conformance corpus

The executable mixed-source corpus contains 18 scenarios normalized from the
openCypher TCK copy in Uni, AGE, Grafeo, pgGraph, Ladybug, SparrowDB, CQLite, and
Samyama. Provenance and license information is pinned in `graph/PROVENANCE.md`.

Current report:

| Classification | Count | Meaning |
|---|---:|---|
| Supported | 12 | Executes end-to-end and returns the expected rows |
| Failed | 0 | Declared supported but errored or returned incorrect rows |
| Unsupported | 6 | Must fail at the frontend boundary with the recorded limitation |

The harness fails on zero discovery, incomplete provenance, a supported error,
row mismatch, or drift between execution and `graph/CONFORMANCE.md`. Ordered
results retain order; unordered results compare as multisets.

The six explicit unsupported areas are:

1. `CALL` subquery scopes;
2. all-shortest-path multiplicity and memory semantics;
3. Cypher weight expressions;
4. `SHORTEST` keyword syntax;
5. `shortestPath()` source syntax; and
6. independent multi-pattern enumeration and join planning.

This is a curated compatibility slice, not full openCypher TCK conformance.

## Architecture and static boundary checks

Repository searches were used as negative boundary assertions:

- graph IR, Cypher, and frontend code contain no direct `Program` or `Insn`
  construction;
- the graph runtime contains no pgrx, SPI, PostgreSQL OID/regclass, memory
  context, background-worker, or transaction-callback dependency; and
- the implementation creates no `.pggraph` or graph sidecar file.

These checks support the intended dependency direction: donor-informed parser,
IR, lowering rules, and CSR algorithms sit above Turso-owned catalog, planner,
VDBE, storage, and transaction machinery.

## Evidence limitations and next measurements

The current evidence does not establish:

- one-million-node snapshot behavior; the recorded snapshot profile stops at
  100,000 nodes;
- end-to-end Cypher latency, including parse, bind, relational planning, VDBE,
  row hydration, and result materialization;
- p95/p99 traversal latency or cancellation latency under concurrent load;
- refresh frequency and write amplification under sustained mixed reads and
  writes;
- independently profiled allocator or resident-set memory; current memory
  figures are internal retained/conservative estimates;
- performance across different hardware, release profiles, or storage modes;
- full TCK, fuzz, long-running stress, or deterministic simulator coverage; or
- public Cypher shortest-path syntax, all-shortest semantics, weighted Cypher
  expressions, or multi-pattern join enumeration.

The next performance phase should add release-profile, end-to-end query
benchmarks at 100,000 and 1,000,000 nodes, external allocation/RSS measurement,
concurrent refresh/read workloads, and cancellation-latency distributions. If
those measurements breach the accepted envelope, the persistence decision
should be reopened for versioned derived chunks inside the Turso file. A
sidecar remains outside the current product contract.
