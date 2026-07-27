# Graph frontend internals

A map into how the Cypher frontend actually works, for someone about to change
it. The consumer-facing guide is [`docs/graph.md`](graph.md); this document is
the other half — the pipeline, the seams, the invariants you can break without
the compiler noticing, and the work that is deliberately not done yet.

Everything here is anchored to a file and, where it helps, a symbol. Line
numbers drift; symbol names and file names are the durable coordinates.

## Crate topology

```text
turso_graph_ir
    ^       ^
    |       |
cypher   runtime
    \       /
     frontend -> turso_core
```

| Crate | Owns | Must not own |
|---|---|---|
| `turso_graph_ir` | Stable graph identities (`GraphId`, `LabelId`, `RelationshipTypeId`, `RoleId`), bound plans, catalog traits, semantic errors | Source text, spans, SQL |
| `turso_graph_cypher` | Lexer, parser, source AST, spans, diagnostics | Anything downstream of binding; donor AST types never leave the crate |
| `turso_graph_runtime` | Adjacency (CSR), traversal, path policy, shortest-path | Canonical rows, transactions |
| `turso_graph_frontend` | Binder, lowering, mutation orchestration, catalog, session, dialect | VDBE instructions — it never emits bytecode directly |

The dependency direction is enforced by the Cargo manifests, not by convention.
`turso_graph_temporal` sits to the side: it is a static extension registering
scalar functions, depended on by `frontend`.

## The read path

Cypher reads are a *compiler plugin*, not a second engine. Core owns translate,
reprepare, and step; the graph frontend hands core an engine AST.

```text
source text
  │  turso_graph_cypher::parse                      graph/cypher/src/parser.rs
  ▼
cypher AST (Query → Vec<Clause>)                    graph/cypher/src/ast.rs
  │  binder::bind                                   graph/frontend/src/binder.rs:588
  ▼
ir::Plan (PlanKind tree)                            graph/ir/src/plan.rs
  │  lowering::lower_relational                     graph/frontend/src/lowering.rs:313
  ▼
SQL text → engine AST
  │  GraphCompiler::compile (FrontendCompiler impl)  graph/frontend/src/compiler.rs:167
  ▼
core translate → VDBE → step
```

The seam is core's per-connection compiler registry. `GraphCompiler` implements
`turso_core::FrontendCompiler`; `Connection::register_frontend_compiler`
installs it under the id from `graph_frontend_id()`, and reads reach it through
`prepare_frontend("graph-cypher")`. `compile_outcome`
(`compiler.rs:103`) is shared by `prepare_frontend` and session result-type
recovery so one source string is parsed and bound once, with a cache behind it
(`compile_misses`, `compiler.rs:162`).

`binder::classify_statement` (`binder.rs:538`) decides `StatementKind::{ReadOnly,
WriteReturningRows, WriteWithoutRows}` before anything else runs. That is what
routes a statement down the read path versus the mutation path, and it is why
`GraphConnection::execute` on a read reports a mutation-binding failure rather
than quietly working.

### The IR

`ir::PlanKind` (`graph/ir/src/plan.rs:39`) is the whole bound-plan vocabulary:

- **Sources** — `NodeScan`, `RelationScan`, `Unit`
- **Relation traversal** — `RoleExpand` (fixed hop, node-anchored),
  `RoleJoin` (role-anchored, the standalone role pattern), `GraphExpand`
  (variable-length `*` / `*min..max`)
- **Relational algebra** — `Join`, `Filter`, `Projection`, `Aggregation`,
  `Distinct`, `Sort`, `Skip`, `Limit`, `Union`, `LeftApply`, `Unwind`
- **Procedures** — `ProcedureCall` with `ProcedureIdentity`

`RolePlayer` (`plan.rs:132`) distinguishes `Fresh` (introduce a binding) from
`Bound` (constrain an existing one). Both arms exist for every role-consuming
node, and a change that handles only `Fresh` will pass most tests while
silently refusing or mis-binding the other — this has happened.

There is no `Direction` in the IR. Direction is a *parser* concept
(`cypher::Direction`, `ast.rs:176`) that the binder resolves into a **role
pair**. Anything downstream reasoning about "incoming" or "outgoing" is a bug.

## The role model

This is the branch's central abstraction and the one most easily broken.

A relationship source declares an ordered list of **roles**
(`RelationshipSourceRegistration.roles`, `graph/frontend/src/catalog.rs`). Each
role carries:

- `targets: Vec<RoleTarget>` — `RoleTarget::Node(LabelId)` or
  `RoleTarget::Relation(RelationshipTypeId)` (`graph/ir/src/role.rs`). A role
  may target node sources, relation sources, or both. Relation-as-player is not
  a special path; it is membership in this list.
