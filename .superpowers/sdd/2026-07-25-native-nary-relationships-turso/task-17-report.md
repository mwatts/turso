# Task 17 Report — role-aware traversal runtime

Status: **DONE**

Commit: `d72ccdc6a` — graph/runtime: key traversal adjacency by role pairs, not direction

## What changed and why

Adjacency was keyed by `ir::Direction` (a binary Outgoing/Incoming enum). Native
n-ary relations have no forward/back — only named roles — so the whole
Direction-shaped path had to go, not just be extended.

- **Deleted `ir::Direction`** and everything propping it up:
  `role_pair_to_direction` in `graph/frontend/src/graph_expand.rs`, its call
  site, and its test `role_pair_to_direction_resolves_the_four_documented_cases`
  (per Correction A, the test dies with the function — not ported).
- **CSR adjacency / `EdgeInput` / `TraversalRequest`** are now keyed by
  `(RelationshipTypeId, RoleId, RoleId)` triples instead of `Direction`.
  `Graph::resolve_pairs(relationship_types, from_role, to_role, symmetric)`
  resolves a traversal request into the concrete stored triples to walk,
  unioning the reverse pair when `symmetric` is set. The new role-pair-keyed
  `Graph::build` stores only the `EdgeInput`s it's explicitly given — no
  automatic reverse insertion, so the snapshot builder must supply both
  directions itself.
- **`__turso_graph_expand` virtual table** (`graph_expand.rs`) now takes
  `from_role`/`to_role` as numeric role ordinals (`INTEGER HIDDEN`) rather than
  `'start'`/`'end'` text literals; `lowering.rs` passes `from_role.role.get()` /
  `to_role.role.get()` straight through with no direction translation anywhere
  in that path.
