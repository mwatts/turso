---
task_id: graph-transactions-postgres-operations
complexity: high
risk: high
ambiguity: medium
agent_pattern: pipeline
estimated_tokens: 30000
---

# Graph transactions, Postgres surface, and operations plan

## Goal

Complete transactional graph mutation semantics, expose the shared graph stack
through a narrow PostgreSQL `graph.*` adapter, and make derived-state lifecycle
and conformance operationally explicit.

## Required skills

| Skill | Path | Relevance |
|-------|------|-----------|
| TursoDB | `/Users/markwatts/.agents/skills/tursodb/SKILL.md` | Frontend and engine boundaries |
| Rust | `/Users/markwatts/.agents/skills/rust/SKILL.md` | Public APIs and typed errors |
| Code quality | `.claude/skills/code-quality/SKILL.md` | Database invariants |
| Testing | `.claude/skills/testing/SKILL.md` | Test placement and commands |
| Async I/O | `.claude/skills/async-io-model/SKILL.md` | Resumable maintenance and execution |
| Yield injection | `.claude/skills/yield-injections/SKILL.md` | Re-entry and abandonment tests |
| Transaction correctness | `.claude/skills/transaction-correctness/SKILL.md` | Commit, rollback, WAL, and recovery |
| MVCC | `.claude/skills/mvcc/SKILL.md` | MVCC visibility and tests |

Read each skill completely before implementation. Use transaction-correctness
and MVCC guidance for any change below the graph session layer.

## Prerequisite

The foundation and query/runtime plans must be green. Do not implement a second
Postgres-specific catalog, IR, CSR, or traversal executor.

## Task 1: add mutation IR and relational lowering

Add `CREATE`, `SET`, `REMOVE`, `DELETE`, `DETACH DELETE`, then `MERGE` to the
Turso graph IR and Cypher binder. Lower canonical row changes to ordinary Turso
DML and keep all node, relationship, label/type, property, and generation
changes in the same transaction.

Write failing tests first for atomicity, constraints, missing entities,
duplicate matches, detach behavior, statement errors, and rollback.

## Task 2: implement read-your-writes correctly

Start with a correctness fallback: after an in-transaction graph write, build a
transaction-local snapshot from rows visible to that connection before its
next variable traversal. Cache it by transaction and generation; discard it on
rollback, savepoint rollback, commit completion, or statement abandonment.

Do not publish it globally. Once correct, profile and replace rebuilds with a
base-snapshot plus transaction-local delta overlay only if measurements justify
the added state machine.

Test explicit transactions, autocommit, savepoints, failed statements,
abandoned statements, concurrent readers/writers, and supported MVCC modes.

## Task 3: add the thin Postgres graph adapter

Make `postgres/frontend` depend on `turso_graph_frontend`. Add a deliberately
scoped built-in `graph.*` API that resolves arguments and delegates to shared
graph services.

- Resolve names to Turso graph identifiers; never expose synthesized OIDs as
  durable identity.
- Rewrite schema-qualified functions to collision-free internal names where
  the translator loses qualification.
- Return result shapes supported by the existing Postgres type/wire layer.
- Optionally accept `CREATE EXTENSION graph` as activation syntax only.
- Reject unsupported named arguments, compound values, ACL/RLS expectations,
  background maintenance, triggers, and generic pgrx extension loading with
  precise errors.

Add a compatibility matrix under `postgres/` and wire tests proving Cypher and
Postgres surfaces query the same registered graph and snapshot.

## Task 4: choose and implement derived-state persistence

Measure build time, peak memory, startup cost, write amplification, and refresh
frequency before selecting a mode:

1. Keep explicit in-memory rebuild if it meets targets.
2. Otherwise persist versioned opaque chunks/internal tables in the same Turso
   file and atomically publish a complete version.
3. Use a sidecar only after an explicit change to the one-file product contract.

