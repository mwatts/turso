# Task 14a report: write and delete many-valued role players

## Scope

Per the brief's "CONTROLLER CORRECTIONS" section: Steps 1-4, 6, 7, 8 of the
original Task 14 body, for `RoleCardinality::Many` roles only. Step 5 (spill
join / hop-through-Many-role reads, e.g. `role_join_expression`) is explicitly
out of scope, deferred to Task 14b (blocked on Task 13b, the MATCH-side
standalone role pattern).

## What changed and why

1. **`graph/frontend/tests/fixture.rs`** — added `witnessed_session()`, a new
   fixture: one `Person` node source, one `KNOWS` relationship source with
   `start`/`end` (single-valued, arrow-syntax shape) plus `witness`
   (many-valued, no column). This is the only role shape that lets a relation
   be both created through the standalone role pattern (Task 13a) and bound
   for deletion through today's arrow syntax, without depending on the
   unimplemented Task 13b. Modeled on `ternary_session` (no eager
   `SnapshotStore::refresh`), not `social_graph_connection`, since none of
   these tests run a variable-length traversal.

2. **`graph/frontend/src/mutation.rs` — `insert_relationship`**: removed the
   `assert!(spilled.is_empty(), ...)` guard that made writing a `Many` role
   player unreachable. After `insert_entity` returns `(identity, created)`,
   each spilled `(role, player)` pair is now written with its own
   `INSERT INTO <spill_table>(relation_id, node_id) VALUES ($p, $p)`, executed
   through `run_ignore` with named parameters bound via an internal
   `HashMap<String, Value>` — never string-interpolated, matching
   `delete_entity`'s existing pattern. The write is skipped when
   `created == false` (a MERGE match on an existing relation), so re-running a
   MERGE does not duplicate spill rows for a relation that already exists.

3. **`graph/frontend/src/mutation.rs` — `delete_entity`, relationship-delete
   branch**: added a per-`Many`-role `DELETE FROM <spill_table> WHERE
   relation_id = $id` before the relation row itself is deleted, using the
   same `identity_parameter(delete.entity)` already used for the relationship
   type-table cleanup.

4. **`graph/frontend/src/mutation.rs` — `delete_entity`, node DETACH DELETE
   branch**: this was "the larger hole" per the brief — the existing code did
   a bare `DELETE FROM <relation> WHERE start=$id OR end=$id`, leaving spill
   rows referencing the deleted relation dangling forever. Added a per-role
   spill delete via a subquery, `DELETE FROM <spill_table> WHERE relation_id
   IN (SELECT <identity_column> FROM <relation> WHERE <predicate>)`, executed
   *before* the bare relation-row delete (ordering matters: the subquery needs
   the relation rows to still exist when it runs).

5. **`graph/frontend/src/binder.rs` — `bind_create_role_pattern`**: removed
   the `at_unsupported` guard that rejected any `Many` role outright. The
   duplicate-role-argument check (`DuplicateRoleArgument`) is preserved for
   `One` roles (repeating a single-valued role name is still a mistake) but no
   longer fires for `Many` roles, which legitimately take one argument per
   player.

6. **`graph/frontend/tests/nary_relations.rs`** — four new tests against
   `witnessed_session()`:
   - `a_many_valued_role_holds_several_players_in_one_relation` — one relation
     row, two spill rows for two witnesses.
   - `deleting_a_relation_removes_its_spilled_players` — arrow-syntax
     `MATCH ... DELETE r` purges both the relation row and its spill row.
   - `detach_deleting_a_node_removes_its_relations_spilled_players` — node
     `DETACH DELETE` purges both the relation row and its spill row.
   - `a_single_valued_role_given_two_players_is_refused` — a repeated `start`
     argument still errors, confirming the duplicate check still holds for
     `One` roles after the guard change.

## Deviation from the brief (found, not specified)

`bind_create_role_pattern`'s final `roles` construction used
`.filter_map(|role| fills.iter().find(...).map(...))` — `.find()` only keeps
the *first* fill matching a role, by construction. With the `Many`-role guard
removed, a role with two or three fills (e.g. two witnesses) would have
silently dropped everything after the first player, with no error and no test
failure to signal it — the relation would just be created with one witness
instead of two. Found by reading the code path before running anything, not
by a failing test (though `a_many_valued_role_holds_several_players_in_one_relation`
now exercises it and would have failed with only one spill row had the
`.find()` version been left in place). Fixed by switching to
`.flat_map(|role| fills.iter().filter(...).map(...))`, keeping every fill per
role in declaration order across roles. This is required for correctness, not
optional scope creep — reported per the "surface deviations" instruction.

## Found but explicitly out of scope, not fixed

