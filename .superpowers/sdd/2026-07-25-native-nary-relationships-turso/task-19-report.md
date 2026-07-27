# Task 19 report: create atomicity and relation-as-player

Commit: `e963b573a` on `feature/graph-nary`
("graph/frontend: let a relation be a role's player, prove create atomicity")

## Part A -- atomicity: mechanism chosen and why

Used `SemanticRequiredProperty` (`semantic_constraints.rs`), not the plan's
"last role's target-type check" alternative. `Citation.label` is registered
required via `register_semantic_constraints`. Cypher `CREATE` does not check
"required" at bind time -- it is enforced only by
`constraints.validate_state(connection)`, called from `run()` in
`mutation.rs:289-298` *after* `try_single_program_mutation`/`execute_bound`
returns. A `CREATE [c:Citation](cited: x, witnesses: w1, witnesses: w2)` that
omits `{label: ...}` therefore binds successfully, writes the relation row and
both `witnesses` spill rows, and only then fails the post-insert NULL scan
(`semantic_constraints.rs:401-415`) -- everything already on disk, still
inside the open transaction. This is exactly the "partway through" condition
the brief describes; no injection hook or production change was needed.

Test: `a_failure_partway_through_an_n_ary_create_leaves_nothing_behind`
(`graph/frontend/tests/nary_relations.rs`). Asserts on the typed error
(`FrontendError::Mutation(MutationError::SemanticConstraintViolation(detail))`
with `detail.contains("Citation.label") && detail.contains("required")`), then
asserts **both** `SELECT count(*) FROM citations` and
`SELECT count(*) FROM citations__witnesses` are zero via a second connection.

## Part A -- sabotage 1 (verbatim)

Change: inserted, at the very top of `execute_cypher_mutation` in
`mutation.rs` (before the `BEGIN IMMEDIATE`/`SAVEPOINT` window opens), an
error-ignored raw insert into the fixture's spill table:

```rust
let _ = connection.execute(
    "INSERT INTO \"citations__witnesses\"(relation_id, node_id) VALUES (999999, 999999)",
);
```

Ran `cargo test -p turso_graph_frontend --test nary_relations -- a_failure_partway_through_an_n_ary_create_leaves_nothing_behind --exact --nocapture`. Verbatim failure:

```
thread 'a_failure_partway_through_an_n_ary_create_leaves_nothing_behind' panicked at graph/frontend/tests/nary_relations.rs:1696:5:
assertion `left == right` failed: both spill rows, already inserted before validate_state ran, must not survive either
  left: [[Numeric(Integer(3))]]
 right: [[Numeric(Integer(0))]]
```

(Count is 3, not 1, because the sabotaged insert runs unconditionally on
every `execute_cypher_mutation` call in the test -- the two earlier,
successful statements each commit one stray row outside any transaction,
plus the failing statement's own attempt, which correctly rolls back its
real spill rows. All three cases confirm the test is sensitive to any leak,
not just the specific failure path.) Reverted immediately after; `git diff
--stat graph/frontend/src/mutation.rs` shows no diff post-revert, and the
full `cargo test -p turso_graph_frontend` (334 passed, 1 ignored) was
re-confirmed green.

## Part B -- production fix

`graph/frontend/src/binder.rs`, `bind_role_player` (~line 1830). Original
code discarded `RoleTarget::Relation(_) => None` when building the allowed
list and checked only `catalog.label(...)` against `RoleTarget::Node`. Fixed
to build two separate allowed lists (`allowed_labels` from `RoleTarget::Node`,
`allowed_relations` from `RoleTarget::Relation`) and check a binding against
the target kind it actually is: `CatalogEntity::Node` bindings resolve names
through `catalog.label` against `allowed_labels` (unchanged from before);
`CatalogEntity::Relationship` bindings now resolve names through
`catalog.relationship_type` against `allowed_relations`, the mirror of the
node-side logic. No writer change: `insert_relationship` already stores a
role player as an opaque identity value regardless of what catalog space it
came from, so "do not special-case the writer" required no branch -- there
was nothing to add.

### `binder.rs:1666` (arrow-form start/end check)