Persisted state must be discardable. Test crash during build, crash during
publish, stale versions, schema changes, corruption, interrupted refresh, and
rebuild. No derived-state failure may change canonical graph rows.

### D4 decision: in-memory rebuild on demand

The initial operational mode is `InMemoryRebuildOnDemand`. Persistence inside
the Turso file is deferred, and no sidecar is created. The reproducible profile
command is:

```sh
cargo run -q -p turso_graph_frontend --example snapshot_profile -- 1000 10000 100000
```

The 2026-07-17 debug-build profile on the development host produced:

| Nodes | Relationships | Build | Refresh | Retained estimate | Conservative build peak | Durable derived writes |
|---:|---:|---:|---:|---:|---:|---:|
| 1,000 | 999 | 6.37 ms | 6.96 ms | 0.18 MiB | 0.27 MiB | 0 bytes |
| 10,000 | 9,999 | 55.69 ms | 55.79 ms | 1.83 MiB | 2.75 MiB | 0 bytes |
| 100,000 | 99,999 | 542.51 ms | 540.80 ms | 18.31 MiB | 27.47 MiB | 0 bytes |

Store startup itself remained below 0.003 ms at every size. The accepted MVP
envelope is a 100,000-node/relationship sparse graph rebuilt in at most one
second with at most 64 MiB conservative peak memory. D5 must measure dense,
skewed, cyclic, and high-degree inputs; exceeding that envelope reopens the
same-file-chunk decision rather than silently adding a sidecar.

Every snapshot now exposes catalog version, source generation, node/edge count,
build duration, retained heap estimate, and conservative peak-build estimate.
The store reports `Missing`, `Current`, or `Stale`, and `GraphExpand` refuses a
snapshot whose catalog version or transaction-visible source generation no
longer matches. A session reuses a current snapshot and rebuilds only once per
visible generation. This is the post-commit maintenance policy: the next graph
read refreshes through an ordinary Turso connection, with no background worker.

The recovery contract follows from having no durable derived bytes. A crash or
restart produces `Missing`; explicit discard handles suspected process-local
damage; both rebuild from canonical rows. Cancellation, resource exhaustion,
invalid endpoints, schema damage, and stale publication leave the last complete
snapshot untouched. Tests assert that discard/restart, interrupted build,
stale publish, schema failure, and rebuild never change canonical node or
relationship rows.

## Task 5: expand conformance and optimization

Add normalized cases from the openCypher TCK, AGE, Grafeo, pgGraph, Ladybug,
SparrowDB, CQLite, and Samyama according to their licenses and provenance.

- Report supported, failed, and unsupported scenarios separately.
- Fail if zero scenarios are discovered.
- Preserve required ordering and compare unordered results as multisets.
- Import optimizer ideas only after a benchmark identifies a concrete plan or
  runtime deficiency.

Add graph benchmarks with representative sparse, dense, skewed, cyclic, and
high-degree datasets. Enforce resource caps in tests rather than relying on
machine exhaustion.

## Task 6: optional protocol surfaces

Add HTTP/JSON only after the shared session behavior is stable. Keep protocol
code limited to authentication/namespace, request decoding, parameter/result
conversion, cancellation, timeout, and transaction/session lifecycle.

Add Bolt only as a separately approved plan with its own compatibility matrix.
Do not move parsing, binding, planning, or traversal into protocol crates.

## Verification and completion

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy --workspace --all-features --all-targets -- --deny=warnings
rtk cargo test -p turso_graph_ir -p turso_graph_cypher \
  -p turso_graph_runtime -p turso_graph_frontend
rtk cargo test -p turso_pg_tests
rtk cargo test -p core_tester --test integration_tests graph
rtk make -C testing/sqltests run-rust ARGS='--snapshot-filter __never__'
```

Run relevant simulator and failure-injection suites when transaction or core
state machines change. The plan is complete when mutations are atomic and
read-your-writes, Postgres and Cypher share one graph implementation, snapshot
recovery is deterministic, and conformance/compatibility results are published
without silent skips.
