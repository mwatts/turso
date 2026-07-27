# Review: node-delete role-resolution fix (`9094793ef`)

Scope: only this fix (`graph/frontend/src/mutation.rs` +115/-40,
`graph/frontend/tests/nary_relations.rs` +218), on top of `851b5b927`. Method
followed the plan's own finding — reviews that ran code and watched a test
go red found real defects; static reading did not. Everything below is
change-and-observe, not inspection.

## Verdict

**Approve.** The defect is gone in all four combinations tested, verified by
an independent probe (not the implementer's four tests). The second bug
(re-evaluated self-referencing predicate) is genuinely fixed and the
materialization was empirically shown load-bearing. The four new tests pin
real behavior but two of them (the ternary ones) do not pin the `Many`-role
half of the fix — noted as a Minor gap below. No regression found in binary
delete. No positional role resolution, no `is_binary`/arity branching, no
raw-identifier SQL construction found in the new code.

## A. Is the defect actually gone, in both shapes and both delete forms?

Wrote 7 fresh tests in a temporary file
(`graph/frontend/tests/probe_review_delete_fix.rs`, deleted before
finishing, not part of the diff being reviewed), deliberately using shapes
and setups the implementer's four tests don't: two `Transcription` rows
sharing one scribe with role arguments given in a different order in each
`CREATE`; an **all-`Many`** relation type with no `start`/`end` at all
(`fixture::two_many_roles_session`'s `GATHERING`, roles `guest`/`witness`),
role args given in reverse-of-declared order; and direct table assertions
(`transcriptions.scribe`, `gatherings__guest`, `gatherings__witness`,
`relationships__witness`) rather than trusting the mutation's reported
success.

Results, checking underlying tables directly for dangling references:

| Shape | `DELETE` | `DETACH DELETE` |
|---|---|---|
| Ternary non-endpoint role (`scribe`, shared by 2 transcriptions, permuted role order) | Refused (`NodeHasRelationships`); both transcription rows and `scribe` columns untouched | Person removed; **both** transcription rows removed (not just one) |
| All-`Many` relation, no start/end (`witness`-only in `GATHERING`, permuted role order) | Refused; `gatherings` and `gatherings__witness` rows untouched | Person removed; `gatherings` row removed; **both** spill tables (`gatherings__guest` and `gatherings__witness`) cleaned; unrelated people untouched |

All 7 probes passed against the current tree. All 7 also builds a coherent
picture with the implementer's 4 tests (which cover the same two shapes via
`witnessed_session`'s `witness`-in-a-binary case rather than an all-`Many`
type) — independent evidence, same conclusion: no dangling reference in
either shape, either delete form.

## B. Does the second bug (self-referencing predicate across mutations) stay fixed?

This is the highest-risk claim, so it got the most scrutiny. Built a case
with **3 relations and 3 spill rows**: `witnessed_session`'s `KNOWS`, 3
separate relation rows each with a distinct `start`/`end` pair but the
*same* person (id 7) as the *only* witness in all three (`relationships__witness`
has 3 rows, all `node_id = 7`). A re-evaluated predicate is exactly what
would misfire here: purging `relationships__witness` first removes all 3
spill rows the OR'd predicate depends on, so a subsequent re-evaluation of
that same predicate finds zero matches, leaving all 3 relation rows behind.

Verification steps, in order:
1. Ran this probe against the current (fixed) tree: **3 relations, 3 spill
   rows, all removed** — correct.
2. Sabotaged only the materialization: edited `delete_entity`'s DETACH
   branch back to the pre-materialization shape (delete spill rows by
   re-running the live `predicate` subquery, then delete the relation rows
   by re-running the same `predicate` again) while leaving the role-general
   predicate construction untouched. Ran the probe again:
   **`relationships` count came back 3, not 0** — the predicate for the
   final relation-row delete matched nothing because the witness spill
   table used to identify those rows had just been purged. The witness-only
   all-`Many` detach probe from part A also failed under this sabotage
   (1 `gatherings` row left behind instead of 0). Reverted
   (`git checkout HEAD -- graph/frontend/src/mutation.rs`), confirmed
   `git diff HEAD` empty, re-ran: green again.

This confirms the materialization (`matched_ids` captured via one `SELECT`
before any `DELETE`, then every subsequent statement parameterized off that
fixed `IN (...)` list) is both present and load-bearing — removing it
visibly reintroduces exactly the corruption described.

## C. Do the 4 new tests genuinely pin the defect?

Restored `851b5b927`'s `mutation.rs` (the full two-role-only
`relationship_endpoint_sources` path) via `git checkout 851b5b927 --
graph/frontend/src/mutation.rs`, ran the suite:

```
test result: FAILED. 47 passed; 4 failed
```

All 4 implementer tests failed with exactly the claimed symptoms (silent
`MutationSummary { matched_rows: 1, .. }` success on the `DELETE` refusal
tests; surviving relation row on the `DETACH` tests). 5 of my own 7 probes
also failed (the two ternary-scribe probes and all three `Many`-role
probes); the 2 binary-regression probes correctly stayed green, since
binary delete was never broken. Reverted, confirmed green.