**Conclusion: left alone.** It carries the same `RoleTarget::Relation(_) =>
None` discard, plus hard-coded `"start"`/`"end"` role names, same as the
role-pattern check did before this fix. But arrow-form grammar
(`(a)-[r:T]->(b)`) only ever permits a `NodePattern` at each endpoint, and
`bind_created_node` (which binds `to`/`from`) always yields
`CatalogEntity::Node` -- there is no Cypher syntax that lets a relation
identity occupy an arrow-form endpoint position, so `RoleTarget::Relation`
can never reach that code path in the first place. None of the five new
tests exercise arrow-form syntax against a relation-targeting role (all use
the standalone role-pattern `[x:T](role: player)` form), so no test proves
it reachable, consistent with the brief's instruction to leave it alone
absent such a test. This is a "known deferred item" finding, not newly
verified.

## Part B -- tests and verbatim error messages

Four tests added to `graph/frontend/tests/nary_relations.rs`, function names
and doc comments exactly as specified in the brief. New fixture
`citation_session()` added to `graph/frontend/tests/fixture.rs`: `Text` node
source (with a declared `title` property -- semantic-schema mode resolves
properties only through declared `SemanticProperty` entries, never falling
back to the identity column, confirmed by reading
`SchemaCatalog::resolve_owned_property`/`property`, so `{id: N}` does not
bind against a semantically-typed node with no declared properties; every
other fixture in this file is schemaless, so this file's existing tests
never hit that path), `Transcription` relationship type (`source` role,
targets `Text` only), `Citation` relationship type (`cited` role,
relation-only, targets `Transcription`; `reference` role, mixed, targets
`Text` and `Transcription`, optional; `witnesses` role, `Many`-cardinality,
targets `Text`, optional -- exists only to give Part A's test a spill table).

Error messages observed (from sabotage runs against the real fixed check,
quoted verbatim from panic output):

- `a_role_that_accepts_only_relations_refuses_a_node_player` asserts
  `RoleTargetTypeViolation { relationship_type: "Citation", role: "cited",
  found: "Text", .. }`.
- `a_role_that_does_not_accept_relations_refuses_a_relation_player` asserts
  `RoleTargetTypeViolation { relationship_type: "Transcription", role:
  "source", found: "Transcription", .. }`.

Both assert on the concrete typed enum variant and its fields
(`FrontendError::Mutation(MutationError::Bind(BindError::RoleTargetTypeViolation
{ .. }))`), matching the existing `semantic_schema.rs` idiom, not a substring
of the rendered message.

## Sabotages 2-4 (verbatim)

**Sabotage 2** -- restored `RoleTarget::Relation(_) => None` in
`bind_role_player` (reverting to the single `allowed`/label-only check).
`cargo test -p turso_graph_frontend --test nary_relations -- a_role_that_accepts_only_relations_refuses_a_node_player a_role_with_both_node_and_relation_targets_accepts_either --exact`:

```
---- a_role_that_accepts_only_relations_refuses_a_node_player stdout ----
thread 'a_role_that_accepts_only_relations_refuses_a_node_player' panicked at graph/frontend/tests/nary_relations.rs:1761:10:
a role targeting only relations must refuse a node player: MutationSummary { matched_rows: 1, operations_executed: 1, rows: [], result_types: [] }

---- a_role_with_both_node_and_relation_targets_accepts_either stdout ----
thread 'a_role_with_both_node_and_relation_targets_accepts_either' panicked at graph/frontend/tests/nary_relations.rs:1851:10:
a Transcription relation must also be an accepted `reference` player: Mutation(Bind(RoleTargetTypeViolation { relationship_type: "Citation", role: "reference", found: "Transcription", span_start: 90, span_end: 102 }))

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 42 filtered out; finished in 0.06s
```

Both required tests went red. Reverted; `git diff --stat` on `binder.rs`
confirmed back to the committed fix, and both tests pass again.

**Sabotage 3** -- changed the `CatalogEntity::Relationship` arm to
`names.iter().all(|name| self.catalog.relationship_type(self.graph,
name).is_some())`, dropping the `allowed_relations.contains(...)` check
entirely (accept any resolvable relationship type). Ran
`cargo test -p turso_graph_frontend --test nary_relations -- a_role_that_does_not_accept_relations_refuses_a_relation_player --exact`:

```
thread 'a_role_that_does_not_accept_relations_refuses_a_relation_player' panicked at graph/frontend/tests/nary_relations.rs:1800:10:
`source` targets Text only, so a Transcription is not a legal player: MutationSummary { matched_rows: 1, operations_executed: 1, rows: [], result_types: [] }

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.06s
```

Went red. Reverted.

**Sabotage 4** -- in `fixture.rs`'s `citation_session`, swapped the `name`
field only between the `cited` and `reference` `SemanticRoleRegistration`
entries (declared order and each entry's `targets`/`optional`/`cardinality`
left exactly as they were at that position -- i.e. the entry declared first
kept `targets: ["Transcription"], optional: false` but is now named
`"reference"`; the second kept `targets: ["Text","Transcription"], optional:
true` but is now named `"cited"`). Ran
`cargo test -p turso_graph_frontend --test nary_relations -- a_relation_may_be_a_player_of_another_relation a_role_that_accepts_only_relations_refuses_a_node_player a_role_that_does_not_accept_relations_refuses_a_relation_player a_role_with_both_node_and_relation_targets_accepts_either a_failure_partway_through_an_n_ary_create_leaves_nothing_behind --exact`:

```
---- a_role_that_accepts_only_relations_refuses_a_node_player stdout ----
thread 'a_role_that_accepts_only_relations_refuses_a_node_player' panicked at graph/frontend/tests/nary_relations.rs:1762:5:
Mutation(Bind(MissingRequiredRole { relationship_type: "Citation", role: "reference", span_start: 38, span_end: 73 }))

---- a_relation_may_be_a_player_of_another_relation stdout ----
thread 'a_relation_may_be_a_player_of_another_relation' panicked at graph/frontend/tests/nary_relations.rs:1730:10:
a relation must be an accepted player of a role targeting only relations: Mutation(Bind(MissingRequiredRole { relationship_type: "Citation", role: "reference", span_start: 42, span_end: 83 }))

---- a_role_with_both_node_and_relation_targets_accepts_either stdout ----
thread 'a_role_with_both_node_and_relation_targets_accepts_either' panicked at graph/frontend/tests/nary_relations.rs:1844:10:
a Text node must be an accepted `reference` player: Mutation(Bind(RoleTargetTypeViolation { relationship_type: "Citation", role: "reference", found: "Text", span_start: 116, span_end: 128 }))

---- a_failure_partway_through_an_n_ary_create_leaves_nothing_behind stdout ----
thread 'a_failure_partway_through_an_n_ary_create_leaves_nothing_behind' panicked at graph/frontend/tests/nary_relations.rs:1677:5:
Mutation(Bind(MissingRequiredRole { relationship_type: "Citation", role: "reference", span_start: 110, span_end: 162 }))

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 39 filtered out; finished in 0.06s
```

Four of five tests went red (`a_role_that_does_not_accept_relations_refuses_a_relation_player`
passed, unaffected -- it only exercises `Transcription`'s `source` role,
untouched by this swap). This proves the queries' `cited:`/`reference:`
role-name tokens resolve against whichever schema entry currently carries
that name, not against declaration position. Reverted; `git diff --stat`
confirmed clean, full `cargo test -p turso_graph_frontend` re-confirmed
green (334 passed, 1 ignored).

## Gate

- `cargo fmt`: ran clean (reflowed only the new test code in
  `nary_relations.rs`; no semantic change).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`:
  exit 0, run exactly as written (not narrowed to `-p`).
- `cargo test -p turso_graph_cypher -p turso_graph_frontend`: 358 passed, 1
  ignored (15 suites).
- `mise run corpus`: exit 1 ("task failed"), as documented -- ignored per
  brief. Per-suite counts from `graph/test-results/history.jsonl`, filtered
  to this run's `run_id` (`20260726T235836.378258Z-8e296519275b-corpus-deep`):
  - `age-deep`: 3042 passed (baseline: 3042) -- match
  - `cqlite-deep`: 113 passed (baseline: 113) -- match
  - `grafeo-deep`: 277 passed (baseline: 277) -- match
  - `sparrowdb-deep`: 2164 passed (baseline: 2164) -- match
  - `tck-deep`: 3332 passed (baseline band: 3329-3332) -- within band
  No suite moved off baseline.
- `mise run cypherbench-sample`: exit 0. Per-domain `matched`/`mismatched`
  counts (company 13/12, fictional_character 14/11, flight_accident 24/1,
  geography 11/14, movie 6/19, nba 25/0, politics 15/10) are identical, per
  domain, to the last recorded baseline run in
  `graph/test-results/benchmarks.jsonl` (commit `8343d8fd4`). No change.

`git add` used explicit paths (`graph/frontend/src/binder.rs`,
`graph/frontend/tests/fixture.rs`, `graph/frontend/tests/nary_relations.rs`);
nothing under `graph/test-results/` was staged or committed (left modified
in the working tree for the controller).
