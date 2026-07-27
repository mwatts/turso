# Task 11 Report: Delete `from`/`to`/`direction` and rename to `CreateRelation`

BASE: `2329d3fd1`. Commit: `8f69c19f5ece9cbedd99fbf4506524246fcb972d`.

## Summary

Renamed `ir::CreateRelationship` -> `ir::CreateRelation` and `ir::MergeRelationship`
-> `ir::MergeRelation` (and the matching `Mutation::CreateRelation(ship)` /
`Mutation::MergeRelation(ship)` variants), deleting the `from: BindingId`,
`to: BindingId`, and `direction: Direction` fields from the create struct.
`roles: Vec<RoleBinding>` is now the only statement of who participates in a
created relation.

## Files changed

- `graph/ir/src/mutation.rs` -- struct/enum renames, field deletion, deleted
  `CreateRelationship::default_direction()` and its doc comment (its own
  comment predicted this: "Task 11 deletes `direction` from this struct...
  and this accessor goes with it"), dropped the now-unused `Direction` import
  from the `use crate::{...}` list. Renamed the test fixture
  `sample_create_relationship()` -> `sample_create_relation()`, extended it
  to a three-role fixture, updated
  `a_created_relationship_lists_its_role_bindings_in_declaration_order` ->
  `a_created_relation_lists_its_role_bindings_in_declaration_order` to check
  all three exact `(role, value)` pairs, and added the brief's Step-1 test
  `a_create_relation_names_only_roles`, strengthened per instructions to
  assert both `roles.len() == 3` AND the exact `Vec<RoleBinding>` contents
  (not just the count), so a bug that fills the right number of roles with
  the wrong players still gets caught.
- `graph/ir/src/lib.rs` -- updated the `pub use mutation::{...}` re-export list.
- `graph/frontend/src/binder.rs` -- `bind_create_path` now builds
  `ir::CreateRelation { binding, source, relationship_types, properties, roles }`
  (dropped `from`/`to`/`direction` initializers, kept the `relationship_from`/
  `relationship_to` role-binding resolution that already existed); updated
  `Mutation::MergeRelation(ship)` match arm in `attach_merge_actions`. Adapted
  (not deleted) the test `binds_created_path_to_stable_sources_and_endpoints`:
  replaced its `relationship.from`/`relationship.to` assertions (fields that no
  longer exist) with an assertion of the exact `relationship.roles` `(role,
  value)` pairs, made consistent with the sibling test
  `binds_created_path_role_bindings_pair_the_physical_role_with_its_resolved_value`
  immediately below it, which already asserted the same pairing and remains
  unchanged. Coverage of the endpoint-resolution behavior did not shrink.
- `graph/frontend/src/mutation.rs` -- updated `execute_operation`'s match arms
  (`ir::Mutation::CreateRelation`/`MergeRelation`), `insert_relationship`'s
  parameter type (`&ir::CreateRelation`), and the `mod tests` helper
  `ternary_create()` (built `ir::CreateRelationship` with a throwaway
  `from`/`to`/`direction`; now builds `ir::CreateRelation` without them). This
  file's `mod tests` already called the `pub(crate)` `insert_relationship`
  directly -- no public write-path test helper exists here to update.
- `graph/frontend/src/schema_catalog.rs:1520` -- renamed `CreateRelationship.roles`
  to `CreateRelation.roles` in a doc comment (prose only, not code).

## Brief corrections applied (as instructed)

1. **No `Session::execute_create_relation` exists.** Verified via
   `rg -n execute_create_relation` across `graph/` -- zero matches, confirmed
   before editing. Updated `graph/frontend/src/mutation.rs`'s own `mod tests`
   (`insert_relationship` callers) instead, per the correction.
2. **Deleted `CreateRelationship::default_direction()`** and its doc comment
   from `graph/ir/src/mutation.rs`, not mentioned in the brief's file list.
3. **Renamed the doc-comment reference in `schema_catalog.rs:1520`**, not in
   the brief's file list.

## Additional wrong reference found in the brief (not one of the three
flagged gaps)

The brief's Step 5 gate command uses `-p turso_cypher`. The actual package
name (per `graph/cypher/Cargo.toml`) is `turso_graph_cypher` --
`cargo test -p turso_cypher` fails with `package ID specification
'turso_cypher' did not match any packages`. Used the correct package name to
run the gate; did not change the crate name itself.

## Critical distinction verified