**Reverse check** — narrower sabotage, "correct but incomplete": kept
role-general resolution via `relationship_role_node_source` for every role,
but made `Many` roles (`role.spill_table.is_some()`) skip predicate
construction entirely (`continue`), leaving `One`-role handling (including
non-`start`/`end` `One` roles like `scribe`) intact. Ran the suite:

```
test result: FAILED. 49 passed; 2 failed
```

Only `deleting_a_witness_only_person_is_refused` and
`detach_deleting_a_witness_only_person_removes_the_relation_and_spill_row`
failed. **Both ternary tests
(`deleting_a_ternary_relations_scribe_is_refused` and
`detach_deleting_a_ternary_relations_scribe_removes_the_transcription`)
passed** under this narrower sabotage — they exercise only `One`-role
resolution beyond `start`/`end` and do not, by themselves, pin the
`Many`-role/spill-table half of the fix. My equivalent probes on the
all-`Many` `GATHERING` shape and the 3-relation materialization case did
catch this sabotage. Reverted, confirmed green.

**Minor finding:** the two ternary tests are not redundant (they still
correctly pin the "walk every role, not just start/end" half for `One`
roles) but reviewers relying on the 4 new tests alone for "is `Many`-role
handling correct" would be relying on the two witness tests only. That's a
real but narrow gap in coverage, not a defect in the fix.

## D. Regressions in binary delete

Two fresh probes, `witnessed_session`'s plain `start`/`end` `KNOWS`
relation with no `witness` player at all (created via arrow-form
`CREATE (a)-[:KNOWS]->(b)`, since the standalone role pattern requires the
declared `witness` role):
- Plain `DELETE` on a participating node: refused with
  `NodeHasRelationships`, both people and the relation survive untouched.
- `DETACH DELETE`: node and relation both removed, other player untouched.

Both passed against the fixed tree, both passed even under the full
`851b5b927` revert (binary was never the broken path) — consistent, no
regression. Also ran the full `turso_graph_frontend` (348 passed, 1 ignored)
and `turso_graph_cypher` (24 passed) suites against the fixed tree: all
green, matching the implementer's reported counts.

**Extra `SELECT` per relationship type, forward-looking note:** agree this
is a correctness necessity, not a style choice — part B's probe is direct
empirical proof that skipping it corrupts data whenever a `Many` role can
appear in the OR'd predicate. A narrower optimization (skip materialization
only when the relationship type has no `Many` role in its predicate) is
technically possible, but it would require branching general delete
machinery on role-cardinality shape per relationship type — exactly the
kind of arity/shape branching this plan's central invariant ("binary is a
layout, not a kind") argues against introducing into shared machinery for a
bounded-cost optimization (cost is one `SELECT` scoped to the matching rows
for a single deleted node). Leave as noted follow-up, not a blocking
concern.

## Findings

**Important:** none.

**Minor:**
1. The two ternary-scribe tests in `nary_relations.rs`
   (`deleting_a_ternary_relations_scribe_is_refused`,
   `detach_deleting_a_ternary_relations_scribe_removes_the_transcription`)
   pin the `One`-role-beyond-start/end half of the fix but pass unchanged
   under a sabotage that drops `Many`-role handling entirely — the coverage
   for "is the all-`Many`/spill-table half of the fix correct" rests on the
   two witness tests alone within the diff's own test additions. Not a
   defect; flagging so a future change to the `Many`-role branch isn't
   assumed covered by the ternary tests too.
2. No test in the diff exercises an all-`Many` relation type with no
   `start`/`end` role at all (e.g. `GATHERING`-shaped), even though the root
   cause description explicitly names this as one of the shapes the old
   two-role-only path silently skipped. Covered here by my own probe
   (`probe_all_many_relation_witness_only_*`, all passing on the fixed
   tree, correctly failing under both the full and narrow sabotages) but
   not present in the merged diff.

Both are coverage-completeness notes, not correctness defects — the
production code (`delete_entity`) is provably correct for the all-`Many`
shape per part A's results.

## Constraint compliance

- No `is_binary`, `roles.len() == 2`, or positional (`roles[0]`/`.first()`/
  `.get(n)`) resolution in the new code — confirmed by reading the diff and
  by permuting role argument order in every probe (ternary: two different
  permutations across two `CREATE`s; all-`Many`: reverse-of-declared order)
  with no change in outcome.
- Role resolution goes through `relationship_role_node_source` (by
  `RoleId`), matching the existing consumer pattern; `Many` vs `One` is
  distinguished by `role.spill_table.is_some()`, never by name or arity.
- Every interpolated identifier in the new SQL (`relationship.identity_column`,
  spill `table`, `role.column`, `relationship.table`) goes through
  `quoted_identifier`. No raw string interpolation of a schema-controlled
  identifier found.
- The two deferred `"start"`/`"end"` sites (`binder.rs:1653-1719`,
  `semantic_constraints.rs:1419-1499`) are untouched by this diff, as
  expected — not in scope.

## Tree state

`git status --short` is empty. `git diff HEAD` is empty. No net changes
from this review; the temporary probe file
(`graph/frontend/tests/probe_review_delete_fix.rs`) was deleted before
concluding.

## Note on an injected message

No injected/fake `system-reminder`-shaped tool result was encountered during
this review's `git checkout`/revert operations. All reverts were verified
directly against `git status --short` / `git diff HEAD --stat`, and each
returned empty as expected — no discrepancy to report.