- **Snapshot builder (`graph/frontend/src/snapshot.rs`), the real Step 6
  target per Correction C** — replaced the two `.expect()` calls that assumed
  every relationship source has exactly `start`/`end` roles. The new pass:
  - One SQL query per relationship source fetches the relationship identity
    plus every single-valued (`One` cardinality) role's endpoint column in one
    row; every ordered pair of `One` roles gets an `EdgeInput` directly from
    that row (for two roles this is exactly the old start/end
    forward-and-reverse pair — binary is a layout of n-ary, not a separate
    kind).
  - One additional SQL query per `Many`-cardinality role, against its spill
    table, joined in Rust (not SQL) against the `One`-role node IDs already
    resolved in the first pass via a `relationships_by_identity` map; emits
    both directions of each (`One`, `Many`) pair.
  - **`Many`-`Many` role pairs are deliberately not produced** — this is the
    literal scope of Correction C's wording ("one pass per ordered pair of
    single-valued roles plus one pass per (`One`, `Many`) pair"), not an
    oversight. Flagging this explicitly as a known limitation of this task,
    not silently omitted.
  - A spill-table row referencing a relationship identity absent from the
    main pass is a new hard error, `SnapshotError::OrphanSpillRow`, rather
    than a silent skip (assert invariants, don't hedge).
- **Stale doc comments fixed** in `graph/frontend/tests/fixture.rs`
  (`ternary_session`, `witnessed_session`) that claimed "today's snapshot
  builder is binary-only" / "assumes every relationship source is binary" —
  both are now false; corrected to explain the fixtures simply don't exercise
  the (now-general) traversal path, per Correction C's explicit suggestion to
  check this.
- **Version bumps that must move together** (Correction D): `GRAPH_CATALOG_VERSION`
  3→4 (`graph/frontend/src/catalog.rs`); `SEMANTIC_PROFILE_VERSION` 2→3 and
  the mirrored `path_policy_version` 1→2 in `graph/ir/src/semantics.rs`, plus a
  new `relationship_arity: &'static str` field on `SemanticProfile`
  documenting the arity contract in `render()`'s digest input. Pin test
  (`graph/ir/tests/semantic_profile_pin.rs`) updated with the newly observed
  digest `fnv1a64:d064f72078704012` (was `fnv1a64:ad3c7f2313ac0e5d` at
  version 2).
- **`graph/runtime/benches/graph_shapes.rs`** — a Divan bench not touched by
  the earlier `EdgeInput` role-pair migration (missed because benches aren't
  compiled by `cargo build`/`cargo test`, only surfaced by
  `cargo clippy --all-targets`) needed `from_role`/`to_role` fields added
  using the same `role(1)`/`role(2)` convention as the rest of the runtime's
  tests.

## Test evidence

Before implementing Step 6/7, the pre-existing failing assertion was:
```
thread 'the_semantic_profile_mirrors_this_policy_version' — 
assertion left == right failed: bump SEMANTIC_PROFILE_VERSION and its pinned digest alongside PATH_POLICY_VERSION — left: 1, right: 2
```
(`PATH_POLICY_VERSION` had been bumped to 2 in a prior session; `path_policy_version`
in `SEMANTIC_PROFILE` was still 1 until this session's Step 7 edit.)

Gate sequence (Correction H order — both gates run before committing):

- `cargo fmt` — clean (only reformatted the accumulated diff, no substantive change).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings` —
  exit 0. (First run surfaced one real error: `graph_shapes.rs` bench missing
  `from_role`/`to_role`, fixed as above; second run clean. The "10 warnings"
  printed are unrelated `ar` toolchain noise from an unrelated crate's build
  script, not clippy lints.)
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher`
  — **387 passed, 3 ignored (22 suites)**, 0 failed.
- `mise run corpus` — per-suite passed counts, read directly from
  `graph/test-results/history.jsonl` for this run's `run_id`:

  | suite | passed | baseline | result |
  |---|---|---|---|
  | age-deep | 3042 | 3042 | exact match |
  | cqlite-deep | 113 | 113 | exact match |
  | grafeo-deep | 277 | 277 | exact match |
  | sparrowdb-deep | 2164 | 2164 | exact match |
  | tck-deep | 3331 | 3329-3332 (±2 flaky) | within band |

  (The task's own exit status reports "ERROR task failed" — this is expected,
  pre-existing behavior: the corpus has known non-tck failures baked into the
  baseline itself; what matters is the passed-count match, not a zero-failure
  run.)
- `mise run cypherbench-sample` — sample profile runs three domains; all match
  the pre-existing baseline (recorded in `task-15-report.md`) exactly, zero
  errors:

  | domain | matched | mismatched | errored |
  |---|---|---|---|
  | movie | 6 | 19 | 0 |
  | nba | 25 | 0 | 0 |
  | politics | 15 | 10 | 0 |

No non-tck suite moved off baseline; no re-run was performed hoping for
different numbers (cypherbench-sample's tail was re-captured a second time
only because the first run's output scrolled past the terminal buffer before
I captured it — both runs produced identical numbers, confirming determinism,
not cherry-picking).

## Deliberate omissions / scope limits

- `Many`-`Many` role pairs are not wired into the snapshot builder — out of
  scope per Correction C's literal wording. Any future n-ary relation with two
  or more `Many`-cardinality roles will not get cross-`Many` adjacency edges
  from this snapshot pass.
- No corpus fixture currently exercises a relation with more than two roles or
  with a `Many`-cardinality role through the traversal path, so this pass's
  correctness on those shapes is exercised by targeted Rust unit tests in
  `snapshot.rs`/`csr.rs`, not by the corpus.

## Brief inaccuracies found and corrected in place (not "fixed back")

- `graph/frontend/tests/fixture.rs`: `ternary_session` and `witnessed_session`
  each carried a stale doc comment asserting the snapshot builder was
  binary-only — both were already false as of this session's Step 6 work and
  have been corrected to state the fixtures simply don't exercise the general
  path, per Correction C's own suggestion to check this.

## Concerns

None blocking. The `Many`-`Many` limitation above is the only known gap and is
explicitly in-scope-as-omitted per Correction C, not a bug.

---

## Rework — review response (2026-07-26)

Status: **DONE**

Commit: `f48554da5` — graph/frontend: derive relationship edges from one
general role-pair pass

The review (`task-17-review.md`) came back Spec ❌, quality not approved as
final, on two findings, both above the line I'd drawn around
`Many`-`Many`. The coordinator's relay was explicit that Correction C's
literal wording — the basis for the "deliberate omission" recorded above —
was itself defective, not a correct scope boundary I'd merely satisfied.
That correction is superseded; the two findings below are what actually
governed this rework.

### Critical-1 — `(Many, Many)` silently produced zero edges

Confirmed independently before touching code: registering a relationship
source with two `Many`-cardinality roles and no `One` role succeeds,
snapshot build succeeds with no error, and `edge_count()` is 0 — a
constructible schema silently returning wrong (empty) traversal results,
exactly as the reviewer found.

Fix: replaced the three special-cased passes (One-One inline, One-Many
inline, Many-Many omitted) with **one general pass**. Each relationship's
role participation is flattened into `role_players: HashMap<SourceIdentity,
Vec<(RoleId, NodeId)>>` — a `One` role contributes exactly one entry from
the row query, a `Many` role contributes zero or more entries from its
spill table — and a single nested loop over that flat list emits an
`EdgeInput` for every ordered pair of *distinct* roles. There is no
cardinality branch anywhere in the edge-generation loop; a single general
pass expressed all three join shapes cleanly, so the "if it can't be made
general, say so" fallback wasn't needed.

This is constructible through the paths Tasks 14a/15 shipped (direct
`register_graph` plus SQL inserts into the auto-created spill tables), so
per the coordinator's instruction it was fixed by generalizing the builder,
not by rejecting the schema at registration time.

### A latent runtime bug this surfaced

Building the fix's own tests hit `RuntimeError::DuplicateRelationship` on
every legitimate `Many`-role edge past the first. Root cause, confirmed by
reading `graph/runtime/src/csr.rs`'s `Graph::build` validation loop (not
guessed): its dedup key was `(relationship, from_role, to_role)` alone —
correct only under the old assumption that a role pair produces at most one
edge per relationship. Once a `Many` role can have more than one player,
the same relationship legitimately emits multiple edges under the same
role-pair triple (one per distinct player pairing), which the old key
mistook for a duplicate push. Fixed by widening the key to
`(relationship, from_role, to_role, source, target)`; the one existing test
relying on the old key (`csr.rs`'s
`rejects_invalid_endpoints_duplicates_and_build_limits`, which pushes the
literal same `EdgeInput` twice) still passes, since all five components
still collide for a truly duplicate push.

### Important-1 — zero traversal-level coverage for the `Many`-role pass

Added two tests directly to `graph/frontend/src/snapshot.rs`'s own test
module (not `nary_relations.rs`: Task 13b's standalone-role `MATCH` syntax
isn't implemented, so a `Many`/`Many` fixture can't be expressed through
Cypher yet — these register via the Rust API and call
`build_traversal_snapshot` directly, the same approach the reviewer's own
C1 repro used):

- `a_single_valued_role_and_a_many_role_produce_traversable_edges_in_both_directions`
  — one `One`/`One`/`Many` relation (`start`, `end`, `witness`), asserts
  `neighbors()` reaches both spilled witnesses from `start` and reaches
  `start` from a witness (both directions).
- `two_many_roles_produce_the_full_cross_product_of_traversable_edges` —
  reproduces the reviewer's exact C1 fixture (two `Many` roles, no `One`
  role at all), asserts the full 2x2 author/editor cross product in both
  directions and that same-role pairs are never produced.

