# Task 10 report — write relations from their role bindings

Status: DONE
Commit: 65fce9e8b154187060e6d1cfedc99c5f241107f7 (branch feature/graph-nary, amended into what was fa67a216c)

## Review round 2 — findings and remediation

The first pass (commit `fa67a216c`) came back **spec compliance ❌**. Three
findings, all fixed in this amended commit:

1. **CRITICAL — production public API.** `GraphConnection::execute_create_relation`
   was a `pub fn` on a crate-root-exported type, panicking via
   `unwrap_or_else(|| panic!(...))` on five caller-controlled lookups, with
   a doc comment admitting it skips relationship-type junction membership
   — an `Ok` masking a structurally incomplete write. The brief's
   Interfaces line says "Produces: no new public API." **Fix:** deleted
   the method from `graph/frontend/src/session.rs` entirely (confirmed
   `git diff dffbc8bff -- graph/frontend/src/session.rs` is now empty —
   the file is byte-identical to before this task). The direct-IR writer
   tests moved into `graph/frontend/src/mutation.rs`'s existing `mod
   tests`, calling the free (non-`pub`, `pub(crate)`) function
   `insert_relationship` directly — no public surface at all. This also
   dissolved the second finding (the panicking lookups) since panicking
   inside a unit test is fine.
2. **IMPORTANT — both core invariants were untested.** Verified by
   sabotage (see below): a positional `layout.roles[i]` lookup and a
   same-player-collapses-to-one-column bug both passed the previously
   shipped test unchanged, because the ternary fixture's three orderings
   (layout declaration order, `RoleId` order, binding order) all
   coincided, and no test exercised a repeated role player through the
   direct-IR path. **Fix:** `TernaryCatalog` in `mutation.rs`'s test
   module now declares its three roles in an order that differs from
   both `RoleId` order and the binding order the new tests use (see
   `ternary_create`'s doc comment), and a new
   `a_repeated_player_fills_two_roles_of_one_relation` test binds one
   player to two different roles and asserts both columns hold it.

Not fixed, per explicit ruling: `assert!(spilled.is_empty(), ...)` stays
exactly as written — `Many` roles cannot reach that code before Task 14,
so no test can exercise removing it; this is inherent, not a gap.

## What changed (final)

- `graph/frontend/src/mutation.rs`: `insert_relationship` iterates
  `create.roles`, resolves each via `layout.role(binding.role)` (never a
  vector index), routes `One`-cardinality players into `fixed`,
  `Many`-cardinality into `spilled` (asserted empty). Added
  `MutationError::UnknownRole { role: ir::RoleId }`. `insert_relationship`
  is `pub(crate)` (needed by nothing outside the crate now, but the
  module boundary itself required it before, and no narrower visibility
  was needed after removing `session.rs`'s caller — left as `pub(crate)`
  since the free-function/module structure is unchanged, and it is only
  ever called from within `graph_frontend`). Added `TernaryCatalog`,
  `setup_ternary`, `ternary_create`, and two tests:
  `role_players_are_resolved_by_role_id_not_by_position` and
  `a_repeated_player_fills_two_roles_of_one_relation`.
- `graph/frontend/src/session.rs`: **no net change** — `execute_create_relation`
  and its supporting imports were added and then fully removed within
  this task; `git diff dffbc8bff -- graph/frontend/src/session.rs` is empty.
- `graph/frontend/tests/fixture.rs`: `ternary_session()` (unchanged from
  round 1) still backs the two `#[ignore]`d surface-syntax tests.
- `graph/frontend/tests/nary_relations.rs`: now holds only the two
  `#[ignore = "surface syntax lands in Task 12"]` tests
  (`a_three_role_relation_writes_one_row_with_three_endpoint_columns`,
  `the_same_player_may_fill_two_roles_of_one_relation`); the
  `execute_create_relation`-based test was removed along with the API it
  called. Doc comment now points at the `mutation.rs` unit tests as where
  the writer is actually exercised today.

## Byte-identical-binary invariant

Unchanged from round 1: the sole `ir::CreateRelationship` construction
site (`binder.rs`) always builds `roles: vec![start_binding, end_binding]`
in that fixed order, so iterating `create.roles` in declared order
preserves identical column order for every binary relation. No
`roles.len() == 2` special case, no `is_binary`, no hard-coded
`"start"`/`"end"` fast path in the general loop.

## Sabotage proof #1 — role identity must be `RoleId`, never position

