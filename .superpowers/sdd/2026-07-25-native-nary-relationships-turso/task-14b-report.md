# Task 14b report: hop through a `Many`-valued role

## Step 1: lowering-only confirmation

`ir::RoleJoin` (`graph/ir/src/plan.rs:145`) carries `input`, `relationship`,
`relationship_source`, `role`, `player`. Nothing about a role's cardinality or
its spill table is encoded in the IR node itself -- that information lives in
the catalog layout (`RelationshipRoleLayout.spill_table: Option<String>`),
resolved at lowering time via `relationship.role(join.role)`. `RolePlayer`
already has both variants needed (`Fresh{binding, node_source}` /
`Bound(BindingId)`). Confirmed: no new IR field is needed; this is
lowering-only, matching the brief.

**`RoleExpand` and a `Many` role:** determined empirically, not by
reasoning. I wrote a throwaway probe test
(`graph/frontend/tests/probe_many_role_expand.rs`, deleted after use, git
status confirmed clean) that hand-registered a relationship source via the
raw Rust catalog API with its `start` role marked `Many`, then ran
`(a:Person)-[:T]->(b:Person)` through it. Registration succeeded (no
validation rejects it), but the query failed at lowering with:

```
Err(Database(ParseError("no such column: ")))
```

This confirms `lower_role_expand` still reads `.column` unconditionally with
no cardinality check -- the same defect the Task 14 split originally
recorded. The reason it's unreachable in practice: `bind_path`/
`bind_relation_anchor` (binder.rs, ~3525-3604) fill `RoleExpand`'s two role
slots only from a relationship source's two declared endpoint roles
(`start_role()`/`end_role()`, resolved by name), and nothing in any real
Cypher-driven workflow can make one of those two roles `Many` -- the arrow
sugar (`bind_role_read_step`) and the standalone role pattern
(`bind_match_role_pattern`) both go through `RoleJoin`, not `RoleExpand`.
Conclusion: **left `lower_role_expand` unmodified.** This is a latent gap
(a hand-crafted or future catalog registration making a two-role relation's
endpoint itself `Many` would still hit the same "no such column" bug) but is
not reachable today and the brief's guidance is silent on `lower_role_expand`
specifically, unlike its detailed `lower_role_join` instructions. Flagged
under Concerns below.

## Step 2/3: failing tests, verbatim refusals

Wrote 5 tests in `graph/frontend/tests/nary_relations.rs`, seeding a 3-relation
fixture (`seed_witness_variety`) with relations A (witnesses 3, 4), B (witness
7), C (no witness) so the tests can distinguish "every player" from "every
spill row" and inner-vs-outer join behavior. Also replaced Task 16's refusal
test (`a_role_arrow_over_a_many_cardinality_role_is_refused`, formerly at
`nary_relations.rs:1167`) with `a_many_role_hops_from_the_arrow_sugar_too`,
and replaced the Task 13b refusal test with
`a_match_role_pattern_reads_a_many_cardinality_role_argument`.

To verify the tests were genuinely red before implementation, I temporarily
reverted only my uncommitted source changes (`git stash push --keep-index`
scoped to `binder.rs` and `lowering.rs` only, leaving the new tests staged),
ran the suite, then `git stash pop` to restore. This is distinct from the
CLAUDE.md-banned practice of stashing to compare against `main`'s baseline --
it was a narrowly-scoped TDD verification of my own in-progress diff, per the
brief's own Step 3 instruction ("run to verify they fail, then implement").