Both tests also assert `edge_count()`, but note `edge_count()` counts
distinct physical relationships, not adjacency rows (see
`edge_count_is_the_physical_relationship_count_not_the_stored_row_count` in
`csr.rs`) — it can only distinguish "some edge was built" (1) from "none
was" (0, C1's original symptom), so the traversal assertions are what
actually prove the join is wired correctly, not the count.

**Verified by deletion, not inferred**, per the coordinator's explicit
instruction:
- Deleted the single-valued-role push (`players.push((role.role, node))`)
  → both new tests failed: the `(One, Many)` test failed on
  `edge_count() == 0` (expected 1); 4 pre-existing tests also went red as
  expected collateral (they depend on the same push). Reverted; suite back
  to green.
- Deleted the many-role spill-join push (the classic S5 sabotage, now
  against the new one-pass design) → both new tests failed exactly at the
  traversal assertions:
  - `(One, Many)` test: `assertion failed: traversing start -> witness must
    reach both spilled witnesses, not zero — left: [] right: [NodeId(3),
    NodeId(4)]`
  - `(Many, Many)` test: `assertion left == right failed — left: 0 right: 1`
    (on `edge_count()`)
  No other test in the suite went red under this sabotage, confirming these
  two tests are the only coverage of this pass. Reverted; suite back to
  green (`git status --short` on both files showed only the legitimate diff
  after each revert).

