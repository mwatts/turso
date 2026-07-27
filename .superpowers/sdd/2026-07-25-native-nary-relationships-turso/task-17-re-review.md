# Task 17 fix-round re-review — 6236ef7c9..f48554da5

Scoped re-review: two findings from `task-17-review.md` (Critical-1, Important-1),
plus the unreviewed CSR dedup-key widening the fix dragged in. Sabotage and
judgment only, per instructions; full suites not re-run (implementer already ran
389 passed/3 ignored, fmt/clippy clean; corpus and cypherbench rows independently
spot-checked against `graph/test-results/runs.jsonl` — all exact at baseline,
tck-deep 3331 in band).

## Verdicts

**CRITICAL-1 — genuinely addressed.** The three special-cased passes
(One-One inline, One-Many inline, Many-Many omitted) were replaced with one
general pass over `role_players: HashMap<SourceIdentity, Vec<(RoleId, NodeId)>>`
that emits an edge for every ordered pair of distinct roles, with no cardinality
branch. Verified independently (R2, R4 below), not just by reading the new
tests.

**IMPORTANT-1 — genuinely addressed.** Two new traversal-level tests were
added directly to `snapshot.rs`'s test module (registered via the Rust API,
since Task 13b's standalone-role Cypher syntax can't express a Many/Many
fixture yet). The exact sabotage that previously found nothing (deleting the
spill-table join pass) now turns a test red (R3 below).

## R1-R4 sabotage results

- **R1** (revert CSR dedup key `(relationship, from_role, to_role, source,
  target)` → `(relationship, from_role, to_role)`): **RED**, both new
  `snapshot.rs` tests fail with `Runtime(DuplicateRelationship(RelationshipId(1)))`
  the instant a role has more than one player. `csr.rs`'s own 8 unit tests
  still pass under the narrow key (none of them exercise multi-player
  fanout), confirming the gap was invisible until this task's own new
  coverage existed. This is the highest-value check in this round and it
  bites hard.

- **R2** (general pass skips `Many`-cardinality roles — filtered pairs where
  either role is in the `Many` set): **RED** on the Many-Many test:
  `edge_count()` left: 0, right: 1 (same symptom as the original Critical-1).
  The One-Many test also went red on its traversal assertion, as expected
  collateral.

- **R3** (delete the spill-table player push, the exact S5 sabotage that
  previously found nothing): **RED**, now on the traversal assertion in the
  `(One, Many)` test: `traversing start -> witness must reach both spilled
  witnesses, not zero — left: [] right: [NodeId(3), NodeId(4)]`. Confirms
  Important-1 is closed — this sabotage now bites where it silently didn't
  before.

- **R4** (independent reproduction, not trusting the new tests' own
  assertions): wrote a temporary test with a different, asymmetric fixture
  (3 authors × 2 editors, 5 people, no overlap with the implementer's 2×2
  fixture) directly in `snapshot.rs`, run in isolation
  (`cargo test -p turso_graph_frontend --lib snapshot::tests::re_review_r4`),
  then reverted. Result: `edge_count() == 1`; every one of the 3 authors
  reaches both editors; every one of the 2 editors reaches all 3 authors;
  `resolve_pairs(&[], authors, authors, false)` is empty (no same-role
  bucket exists at all). Confirms the fix produces real, non-degenerate
  cross-product edges at a scale/shape the implementer's own test didn't
  cover, not just at the implementer's chosen 2×2 case.

Tree confirmed clean (`git status --short` / `git diff --stat` both empty)
after every revert, and `cargo build -p turso_graph_frontend -p
turso_graph_runtime` clean (0 errors) at the end.

## Dedup-key widening verdict: correct, and not vacuous

1. **Rationale is correct.** The narrow key `(relationship, from_role,
   to_role)` encodes "at most one edge per role-pair per relationship,"
   which was true only when every role has ≤1 player (the old binary/One-Many
   world). R1 above is direct, non-hypothetical proof: reverting to the
   narrow key makes the new tests' *legitimate*, non-duplicate multi-player
   edges collide and get rejected as `DuplicateRelationship`. The old key
   was genuinely wrong for multi-player roles, not "correctly rejecting
   something that should collapse."

