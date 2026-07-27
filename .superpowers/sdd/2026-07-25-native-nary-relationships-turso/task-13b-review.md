# Task 13b review — relation-anchored MATCH over a standalone role pattern

Diff range reviewed: `eab179db4..075676383` (`075676383 graph/frontend: bind
MATCH-side standalone role patterns`). The later commit `a230c2f38` (test-results
bookkeeping) is out of scope and was not reviewed.

Method: read the brief, report, and diff; then edited the working tree to run
three sabotages (two prescribed by the brief/controller, one I added myself to
probe an untested code path), watched tests fail, and reverted every edit.
Working tree is clean at the end (`git status --porcelain` empty).

---

## Verdict 1 — spec compliance (`task-13b-brief.md`)

- **Step 1 (new plan node, no `RoleExpand` reuse, no arity branch):** ✅ met.
  `RelationScan`/`RoleJoin`/`RolePlayer` are new IR (`graph/ir/src/plan.rs`).
  `bind_match_role_pattern` (`binder.rs:2770`) loops `pattern.roles` once per
  named argument, wrapping one `RoleJoin` per iteration — no `if roles.len()
  == 2`, no `is_binary`, no hard-coded `start`/`end` anywhere in this path.
  `Many` roles are rejected by `role.cardinality == RoleCardinality::Many`
  (binder) and defended again by `role.cardinality != RoleCardinality::One`
  (lowering) — both structural, never by name or position.
- **Step 2 (failing tests, non-vacuous subset test):** ✅ met. Both tests
  added to `nary_relations.rs` use `fixture::ternary_session`/
  `witnessed_session`-style idioms already used elsewhere in that file (no
  invented `session.run`/`.sql`/`.query returning Vec<Vec<&str>>` API).
  `a_match_role_pattern_may_leave_roles_unnamed` seeds **two** relations
  sharing `start` but differing in `end` (2 vs 3) and asserts both rows come
  back from a `start`-only match — this is the non-vacuous shape the brief
  requires, confirmed by reading the fixture, not assumed.
