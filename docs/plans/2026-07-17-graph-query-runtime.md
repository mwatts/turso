---
task_id: graph-query-runtime
complexity: high
risk: high
ambiguity: medium
agent_pattern: pipeline
estimated_tokens: 30000
---

# Graph query lowering and traversal runtime plan

> **Status as of 2026-07-21:** this document predates the graph frontend
> delivery on `feature/graph-frontend` and is retained as an archival plan.
> Since it was written: Ladybug/Kuzu was removed from the corpus, the deep
> corpus grew to ~10k identities, `PreparedSource` + `FrontendCompiler`
> replaced the `ReprepareRecipe` naming, and `__turso_graph_expand`
> (GraphExpand) shipped. Where this text and the code disagree, the code and
> `graph/test-results/REPORT.md` are authoritative.

## Goal

Execute fixed Cypher patterns through Turso's ordinary relational planner and
bounded variable/shortest paths through a pgrx-free pgGraph-derived CSR runtime.
Prove the smallest `GraphExpand` integration before adding graph opcodes.

## Required skills

| Skill | Path | Relevance |
|-------|------|-----------|
| TursoDB | `/Users/markwatts/.agents/skills/tursodb/SKILL.md` | Frontend and engine boundaries |
| Rust | `/Users/markwatts/.agents/skills/rust/SKILL.md` | Runtime APIs, errors, and newtypes |
| Code quality | `.claude/skills/code-quality/SKILL.md` | Database invariants |
| Testing | `.claude/skills/testing/SKILL.md` | Test placement and commands |
| Async I/O | `.claude/skills/async-io-model/SKILL.md` | Resumable core execution |
| Yield injection | `.claude/skills/yield-injections/SKILL.md` | Deterministic re-entry testing |

Read each skill completely before changing code. Async I/O and yield injection
become operationally mandatory if this plan adds a core cursor or opcode.

## Prerequisite

The foundation plan must be complete and green. Its graph identifiers, bound
IR, catalog snapshot, frontend compiler, and generation metadata are inputs and
must not be redesigned opportunistically.

## Task 1: add fixed-pattern lowering tests

Create frontend tests for node scans, directed and undirected one-hop matches,
multi-hop fixed patterns, property/label/type predicates, parameters,
projection, aggregation, `WITH`, `OPTIONAL MATCH`, `UNWIND`, ordering, skip,
and limit.

Normalize selected AGE and Grafeo cases into repository-owned fixtures with
provenance. Add `.sqltest` coverage only where the resulting semantics are
meaningful through the SQL harness; use graph frontend integration tests for
Cypher parsing and binding.

## Task 2: implement AGE-informed relational lowering

In `graph/frontend`, lower relationally expressible graph IR into
`turso_parser::ast`. Keep row hydration, expressions, joins, aggregation, and
sorting in Turso.

Pay particular attention to:

- optional-match predicates remaining in the left-join condition;
- Cypher null propagation and missing properties;
- clause scope across `WITH`;
- relationship direction and endpoint aliasing;
- deterministic ordering only where the language/query requests it.

Do not construct `Program`, `Insn`, or planner internals.

## Task 3: extract the portable pgGraph runtime

In `graph/runtime`, adapt the pinned pgGraph CSR and traversal algorithms:

- CSR build with forward and reverse adjacency;
- bounded BFS/DFS;
- shortest and weighted path only where existing tests specify semantics;
- relationship-type filtering;
- path/walk/trail uniqueness;
- cancellation checks and explicit node, edge, path, hop, work, and memory
  limits.

Replace pgrx, SPI, OIDs, `regclass`, GUCs, PostgreSQL errors/memory contexts,
background workers, transaction callbacks, and sidecar paths. Accept typed row
input and graph-IR identifiers. Use typed errors; no `anyhow` in public APIs.

Port applicable pgrx-free unit tests and add differential fixtures against the
pinned source behavior. Update `graph/PROVENANCE.md` with every adapted area.

## Task 4: build versioned snapshots from Turso rows

In `graph/frontend`, scan registered node/relationship sources in one
consistent Turso read transaction and construct an immutable
`TraversalSnapshot`. Tag it with graph id, catalog version, and source
generation.

Before publishing, compare against the current committed generation. Discard a
snapshot that became stale during build. The initial implementation is explicit
build/refresh plus in-memory storage; do not add a sidecar or persistence.

Test empty graphs, missing endpoints, duplicate ids, concurrent invalidation,
cancelled builds, failed builds, and rebuild replacement.

## Task 5: implement the `GraphExpand` virtual-table spike

Register an internal graph-expansion virtual table through a reusable graph
catalog-registration function called by graph-capable dialects. Its constrained
inputs are graph id, start node, direction, relationship types, hop bounds,
uniqueness mode, and resource limits. Its outputs are path position, node id,
relationship id/type, depth, and path identity as required by lowering.

The cursor must perform bounded incremental work and keep traversal state
between calls. It may access only an already-built immutable snapshot; no
database I/O is permitted from synchronous virtual-table cursor methods.

Lower bound graph expansion to a normal virtual-table scan joined with Turso
tables for properties. This deliberately reuses existing `VOpen`, `VFilter`,
`VColumn`, and `VNext` planning/execution.

## Task 6: run the core-cursor decision gate

Benchmark and test worst-case work before the first result, cancellation
latency, memory caps, and fairness. Retain the virtual-table implementation if
every cursor call has bounded work and acceptable cancellation behavior.

If it cannot meet the gate, replace only the execution adapter with a dedicated
graph cursor/opcode state machine. Follow `IOResult`, mutate into a resumable
state before yielding, add yield-point variants only at enum ends, and test
resume plus statement abandonment. Do not make every existing internal virtual
table async to accommodate graph traversal.

Record the decision and measurements in the compatibility documentation.

## Verification and completion

```bash
rtk cargo fmt --all -- --check
rtk cargo clippy -p turso_graph_ir -p turso_graph_cypher \
  -p turso_graph_runtime -p turso_graph_frontend --all-targets --all-features \
  -- --deny=warnings
rtk cargo test -p turso_graph_runtime -p turso_graph_frontend
rtk cargo test -p core_tester --test integration_tests graph
rtk make -C testing/sqltests run-rust ARGS='--snapshot-filter __never__'
```

If a core cursor/opcode is added, also run its narrow core tests and the
concurrent simulator scenario added by this plan. Completion requires passing
fixed-pattern, bounded expansion, shortest-path, stale-snapshot, cancellation,
and resource-exhaustion cases.