2. **Does the wider key admit duplicates that ought to collapse? No.**
   `(relationship, from_role, to_role, source, target)` colliding means: same
   physical relationship, same directed role pair, same source `NodeId`,
   same target `NodeId`. Since `NodeId`s are unique per node, that 5-tuple
   can only repeat if the *identical directed player pair* is pushed twice
   for the same relationship — i.e. either a genuine duplicate row in a
   spill table (a data-integrity bug) or the snapshot builder itself
   double-emitting the same pairing (an implementation bug). Both of those
   were caught under the old key too (a duplicate spill row collided on
   `(relationship, from_role, to_role)` alone, just less precisely) and
   remain caught under the new key (all 5 components still match for an
   exact duplicate). There is no legitimate path that produces two distinct
   player-pairings with an identical 5-tuple — the widened key is exactly as
   tight as the finest real distinction available (a specific directed
   player pair), so nothing that should collapse is let through.

3. **The pre-existing test is still load-bearing.**
   `rejects_invalid_endpoints_duplicates_and_build_limits` pushes the
   literal same `EdgeInput` value twice (`[edge, edge]`) — all 5 tuple
   components are identical by construction, so it still exercises the
   `HashSet::insert` returning `false` under the new key exactly as it did
   under the old one. Confirmed by running `cargo test -p turso_graph_runtime
   --lib csr` both with the key reverted (R1) and restored: 8/8 pass either
   way, and the duplicate-detection branch is genuinely hit in both cases —
   not vacuous.

## Scale/quadratic question: unbounded during construction, and it's a real gap

Nothing bounds the O(N×M) nested pair loop in `graph/frontend/src/snapshot.rs`
(`for identity in &relationship_order { let players = &role_players[identity];
for (from_role, from_node) in players { for (to_role, to_node) in players
{ ... edges.push(...) } } }`) while it runs. There is no `check_cancelled`
call inside that nested loop and no running comparison against
`limits.max_edges` (or any other budget) as `edges: Vec<EdgeInput>` grows.
The only existing guard, `BuildLimits::max_edges` (default 100,000,000),
is enforced later, inside `Graph::build_cancellable`'s own edges loop in
`graph/runtime/src/csr.rs` — by which point the frontend has already fully
materialized the oversized `Vec<EdgeInput>` in memory. A single relation with
two large `Many` roles (say, tens of thousands of players each) would attempt
to allocate and push tens of billions of `EdgeInput` values before that later
check ever gets a chance to reject the build — an OOM/hang risk, not a
bounded rejection. The corpus and cypherbench fixtures cannot detect this
because none of them has a `Many` role anywhere near that size.

Judgment: this is a real defect, but I would not block this fix round on it.
Two reasons: (a) the quadratic *edge count* itself is inherent to correct
Many-Many semantics — a relation with N and M players in two `Many` roles
genuinely has N×M directed pairs per direction, the same way any all-pairs
adjacency materialization does, so the fix isn't wrong to produce that count,
it's just missing an early-exit check while producing it; (b) Task 17's brief
was traversal correctness/generality, not resource governance, and the
existing `BuildLimits` machinery already models the intended safety valve —
it's applied one phase too late, not absent in concept. Recommend a narrow
follow-up: add an incremental check inside the nested pair loop in
`snapshot.rs` (compare `edges.len()` against `limits.max_edges` — or better,
pass `limits` into that loop and reuse the exact check `Graph::build_cancellable`
already has) so a pathological `Many`×`Many` shape fails fast with
`LimitExceeded` instead of attempting the full materialization first. This
should be a follow-up task, not a blocker for this re-review.

## Same-role semantics: intentional, not an omission

The pair loop's `if from_role == to_role { continue; }` compares `RoleId`
(the schema-level role identity, e.g. "authors"), not player identity — so
*any* two players sharing the same role are skipped regardless of which
specific players they are. This is the correct semantics, not a repeat of
Critical-1's bug class: Critical-1 was two *distinct, declared* roles
(`authors`, `editors`) that the schema says should be traversable producing
zero edges. Same-role co-players have no declared role pair at all — the
schema only declares roles "authors" and "editors," never "authors ↔
authors" — so there is nothing for the traversal layer to silently drop;
"authors→authors" was never a role pair the binder or catalog exposes as
traversable in the first place. `resolve_pairs(&[], authors, authors, false)`
correctly returning empty (confirmed directly in R4) reflects that no
adjacency bucket was ever supposed to exist for that pair, not a gap.

## New findings

- **Important** (new, not previously flagged): the O(N×M) edge-generation
  loop in `snapshot.rs` has no incremental limit or cancellation check,
  unlike every other loop in this function (`check_cancelled` is called once
  per relationship source and once per `Many` role's spill query, but not
  inside the pair-generation loop itself). See scale-question section above
  for the concrete risk and recommended fix shape. Not blocking this fix
  round; recommend a follow-up task.
- No Critical or additional blocking findings. Minor: none new beyond what
  the prior review already recorded (M1/M2, both unaffected by this fix).
