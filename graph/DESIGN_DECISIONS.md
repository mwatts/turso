# Graph frontend: design decisions

Status: all decisions are made and implemented on this branch
(variable-length path materialization is the one outlined follow-up).

- Labels: **junction table** (chosen, implemented — filtered scans,
  labels()/label(), ±labels side effects; extended to a relationship-type
  junction with a persistent identity registry).
- Path values: **first-class Path IR** (chosen, fixed-length implemented —
  PathValue IR, nodes()/relationships()/length(); variable-length paths
  await traversal-emitted path data).
- Temporal: **core time_* functions** (chosen, implemented — constructors,
  accessors, ISO rendering; durations remain).
- CALL: **minimal registry** (chosen, implemented — db.labels,
  db.relationshipTypes).

Original option analysis below.

Corpus state when written: 6,161 / 10,392 passing. Each decision below
blocks a measured family of remaining failures. Options are ordered by
increasing scope; the recommendation states a default, not a commitment.

## 1. Label storage and filtering (~500 failures, plus silent wrong results)

The engine registers one node table per graph. Labels resolve to catalog
identities at bind time and are then dropped: scans do not filter by label,
`labels(n)` only knows pattern-declared names, results cannot render
`(:A {p: 1})`, and any query that distinguishes labels over a shared table
returns wrong counts (e.g. `MATCH (p:Person)` also matching an `Anchor`).

- **Option A — label column on the registered node table.** Registration
  gains an optional `label_column`; `CreateNode` writes the label name (or a
  JSON array for multi-label), `NodeScan` lowering appends
  `WHERE label_column = 'X'` (JSON containment for multi-label), `labels()`
  reads the column, and the testkit fixtures adopt it through the dynamic
  catalog. Smallest model change; single-table layout preserved; unlocks
  filtering, rendering, and label side-effect counts at once. Existing
  registrations without the column keep today's permissive behavior.
- **Option B — table per label.** Each label maps to its own table (the
  registration vec already has that shape). Clean relational mapping, but
  unlabeled scans become unions over every table, multi-label nodes need
  duplication, and dynamically created labels need DDL per label.
- **Option C — junction table** `node_labels(node_id, label)` maintained by
  mutations; scans join it. Full multi-label fidelity including later
  SET/REMOVE label support, at the cost of a join on every labeled scan and
  more registration surface.

Recommendation: A now (with the JSON-array variant it is semantically
C-equivalent), C as the long-term fidelity end-state if label mutation
support is wanted.

## 2. Path values and relationship-list materialization (~150–300 failures)

Named paths (`p = (a)-[]->(b)`) and named variable-length relationships
(`[r:T*1..3]`) currently register scope bindings with no backing value;
`nodes(p)`, `relationships(p)`, `length(p)`, and list operations over `r`
fail at planning.

- **Option A — JSON path values, staged.** Fixed-length paths materialize at
  lowering time as a JSON structure (`{nodes: [...], rels: [...]}`) built
  from the already-bound identity columns; `nodes()/relationships()/length()`
  become JSON extractions. Variable-length paths follow by having the
  traversal snapshot emit a path-JSON column (graph/runtime work, no core
  changes). Fits the existing lists-are-JSON design.
- **Option B — fixed-length only.** The subset of A that needs no runtime
  work. Immediate wins for `length(p)`/`nodes(p)` on fixed patterns; the
  frequent donor `nodes(p)[0]` over `[*]` patterns stays unsupported.
- **Option C — first-class Path IR value.** A typed
  `ir::Expression::PathValue` with dedicated lowering and accessors. Most
  principled, most work, and still needs decision 1 to render entities
  inside paths.

Recommendation: A, shipped in the B-first order.

## 3. Temporal value model (~150 failures)

`datetime({year: ...})`, `duration(...)`, `localtime`, and friends have no
value model; the namespaced constructors parse but nothing executes.

- **Option A — text-encoded temporals over SQLite conventions.** Map
  constructors onto strftime/julianday encodings (ISO-8601 text); component
  access (`.year`) via strftime; comparisons work lexically on ISO text and
  many TCK expectations are exact ISO strings. Duration arithmetic and
  component-map constructors are fiddly; timezones are poor.
- **Option B — custom scalar types.** The engine already supports
  `CREATE TYPE ... BASE ... ENCODE/DECODE`; define datetime/duration custom
  types and lower constructors/accessors to their functions. Typed storage,
  but depends on the custom-type function surface and fixture registration.
- **Option C — core temporal types.** Real temporal support in turso core.
  Out of scope for the frontend, and the long-term correct answer.

Recommendation: A for corpus coverage now; raise C with core.

## 4. CALL procedures (~100 failures)

`CALL db.labels()` and donor procedures fail at "unsupported starting
clause".

- **Option A — minimal built-in registry.** Parse
  `CALL name(args) [YIELD cols]`; implement a handful backed by the catalog
  (`db.labels`, `db.propertyKeys`, `db.relationshipTypes`, a `dbms.components`
  stub); unknown procedures get a clean error. YIELD needs a small
  projection shim.
- **Option B — parse-and-reject.** Grammar only; every CALL errors with
  "unknown procedure". Moves parser failures to execution failures with few
  real passes.
- **Option C — defer entirely.**

Recommendation: A with three to five procedures.

## Acknowledged hard blocks (no decision needed)

- `reduce()` (71) needs recursive CTEs; turso core rejects them today.
- Runtime TypeErrors for entity values flowing through `Any`-typed lists
  (~19) need an error-raising SQL function; SELECT cannot raise.
- AGE jsonb (`?`, `@>`, `#>`), pgvector `OPERATOR(...)`, and ~40
  expected-error adapter artifacts are donor-semantic conflicts with the
  TCK-normative behavior this frontend follows.
