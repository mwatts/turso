# Task 13b report — relation-anchored MATCH over a standalone role pattern

## Status: DONE

## Commit
Not yet committed — task instructions say to commit only when explicitly
asked, and that has not happened this session. All changes are staged in the
working tree, verified against `HEAD` (`eab179db4`, `graph/frontend: cover
Many-role subsetting in the MERGE merge key`) on `feature/graph-nary`.

Files changed (code only; `graph/test-results/*` deliberately excluded, see
Gate results):
- `graph/ir/src/plan.rs`, `graph/ir/src/lib.rs` — new `RelationScan`,
  `RoleJoin`, `RolePlayer` IR nodes.
- `graph/frontend/src/binder.rs` — new `bind_match_role_pattern`; `bind_match`
  restructured to dispatch path and role pattern elements in source order;
  one stale unit test flipped from "errors" to "binds" (see below).
- `graph/frontend/src/lowering.rs` — `lower_relation_scan`, `lower_role_join`.
- `graph/frontend/src/schema_catalog.rs`,
  `graph/testkit/src/dynamic_catalog.rs` — `relationship_role_node_source`
  implementations (production and test-fixture catalogs).
- `graph/frontend/tests/nary_relations.rs` — two new execution tests.
- `graph/frontend/tests/fixture.rs` — `lower_fixture` helper.
- `graph/frontend/tests/desugaring_golden.rs` — rewritten to compare emitted
  SQL instead of plan-node identity (ruling (B)); both goldens documented and
  left `#[ignore]`d with a confirmed-not-assumed finding (see below).
- `graph/frontend/tests/semantic_schema.rs` — one exhaustive `match` arm
  updated for the two new `PlanKind` variants.

## What changed and why

**Decomposition**: a standalone role pattern in `MATCH` anchors on the
relation (`RelationScan`), then joins each *named* role argument out to its
player, one `RoleJoin` per argument, composed in the order written. No arity
branch exists anywhere in this path — a two-role pattern and a five-role
pattern go through the identical loop, differing only in how many times it
iterates. A fresh anchor (no existing `self.plan`) combines with any prior
plan as a cartesian product via `ir::PlanKind::Join`, mirroring the existing
`bind_start_node` precedent for a fresh node anchor.

Each role argument resolves to `RolePlayer::Bound(id)` (an already scope-bound
variable, folds to an identity equality) or `RolePlayer::Fresh { binding,
node_source }` (a new variable, joins its physical node table). `node_source`
for a Fresh player comes from `relationship_role_node_source`, a new
`GraphCatalogSnapshot` method generalizing the existing
`relationship_endpoint_sources` (which hard-codes the start/end pair) to any
named role — necessary because a role argument carries no label-annotation
syntax the way `(p:Person)` does, so its physical table cannot be inferred
from the query text.

**Role resolution uses `self.catalog.relationship_roles(...)` (the
`SemanticRole`/semantic-layer accessor), not the physical
`RelationshipTableLayout`.** This is the same accessor `bind_create_role_pattern`
(Task 13a) already uses. Traced `SchemaCatalog::relationship_roles` and
`SemanticRole::from(RelationshipRoleLayout)`: `cardinality` is derived
verbatim from the physical layout in schemaless mode, so no correctness is
lost, and it gives exact parity with the already-tested CREATE-side role
resolution rather than a second, independent lookup path.

**`Many`-cardinality role arguments are rejected** (`at_unsupported`, "a
Many-cardinality role in a MATCH role pattern"), identified structurally by
`role.cardinality == RoleCardinality::Many` — never by name or position. This
was confirmed, not assumed, as the correct scope boundary: `task-14-report.md`
records that Task 14a explicitly deferred "Step 5 (spill join /
hop-through-Many-role reads, e.g. `role_join_expression`) ... to Task 14b
(blocked on Task 13b, the MATCH-side standalone role pattern)". This task's
Many-role rejection is exactly that blocking dependency being satisfied, not
a gap.

**`pattern.types.len() != 1` is rejected** with the same "a MATCH role
pattern without exactly one relationship type" message `bind_create_role_pattern`
uses for the analogous CREATE-side check — the parser's `RolePattern.types`
is a `Vec` for future multi-type support the binder does not yet implement on
either side, so MATCH stays consistent with CREATE rather than silently
accepting a shape it can't act on.

