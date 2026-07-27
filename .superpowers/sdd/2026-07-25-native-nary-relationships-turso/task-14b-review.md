# Task 14b review: hop through a `Many`-valued role

Range reviewed: `f5c009b2f..b823d0853` (single commit `b823d0853`, plus an
unrelated `graph/test-results` commit `6a5036017` already on top of it in the
working tree — not part of this review). Working tree at review time was
`b823d0853` + the test-results commit; confirmed clean (`git status --short`
empty) before and after every experiment below.

## Method

For every property claimed by the diff or the report, I broke it in the
working tree, ran `cargo test -p turso_graph_cypher -p turso_graph_frontend`
(or the narrower `--test nary_relations` while iterating), recorded the
verbatim failure, then `git checkout -- <file>` to revert. Baseline before and
after every sabotage: `351 passed, 1 ignored` (workspace gate) / `37 passed`
(`nary_relations.rs` alone). Tree was byte-identical to `b823d0853` +
test-results at the end (`git status --short` empty, `git diff --stat` empty).

Two additional throwaway probe files were written outside the diff to test
scenarios the shipped tests do not cover (three-or-more players, two `Many`
roles at once, a duplicated player). Both were deleted after use; `git status`
confirmed clean.

---

## Verdict 1: SPEC COMPLIANCE

**Step 1 (lowering-only confirmation + `RoleExpand` reachability):** ✅ met.
- `ir::RoleJoin` (`graph/ir/src/plan.rs:145`) carries `input`, `relationship`,
  `relationship_source`, `role`, `player` — no cardinality or spill field. I
  read this directly; matches the report and requires no IR change, since the
  spill table name is resolved from `RelationshipRoleLayout.spill_table` at
  lowering time via `relationship.role(join.role)`. Confirmed.