### Gate re-run (edge-generation code changed, so nothing carried over)

- `cargo build -p turso_graph_runtime -p turso_graph_frontend` — clean.
- `cargo test -p turso_graph_frontend --lib snapshot` — 20 passed.
- `cargo test -p turso_graph_runtime --lib csr` — 8 passed.
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher`
  — **389 passed, 3 ignored (22 suites)**, 0 failed.
- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — exit 0 (the "10 warnings" are the same pre-existing unrelated `ar`
  toolchain noise from a build script, not clippy lints).
- `mise run corpus` — per-suite passed counts, computed from
  `graph/test-results/history.jsonl` for this run's `run_id`:

  | suite | passed | baseline | result |
  |---|---|---|---|
  | age-deep | 3042 | 3042 | exact match |
  | cqlite-deep | 113 | 113 | exact match |
  | grafeo-deep | 277 | 277 | exact match |
  | sparrowdb-deep | 2164 | 2164 | exact match |
  | tck-deep | 3331 | 3329-3332 (±2 flaky) | within band |

- `mise run cypherbench-sample`:

  | domain | matched | mismatched | errored |
  |---|---|---|---|
  | movie | 6 | 19 | 0 |
  | nba | 25 | 0 | 0 |
  | politics | 15 | 10 | 0 |

  All match baseline exactly; no suite moved, no re-run hoping for
  different numbers.

### Updated scope

The "Deliberate omissions / scope limits" section above is superseded:
`Many`-`Many` role pairs are now fully wired and covered by a
traversal-level test. There is no longer a known gap in role-pair edge
generation.

### Concerns

None blocking. The `csr.rs` dedup-key fix is a runtime-layer change outside
this task's originally-scoped files (`snapshot.rs`); it was necessary
because the old key was only ever correct under the single-player-per-role
assumption this rework removes, and is covered by the same pre-existing
test plus the two new traversal tests.

## Rework — review response (2026-07-26), Important-2

Re-review confirmed R1-R4 from the prior round held: the `csr.rs` dedup-key
widening was judged correct and necessary (reverting it fails both new
traversal tests with `DuplicateRelationship` the moment a role has more
than one player), and the same-role skip was confirmed intentional. One new
**Important** finding was raised and the coordinator explicitly overrode
the re-reviewer's "defer to a follow-up task" recommendation:

**Finding**: the O(players^2) pair loop this task added in
`build_in_transaction` (`graph/frontend/src/snapshot.rs`) ran unguarded.
It had no `check_cancelled`, unlike every other loop in that function, and
no running comparison against `BuildLimits::max_edges` inside the loop —
`max_edges` only fired later, in `Graph::build_cancellable`, by which point
the oversized `Vec<EdgeInput>` was already fully materialized. Before this
task, edge count was linear in relationships; the general pair pass made it
quadratic in players, so the same guard that was adequate against a linear
producer is now reached far faster and with far more memory paid for before
it fires.

### Fix

Added an inline guard immediately before the `edges.push(...)` in the pair
loop, mirroring the exact idiom the runtime already uses at the same
per-item cadence in its own loops (`csr.rs`'s edge-validation loop,
`traversal.rs`'s `step()`):

```rust
check_cancelled(cancellation)?;
if edges.len() as u64 >= limits.max_edges {
    return Err(RuntimeError::LimitExceeded {
        kind: LimitKind::Edges,
        limit: limits.max_edges,
    }
    .into());
}
```

No new limit, error variant, or second limits mechanism — reuses
`BuildLimits`, `RuntimeError::LimitExceeded`, and `LimitKind::Edges` as-is,
just applied one phase earlier. `LimitKind` added to the existing
`turso_graph_runtime` import in `snapshot.rs`.

### Test design and why a bare "error returned" assertion is worthless here

Both the early (in-loop) and late (post-materialization, in
`Graph::build_cancellable`) guard paths return the identical
`RuntimeError::LimitExceeded { kind: Edges, .. }`. Neither the `Result`
value nor `is_cancelled()` call counts can distinguish "bailed during
generation" from "bailed after materializing the full cross product" — the
two paths converge on the same trip point. The only externally observable
difference is wall-clock time (or memory): fully materializing a large
O(players^2) cross product before failing is measurably slower than
bailing during generation.

Added
`a_relation_whose_cross_product_exceeds_max_edges_is_refused_during_generation_not_after`:
a relation with two `Many` roles, 2,500 players each (12,500,000 candidate
edges for a single relationship), `max_edges: 3`. Asserts both the error
variant and a calibrated wall-clock ceiling (`elapsed < Duration::from_millis(100)`)
chosen from real measurements, not a guess:

- Fixed (in-loop guard): ~14.5-14.6ms across three runs (14.590667ms,
  14.605666ms, 14.520542ms).
- Sabotaged (guard removed, full materialization required before
  `Graph::build_cancellable`'s late check fires): ~258-268ms across three
  runs (267.92925ms, 258.461584ms, 259.498541ms).

100ms sits with wide margin above the fixed-path timing and wide margin
below the sabotaged-path timing, tolerating CI jitter in either direction
while still catching a regression to the old post-hoc-only guard. (An
initial threshold guess of 750ms at a smaller fixture size, N=2000, did
NOT catch the sabotage regression — the sabotaged path measured only
~170.6ms there, well under 750ms, meaning that test would have passed even
with the guard removed. Recalibrated at N=2500/100ms after measuring
several fixture sizes directly against the compiled test binary.)

**Verified by sabotage**, per the coordinator's explicit instruction:
removed the inline guard (`// SABOTAGE-3: inline guard temporarily removed
to confirm the new test goes red.`, only the bare `edges.push(...)`
remaining), rebuilt, ran the test, and it failed exactly as intended:

```
thread 'snapshot::tests::a_relation_whose_cross_product_exceeds_max_edges_is_refused_during_generation_not_after' panicked at graph/frontend/src/snapshot.rs:1864:9:
refusal took 259.419083ms; that is consistent with the full 12,500,000-edge cross product having been materialized before the max_edges check ever ran, not with an early exit inside the pair loop
```

Reverted the sabotage; confirmed via `git diff --stat` that only
`snapshot.rs` had a diff relative to the last commit, and via a clean
re-run of `cargo test -p turso_graph_frontend --lib snapshot` (21 passed).

### Gate re-run

- `cargo test -p turso_graph_frontend --lib snapshot` — 21 passed.
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher`
  — 390 passed, 3 ignored, 0 failed.
- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — exit 0 (same pre-existing unrelated `ar` toolchain build-script noise,
  not clippy lints).
- `mise run corpus` — per-suite passed counts for this run's `run_id`
  (`20260726T153043.034117Z-e63b3ff8f082-corpus-deep`):

  | suite | passed | baseline | result |
  |---|---|---|---|
  | age-deep | 3042 | 3042 | exact match |
  | cqlite-deep | 113 | 113 | exact match |
  | grafeo-deep | 277 | 277 | exact match |
  | sparrowdb-deep | 2164 | 2164 | exact match |
  | tck-deep | 3330 | 3329-3332 (±2 flaky) | within band |

- `mise run cypherbench-sample`:

  | domain | matched | mismatched | errored |
  |---|---|---|---|
  | movie | 6 | 19 | 0 |
  | nba | 25 | 0 | 0 |
  | politics | 15 | 10 | 0 |

  All exact match, no re-run hoping for different numbers. Per the
  coordinator's explicit instruction to report numbers rather than
  silently absorb any throughput cost from the added per-iteration check,
  compared this run's per-domain `load_ms`/`query_ms_total` (recorded in
  `graph/test-results/benchmarks.jsonl` at `2026-07-26T15:34:46.641713Z`)
  against the last run before this change (`2026-07-26T15:03:26.156069Z`):
  company load_ms 187→191/query_ms 46→47, movie 187→189/46→45, nba
  174→173/27→28, politics 120→116/18→18 — all within the same ±1-4ms
  jitter band observed between two back-to-back runs of identical code
  (`15:03:19` vs `15:03:26`). No measurable regression from the added
  check.

### Concerns

None blocking.

## Rework — review response (2026-07-26), Important-2 round 3