`graph/frontend/src/binder.rs` still contains 19 references to
`cypher::Direction` (the parser AST arrow spelling used at lines ~1530, 1593,
2297/2792-area direction matching, 6368, and elsewhere) -- all untouched.
Only the `direction: Direction` **field on the IR create struct**, its two
initializers, and `default_direction()` were deleted. `ir::Direction` itself
(defined in `graph/ir/src/scope.rs`, re-exported from `lib.rs`) is untouched
and remains in active use by `graph/runtime/src/{csr,shortest,traversal}.rs`
and `graph/frontend/src/graph_expand.rs` for traversal direction -- confirmed
by `rg` before and after the edit. The only import removed was the now-dead
`Direction` import from `graph/ir/src/mutation.rs`'s `use crate::{...}` list.

## Test-driven verification

- Step 1/2 (compiler-enforced): before the rename, `cargo test -p turso_graph_ir
  --lib mutation::` referencing `CreateRelation`/`sample_create_relation` failed
  to compile with `cannot find type CreateRelation`/`cannot find function
  sample_create_relation`, confirmed by running the test against the
  brief's pre-rename code. After Step 3's edits, all mutation:: tests compile
  and pass.
- Most of this task is a compiler-enforced rename/deletion: every
  `CreateRelationship`/`MergeRelationship`/`.from`/`.to`/`.direction`/
  `default_direction()` reference across `graph/ir` and `graph/frontend` had
  to be updated for the workspace to compile at all -- the compiler is the
  test for that portion, stated honestly rather than inventing ceremonial
  tests for a rename.
- The behavior-bearing test additions (three-role fixture, exact
  `Vec<RoleBinding>` assertions in both `graph/ir/src/mutation.rs` and the
  adapted `graph/frontend/src/binder.rs` test) assert exact `(role, value)`
  identity, not just counts, per the standing requirement that role identity
  is the `RoleId` and a length check alone is a proxy that survives the
  plan's recurring role/value-mispairing defect class.

## Gate results

- `cargo fmt`: clean (reformatted the 5 touched files to match rustfmt).
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`:
  0 errors, exit 0. The only warnings in the raw log are pre-existing `ar`
  toolchain warnings from `limbo_sqlite_test_ext`'s build script (illegal
  `-D` option on macOS `ar`), unrelated to this change and not clippy lints.
- `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime
  -p turso_graph_cypher` (corrected package name; see above): 349 passed, 5
  ignored, 0 failed, across 22 suites.
- `mise run corpus` (release build): binary exits non-zero by design whenever
  `clean != true` (i.e., whenever any query fails/is unsupported), which is
  the case for every corpus run at this baseline (never 100% clean) -- this
  is expected, not a regression signal. Verified the real numbers against
  `graph/test-results/runs.jsonl`'s freshly-appended last line
  (`run_id: 20260726T094846.915911Z-2329d3fd1aa7-corpus-deep`, timestamp
  matches this run): **total 8926/10242**; age-deep 3042/553, cqlite-deep
  113/11, grafeo-deep 277/95, sparrowdb-deep 2164/61 all exactly match the
  stated baseline; tck-deep 3330/596, within the documented 3330-3332
  legitimate-variance range. No suite outside tck-deep moved.
- `mise run cypherbench-sample` (release build): exit 0. Per-domain
  matched/mismatched, cross-checked against
  `graph/test-results/benchmarks.jsonl`'s newly appended entry
  (`recorded_at: 2026-07-26T09:49:59`): company 13/25, fictional_character
  14/25, flight_accident 24/25, geography 11/25, movie 6/25, nba 25/25,
  politics 15/25 -- identical to the three prior benchmark entries in the
  same file (06:45, 06:48, 07:21), confirming no behavior change. Aggregate:
  108 matched / 175 queries, 0 errored.

## Commit

`8f69c19f5ece9cbedd99fbf4506524246fcb972d`, signed (`git commit -S`), staged
explicit paths only (`graph/frontend/src/{binder,mutation,schema_catalog}.rs`,
`graph/ir/src/{lib,mutation}.rs`). `graph/test-results/{REPORT.md,
benchmarks.jsonl, runs.jsonl}` were touched by the gate runs and left
uncommitted per instructions, to be committed separately.

## Concerns

None. All standing requirements held: two-role all-required all-`One`
relations still land on the pre-existing physical shape (no `roles.len()`
special-casing was touched or needed -- `insert_relationship` was already
role-generic from Task 10); role identity resolved by `RoleId` throughout,
never by vector index; repeated role players remain legal and untested
assumption was not introduced.