Node `DETACH DELETE` resolves which relations touch a node via
`relationship_endpoint_sources`, which is `start`/`end` (two-role) only. For a
relation with more than two roles (e.g. a true ternary, or `witnessed`'s
`start`/`end`/`witness`), `DETACH DELETE` on the node currently cannot even
discover the relation to delete it — this is a pre-existing limitation
unrelated to spill-table cleanup and was called out by the brief as
explicitly out of scope for this task. The fix in this task (item 4 above)
only reaches relations that `DETACH DELETE` already knows how to find (the
`start`/`end` pair); it purges those relations' spill rows correctly, but does
not extend what `DETACH DELETE` can see.

## Touched-but-not-unified duplicate predicates (per correction H)

`single_valued_roles()` (`graph/frontend/src/catalog.rs:136`) and
`structural_columns()` (`graph/frontend/src/lowering.rs:57`) were **read**
while tracing how `Many` roles flow through the physical layout (both already
filter to `One`-cardinality roles and were unaffected by this task's changes),
but **not edited and not unified**, per the brief's explicit instruction not
to do so unprompted.

## Invariant preserved

No `roles.len() == 2` check or hard-coded `"start"`/`"end"` name was added to
any general machinery. All new/edited code resolves roles by `RoleId` or by
declared name (`role.spill_table`, `role.role_id`, `RoleSourceRegistration`
lookups). A binary, all-`One` relation's insert/delete SQL is unchanged by
this task — the spill-write/spill-delete loops iterate over `Many` roles only,
which are absent for a purely binary relation, so `spilled` is empty and the
loops are no-ops.

## Test commands and results

- `cargo build -p turso_graph_frontend --tests` — clean build, no warnings
  from touched code.
- `cargo test -p turso_graph_frontend --test nary_relations` — 12 passed
  (8 pre-existing Task 13a tests + 4 new).
- `cargo test -p turso_graph_frontend` — **298 passed, 3 ignored (13 suites)**,
  no regressions, re-verified as the final sanity check before commit.
- `cargo fmt -p turso_graph_frontend -- --check` — exit 0, no diff.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — 0 errors. (The build also emits pre-existing, unrelated `ar -D` toolchain
  warnings from `limbo_sqlite_test_ext`'s build script on this machine's Xcode
  toolchain; these are not Rust/clippy lints and are not on any file this task
  touched.)

## Gate runs (release, per `mise.toml`)

### `mise run corpus`

Run id `20260726T115028.302572Z-754dce74d819-corpus-deep`, 10242 records,
compared per suite against the Task 13a baseline
(`graph/test-results/runs.jsonl`, e.g. run `20260726T111157.045920Z-de684e6ff864-corpus-deep`
and multiple other same-commit runs recorded before this task's edits):

| suite | baseline passed | this run passed | verdict |
|---|---|---|---|
| age-deep | 3042 | 3042 | exact match |
| cqlite-deep | 113 | 113 | exact match |
| grafeo-deep | 277 | 277 | exact match |
| sparrowdb-deep | 2164 | 2164 | exact match |
| tck-deep | 3331 (band 3329-3332) | 3330 | inside flake band |

Total: 8926/10242 passed (baseline total observed ranges 8926-8928 across
repeated runs of the *same* commit, entirely attributable to tck-deep flake —
every other suite is bit-for-bit identical across all runs inspected). Gate:
**pass**, per suite.

The corpus run itself exits non-zero (`clean=false`, `[corpus] ERROR task
failed`) because of ~100 pre-existing failing queries (unsupported
procedures like `db.index.fulltext.queryNodes`/`db.schema`, missing functions
like `shortestPath`/`vector_similarity`/`out_degree`, parameter-binding gaps,
etc.) — none of these are related to n-ary roles or this task's change; they
reproduce identically in the pre-task baseline runs.

### `mise run cypherbench-sample`

Compared per domain against `graph/test-results/benchmarks.jsonl` baseline
rows (multiple runs recorded before this task's edits, e.g.
`recorded_at: 2026-07-26T09:49:59Z` and `2026-07-26T11:14:05Z`):

| domain | matched | mismatched | errored |
|---|---|---|---|
| company | 13 | 12 | 0 |
| fictional_character | 14 | 11 | 0 |
| flight_accident | 24 | 1 | 0 |
| geography | 11 | 14 | 0 |
| movie | 6 | 19 | 0 |
| nba | 25 | 0 | 0 |
| politics | 15 | 10 | 0 |

Identical to baseline in every domain, `errored=0` throughout. Gate: **pass**.

## Files touched

- `graph/frontend/src/binder.rs`
- `graph/frontend/src/mutation.rs`
- `graph/frontend/tests/fixture.rs`
- `graph/frontend/tests/nary_relations.rs`

`graph/test-results/{REPORT.md,benchmarks.jsonl,runs.jsonl}` were modified by
running the two mise gates (as required, before committing) but are **not**
part of this commit — the controller records `graph/test-results/` changes
separately.