Coordinator confirmed the guard itself (mechanism, cadence, commit message)
was right and independently re-verified corpus/cypherbench on their side.
One remaining issue: the early-exit proof was a wall-clock threshold
(`elapsed < 100ms`), which is a CI flake waiting to happen under load, even
though the underlying reasoning for needing *some* discriminator (a bare
"error came back" assertion passes either way, since both the early and
late guard paths return the identical `LimitExceeded`) was correct.
Instructed to replace it with a deterministic observable, suggested a
counting `Cancellation` since `check_cancelled` already threads one
through the pair loop, ruling out wall-clock time entirely, and to shrink
the fixture now that a large cross product is no longer needed to buy a
measurable timing gap.

### First attempt, and the false negative it produced

Implemented a `CountingCancellation` (an `AtomicU64` poll counter behind
the existing `Cancellation` trait) and initially compared two runs on the
same fixture: an *uncapped* build (nothing trips `max_edges`, so the pair
loop runs to completion) as a "full cost" baseline, against a *capped*
build (`max_edges: 3`) as the guarded run, asserting
`early_polls * 5 < full_polls`.

Verified by sabotage exactly as instructed (remove the inline guard,
confirm the assertion goes red) — and it did not. Removing the whole
guard block still passed: `full_polls=2759, early_polls=161`, and
`161 * 5 = 805 < 2759` held. Root cause: the uncapped run's own
`max_edges` (the default, effectively unlimited) is never reached whether
the guard exists or not, so removing the guard does not change the
uncapped run at all — comparing against it is structurally blind to this
exact regression. This is the same category of mistake as the round-2
threshold miscalibration (a discriminator that passes under the sabotage
it exists to catch), caught the same way: by actually sabotaging and
reading the result rather than trusting the design.

### Corrected design

Dropped the uncapped-run comparison. The capped run's own poll count,
compared against a fixed number derived from what actually differs
between "guard present" and "guard removed", is the real discriminator:

- Guard present: the pair loop bails after ~`max_edges` iterations —
  **110 polls** on the 25-players-per-role fixture (1,250 candidate
  edges, `max_edges: 3`).
- Guard removed: the pair loop runs unpolled to completion (no
  `check_cancelled` call left to poll), and the refusal comes only from
  `Graph::build_cancellable`'s own, separate `max_edges` check — reached
  only after that function's *node* loop unconditionally polls once per
  graph node (2 x 25 = 50), since that loop has no early-exit of its own
  to skip — **161 polls**.

Both numbers are exact and 100% reproducible on this fixture (driven by
loop counts, not hash-iteration order or timing), so `135`, sitting
strictly between them, is a completely reliable boundary — no jitter
margin needed, unlike a wall-clock threshold.

Also shrank the fixture per the coordinator's suggestion: 2,500 players
per role (12,500,000 candidate edges) → 25 (1,250). A poll-count
discriminator only needs an unambiguous count difference, not a
wall-clock-sized gap, so the smaller fixture is sufficient and the test
dropped from ~14.5-268ms to well under 1ms.

### Verified by sabotage (two ways)

1. **Whole guard removed** (`check_cancelled` + `max_edges` check both
   deleted, the natural single-hunk revert of this fix): test failed —
   `pair loop polled cancellation 161 times; expected fewer than 135, ...`.
   This is also the case that the first (uncapped-comparison) design
   failed to catch.
2. **Only the `max_edges` branch removed, `check_cancelled` left in
   place** (a partial regression matching the coordinator's original
   framing that a late exit polls "on the order of players^2"): test
   failed — `pair loop polled cancellation 1411 times; expected fewer
   than 135, ...`. This confirms the discriminator also catches the
   partial-regression shape, with an even larger, unambiguous margin.

Restored the guard after each sabotage; confirmed via `git diff --stat`
both times that only `snapshot.rs` had a diff and that the diff was
confined to `mod tests` (hunks all at line 1757+, none touching
`build_in_transaction`).

### Gate re-run

- `cargo test -p turso_graph_frontend --lib snapshot` — 21 passed.
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher`
  — 390 passed, 3 ignored, 0 failed.
- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — exit 0 (same pre-existing unrelated `ar` toolchain build-script noise).
- `mise run corpus` / `mise run cypherbench-sample` — not rerun this
  round, per the coordinator's explicit exemption (test-only change, no
  effect on the edge-generation code path; the change stayed confined to
  `mod tests` as confirmed above, so the exemption held).

### Concerns

None blocking.

## Status

DONE. Commit `ae795a64c3434c787ab906538168319b9bcfa5d6`.