- `cardinality: RoleCardinality` — `One` or `Many`.
- optionality.

**Storage follows cardinality, not arity:**

| Cardinality | Storage |
|---|---|
| `One` | A column on the relation table (the role's endpoint column) |
| `Many` | A spill table `<relation_table>__<role>(relation_id, node_id)`, indexed `(relation_id, node_id)` **and** `(node_id, relation_id)` so probes from either side are index probes |

Spill tables are created by `install_spill_table` (`catalog.rs:941`).

### The invariants

These are not style preferences. Violating any of them reintroduces the defect
class this work exists to remove.

1. **Binary is a layout, not a kind.** A two-endpoint relation is a two-role
   relation whose roles are named `start` and `end`.
   `RelationshipSourceRegistration::binary(...)` is a convenience constructor
   over the general path. No `is_binary` flag, no `if roles.len() == 2`, no
   arity branch in general machinery.
2. **Roles resolve by `RoleId` or by declared name — never by position.**
   Positional resolution was shipped by five separate tasks on this branch and
   caught by review each time. It passes every test whose fixture happens to
   declare roles in the order the query writes them. The standing test for a
   change here: permute the role *names* in a fixture without changing their
   order, and permute the argument order without changing names. Both must
   still behave correctly, and a sabotage of the resolution must turn something
   red.
3. **A `Many` role is identified by `spill_table.is_some()`** — never by name,
   position, or count.
4. **No hard-coded `"start"` / `"end"` in general machinery.** Two sites
   legitimately retain them, both audited: the arrow-form start/end check in
   `binder.rs` and the endpoint-cardinality constraint path in
   `semantic_constraints.rs`. Arrow syntax is *defined* in terms of that role
   pair, and `SemanticEndpoint` only models the binary layout. Everything else
   must be role-general.

Arrow forms (`(a)-[:KNOWS]->(b)`) are sugar that requires roles literally named
`start` and `end`; a relation without that pair is reachable only through the
standalone role pattern, and the refusal happens at bind time before any row is
touched.

## The write path

Mutations are **multi-statement orchestration**, not a single `PreparedSource`.
This is the largest structural divergence from how the SQL and Postgres
frontends use core, and it is acknowledged debt rather than an accident — see
[`docs/graph-frontend-core-alignment.md`](graph-frontend-core-alignment.md)
§6.1.

Entry point: `execute_cypher_mutation` (`graph/frontend/src/mutation.rs:252`).
The shape that matters:

```text
run() = bind → try_single_program_mutation | execute_bound
              → constraints.validate_state(connection)
```

`run()` is invoked inside a transaction wrapper chosen by host state:

| Host state | Wrapper |
|---|---|
| Autocommit | `BEGIN IMMEDIATE` → `run()` → `COMMIT` / `ROLLBACK` |
| Existing write transaction | `SAVEPOINT __turso_graph_mutation` → `run()` → `RELEASE` / `ROLLBACK TO` |
| Deferred read transaction (bare `BEGIN`) | `MutationError::RequiresWriteTransaction` |

Two consequences worth internalizing:

- **Every spill insert is inside `run()`**, so a relation row and all of its
  role players commit or roll back together. This is the integrity property
  reified modeling cannot offer: reification needs one statement per role, so a
  failure between them leaves a partially stated assertion that reads as
  complete. Do not move a role write outside this window.
- **`validate_state` runs after the inserts**, inside the transaction. It is
  therefore the natural mid-create failure for testing atomicity — no
  failure-injection hook exists in `graph/`, and none is needed.

Mutation helper SQL is prepared with `prepare_internal` (InternalHelper, SQLite
symbol table only), which is why the session-installed temporal extension is
required even under dialect-pinned open.

### Delete is role-general, and must stay that way

`DELETE` / `DETACH DELETE` on a node has to find every relation referencing that
node **through every role**, not through a start/end pair. An earlier
implementation resolved only the two-role layout and silently orphaned role
players for any other shape — no error, no refusal, just dangling rows. Two
rules protect the current implementation:

1. Resolve references through the role list
   (`schema_catalog.rs::relationship_role_node_source`), never through a
   two-role endpoint helper.
2. **Materialize matching relation ids before any mutating statement.** The
   DETACH cleanup loop must not re-evaluate a live predicate across successive
   mutating statements: a predicate that self-references a `Many` role's spill
   table sees its own deletions and stops matching. Sabotaging only the
   materialization — keeping role-general resolution — makes the relation-row
   delete match zero rows and leaves every relation dangling.

## Catalog and storage overlay

There is no graph file format. Graph metadata lives in reserved tables inside
the same `.db` file:

- `__turso_graph_catalog` — registrations, semantic schema, constraints,
  fragments, FTS index metadata
- `__turso_graph_node_labels_*`, `__turso_graph_relationship_types_*`,
  `__turso_graph_relationship_type_registry_*` — label/type junctions
- `__turso_graph_fts_*` — physical FTS index names
- generation-counter triggers

Reserved-name handling and the generation triggers live in core
(`core/schema.rs`, `core/translate/{index,trigger,update}.rs`) — see
[`graph/docs/core-changes.md`](../graph/docs/core-changes.md) for the complete
list of core changes this frontend required.

**Generation counters are the cache-invalidation spine.** A `GraphConnection`
opened via `open`/`open_with_parameters` compares its catalog generation before
preparing a read or executing a mutation and reloads the immutable semantic
snapshot when registration publishes a newer one. Sessions created through
`install` keep their caller-supplied catalog deliberately.

Identities are **table-local coordinates**: equal numeric ids in two source
tables are distinct graph entities. Anything that compares raw identity across
sources is wrong.

## Runtime: snapshots and traversal

Fixed-hop patterns are plain relational joins with ordinary read-your-writes
semantics. Variable-length patterns are not: they run against an in-memory
adjacency snapshot.

- `graph/runtime/src/csr.rs` — the compressed-sparse-row `Graph`, `EdgeInput`,
  `Neighbor`, `NeighborCursor`. Adjacency is keyed by **role pairs**, not by
  direction.
- `graph/runtime/src/traversal.rs` — `traverse`, `TraversalRequest`,
  `TraversalCursor`, `TraversalOrder`, `Uniqueness`, `Path`
- `graph/runtime/src/path_policy.rs` — `resolve_path_algorithm`, `PathSelector`,
  `WeightClass`, `PathAlgorithm`. Algorithm legality is policy, not a planner
  heuristic; see `graph/DESIGN_DECISIONS.md` "Path algorithm legality".
- `graph/runtime/src/shortest.rs` — `shortest_path`, `weighted_shortest_path`

Variable-length lowering targets the internal `__turso_graph_expand` virtual
table, which holds a process-local `SnapshotStore`. Because that store is
derived state and needs a connection snapshot, it **cannot** be installed from
`GraphDialect::register_catalog` at schema build; both open modes activate it
through `install_graph_catalog` inside `GraphConnection::install`, which is
idempotent.

Snapshot rules that surprise people:

- The **connection-local** snapshot is rebuilt inside a nested savepoint before
  a traversal read whenever the generation counter moved, so a session sees its
  own uncommitted writes without publishing them.
- The **shared** store caches the last committed snapshot and refuses to refresh
  inside a transaction (`SnapshotError::RefreshInsideTransaction`).
- Snapshots are never persisted and are always rebuildable from the tables.

`GraphConnection::diagnostics()` is strictly observational — no refresh, no
catalog write, no publication.

## The semantic overlay

`register_graph` alone gives source-derived, schemaless behavior. Adding
semantic rows promotes the graph into **strict semantic mode**, and there is no
partial state: a graph either has semantic rows or it does not.

Layering, from physical to conceptual:

```text
source tables (application-owned columns)
  └─ graph registration      : sources, roles, identities
      └─ semantic schema     : node/relationship types, property ownership
          ├─ fragments       : property interfaces, polymorphic scans
          └─ constraints     : required / key / unique / value / cardinality
```

Implementation sits in `semantic.rs` (types, registration, validation),
`schema_catalog.rs` (the immutable snapshot the binder reads), and
`semantic_constraints.rs` (validation, including the `validate_state` pass that
runs inside the mutation transaction).

Semantic ids are persisted independently of source ids and column positions;
the snapshot maps conceptual ids to physical sources only at lowering time.
Registration is additive, atomic, and idempotent — an identical replay writes
nothing, a conflicting replay is rejected, and changing or removing anything
active is a deliberate future evolution API rather than an accident.

**Integrity boundary:** semantic guarantees are graph-frontend guarantees.
Direct SQL against a registered source table bypasses membership and validation;
only physical constraints (`NOT NULL`, `UNIQUE`, `CHECK`, FKs, triggers) survive
that path. The frontend deliberately does not install a native unique index when
multiple semantic types share a source, because such an index would enforce
uniqueness across types and a partial index cannot express junction-table
membership.

## Where to change what

| Task | File |
|---|---|
| New syntax | `graph/cypher/src/parser.rs` + `ast.rs`, then `binder.rs` |
| New plan shape | `graph/ir/src/plan.rs`, then `lowering.rs` |
| Read SQL generation | `graph/frontend/src/lowering.rs` |
| Write behavior, transactions, delete | `graph/frontend/src/mutation.rs` |
| Registration, roles, spill tables | `graph/frontend/src/catalog.rs` |
| Semantic types, fragments, constraints | `semantic.rs`, `schema_catalog.rs`, `semantic_constraints.rs` |
| Scalar function surface | `binder.rs` (mapping) + `functions.rs` (typing) |
| Temporal / `cypher_*` scalars | `graph/temporal/src/lib.rs` |
| Variable-length traversal | `graph/runtime/`, `snapshot.rs`, `graph_expand.rs` |
| Catalog procedures | `graph/frontend/src/procedures.rs` |
| Session API | `graph/frontend/src/session.rs` |
| Dialect, function resolution | `graph/frontend/src/dialect.rs` |

`binder.rs` is 8.7k lines and `lowering.rs` 3.8k; both are the natural
first-stop for most behavior questions and the hardest to hold in context. Read
the specific arm, not the file.

## Future features

Grouped by how well-understood the work is, not by priority.

### Designed, deliberately deferred

- **Mutation as a single `PreparedSource`.** The multi-statement orchestration
  above is the largest structural gap between this frontend and how core
  intends frontends to work. Closing it removes a second execution path and its
  bespoke transaction wrapper.
- **Non-additive semantic evolution.** Changing or removing an active
  constraint, remapping a type or property, and removing a fragment membership
  are all rejected today by design. They need an explicit evolution API with
  its own migration and validation story.
- **Role cardinality constraints past `start`/`end`.** The general role model
  supports n-ary natively, but `SemanticEndpoint` cardinality constraints still
  only model the two-role binary layout — the last significant place where the
  overlay is narrower than the engine beneath it.
- **FTS-driven outer scan.** Lowering emits a core FTS rowid-set subquery, but
  the layered plan still scans its outer node relation, so the index does not
  yet replace that scan. Teaching normal `MATCH` lowering to drive its outer
  source from the FTS rowid set is the preferred fix, explicitly in preference
  to adding a `db.index.fulltext.queryNodes` procedure that would duplicate the
  scalar surface and create a second optimization path.
- **Variable-length path materialization** — the one outlined follow-up in
  `graph/DESIGN_DECISIONS.md`.
- **Database-wide protection of owned backing tables.**

### Known hard blocks

- Runtime `TypeError`s for entity values flowing through `Any`-typed lists
  (~19 corpus identities) need an error-raising SQL function; a `SELECT` cannot
  raise.
- AGE jsonb operators (`?`, `@>`, `#>`), pgvector `OPERATOR(...)`, and ~40
  expected-error adapter artifacts are donor-semantic conflicts with the
  TCK-normative behavior this frontend follows — they are *divergences*, not
  bugs, and are enforced as such through `registries/divergence.toml`.

### Ergonomics gap

There is no `bindings/rust`-level async wrapper for the graph frontend (nor for
`turso_pg`); only core SQL has the `turso` crate's async `Rows`/`Transaction`
surface. Consumers embed `turso_graph_frontend` synchronously.

### Natural extensions of the role model

The role model makes several things cheap that were previously structural
changes, and none of them are implemented:

- Role-qualified variable-length traversal (today `GraphExpand` discovers
  candidate sources through the `start`/`end` lookup, which is why arrow
  traversal needs that pair).
- Roles targeting fragments directly at query time rather than being expanded
  to a concrete member set at registration.
- Relation-as-player beyond one level — the model permits it; nothing exercises
  deep nesting.

## Historical and process documents

These are artifacts of how the frontend was built, not descriptions of how it
works. They are accurate for their moment and are not maintained against the
current tree:

| Document | What it is |
|---|---|
| `graph/BRANCH_QUALITY_REVIEW.md` | A dated hygiene review with its resolution pass |
| `graph/MAIN_MERGE_LEVERAGE.md` | Notes from one merge with `main` |
| `graph/PROVENANCE.md` | **Not historical — binding.** Pinned donor sources, licenses, adaptation records. Read before importing anything |
| `graph/memory-observability.md` | Phased memory-measurement design |
| `graph/docs/core-changes.md` | The core-side diff this frontend required |
| `docs/graph-frontend-core-alignment.md` | Where graph diverges from core's frontend model, with the closure plan |
| `docs/archive/plans/` | Superseded delivery plans |
