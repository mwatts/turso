---
task_id: multi-frontend-graph-roadmap
complexity: high
risk: high
ambiguity: medium
agent_pattern: pipeline
---

# Multi-frontend graph implementation roadmap

> **Status as of 2026-07-21:** this document predates the graph frontend
> delivery on `feature/graph-frontend` and is retained as an archival plan.
> Since it was written: Ladybug/Kuzu was removed from the corpus, the deep
> corpus grew to ~10k identities, `PreparedSource` + `FrontendCompiler`
> replaced the `ReprepareRecipe` naming, and `__turso_graph_expand`
> (GraphExpand) shipped. Where this text and the code disagree, the code and
> `graph/test-results/REPORT.md` are authoritative.

This roadmap turns the selected architecture in
[`docs/multi-frontend.md`](../multi-frontend.md) into three sequential,
agent-executable plans. Experimental product status is accepted; correctness,
transaction, reprepare, and cooperative-yield invariants are not relaxed.

## Selected architecture

```text
Uni Cypher parser
    + Grafeo-informed Turso graph IR
    + AGE relational lowering rules
    + pgGraph-derived CSR/traversal runtime
    + Turso catalog, planner, VDBE, storage, and transactions
```

pgGraph is a source for portable Rust algorithms and tests. Turso does not host
its `pgrx` extension or emulate PostgreSQL SPI, OIDs, GUCs, background workers,
transaction callbacks, or `$PGDATA`.

## Execution order

1. [Foundation and frontend context](2026-07-17-graph-frontend-foundation.md)
   establishes frontend-aware reprepare, graph crates, IR, parser, binder, and
   catalog generation tracking.
2. [Query lowering and traversal runtime](2026-07-17-graph-query-runtime.md)
   adds fixed-pattern lowering, extracts pgGraph's CSR algorithms, and proves
   bounded `GraphExpand` integration.
3. [Transactions, Postgres surface, and operations](2026-07-17-graph-delivery.md)
   adds mutations/read-your-writes, the thin `graph.*` adapter, derived-state
   persistence, conformance expansion, and optional protocols.

Each plan must land with its own tests and may be executed on a fresh branch
from the prior plan's accepted commit. Do not run plans 2 or 3 against a
partially passing predecessor.

## Task database

The authoritative execution state is the repository-local Beads database,
not GitHub issues or Markdown checkboxes. The root goal is `turso-graph`.

| Phase/task | Bead | Purpose |
|------------|------|---------|
| Goal | `turso-graph` | Native transactional graph frontend |
| Foundation | `turso-graph.1` | Frontend context, IR, parser, binder, catalog |
| F0 | `turso-graph.1.1` | Provenance and feasibility fixtures |
| F1 | `turso-graph.1.2` | Failing frontend-reprepare tests |
| F2 | `turso-graph.1.3` | Prepared source and compiler registry |
| F3 | `turso-graph.1.4` | Postgres compiler migration |
| F4 | `turso-graph.1.5` | Graph crate scaffolding |
| F5 | `turso-graph.1.6` | Turso-owned graph IR |
| F6 | `turso-graph.1.7` | Uni parser slice and graph binder |
| F7 | `turso-graph.1.8` | Graph registration and invalidation |
| Query/runtime | `turso-graph.2` | Relational lowering and traversal runtime |
| Q1 | `turso-graph.2.1` | Fixed-pattern fixtures |
| Q2 | `turso-graph.2.2` | AGE-informed relational lowering |
| Q3 | `turso-graph.2.3` | pgGraph runtime extraction |
| Q4 | `turso-graph.2.4` | Versioned traversal snapshots |
| Q5 | `turso-graph.2.5` | `GraphExpand` virtual-table spike |
| Q6 | `turso-graph.2.6` | Execution-adapter decision gate |
| Delivery | `turso-graph.3` | Transactions, Postgres, persistence, conformance |
| D1 | `turso-graph.3.1` | Mutation IR and lowering |
| D2 | `turso-graph.3.2` | Transactional read-your-writes |
| D3 | `turso-graph.3.3` | Thin Postgres graph API |
| D4 | `turso-graph.3.4` | Persistence decision and implementation |
| D5 | `turso-graph.3.5` | Conformance and benchmarks |
| D6 | `turso-graph.3.6` | Optional protocol surfaces |

Operating rules:

1. Run `bd ready` and claim one unblocked leaf task before implementation.
   Phase and goal epics are containers, not implementation assignments.
2. Set exactly one leaf task to `in_progress` unless explicitly coordinating
   independent work.
3. Treat the bead's acceptance criteria and linked plan as the scope contract.
4. Record material discoveries as notes or new dependency-linked beads rather
   than silently expanding the active task.
5. Run the task's verification commands before closing it. Include the task id
   in the commit body and close only after the commit exists.
6. Close a phase epic only when `bd epic status` reports all children closed
   and the phase-level verification gate passes.

Useful commands:

```bash
bd ready
bd show turso-graph.1.1
bd update turso-graph.1.1 --claim
bd note turso-graph.1.1 "discovery or verification result"
bd close turso-graph.1.1
bd epic status turso-graph
bd dep cycles
bd lint
```

## Shared constraints

- Donor parser, value, catalog, planner, executor, record-id, and storage types
  must not cross a Turso-owned crate boundary.
- Frontends produce Turso AST or a validated Turso graph request; they never
  construct VDBE instructions directly.
- Canonical graph data is stored in ordinary Turso tables. CSR state is
  versioned, derived, discardable, and rebuildable.
- Use strong identifier newtypes and typed library errors.
- No operation may publish derived graph state before its source transaction
  commits.
- Unsupported conformance scenarios remain visible; zero discovered scenarios
  is a test failure.
- Use `rtk` for repository commands and `apply_patch` for edits.

## Pipeline completion criteria

- Cypher and PostgreSQL graph requests use the same catalog, IR, lowering, and
  traversal runtime.
- Fixed patterns execute through the ordinary Turso planner/VDBE.
- Bounded variable traversal and shortest path have deterministic resource and
  cancellation behavior.
- Direct SQL writes invalidate affected graph snapshots transactionally.
- Reprepare never routes retained Cypher or PostgreSQL source through SQLite
  parsing.
- Derived-state loss or corruption can be recovered without changing canonical
  graph rows.