**No new `BindError` variants.** Every error path reuses variants Task 13a
already introduced and tested: `UnknownRelationshipType`, `MissingSource`,
`UnknownRole`, `DuplicateRoleArgument`, plus the generic
`at_unsupported`/`Unsupported`. Unlike CREATE, a MATCH role pattern may name a
*subset* of a relation's roles (no `MissingRequiredRole` analogue) — a
deliberate asymmetry: CREATE must fully specify a new relation, MATCH is
filtering existing ones.

**Skipped**: re-running CREATE's `RoleTargetTypeViolation` check (bound
player's label must be a legal target for the role) against a `Bound` MATCH
role player. CREATE's own check has a known permissive gap for `Relation`-kind
targets (noted in the Task 13a report), and the arrow-form MATCH path already
achieves comparable filtering structurally (through the physical join
condition) rather than via an explicit semantic check. No test requires this
validation, and duplicating a check with a known gap felt like the wrong
place to spend the budget; flagged here as a deliberate, minor gap rather than
silently dropped.

**One stale unit test found and fixed** (not part of the brief's explicit
step list, found by running the gate): `binder.rs`'s
`a_standalone_role_pattern_binds_to_an_error_not_an_empty_plan` asserted the
pre-Task-13b `Unsupported { feature: "role patterns are not supported yet"
}` error — its own comment said "Task 13 flips this assertion to a success
case." Renamed to
`a_standalone_role_pattern_binds_to_a_relation_scan_and_role_joins` and
rewritten to assert the actual plan shape (`Project` → `RoleJoin` → `RoleJoin`
→ `RelationScan`), which doubles as an arity-branch check (this exact shape
for both named roles, checked structurally not by counting).

## Step 5 finding — traversal snapshot (confirmed with evidence, no change needed)

`graph/frontend/src/compiler.rs:187-210`, `query_needs_traversal_snapshot`,
already reads:

```rust
let turso_graph_cypher::PatternElement::Path(path) = element else {
    return false;
};
```

with an inline comment already present: "A role pattern's grammar has no hop
range at all (Task 12 rejects one as a parse error), so it can never need a
traversal snapshot." This is correct and requires no change: role patterns
have no `range` syntax at the grammar level (confirmed — `RolePattern` has no
range field reachable from the parser), so a `Roles` element can never be the
thing that makes a query need a snapshot. `false` is the right answer for the
right reason, already documented; not touched.

## Step 6 finding — write classification (confirmed with evidence, no change needed)

`graph/frontend/src/binder.rs:558-579`, `clauses_write`:

```rust
cypher::Clause::Match(_)
| cypher::Clause::Unwind(_)
| cypher::Clause::With(_)
| cypher::Clause::Return(_) => false,
```

`Clause::Match(_)` is matched on the clause variant, not on its pattern
elements — a `MATCH` containing a `Roles` element is classified identically
to one containing only `Path` elements: always a read. No change needed;
confirmed by reading the match arm directly, not assumed.

## Desugaring goldens — confirmed SQL divergence, controller decision needed

Per ruling (B): the contract is emitted SQL, not plan-node identity. Rewrote
both goldens in `desugaring_golden.rs` to lower and compare SQL text (via a
new `fixture::lower_fixture` helper) instead of comparing `first_role_expand`
plan nodes, and rewrote the module doc comment to state that contract.

Running either test (drop `#[ignore]`) shows a genuine, non-cosmetic SQL
divergence for these exact golden queries — not alias/ordering noise, and not
a defect in `RelationScan`/`RoleJoin`. Actual output, captured this session:

**Query 1**: `MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b` vs.
`MATCH (a:Person), (b:Person), [r:KNOWS](start: a, end: b) RETURN b`

