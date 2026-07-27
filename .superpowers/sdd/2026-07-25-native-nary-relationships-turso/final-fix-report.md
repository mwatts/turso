# Final fix report: `DELETE`/`DETACH DELETE` silently orphans role players

Fixes the single Important finding from `final-review.md`: node deletion
resolved a node's participating relations only through
`relationship_endpoint_sources` (`schema_catalog.rs:454`, two-role
`start`/`end` only), so any relation shape outside that pattern — a ternary
relation's non-endpoint role, or any `Many` role, even inside an otherwise
binary `start`/`end` relation — was silently skipped by `delete_entity`
(`mutation.rs`). No error, no cleanup: the node vanished while the relation
row (and, for `Many` roles, the spill row) kept pointing at the
now-nonexistent identity.

## Which option was taken

**The full fix** — `delete_entity` now resolves participation for every
relation type through the full role model, not the two-role shortcut. This
was tractable as a single change because the general-purpose primitives it
needed already existed and were already load-bearing elsewhere in the
codebase (`relationship_role_node_source` for per-role, name/position-free
source resolution — already a real consumer per the review's finding 4 —
and the `Some(spill_table)`/`None` match idiom `lower_role_join` already uses
to distinguish a `Many` role's spill-table membership test from a `One`
role's column equality). No new resolution mechanism had to be invented; the
node-delete path just had to stop bypassing the ones that already existed.

## What changed

`graph/frontend/src/mutation.rs`, `delete_entity`:

1. **Reference detection is now role-general.** For each relationship
   source, walk every declared role (`&relationship.roles`), not just
   `start`/`end`. For each role, resolve its physical node source via
   `catalog.relationship_role_node_source(graph, relationship_source,
   role.role)` (by `RoleId`, never by name or position) and skip roles that
   don't target this node's source. A matching role contributes a predicate:
   a `One` role (`role.spill_table == None`) contributes `column = $id`; a
   `Many` role (`role.spill_table == Some(table)`) contributes `identity IN
   (SELECT relation_id FROM spill_table WHERE node_id = $id)` — an
   existence test against the spill table, mirroring the join
   `lower_role_join` already builds for a bound `Many`-role player. This
   alone fixes the plain-`DELETE` refusal check and closes the ternary/
   witness-only silent-success gap.

2. **A second, real bug surfaced while fixing the first, and had to be
   fixed too.** The `DETACH` branch originally re-evaluated the same OR'd
   `predicate` string three times in sequence: once per `Many` role to purge
   its spill rows, then again to delete the relation row itself. That was
   safe for the old two-role predicate (built only from immutable `start`/
   `end` columns), but the new predicate can itself contain a `Many` role's
   spill-table subquery. Purging that spill table first — before the later
   re-evaluations run — makes the predicate *stop matching* the very rows it
   just identified (the row it needs to find no longer has the spill entry
   that made it match), silently leaving both other roles' spill rows and
   the relation row itself undeleted. Fix: capture the matching relation
   identities into a concrete list (one `SELECT` before any mutation) and
   drive every subsequent spill purge and the final row delete off that
   fixed, parameterized `IN (...)` list instead of re-running the
   self-referencing predicate. This was caught by writing the DETACH tests
   below, not by inspection — worth flagging in case a similar
   re-evaluated-predicate-over-a-table-you're-about-to-mutate pattern exists
   elsewhere.

No branching on relationship arity was introduced. `role.cardinality`
(already the established idiom in this exact function, used unchanged for
the pre-existing `Many`-role spill-purge loop) and `role.spill_table.is_some()`
(the idiom used for the new predicate construction, matching
`lower_role_join`) are both data-driven per-role properties, never a
`roles.len()` or positional check. `relationship_endpoint_sources` and
`start_role()`/`end_role()` are untouched and still have their other
legitimate consumers (arrow-form sugar in `binder.rs`, `semantic.rs`/
`snapshot.rs`, `schema_catalog.rs` tests) — this fix does not remove or
repurpose them, it just stops `delete_entity` from relying on them as its
only path.

## Tests added

`graph/frontend/tests/nary_relations.rs`, four new tests, using the existing
`fixture::ternary_session()` and `fixture::witnessed_session()` fixtures
(the latter built for exactly this shape: `start`/`end`/`witness`(`Many`)):

- `deleting_a_ternary_relations_scribe_is_refused` — a `scribe` (no
  `start`/`end` at all) still cited by a `Transcription` must refuse plain
  `DELETE` with `NodeHasRelationships`; asserts the person row and the
  transcription's `scribe` column are both untouched.
- `detach_deleting_a_ternary_relations_scribe_removes_the_transcription` —
  `DETACH DELETE` on that scribe must remove the transcription row, not
  leave `transcriptions.scribe` dangling.
- `deleting_a_witness_only_person_is_refused` — a person who is *only* a
  `witness` (`Many` role, never `start`/`end`) must refuse plain `DELETE`;
  asserts the person row and the `relationships__witness` spill row are both
  untouched.
- `detach_deleting_a_witness_only_person_removes_the_relation_and_spill_row`
  — `DETACH DELETE` on that witness must remove the `KNOWS` relation and its
  spill row, while leaving the unrelated `start`/`end` people intact.

## Sabotage verification

Reverted `graph/frontend/src/mutation.rs` to its pre-fix state (`git
checkout --`, restoring the two-role-only `relationship_endpoint_sources`
path) while keeping the new tests, then ran `cargo test -p
turso_graph_frontend --test nary_relations`. All four new tests went red,
the rest of the suite (47 tests) stayed green:

```
---- deleting_a_ternary_relations_scribe_is_refused stdout ----
thread 'deleting_a_ternary_relations_scribe_is_refused' panicked at graph/frontend/tests/nary_relations.rs:597:10:
a scribe still cited by a transcription must refuse plain DELETE: MutationSummary { matched_rows: 1, operations_executed: 1, rows: [], result_types: [] }

---- deleting_a_witness_only_person_is_refused stdout ----
thread 'deleting_a_witness_only_person_is_refused' panicked at graph/frontend/tests/nary_relations.rs:699:10:
a witness-only player still recorded in the spill table must refuse plain DELETE: MutationSummary { matched_rows: 1, operations_executed: 1, rows: [], result_types: [] }

---- detach_deleting_a_witness_only_person_removes_the_relation_and_spill_row stdout ----
thread 'detach_deleting_a_witness_only_person_removes_the_relation_and_spill_row' panicked at graph/frontend/tests/nary_relations.rs:767:5:
assertion `left == right` failed: detaching the only-witness player must remove the relation that referenced it
  left: [[Numeric(Integer(1))]]
 right: [[Numeric(Integer(0))]]

---- detach_deleting_a_ternary_relations_scribe_removes_the_transcription stdout ----
thread 'detach_deleting_a_ternary_relations_scribe_removes_the_transcription' panicked at graph/frontend/tests/nary_relations.rs:662:5:
assertion `left == right` failed: detaching the scribe must remove the transcription referencing it, not leave a dangling scribe column behind
  left: [[Numeric(Integer(1))]]
 right: [[Numeric(Integer(0))]]

test result: FAILED. 47 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

Each failure is exactly the shape of the defect: a mutation that reports
success (`matched_rows: 1`, no error) while leaving the dangling reference in
place, or a `DETACH DELETE` whose target row survives. `git status --short`
and `git diff --stat -- graph/frontend/src/mutation.rs` confirmed only that
file was reverted; the patch was then reapplied (`git apply`) and the full
suite re-verified green.

## Gate results

- `cargo fmt`: clean (reformatted the new test file's line-wrapping only;
  no logic change — re-verified by re-running the full suite after).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`:
  **exit 0**, run exactly as specified (not narrowed to `-p <package>`).
- `cargo test -p turso_graph_cypher -p turso_graph_frontend`: **all green**
  — `turso_graph_frontend` unittests 159 passed; `nary_relations.rs` 51
  passed (47 pre-existing + 4 new); every other suite in both crates
  unchanged and green (365 total passed across both crates' unit + doctest +
  integration suites, 1 pre-existing ignored).
- `mise run corpus`: task reports exit 1 (expected — the task always does
  when any query in the 10k+ corpus fails, which pre-existing unsupported
  syntax always causes). Per-suite counts from `graph/test-results/runs.jsonl`
  for this run (`851b5b927`, dirty tree): `age-deep` 3042/3042 exact,
  `cqlite-deep` 113/113 exact, `grafeo-deep` 277/277 exact, `sparrowdb-deep`
  2164/2164 exact, `tck-deep` 3330 (within the 3329-3332 band). **All at
  baseline, no regression.**
- `mise run cypherbench-sample`: exit 0. Per-domain `matched`/`mismatched`
  counts (`graph/test-results/benchmarks.jsonl`, this run) are identical to
  the immediately preceding baseline run: company 13/12, fictional_character
  14/11, flight_accident 24/1, geography 11/14, movie 6/19, nba 25/0,
  politics 15/10. **No regression.**

`graph/test-results/REPORT.md`, `benchmarks.jsonl`, and `runs.jsonl` were
updated by running these gate tasks (as they always are) but were left
uncommitted per instruction — the user commits `graph/test-results/`
separately. `history.jsonl` is gitignored and untouched by this commit
either way.

## Concerns / follow-up

None identified that block this change. One thing worth flagging for future
work on this delete path: the DETACH branch now does one extra `SELECT`
per relationship type per deleted node (to materialize matching relation
ids before mutating anything) where the old code did none for the pure
`One`-role case. This is a correctness necessity (see point 2 above, not a
style choice) and the extra query is bounded by the number of matching
relations for one node, so it should be a non-issue in practice, but a
future perf pass on bulk `DETACH DELETE` should be aware the delete path per
relationship type is now one read + N writes instead of N writes.

## Follow-up (re-review, 2026-07-26)

The re-review approved the fix (no Important findings) and independently
validated the second bug documented above (the self-referencing predicate
during DETACH cleanup): its own sabotage test reverted only the
id-materialization step while keeping role-general resolution, and on a
3-relation/3-spill-row case the final relation-row `DELETE` matched zero
rows, leaving all three dangling. Confirms both the bug and the fix.

Two Minor coverage gaps were identified. Per instruction, only one is
closed here; the other (the two ternary-scribe tests not pinning the
`Many`/spill half of `delete_entity` — already covered by the separate
witness tests) is explicitly left alone.

**Gap closed: an all-`Many`, no-`start`/`end`-role relation type.**
`ternary_session` and `witnessed_session` both still have at least one
`One` role; nothing previously exercised a relation type where *every*
role is `Many` and none is named `start`/`end` (`GATHERING`: roles
`guest`/`witness`, both `Many`, over `fixture::two_many_roles_session`).
Added to `graph/frontend/tests/nary_relations.rs`:

- `deleting_an_all_many_role_relations_guest_is_refused` — plain `DELETE`
  of a guest still cited by a gathering must return
  `MutationError::NodeHasRelationships`, leaving `people`,
  `gatherings__guest`, and `gatherings__witness` untouched.
- `detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables`
  — `DETACH DELETE` of the same guest must remove the `gatherings` row
  *and* clean both `gatherings__guest` and `gatherings__witness`, not just
  the spill table the deleted node participated through.

**Role-order permutation.** Added `two_many_roles_session_reordered` to
`graph/frontend/tests/fixture.rs` — identical to `two_many_roles_session`
except `witness` is declared before `guest` in
`RelationshipSourceRegistration::roles`. Duplicated the two tests above
against it, deleting/detaching a `witness` instead of a `guest`
(`deleting_an_all_many_role_relations_witness_is_refused_with_roles_declared_in_reverse_order`,
`detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables_with_roles_declared_in_reverse_order`),
so both roles are exercised in both declared positions. All 4 new tests
pass; `cargo test -p turso_graph_frontend --test nary_relations` went from
51 to 55 passing, 0 failing.

**Sabotage verification.** Wrapped the DETACH branch's `Many`-role
spill-purge loop in `delete_entity` (`mutation.rs`) in
`if relationship.roles.iter().any(|role| role.cardinality != ir::RoleCardinality::Many) { ... }`
— i.e. skip spill cleanup entirely when a relation type has no `One` role.
`cargo test -p turso_graph_frontend --test nary_relations` then produced,
verbatim:

```
---- detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables stdout ----

thread 'detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables' panicked at graph/frontend/tests/nary_relations.rs:902:5:
assertion `left == right` failed: the guest's spill row must not dangle behind
  left: [[Numeric(Integer(1))]]
 right: [[Numeric(Integer(0))]]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables_with_roles_declared_in_reverse_order stdout ----

thread 'detach_deleting_an_all_many_role_relation_removes_it_and_both_spill_tables_with_roles_declared_in_reverse_order' panicked at graph/frontend/tests/nary_relations.rs:1031:5:
assertion `left == right` failed: the witness's spill row must not dangle behind
  left: [[Numeric(Integer(1))]]
 right: [[Numeric(Integer(0))]]

test result: FAILED. 53 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

Exactly the two new "both spill tables" tests went red — the other 53,
including the pre-existing single-`Many`-role witness DETACH test, stayed
green, confirming the sabotage is scoped to relation types with no `One`
role and that the new tests (not some unrelated assertion) are what catch
it. Reverted with `git checkout -- graph/frontend/src/mutation.rs`;
`git status` and `git diff` confirmed a clean revert (no residual diff);
re-ran the suite and got 55 passing, 0 failing again.

**Scope.** This follow-up is test-only: `git diff --stat` shows only
`graph/frontend/tests/fixture.rs` (+69) and
`graph/frontend/tests/nary_relations.rs` (+262) changed;
`graph/frontend/src/mutation.rs` is untouched in the committed state. Per
instruction, `mise run corpus` and `mise run cypherbench-sample` were
skipped. `cargo clippy --workspace --all-features --all-targets
-- --deny=warnings` currently fails, but only on pre-existing, unrelated
unused-import errors in `core/mvcc/persistent_storage/logical_log.rs` and
`core/vdbe/mod.rs` — files this change does not touch and that do not
appear in `git status`. Flagging as a pre-existing branch-state issue, not
a regression from this change, and out of scope for a test-only follow-up.

The second Minor gap (ternary-scribe tests not pinning the `Many`/spill
half) was left untouched, per instruction — it is already covered by the
separate witness tests.