- `lower_role_expand` unreachability claim: independently verified, by
  structural reading (not by re-running the implementer's deleted probe,
  which no longer exists to re-run). `bind_path`'s expansion binder
  (`binder.rs:3516-3536`) populates `RoleExpand`'s `from_role`/`to_role`
  exclusively from `layout.start_role()`/`end_role()`, which resolve by the
  literal names `"start"`/`"end"` (`lowering.rs:46-54`). The only way to make
  one of those two roles `Many` is to register a relationship schema that way
  via `register_graph` — and `dialect.rs:178` states plainly "there is no
  marked graph DDL (yet)": `register_graph` is a Rust host-API call, not
  reachable from any Cypher text. No Cypher-driven path can produce a
  `RoleExpand` whose `start`/`end` role is `Many`. The report's claim holds;
  I traced the same reachability chain from a different starting point
  (`dialect.rs`'s DDL comment) rather than trusting the report's probe.
  `lower_role_expand` left unmodified, as directed.

**Step 2 (five failing tests, seeded with ≥2 relations differing in witness
sets):** ✅ met. All five named tests exist in
`graph/frontend/tests/nary_relations.rs`, using `seed_witness_variety` (three
relations: A with two witnesses, B with one, C with none) exactly as the
brief's rationale requires ("a single-relation fixture cannot distinguish...").

**Step 3 (implement, JOIN not scalar subquery, `Bound` membership test):**
✅ met.
- `lower_role_join`'s `Fresh` arm (`lowering.rs:1633-1640`) emits a `JOIN`
  through the spill table then the node table — not a scalar subquery. I
  sabotaged this into the plan's rejected scalar-subquery snippet A myself;
  it broke exactly the claimed test (see sabotage log below).
- `Bound` arm (`lowering.rs:1584-1592`) emits `EXISTS (SELECT 1 FROM
  <spill> AS s WHERE s.relation_id = q.<relation_column> AND s.node_id =
  q.<binding_column>)`, matching the shape at `mutation.rs:1988-1990` the
  brief points to. Verified by sabotage (see below).
- No arity branch, no `is_binary`, no hard-coded `"start"`/`"end"` anywhere
  in the new `lower_role_join` code — confirmed by reading the full diff hunk;
  the `Many`/`One` distinction is made once, via `role.spill_table.is_some()`.
- All new identifiers go through `quote_identifier`: `spill_table`,
  `target.table`, `target.identity_column` are all quoted. `relation_id` and
  `node_id` (the spill table's own fixed schema column names, not
  catalog-derived data) are written unquoted, exactly matching the existing
  convention in `mutation.rs` (e.g. `mutation.rs:1989`, `2035`) for the same
  columns — not a new deviation.

**Step 4 (sabotage verification, 5 required experiments):** ✅ met, and I
independently reproduced all five myself (see below) — every one matched the
report's claimed outcome exactly.

**Step 5 (gates):** ⚠️ partially verifiable by me, but not a concern.
`cargo test -p turso_graph_cypher -p turso_graph_frontend` reproduced
`351 passed, 1 ignored` in my own run, matching the report. Per the task
instructions I did not re-run `cargo clippy`, `mise run corpus`, or `mise run
cypherbench-sample` — the controller verified those already. I have no
independent evidence for the clippy/corpus/cypherbench numbers beyond the
report's own text, which is consistent with the brief's per-suite gate
description.

**Commit hygiene:** ✅ met. `git log -p b823d0853 --stat` shows only
`binder.rs`, `lowering.rs`, `nary_relations.rs` — nothing under
`graph/test-results/` in this commit (that lives in the separate, later
`6a5036017` commit, as expected).

**Global constraints (positional-vs-name resolution, `Many` identified only by
`spill_table.is_some()`):** ✅ met, verified by sabotage (role-swap experiment
below) and by reading — `role.spill_table.is_some()` is the only cardinality
test anywhere in the new code; role lookup is `relationship.role(join.role)`,
a `RoleId`-keyed `.find`, order-independent.

---

## Verdict 2: TASK QUALITY

Ordered most severe first. Nothing rises to **Critical** — I could not break
any of the properties the brief calls load-bearing. The two **Important**
items below are both instances of the same thing: real, defensible SQL
behavior that the brief itself flagged as worth determining by experiment,
which the shipped diff neither tests nor documents.

### Important

1. **Joining two `Many` roles at once produces an untested, undocumented
   Cartesian product.** I hand-registered a relationship type with two
   `Many` roles (`witness`, `attendee`) on the same table (probe file, not
   committed) and seeded one relation with two witnesses and two attendees.
   `MATCH [x:KNOWS](witness: w, attendee: at) RETURN w.id, at.id` returned
   **4 rows** — the full 2×2 cross product
   (`[(3,5),(3,6),(4,5),(4,6)]`) — because each `RoleJoin` composes
   independently onto the previous plan's row set, exactly as the doc
   comment says ("Composing `n` of these... each role resolves
   independently"). This is defensible relational-algebra behavior (the same
   thing plain SQL does composing two 1:N joins), not a bug, but it is a real
   multiplicity trap for a Cypher user who writes
   `[x:KNOWS](witness: w, attendee: a)` expecting `w`/`a` pairs to line up
   1:1 rather than combinatorially, and the brief explicitly names this
   scenario in item (e) as something to resolve by experiment. Nothing in
   the diff tests or documents it. No fixture in the tree today has two
   `Many` roles on one relationship, so this can't yet happen against a
   *real* Cypher-registered schema — same reachability shape as the
   `lower_role_expand` gap — but unlike that gap, this task's own new code
   (`lower_role_join`) is what produces the multiplication, so it is closer
   to this task's scope than `lower_role_expand`'s latent gap is.

2. **A duplicate player in the same `Many` role's spill table silently
   duplicates output rows, untested.** I ran
   `CREATE [x:KNOWS](start: a, end: b, witness: w, witness: w)` with the
   *same* variable named twice for `witness` against `witnessed_session`.
   It succeeds (`MutationSummary { matched_rows: 1, operations_executed: 1,
   ... }`) and inserts two spill rows for the same `node_id`. Reading it back
   with `MATCH [x:KNOWS](witness: w) RETURN w.id` returns **two identical
   rows** (`[[3],[3]]`). This is standard non-deduplicating SQL join
   behavior, not a defect in `lower_role_join` specifically (14a's CREATE
   path is what allows the duplicate insert; this task only reads it back
   faithfully) — but the brief's item (e) asks for exactly this
   determination, and the shipped diff has no test establishing this is
   intended/acceptable behavior rather than an oversight one layer down.

Both items are **experimentally confirmed**, not reasoned about, and both are
scoped as "determine by experiment, report the finding" in the brief rather
than "must be fixed" — I am not asserting either is a bug the implementer
should have prevented in `lower_role_join` itself, since the row-multiplying
behavior in both cases is a direct, correct consequence of composable joins
and of 14a's write-side choice not to deduplicate. I flag them as Important
because the brief explicitly asked whether the behavior is "defensible and
whether anything tests it," and the answer for both is "defensible, and
nothing tests it."

### Minor

3. **Report's stashing justification during red-test verification is a
   judgment call worth flagging, not re-litigating.** The report says it
   used `git stash push --keep-index` scoped to only `binder.rs`/
   `lowering.rs` (leaving the new tests staged) to verify the tests were red
   before implementing, distinguishing this from the CLAUDE.md-banned
   practice of stashing to compare against `main`. I did not attempt to
   reproduce this (it would require replaying the implementer's exact
   in-progress state, which no longer exists — the commit is already
   assembled). Read literally, CLAUDE.md's ban is about using stash/revert
   "to check whether a failure pre-exists" against a baseline; this was a
   scoped stash of the implementer's own uncommitted diff, not a
   comparison against `main` or a prior commit, so it does not appear to be
   the banned pattern. Noting it because the brief's task instructions
   explicitly called out this exact CLAUDE.md clause as binding on the
   *review*, so it is worth a second set of eyes even though it doesn't
   change my verdict.

### Verified findings vs. reasoned-about findings

**Verified by sabotage (built, ran, confirmed red, reverted, confirmed
clean):**
- Scalar-subquery regression (plan's rejected snippet A) → row-count
  collapse in `a_hop_through_a_many_valued_role_returns_every_player`.
- Positional role resolution (`relationship.roles.first()` instead of
  `relationship.role(join.role)`) → 5 tests red with `no such column:
  b1_role3` / `b2_role3`.
- `LEFT JOIN` instead of `JOIN` in the `Fresh` arm → the empty-player-set
  test goes red (relation C leaks in), plus collateral damage to the
  every-player test (a spurious `Null` row for relation C).
- `Bound` membership predicate replaced with `"1 = 1"` → exactly
  `a_bound_player_constrains_a_many_valued_role` goes red.
- Unconditional spill join injected into `lower_relation_scan` (leaking into
  every query over a relation with a `Many` role, named or not) → 8 tests
  red, including the specifically-targeted
  `not_naming_a_many_valued_role_does_not_multiply_rows`.

**Verified by experiment (new probe code, not sabotage of shipped code):**
- Three-or-more players in a `Many` role → three rows (`[3, 4, 5]`).
  Confirms the brief's item (a) "relation with three or more players" case,
  which the shipped tests cap at two witnesses.
- Two `Many` roles named in the same query at once → Cartesian product
  (Important finding #1 above).
- Duplicate player in the same `Many` role's spill table → duplicate output
  rows, no dedup (Important finding #2 above).

**Verified by structural reading, not sabotage (no sensible experiment
applies — these are reachability/shape claims):**
- `RoleJoin`'s IR shape needs no new field (Step 1).
- `lower_role_expand`'s `Many`-role gap is unreachable via any real Cypher
  surface today (Step 1, extended discussion) — traced independently via
  `bind_path`'s `start_role()`/`end_role()` name resolution plus
  `dialect.rs:178`'s "no marked graph DDL (yet)" comment, not by re-running
  the implementer's now-deleted probe test.
- `quote_identifier`/`quoted_identifier` usage on every catalog-derived
  identifier in the new code, and the un-quoted `relation_id`/`node_id`
  literals matching existing convention in `mutation.rs`.

## Summary

Task 14b does what the brief asked: `lower_role_join`'s `Fresh` arm joins
through the spill table (one row per player, verified against 0/1/2/3-player
cases), its `Bound` arm does a membership test rather than an identity
equality, the two binder refusals are removed and their pinned tests
replaced rather than deleted, role resolution is by `RoleId` not position or
name, and none of the five required sabotage experiments were "unfalsifiable"
— every one broke a specific, correctly-targeted test when I reproduced it
myself. `lower_role_expand` was correctly left alone; its gap is real but
independently confirmed unreachable via any current Cypher surface syntax.
The two Important findings (Cartesian product from naming two `Many` roles
at once; unde-deduped duplicate players) are both real, both defensible as
shipped, and both genuinely untested — worth a follow-up test or an explicit
documented note, but not blockers to this commit standing as correct for the
scope the brief defines.