`TernaryCatalog`'s `roles` vec declares `[text(RoleId 2), folio(RoleId 3),
scribe(RoleId 1)]`; `ternary_create` binds players in `[folio, scribe,
text]` order — three different permutations of the same three roles, so a
positional bug cannot hide behind coincidentally aligned indices.

Sabotage: replaced the by-`RoleId` lookup with
`layout.roles.get(position).cloned().unwrap()` keyed by the loop's
`enumerate()` position.

```
cargo test -p turso_graph_frontend --lib mutation::tests::role_players_are_resolved_by_role_id_not_by_position
```
→ **FAILED**:
```
assertion `left == right` failed
  left: [[Numeric(Integer(20)), Numeric(Integer(30)), Numeric(Integer(10))]]
 right: [[Numeric(Integer(10)), Numeric(Integer(20)), Numeric(Integer(30))]]
```
(scribe/txt/folio columns scrambled). Restored the by-`RoleId` lookup,
reran: `test result: ok. 1 passed`.

## Sabotage proof #2 — a repeated role player must not collapse

`a_repeated_player_fills_two_roles_of_one_relation` binds `scribe=7,
text=7, folio=30` and asserts all three columns independently
(`[7, 7, 30]`).

Sabotage: added a `seen_players: Vec<Value>` guard around the `One`-cardinality
push so a role whose player value was already seen is skipped (`fixed`
never gets a second entry for the same value):

```
cargo test -p turso_graph_frontend --lib mutation::tests::a_repeated_player_fills_two_roles_of_one_relation
```
→ **FAILED**:
```
assertion `left == right` failed
  left: [[Numeric(Integer(7)), Null, Numeric(Integer(30))]]
 right: [[Numeric(Integer(7)), Numeric(Integer(7)), Numeric(Integer(30))]]
```
(`txt` column collapsed to `NULL` instead of holding `7`). Restored the
straight-line push, reran: `test result: ok. 1 passed`.

Both sabotages were reverted immediately after producing red output; the
committed code contains neither.

## Gate results (measured, not assumed)

- `cargo fmt`: applied.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`:
  exit 0, 0 warnings attributable to source (only pre-existing
  `limbo_sqlite_test_ext` `ar`-toolchain build-script noise, unrelated).
- `cargo test -p turso_graph_frontend`: all suites passed, **0 failed**
  (5 ignored, all Task-12-gated: the 2 in `nary_relations.rs` plus 3
  pre-existing elsewhere in the crate).
- `mise run corpus` (release build; per-suite counts computed from
  `graph/test-results/history.jsonl` for this run's own `run_id`, not
  eyeballed from console output):
  - age-deep: 3042 passed / 553 non-passed (500 failed + 53 unsupported)
  - cqlite-deep: 113 passed / 11 failed
  - grafeo-deep: 277 passed / 95 failed
  - sparrowdb-deep: 2164 passed / 61 failed
  - tck-deep: 3331 passed / 595 failed
  - **Total: 8927 passed / 10242 records**

  Matches the stable baseline **exactly** on every suite (age-deep
  3042/553, cqlite-deep 113/11, grafeo-deep 277/95, sparrowdb-deep
  2164/61) except tck-deep, whose 3331 sits inside the documented
  3330-3332 stable range — the one suite explicitly allowed to vary. No
  regression to investigate. (Total is 8927 rather than round 1's 8926
  purely because tck-deep landed one higher within its stable band.)
- `mise run cypherbench-sample`: not run — this task does not touch
  traversal runtime.

## Commit

Amended into `fa67a216c` (`git commit -S --amend`) so Task 10 stays one
commit: `65fce9e8b154187060e6d1cfedc99c5f241107f7`. Staged explicit paths
only (`graph/frontend/src/mutation.rs`, `graph/frontend/src/session.rs`,
`graph/frontend/tests/nary_relations.rs`) — no `git add -A`.
`graph/test-results/*` left uncommitted and untouched. Commit is signed
(verified `Good "git" signature`).

## Concerns

- None outstanding from this round. `insert_relationship` remains
  `pub(crate)`, not `pub` — no public API was added or reintroduced.
- `ternary_session()` (in `fixture.rs`) still does not seed
  `Person`/`Text`/`Folio` node rows, since neither test that uses it
  executes yet (`#[ignore]`d pending Task 12). Flagging again for
  whoever un-ignores them.
