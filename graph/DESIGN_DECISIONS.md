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
  accessors, ISO rendering).
- Durations: **graph-owned custom type over the static-extension mechanism**
  (chosen, implemented — `graph/temporal` registers `duration_*` scalar
  functions backed by Rust/jiff arithmetic; fixtures declare
  `CREATE TYPE duration BASE TEXT` with experimental custom types enabled;
  the binder types values with a `cypher_duration` marker and rewrites
  constructors, accessors, `duration.between`, and datetime ± duration).
- CALL: **typed descriptor registry and explicit procedure IR** (chosen,
  implemented — `db.labels`, `db.relationshipTypes`, `db.propertyKeys`).

Original option analysis below — retained as a historical record; several
recommendations were superseded by the implemented decisions listed above
(e.g. junction tables over Option A label columns).

**Live corpus pass/fail counts are not maintained in this file.** Use
[`test-results/REPORT.md`](test-results/REPORT.md) (regenerated from
`test-results/history.jsonl` on recorded baseline runs). Each decision below
blocked a measured family of failures at the time it was written. Options are
ordered by increasing scope; the recommendation states a default, not a
commitment.

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

## Result ordering contract

`SEMANTIC_PROFILE.row_order` is `OrderedOnlyUnderExplicitOrderBy`. A result's
row order is part of its identity only when the outermost RETURN carries
`ORDER BY`. Everything else is a **multiset**: duplicates count, order does not.

| Result | Comparison | Recorded digest |
| --- | --- | --- |
| RETURN … ORDER BY … | sequence | `fnv1a64:` |
| RETURN … (no ORDER BY) | multiset (sorted, duplicates retained) | `fnv1a64u:` |
| aggregate without ORDER BY | multiset | `fnv1a64u:` |
| UNWIND without ORDER BY | multiset | `fnv1a64u:` |
| `labels(n)` list contents | sequence, label-table insertion order | n/a |

UNWIND is the one that looks like an exception and is not. It lowers to
`SELECT q.*, j.value … FROM (…) AS q JOIN json_each(<list>) AS j`
(`graph/frontend/src/lowering.rs`) with no `ORDER BY`, so list order survives
only because the current plan happens to preserve it. Guaranteeing it would mean
emitting an explicit order and adding the guarantee to `SEMANTIC_PROFILE`, with
a version bump — not asserting it in a manifest.

`labels(n)` is a real exception. Relational lowering emits `ORDER BY lbl.rowid`
inside the `json_group_array` subquery for `labels()` and the `LIMIT 1` subquery
for `label()` (`graph/frontend/src/lowering.rs`), so the list is deterministic
within one database. It is not
portable across databases that inserted the same labels in a different order,
and no test may depend on cross-database label order.

Suite manifests may not declare `ordering = "ordered"` for a query with no
`ORDER BY`; `ScenarioManifest::validate` rejects it, and the fixed-pattern
manifest is held to the same rule by
`graph/frontend/tests/fixed_pattern_fixtures.rs`. For a mutation the rule reads
`verification_query`, since that is what produces the compared rows.

## Path algorithm legality

Written before the syntax exists. `graph/cypher/src/cypher.pest` has
`range_literal` (`[r:T*1..3]`) but no `SHORTEST`, `ALL SHORTEST`, `TRAIL`, or
`ACYCLIC` selector. `turso_graph_runtime::resolve_path_algorithm`
(`graph/runtime/src/path_policy.rs`) is the enforcing copy of this table; it is
total, and a combination it refuses cannot be reached by any search entry
point.

Uniqueness is `turso_graph_ir::PathUniqueness`: `Walk` may repeat nodes and
edges, `Trail` may not repeat an edge, `Path` may not repeat a node.

| Selector | Weights | Walk | Trail | Path |
| --- | --- | --- | --- | --- |
| ANY | any | BFS | BFS | BFS |
| ALL | any | not supported | DFS enumeration | DFS enumeration |
| SHORTEST | unweighted | BFS | BFS | BFS |
| SHORTEST | non-negative | Dijkstra | Dijkstra | Dijkstra |
| SHORTEST | negative | not supported | not supported | not supported |
| ALL SHORTEST | unweighted | BFS level set | BFS level set | BFS level set |
| ALL SHORTEST | non-negative | Dijkstra level set | Dijkstra level set | Dijkstra level set |
| ALL SHORTEST | negative | not supported | not supported | not supported |
| SHORTEST k | unweighted | not supported | Yen | Yen |
| SHORTEST k | non-negative | not supported | Yen | Yen |
| SHORTEST k | negative | not supported | not supported | not supported |

Reasons for each refusal:

- **ALL over walks.** One cycle makes the answer infinite. The hop limit would
  bound it, but the result would then be an arbitrary prefix, which is the
  silent truncation `graph/runtime/src/traversal.rs` deliberately refuses.
- **SHORTEST over walks with negative weights.** A negative cycle means no
  shortest walk exists.
- **SHORTEST over trails or paths with negative weights.** Shortest simple path
  with negative weights is NP-hard; there is no correct polynomial algorithm to
  offer.
- **SHORTEST k over walks.** Yen's algorithm requires a simple-path constraint.

Two things this table does not say. It does not claim an algorithm is
implemented: `PathAlgorithm::YenKShortest`, `BreadthFirstAllShortest`, and
`DijkstraAllShortest` are sound and unbuilt (`PathAlgorithm::is_implemented`
answers that separate question), and reaching them yields
`RuntimeError::PathAlgorithmNotImplemented`, distinct from
`RuntimeError::UnsupportedPathCombination`. And it does not describe reachable
state today: `EdgeInput.weight` and `Path.total_weight` are `u64`, so
`WeightClass::Negative` is unreachable from the current type. Those rows exist
so that widening the weight type trips a policy error rather than quietly
feeding negative edges to Dijkstra.

`PATH_POLICY_VERSION` is mirrored into
`turso_graph_ir::SEMANTIC_PROFILE.path_policy_version`, so a change to this
table moves the semantic profile digest recorded with every test run.

## Acknowledged hard blocks (no decision needed)

- ~~`reduce()` (71) needs recursive CTEs; turso core rejects them today.~~
  Superseded twice: `reduce()` first shipped with polymorphic `+`/`/`
  semantics over an unrolled ten-rung ladder, and now folds with a real
  `WITH RECURSIVE` after core implemented recursive CTEs (`4360b24f5`), so
  the ten-element ceiling is gone. Aggregates anywhere inside a `reduce()`
  are rejected at bind time: the fold's rows are not the outer rows, so an
  aggregate there cannot mean what it reads as. AGE and Neo4j reject it too.
  Aggregate first in an earlier clause (`WITH collect(x) AS xs`) and fold the
  result.
- Runtime TypeErrors for entity values flowing through `Any`-typed lists
  (~19) need an error-raising SQL function; SELECT cannot raise.
- AGE jsonb (`?`, `@>`, `#>`), pgvector `OPERATOR(...)`, and ~40
  expected-error adapter artifacts are donor-semantic conflicts with the
  TCK-normative behavior this frontend follows.