- **Step 3 (verify red before the fix):** ⚠️ cannot verify directly from the
  diff alone (diffs don't show intermediate red state), but the report's
  claim is consistent with the tree: prior to this commit `bind_match` calls
  `only_paths(&clause.paths)?`, which errors on any `Roles` element, so the
  new tests could not have passed against the pre-diff binder.
- **Step 4 (rewrite goldens to compare SQL, not plan identity):** ✅ met.
  `desugaring_golden.rs` now calls `lower_fixture`/`lower_relational` and
  compares `ast::Stmt`-derived SQL text via `assert_eq!` with both statements
  quoted on failure. The module doc comment was rewritten to state the SQL/
  layout contract instead of the identity claim. Both tests remain
  `#[ignore]`d with a documented, evidenced SQL divergence — per the task
  framing this is a settled, parked decision, not a re-litigated finding here.
  `first_role_expand` is left in `fixture.rs` (still `#[allow(dead_code)]`,
  still used by other test files) — correct per the brief's "check before
  deleting" instruction.
- **Step 5 (traversal-snapshot decision):** ✅ met — no change needed, and the
  brief itself says the controller already confirmed this independently. Not
  re-litigated.
- **Step 6 (write classification):** ✅ met — confirmed correct as-is, brief
  says this was already settled. Not re-litigated.
- **Step 7/8 (sabotage verification, gate, corpus/cypherbench):** ✅ met. Gate
  numbers in the report (338 passed/3 ignored; corpus exact per suite;
  cypherbench exact per domain) were not re-run per instructions (corpus/
  cypherbench are release builds and explicitly marked "do not re-run"), but
  `cargo test -p turso_graph_cypher -p turso_graph_frontend` was re-run here
  and is green. The two required sabotages were independently reproduced (see
  Verdict 2) with output matching the report's claims exactly.

**Overall Verdict 1: the diff meets the brief, step by step, with the two
already-settled items (5, 6) correctly left alone and the two golden tests
correctly parked per ruling (B).**

---

## Verdict 2 — task quality

### Sabotage results (all reverted; tree confirmed clean)

1. **Permute the role a `RoleJoin` resolves.** Changed the `RoleJoin`
   construction in `bind_match_role_pattern` so `role:` used a different
   declared role than the one resolved by name (`declared.iter().find(|c|
   c.name != role.name)...`) instead of `role.role`. Ran
   `cargo test -p turso_graph_frontend --test nary_relations`:
   `a_match_role_pattern_binds_the_named_players` went red with
   `left: [[Numeric(Integer(2)), Numeric(Integer(1))]] right:
   [[Numeric(Integer(1)), Numeric(Integer(2))]]` — the exact swapped-columns
   signature, and an exact match to what the implementer's report independently
   claims it observed. Roles are resolved by `RoleId`, not position. Reverted;
   confirmed clean.
2. **Break the subset projection.** Added `if seen.len() != declared.len() {
   return Err(...) }` after the role-argument loop in
   `bind_match_role_pattern`, forcing every declared role to be named. Both
   Step-2 tests went red (`witnessed_session`'s `KNOWS` declares three roles —
   `start`/`end`/`witness` — and both new tests name only a subset):
   `Database(ParseError("SABOTAGE: role subset is not supported ..."))` for
   both `a_match_role_pattern_binds_the_named_players` and
   `a_match_role_pattern_may_leave_roles_unnamed`. Reverted; confirmed clean.
   The fixture is non-vacuous as required (see Verdict 1, Step 2).
3. **Scrutinize the flipped test.** `git log -S "Task 13 flips this assertion
   to a success case"` shows this comment was introduced in commit
   `1e2c16b16` ("cypher: parse standalone role pattern syntax"), 18 commits
   before this diff on the branch — i.e. it predates and explicitly
   pre-authorizes today's flip; it was not invented in this diff to justify
   itself. The rewritten test
   (`a_standalone_role_pattern_binds_to_a_relation_scan_and_role_joins`)
   asserts a real, structural plan shape — `Project → RoleJoin → RoleJoin →
   RelationScan` — not merely "it returned `Ok`". This is a meaningful
   assertion, not the weakest thing that passes: I confirmed via sabotage 1
   that a positional-resolution bug would still produce this exact shape but
   wrong data, so this unit test alone would not catch a positional bug — but
   the execution-level tests in `nary_relations.rs` do, and together they
   cover both "right shape" and "right values." No defect found here.

### Additional finding from my own probe (not in the brief's sabotage list)

- **The `Many`-cardinality-role-in-MATCH rejection path is implemented but
  has zero test coverage in this diff.** Both `bind_match_role_pattern`'s
  `at_unsupported("a Many-cardinality role in a MATCH role pattern")` check
  and `lower_role_join`'s defensive `role.cardinality != One` guard are new
  code with no test in `nary_relations.rs` (or elsewhere in the diff)
  exercising `MATCH [x:KNOWS](witness: w)` against `witnessed_session`'s
  three-role relation. I added a throwaway test doing exactly that
  (`CREATE [x:KNOWS](start: a, end: b, witness: w)` then `MATCH
  [x:KNOWS](witness: w) RETURN w.id`) and confirmed it does error correctly:
  `Cypher mutation binding failed: a Many-cardinality role in a MATCH role
  pattern is not supported ...`. So the behavior is *correct*, but it shipped
  untested — the report's own justification (deferring the `Many`-hop case to
  Task 14b, evidenced by `task-14-report.md`'s "Step 5 ... deferred to Task
  14b (blocked on Task 13b)") explains *why* the feature isn't implemented,
  but not why the rejection path itself has no assertion. Classified
  **Minor**: verified correct by me, evidenced deferral rationale exists, but
  test coverage for the new error path is missing.
  Reverted (via `git checkout --`); tree confirmed clean.

### Design review of new IR (`graph/ir/src/plan.rs`)

- `RelationScan { graph, source, binding, relationship_types }` and
  `RoleJoin { input, relationship, relationship_source, role, player }` are
  minimal for what the binder/lowering actually read. `relationship_types` is
  a `Vec` (always populated with exactly one element today, since
  `pattern.types.len() != 1` is rejected earlier) — this mirrors the existing
  `NodeScan.labels: Vec<LabelId>` precedent and carries the parser's own
  already-`Vec`-typed AST field through rather than adding new complexity; not
  YAGNI bloat.
- `RelationScan.graph` is not read in `lower_relation_scan` — but `NodeScan.graph`
  is equally unread in `lower_node_scan` in the pre-existing code, so this
  matches established convention rather than being a new dead field introduced
  by this diff.
- `RolePlayer::{Fresh, Bound}` cleanly mirrors the existing
  `RoleExpand.bound_target: Option<BindingId>` idea but as a proper enum
  instead of an `Option` bolted onto a single "target" field — arguably a
  small improvement in clarity for this new node, and appropriately scoped
  (no extra fields).
- No field on either struct goes unread by both the lowering and the test
  helpers (`fixture.rs`, `semantic_schema.rs`) that pattern-match over
  `PlanKind` — every match arm added for the two new variants does real work
  (walks `.input`, or terminates as a leaf), so this isn't a case of new IR
  carrying unread fields.

### Arrow-path / ruling (B) compliance

- Confirmed via `git log -p eab179db4..075676383 -- graph/frontend/src/binder.rs`
  that `bind_path`, `should_reverse_path`, and `reverse_path` are **untouched**
  by this diff — only `bind_match`'s top-level dispatch loop changed (from
  pre-filtering to `paths` via `only_paths` to iterating
  `clause.paths.elements` and matching on `Path`/`Roles`), and the `Path` arm's
  body is a verbatim copy of the old loop body.
- Confirmed via `git diff -- graph/frontend/src/lowering.rs` that
  `lower_role_expand` (the arrow form's lowering) is untouched — only new
  functions were added alongside it.
- The corpus being exact on all five suites (per the report, not re-run here
  per instructions) is consistent with, and further corroborated by, this
  direct diff inspection: the arrow path was not re-planned.

### Identifier quoting

- New SQL-building code (`lower_relation_scan`, `lower_role_join`) uses
  `quote_identifier` (no `d`), not the `quoted_identifier` wrapper. This
  is **not** a defect: within `lowering.rs` itself, `quote_identifier` is the
  module-private function and is used the same way 76 other times in this
  file; `quoted_identifier` is a `pub(crate)` re-export of it that only
  *other* modules (`mutation.rs`, `semantic_constraints.rs`) import, since
  they don't have direct access to the private fn. The new code is
  consistent with the file's own established convention.

### Findings summary

| # | Finding | Severity | How found |
|---|---|---|---|
| 1 | `Many`-cardinality role rejection in MATCH role patterns has no test coverage (behavior verified correct by hand) | Minor | Sabotage-style probe (test added, run, reverted) |

No Critical or Important findings. Both prescribed sabotages (role permutation,
subset-projection break) produced exactly the failures the brief and the
implementer's report describe, independently reproduced. The flipped unit
test's authorizing comment predates this diff by 18 commits. The arrow path's
binder and lowering are untouched. No two-role assumption, no positional role
resolution, and no un-quoted/hard-coded identifier interpolation were found in
the new code.

**Overall Verdict 2: the implementation is correct, adequately tested (with
one minor coverage gap), respects every global constraint (binary-is-a-layout,
no arity branch, `Many` identified structurally, arrow path untouched), and
the new IR is minimal and fully consumed.**