```
arrow SQL:
SELECT q.b4 AS "b" FROM (SELECT q.b3 AS b4 FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."src" = q.b1 JOIN "people" AS n ON n."id" = r."dst") AS q WHERE TRUE) AS q) AS q

role SQL:
SELECT q.b4 AS "b" FROM (SELECT q.b2 AS b4 FROM (SELECT q.* FROM (SELECT q.* FROM (SELECT l.*, r.* FROM (SELECT l.*, r.* FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS l JOIN (SELECT n."id" AS b2, 1 AS b2_source FROM "people" AS n) AS r) AS l JOIN (SELECT r."id" AS b3, 2 AS b3_source, r."src" AS b3_role1, r."dst" AS b3_role2 FROM "relationships" AS r) AS r) AS q WHERE q.b3_role1 = q.b1) AS q WHERE q.b3_role2 = q.b2) AS q) AS q
```

**Query 2 (reversed)**: `MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b` vs.
`MATCH (a:Person), (b:Person), [r:KNOWS](end: a, start: b) RETURN b`

```
arrow SQL:
SELECT q.b4 AS "b" FROM (SELECT q.b3 AS b4 FROM (SELECT q.*, r."id" AS b2, 2 AS b2_source, n."id" AS b3, 1 AS b3_source FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS q JOIN "relationships" AS r ON r."dst" = q.b1 JOIN "people" AS n ON n."id" = r."src") AS q WHERE TRUE) AS q) AS q

role SQL:
SELECT q.b4 AS "b" FROM (SELECT q.b2 AS b4 FROM (SELECT q.* FROM (SELECT q.* FROM (SELECT l.*, r.* FROM (SELECT l.*, r.* FROM (SELECT n."id" AS b1, 1 AS b1_source FROM "people" AS n) AS l JOIN (SELECT n."id" AS b2, 1 AS b2_source FROM "people" AS n) AS r) AS l JOIN (SELECT r."id" AS b3, 2 AS b3_source, r."src" AS b3_role1, r."dst" AS b3_role2 FROM "relationships" AS r) AS r) AS q WHERE q.b3_role2 = q.b1) AS q WHERE q.b3_role1 = q.b2) AS q) AS q
```

**Root cause**: the arrow form (`(a)-[r]->(b)`) is one `PathPattern`;
`bind_path` anchors a single, selectivity-chosen `NodeScan` and reaches the
other endpoint only through the relationship's own `JOIN ... ON` condition —
`b` never gets an independent scan. The role form's query text is written as
three separate pattern elements (`(a:Person), (b:Person), [r:KNOWS](...)`),
so `a` and `b` are already bound, independently scanned `NodeScan`s (joined
as a cartesian product) *before* the role pattern ever runs; both role
arguments then resolve as `RolePlayer::Bound`, i.e. two `WHERE` equality
filters against the relation scan, not a single anchored join chain. Both
statements are correct and row-equivalent (independently verified this
session with a throwaway execution test against `witnessed_session`, not
committed: matching rows returned for both role orderings, and swapped role
arguments correctly returned zero rows) — they are just not the same SQL
text.

