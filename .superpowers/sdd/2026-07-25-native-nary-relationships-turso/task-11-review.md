# Task 11 Review

## Verdict 1: Spec compliance — ✅

Step-by-step against `task-11-brief.md`:

- **Step 1 (failing test):** `a_create_relation_names_only_roles` added in
  `graph/ir/src/mutation.rs`, built on a `sample_create_relation()` fixture
  that produces a genuine three-role sample (`RoleId(1..3)` bound to distinct
  `BindingId`s). Matches the brief's Step 1 intent.
- **Step 2 (verify fail):** Reported as compiler-verified pre-edit
  (`cannot find type CreateRelation`); plausible and consistent with the
  nature of the change (this is not independently re-verifiable after the
  fact without reverting the whole commit, which is out of scope).
- **Step 3 (delete and rename):** Verified directly.
  - `CreateRelationship`→`CreateRelation`, `MergeRelationship`→`MergeRelation`
    renamed throughout `graph/ir/src/mutation.rs`, `graph/ir/src/lib.rs`,
    `graph/frontend/src/{binder,mutation}.rs`. `rg -n
    "CreateRelationship|MergeRelationship" --type rust` across the repo:
    zero hits outside `.superpowers/sdd` review artifacts and pre-existing,
    out-of-scope planning/architecture docs (`docs/superpowers/**`,
    `architecture_graph.json/html`), none of which were in this task's file
    list.
  - `from`, `to`, `direction` fields and `default_direction()` deleted from
    `CreateRelation`; `rg -n "default_direction"` across `graph/`: zero hits.
    The remaining `.direction` hits in `binder.rs` (lines ~1530, 1593, 2294,
    2348, 2789, 6365) are all `cypher::Direction` on the AST relationship
    pattern (parser arrow spelling), untouched and correctly out of scope
    per Task 7's design — confirmed by reading each site.
  - `ir::Direction` (in `graph/ir/src/scope.rs`) is untouched, still
    re-exported from `lib.rs`, and still consumed by
    `graph/runtime/src/{csr,shortest,traversal}.rs` — confirmed live usage,
    not dead code, so its survival is correct (Task 17 owns its removal).
  - No `roles.len() == 2` / `is_binary` shortcut anywhere in the touched
    files — `rg` came back empty.
- **Step 4 (tests pass):** Reproduced independently:
  `cargo test -p turso_graph_ir -p turso_graph_frontend --lib` → 156 + 18
  passed, 0 failed. `cargo test -p turso_graph_cypher --lib` → 17 passed
  (crate exists, has real tests, none reference the renamed types — it's
  upstream of this change, correctly untouched).
- **Step 5 (gate):** `cargo build` for all four crates reproduced clean
  (only pre-existing, unrelated `core/mvcc` warnings). Corpus/cypherbench
  results were not re-run per instructions; the report's numbers were
  cross-checked against `graph/test-results/{runs.jsonl,benchmarks.jsonl}`
  contents already present in the tree and match the reviewer's
  independently-verified baseline stated in the task prompt.

Context items independently confirmed:
1. `ir::Direction` correctly not deleted; only the create-struct field and
   accessor were in scope. Confirmed above.
2. The adapted `binds_created_path_to_stable_sources_and_endpoints` test
   replaces `.from`/`.to` assertions with an exact `roles` vector assertion.
   Sabotage-tested: swapped the two `(role, value)` pairs in this test —
   `cargo test` failed with the expected mismatch, then restored via a
   targeted edit (verified `git status`/`git diff --stat` shows only the
   pre-existing, intentionally-uncommitted `graph/test-results/*` changes
   afterward). The assertion has real teeth, not a rubber-stamp.
3. Step 1's test was strengthened exactly as instructed: asserts both
   `roles.len() == 3` and the exact `Vec<RoleBinding>` contents, over a
   fixture with three distinct `RoleId`/`BindingId` pairs.
4. Crate name confirmed: `graph/cypher/Cargo.toml` declares
   `name = "turso_graph_cypher"`; `turso_cypher` does not exist as a package
   anywhere in the workspace. The gate the implementer ran
   (`-p turso_graph_cypher`) is the crate that actually needed covering, and
   its 17 tests ran and passed.

## Verdict 2: Task quality — Approved

Correctness: rename/deletion is exactly compiler-scoped, nothing untouched
that should have moved, nothing moved that should have stayed (`ir::Direction`
distinction handled correctly, the trickiest part of this task). Role
resolution remains name/`RoleId`-based, never positional, both before and
after this diff. `MergeRelation` correctly forwards to `CreateRelation` with
no re-introduction of endpoint fields.

Test quality: the Step-1 IR test is a genuine (not proxy) assertion of role
identity, matching the plan's standing requirement. The `binder.rs` adaptation
preserves rather than deletes coverage, as instructed, and was independently
verified via sabotage to still catch a role/value swap.

YAGNI / one finding: the adapted `binds_created_path_to_stable_sources_and_
endpoints` now asserts the *exact same* `roles` vector, over the *same*
fixture query, as the immediately-following sibling test
`binds_created_path_role_bindings_pair_the_physical_role_with_its_resolved_
value`. This is deliberate per the explicit "adapt, not delete" instruction
and the report is honest about it ("made consistent with the sibling...
Coverage... did not shrink") — the right call given the instruction, but it
is now byte-for-byte duplicated assertion logic between two tests. Left as a
Minor note rather than a blocker, since the alternative (deleting the
duplicate portion) risks exactly the coverage-shrinkage the instruction was
guarding against, and the redundancy costs nothing but a few lines.

## Findings

- Minor: `binds_created_path_role_bindings_pair_the_physical_role_with_its_
  resolved_value`'s doc comment still describes
  `binds_created_path_to_stable_sources_and_endpoints` as "which only checks
  `from`/`to`" — that's now stale, since the adapted test checks the identical
  `roles` assertion as the sibling. Cosmetic; does not affect behavior or
  coverage, just a doc-comment accuracy nit that predates this diff's
  adaptation and wasn't updated.
- Minor: the two tests noted above now carry duplicated exact-role-pair
  assertion logic. Correct outcome of the "adapt, don't delete" instruction;
  flagged for awareness, not a blocker.

No Critical or Important findings. No "⚠️ Cannot verify from diff" items —
everything material was checked against the live tree (build, tests,
sabotage) rather than taken on the report's word alone.