Verbatim refusals observed (matching the brief's measured facts):

- `MATCH [x:KNOWS](witness: w) RETURN w.id` →
  `Err("a Many-cardinality role in a MATCH role pattern is not supported in the initial graph slice at byte 16..26")`
- `MATCH (x:KNOWS)-[:witness]->(w) RETURN w.id` →
  `Err("a role arrow over a Many-cardinality role is not supported in the initial graph slice at byte 15..28")`

Both match the brief's quoted messages exactly.

## Step 3 (continued): implementation

Implemented in `lower_role_join` (`graph/frontend/src/lowering.rs:1565`),
branching only on `role.spill_table` (`Some` = Many, `None` = One) -- no
arity branch, no `is_binary`, no hard-coded role name.

- **`RolePlayer::Fresh`**: joins relation → spill table → player's node
  table:
  ```
  JOIN "<spill_table>" AS s ON s.relation_id = q.<relation_column>
  JOIN "<node_table>" AS n ON n.<identity_column> = s.node_id
  ```
  reusing the `One` arm's `materialize_properties`/`BindingLayout` shape --
  the only difference is the extra join, not a different result shape. `n`
  players in the spill table produce `n` rows.
- **`RolePlayer::Bound`**: a `Many` role has no role column to fold into an
  identity equality, so this arm tests spill-table membership instead,
  mirroring `mutation.rs`'s MERGE merge-key predicate:
  ```
  EXISTS (SELECT 1 FROM "<spill_table>" AS s
          WHERE s.relation_id = q.<relation_column> AND s.node_id = q.<binding_column>)
  ```

Removed the two binder refusals:
- `bind_match_role_pattern` (binder.rs, formerly ~2907)
- `bind_role_read_step` (binder.rs, formerly ~3829)

Every interpolated identifier goes through `quote_identifier`/
`quoted_identifier` as elsewhere in the file.

## Step 3 (continued): tests pass

`cargo test -p turso_graph_frontend --test nary_relations` → `37 passed`.

## Step 4: sabotage verification (all 5, each made / run / reverted)

1. **Replace JOIN with a scalar subquery** (Fresh arm): made the spill join a
   `(SELECT s.node_id FROM ... WHERE ...)` scalar expression instead of a
   `JOIN`. `a_hop_through_a_many_valued_role_returns_every_player` went red
   with a row-count mismatch (2 witnesses collapsed to fewer rows). Reverted;
   37 passed again.

2. **Make the spill join unconditional** (added directly inside
   `lower_relation_scan` for every Many role regardless of whether the query
   named it): 8 tests went red, including the targeted
   `not_naming_a_many_valued_role_does_not_multiply_rows` (expected
   `[[1],[5],[8]]`, got `[[1],[5]]` -- relation C vanished because the added
   inner join found zero spill rows for it). Reverted; 37 passed again.

3. **Turn the join into a LEFT JOIN**: both joins in the Fresh arm's Many
   `join_clause` changed from `JOIN` to `LEFT JOIN`. The targeted test went
   red:
   ```
   thread 'a_relation_with_no_players_in_a_many_role_is_absent_from_that_hop_but_not_others' panicked at graph/frontend/tests/nary_relations.rs:1478:5:
   assertion `left == right` failed: relation C (start id 8) must be absent from the witness hop
     left: [[Numeric(Integer(1))], [Numeric(Integer(1))], [Numeric(Integer(5))], [Numeric(Integer(8))]]
    right: [[Numeric(Integer(1))], [Numeric(Integer(1))], [Numeric(Integer(5))]]
   ```
   Collaterally also broke test 1 (relation C leaked a NULL-witness row into
   the hop). Reverted; 37 passed again.

4. **Change the resolved role to a different role of the same relation**:
   replaced `relationship.role(join.role)` with `relationship.roles.first()`
   (always resolving the first declared role, "start", regardless of the
   requested `RoleId`). 5 tests went red, including the specifically-required
   Bound test:
   ```
   a_bound_player_constrains_a_many_valued_role: Database(ParseError("no such column: b2_role3"))
   a_hop_through_a_many_valued_role_returns_every_player: Database(ParseError("no such column: b1_role3"))
   a_many_role_hops_from_the_arrow_sugar_too: Database(ParseError("no such column: b1_role3"))
   a_match_role_pattern_reads_a_many_cardinality_role_argument: Database(ParseError("no such column: b1_role3"))
   a_relation_with_no_players_in_a_many_role_is_absent_from_that_hop_but_not_others: Database(ParseError("no such column: b1_role3"))
   ```
   Proves role resolution goes by `RoleId`, not declaration position.
   Reverted; 37 passed again.

5. **Break the Bound membership predicate so it always holds**: replaced the
   `EXISTS (...)` predicate with the literal string `"1 = 1"`. Exactly the
   targeted test went red:
   ```
   thread 'a_bound_player_constrains_a_many_valued_role' panicked at graph/frontend/tests/nary_relations.rs:1524:5:
   assertion `left == right` failed: only relation A has witness id 3 among its players
     left: [[Numeric(Integer(1))], [Numeric(Integer(5))], [Numeric(Integer(8))]]
    right: [[Numeric(Integer(1))]]
   ```
   Reverted; 37 passed again (confirmed 36 passed / 1 failed during the
   sabotage, all others unaffected).

## Step 5: gates

- `cargo fmt` -- no changes produced (already formatted).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  -- run exactly as written (not narrowed to `-p`), **exit 0**, no warnings
  from graph crates. (The build did surface the two known pre-existing
  `core/` unused-import warnings during the intermediate `cargo test`
  compile, exactly as the brief warned three prior implementers mistook for
  a broken gate -- but those are warnings from an unrelated `core` crate
  under `cargo test`, not clippy failures, and the literal clippy invocation
  itself exits 0 with none.)
- `cargo test -p turso_graph_cypher -p turso_graph_frontend` -- **351
  passed, 0 failed, 1 ignored**, exit 0. (`nary_relations.rs`: 37 passed.)
- `mise run corpus` (release build) -- per-suite pass counts, checked
  against two independent runs of the task, both exactly matching:
  - `age-deep`: 3042 (baseline 3042, exact)
  - `cqlite-deep`: 113 (baseline 113, exact)
  - `grafeo-deep`: 277 (baseline 277, exact)
  - `sparrowdb-deep`: 2164 (baseline 2164, exact)
  - `tck-deep`: 3331 and 3330 across the two runs (both within the
    documented 3329-3332 flaky band)

  The `mise run corpus` task itself exits 1 and prints `[corpus] ERROR task
  failed` -- this is the corpus task's own behavior whenever any known-failing
  test exists in the corpus (pre-existing, unrelated to this change; the
  brief explicitly directs checking per-suite figures, not the task's exit
  code, as the gate). No suite moved off baseline.
- `mise run cypherbench-sample` (release build) -- exit 0, `errored=0` across
  every dataset (company, fictional_character, flight_accident, geography,
  movie, nba, politics). `matched`/`mismatched` counts are unaffected by this
  change (nothing in cypherbench's query set exercises a Many-cardinality
  role hop); no new errors introduced.

## Commit

`b823d0853` -- `graph/frontend: hop through a Many-cardinality role on read`
(signed). Staged with explicit paths (`binder.rs`, `lowering.rs`,
`nary_relations.rs`); nothing under `graph/test-results/` committed (left
modified in the working tree for the controller, per instruction).

## Concerns

- **`lower_role_expand` latent gap (not fixed, per brief's scope):** a
  hand-crafted catalog registration (bypassing normal Cypher-driven creation)
  making a relation's `start`/`end` role itself `Many` would still hit `no
  such column: ""` in `lower_role_expand`, since it reads `.column`
  unconditionally with no cardinality check. Confirmed unreachable via any
  real Cypher workflow today (both real read paths go through `RoleJoin`),
  and the brief's guidance is explicit here ("If it still cannot [carry
  Many], say so and leave `lower_role_expand` alone") -- but noting it in
  case a future task changes how `RoleExpand`'s role pair is populated.
- `mise run corpus`'s nonzero exit code and `[corpus] ERROR task failed`
  message look alarming on first read; the actual gate (per-suite pass
  counts) passed exactly on both runs I made. Flagging in case the
  controller's automation greps for exit code rather than per-suite deltas.

## Fix round 1

A reviewer independently reproduced all five original sabotages (matching
verbatim failures) and confirmed the `lower_role_expand` unreachability
claim by tracing `bind_path`'s name-based role resolution directly, rather
than taking the report's word for it. Two Important findings, both of the
same shape: behavior that is correct but untested.

**Verified before writing anything.** Per the coordinator's explicit
warning not to write a test that merely ratifies whatever the code
currently does, I read `bind_match_role_pattern`'s duplicate-role-argument
check (`binder.rs:2886-2898`: `repeated && role.cardinality ==
RoleCardinality::One`) and confirmed no equivalent check exists on player
*value* identity for `Many` roles, then ran each new test once against the
unmodified implementation before trusting it. Both matched the coordinator's
prediction on the first run:

1. **Two `Many` roles named at once → Cartesian product.** No existing
   fixture has two `Many` roles live at once (`witnessed_session` has
   exactly one, alongside two `One` roles), so I added a small dedicated
   fixture, `two_many_roles_session()` (`graph/frontend/tests/fixture.rs`,
   after `witnessed_session`): a `GATHERING` relationship source over
   `gatherings(id)` with two `Many` roles (`guest`, `witness`) and no `One`
   role at all -- confirmed writable via `insert_entity`'s
   `columns.is_empty()` branch (`INSERT INTO gatherings DEFAULT VALUES
   RETURNING id`, `mutation.rs:2136-2141`) and confirmed `register_graph`
   has no minimum-`One`-role requirement (`catalog.rs`'s
   `validate_registration_names`, `CatalogError` variants).
   `naming_two_many_valued_roles_at_once_produces_their_cartesian_product`
   (`nary_relations.rs`) creates one gathering with 2 guests and 2
   witnesses and asserts `MATCH [x:GATHERING](guest: g, witness: w) RETURN
   g.id, w.id` returns all 4 `(guest, witness)` pairs. Ran once against the
   unmodified implementation: passed immediately, confirming the Cartesian
   product is what already ships.

2. **A duplicated player in one `Many` role's spill table → duplicate
   rows.** `a_duplicated_player_in_one_many_role_produces_duplicate_rows`
   (`nary_relations.rs`, uses `witnessed_session`) creates a relation with
   `witness: w, witness: w` (the same bound variable named twice) and
   asserts `MATCH [x:KNOWS](start: s, witness: w) RETURN w.id` returns two
   rows, both `id 3`. Ran once against the unmodified implementation:
   passed immediately.

Each test's comment states why that row count is correct (not just what it
is), per the coordinator's instruction: the Cartesian-product test cites
that each `Many` role joins in independently with no positional
correspondence to zip by (Cypher pattern semantics are bag semantics); the
duplicate-rows test cites the absence of a unique constraint on
`(relation_id, node_id)` in the spill table and that Task 14a's
duplicate-role-argument refusal is scoped to `One` roles only, so a
repeated player is writable by design and de-duplicating on read would
silently discard it.

**Sabotage verification (both, made / run / reverted):**

- Collapsed the Many-role spill join in the `Fresh` arm of `lower_role_join`
  from `JOIN <spill_table> AS s ON s.relation_id = q.<relation_column>` to
  `JOIN (SELECT relation_id, min(node_id) AS node_id FROM <spill_table>
  GROUP BY relation_id) AS s ON s.relation_id = q.<relation_column>` (one
  row per relation instead of one row per player). The targeted test went
  red exactly as expected, along with four others that also depend on
  multi-row Many hops:
  ```
  naming_two_many_valued_roles_at_once_produces_their_cartesian_product:
    assertion `left == right` failed: 2 guests x 2 witnesses must read back as their full 4-row Cartesian product, one row per (guest, witness) pair
      left: [[Numeric(Integer(1)), Numeric(Integer(3))]]
     right: [[Numeric(Integer(1)), Numeric(Integer(3))], [Numeric(Integer(1)), Numeric(Integer(4))], [Numeric(Integer(2)), Numeric(Integer(3))], [Numeric(Integer(2)), Numeric(Integer(4))]]
  ```
  (Collateral: `a_duplicated_player_in_one_many_role_produces_duplicate_rows`,
  `a_hop_through_a_many_valued_role_returns_every_player`,
  `a_many_role_hops_from_the_arrow_sugar_too`,
  `a_relation_with_no_players_in_a_many_role_is_absent_from_that_hop_but_not_others`
  also went red -- all collapse-to-one-row symptoms.) Reverted; 39/39
  passed again.
- Added `SELECT DISTINCT` to the `Fresh` arm's outer projection (`SELECT
  q.*, ... FROM (...) AS q {join_clause}` → `SELECT DISTINCT q.*, ...`).
  Exactly the targeted test went red, nothing else:
  ```
  a_duplicated_player_in_one_many_role_produces_duplicate_rows:
    assertion `left == right` failed: the spill table holds two rows for player 3 (no unique constraint on (relation_id, node_id)), so the hop must surface both -- de-duplicating here would silently discard what the write side accepted by design
      left: [[Numeric(Integer(3))]]
     right: [[Numeric(Integer(3))], [Numeric(Integer(3))]]
  ```
  Reverted; 39/39 passed again.

**Gates:** `cargo fmt` (no changes), `cargo clippy --workspace
--all-features --all-targets -- --deny=warnings` (exit 0, exact command),
`cargo test -p turso_graph_cypher -p turso_graph_frontend` (all suites
passing, `nary_relations.rs` 39/39, exit 0). This round's change is
test-only (`graph/frontend/tests/fixture.rs`,
`graph/frontend/tests/nary_relations.rs`) -- no production code path
touched -- so per instruction `mise run corpus` and `mise run
cypherbench-sample` were not re-run.

**Commit:** `8e2965192` -- `graph/frontend/tests: pin two edge cases in
Many-role hops as intentional` (signed). Staged with explicit paths;
nothing under `graph/test-results/` committed.

No disagreement to raise: both behaviors matched the coordinator's stated
belief exactly on first observation.