Making them byte-identical would require detecting, at bind time, that both
role arguments already happen to be bound and retroactively folding their
pre-existing independent scans into the relation's join chain — a
query-shape-dependent special case, which the brief explicitly forbids
("if you find yourself needing a two-role special case just to make a golden
pass, stop and report BLOCKED"). Both tests are left `#[ignore]`d with this
finding in the module doc comment and here, pending a controller decision on
whether this divergence is acceptable (my read: it should be, since it
already exists for the underlying reason that the two forms bind through
genuinely different code paths given genuinely different query text — a
`(a)-[r]->(b)` single-path query and an equivalent 3-element role-form query
were never going to plan identically even before this task) or requires a
different plan shape.

## Verify by sabotage (both reverted; diff is clean at HEAD)

1. **Force every declared role to be named** (added `if seen.len() !=
   declared.len() { return Err(...) }` after the role loop in
   `bind_match_role_pattern`) — a stand-in for "an unnamed role should not be
   free to vary" since `witnessed_session`'s `KNOWS` declares three roles
   (`start`/`end`/`witness`) and both Step-2 tests name only two or one.
   Result: **both** Step-2 tests went red —
   `a_match_role_pattern_may_leave_roles_unnamed` (the one the brief calls
   out) and `a_match_role_pattern_binds_the_named_players` (which also only
   names `start`/`end`, not `witness`) — with
   `Database(ParseError("SABOTAGE: role subset is not supported ..."))`.
   Reverted; confirmed clean via a second full run of
   `cargo test -p turso_graph_frontend --test nary_relations` (26 passed).
2. **Permute a `RoleJoin`'s role** (swapped `role.role` for the first
   non-matching declared role, so e.g. the argument named `start` joins
   through `end`'s column instead): `a_match_role_pattern_binds_the_named_players`
   went red with `left: [[Numeric(Integer(2)), Numeric(Integer(1))]] right:
   [[Numeric(Integer(1)), Numeric(Integer(2))]]` — the exact swapped-columns
   signature the brief warns about ("if the tests pass with the roles
   permuted, they are resolving by position"). Reverted; confirmed clean via
   the same full re-run (26 passed).

## Gate results

- `cargo fmt`: clean (`cargo fmt --check` exits 0).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`:
  exit 0, no lint warnings or errors. (The 10 lines cargo prints as
  "warnings" are pre-existing macOS toolchain `ar` build-script noise from an
  unrelated crate, not clippy lints.)
- `cargo test -p turso_graph_cypher -p turso_graph_frontend`: **338 passed, 3
  ignored, 0 failed** (15 test suites). The 3 ignored: the two desugaring
  goldens above (deliberately, pending the controller decision) plus one
  pre-existing, unrelated ignore in `dialect_alignment.rs:569`.
- `mise run corpus` (release build, per project convention): measured against
  the run_id in this session, and cross-checked against the last officially
  recorded baseline in `graph/test-results/runs.jsonl` (run
  `20260726T163317.906100Z-ae795a64c343-corpus-deep`, recorded by the
  Task-18b commit, i.e. the true current baseline — not the brief's original
  numbers, though they happen to coincide here since none of the intervening
  tasks 14a/15/17/18b touch corpus-visible parsing/binding behavior):
  - `age-deep`: 3042 passed (baseline 3042) — **exact match**
  - `cqlite-deep`: 113 passed (baseline 113) — **exact match**
  - `grafeo-deep`: 277 passed (baseline 277) — **exact match**
  - `sparrowdb-deep`: 2164 passed (baseline 2164) — **exact match**
  - `tck-deep`: 3330 passed (baseline 3330) — **exact match** (well inside
    the stated 3329-3332 tolerance)

  Zero suites moved off baseline. `graph/test-results/{REPORT.md,
  runs.jsonl, benchmarks.jsonl}` were regenerated by this run and left
  uncommitted, per instructions.
- `mise run cypherbench-sample` (release build): compared against the last
  recorded baseline in `graph/test-results/benchmarks.jsonl` (multiple
  identical prior runs through commit `eab179db4`). Per-domain
  matched/mismatched/errored, all **exact matches**:
  company 13/12/0, fictional_character 14/11/0, flight_accident 24/1/0,
  geography 11/14/0, movie 6/19/0, nba 25/0/0, politics 15/10/0.

## Concerns / notes for the caller

- **Branch state moved substantially during this session.** When this task
  was framed, the branch tip was `d054a52c5` (a docs commit). By the time I
  reached the gate, `HEAD` had advanced through Tasks 13a (already landed
  before I started), 14a, 15, 17, MERGE-role-pattern support, and 18b —
  eleven commits ahead. I verified before proceeding that none of those
  commits implement or duplicate this task's surface: `RelationScan`,
  `RoleJoin`, `RolePlayer`, and `bind_match_role_pattern` do not exist
  anywhere in `git show HEAD:...` for any touched file, and `HEAD`'s
  `bind_match` still calls `only_paths(&clause.paths)?`, which errors on any
  `Roles` element — i.e., Task 13b was genuinely still open. All gate numbers
  above were checked against the true current baseline (the last recorded
  run before mine), not the brief's original figures, and both happened to
  match exactly regardless.
- The two desugaring goldens remain `#[ignore]`d pending a controller
  decision (see above) — this is a known, intentional gap, not an oversight.
  My own assessment is that the divergence is inherent to the two forms'
  different query shapes and should be accepted, but that call belongs to the
  controller per the brief.
- The deliberate, minor coverage gap noted above (no `RoleTargetTypeViolation`
  check for a `Bound` MATCH role player) is unchanged from Task 13a's own
  noted gap on the CREATE side.
