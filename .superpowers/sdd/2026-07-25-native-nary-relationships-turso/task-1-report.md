# Task 1 Report: `RoleId` identity and the role model

## Status: DONE

## Files changed

### `graph/ir/src/identity.rs`
- Added `assert!(RoleId::new(0).is_err());` to the existing `all_public_identities_reject_zero` test (extended, not duplicated).
- Added `define_u32_id!(RoleId, "role");` immediately after the `PropertyId` line, before `BindingId`, matching the brief's placement instruction and the existing macro-invocation style used by `LabelId` / `RelationshipTypeId` / `PropertyId` / `BindingId`.

### `graph/ir/src/role.rs` (new)
- Created verbatim per the brief:
  - `RoleTarget` enum: `Node(LabelId) | Relation(RelationshipTypeId)` — deliberately keeps the two identity spaces distinct rather than flattening to one integer.
  - `RoleCardinality` enum: `One | Many`.
  - `RoleDef` struct: `{ role: RoleId, name: String, target_types: Vec<RoleTarget>, optional: bool, cardinality: RoleCardinality }`, with `accepts_any_target()` returning `target_types.is_empty()`.
  - `RoleBinding` struct: `{ role: RoleId, value: BindingId }`.
  - `mod tests` with the three tests from the brief, verbatim.

### `graph/ir/src/lib.rs`
- Added `mod role;` in alphabetical position among the existing `mod` declarations (`error, expression, identity, mutation, plan, role, scope, semantics`).
- Added `RoleId` to the `identity` re-export list.
- Added `pub use role::{RoleBinding, RoleCardinality, RoleDef, RoleTarget};` in alphabetical position among the existing re-export lines (after `plan`, before `scope`).

No other files were touched (side-effect files from running `mise run corpus` — `graph/test-results/REPORT.md` and `graph/test-results/runs.jsonl` — were left unstaged and are not part of the commit, since the brief's `git add` step names only the three source files).

## Test-driven steps, exact commands and output

**Step 2 — verify the identity test fails to compile:**
```
cargo test -p turso_graph_ir all_public_identities_reject_zero
```
Output (before Step 3):
```
error[E0433]: failed to resolve: use of undeclared type `RoleId`
  --> graph/ir/src/identity.rs:82:17
   |
82 |         assert!(RoleId::new(0).is_err());
   |                 ^^^^^^
```
Matches the brief's expected failure exactly.

**Step 4 — verify it passes after defining `RoleId`:**
```
cargo test -p turso_graph_ir all_public_identities_reject_zero
```
Output: `cargo test: 1 passed, 13 filtered out (2 suites, 0.00s)`

**Step 6 — verify the role module fails to compile before the type definitions exist:**
```
cargo test -p turso_graph_ir --lib role::
```
Output (before Step 7):
```
error[E0433]: failed to resolve: use of undeclared type `RoleTarget`
  --> graph/ir/src/role.rs:11:20
...
error[E0433]: failed to resolve: use of undeclared type `RoleCardinality`
```
Matches the brief's expected failure (cannot find `RoleTarget`).

**Step 8 — verify the full crate passes:**
```
cargo test -p turso_graph_ir
```
Output: `cargo test: 17 passed (3 suites, 0.00s)`
(14 pre-existing tests + 3 new `role::tests` = 17.)

## Gate results

- **`cargo fmt`**: ran clean; `git diff --stat` after fmt showed no additional changes beyond what was authored by hand (identity.rs +2, lib.rs +4/-1), confirming the hand-written code was already correctly formatted.
- **`cargo clippy --workspace --all-features --all-targets -- --deny=warnings`**: exit code 0. The reported "10 warnings" in the summary line are unrelated build-script noise from `limbo_sqlite_test_ext`'s use of `ar -D` (a toolchain/`ar` option incompatibility, pre-existing, nothing to do with this change) — verified by capturing full output; no clippy lint warnings were present.
- **`cargo test -p turso_graph_ir`**: 17 passed, 0 failed.
- **`mise run corpus`** (release build, as required by this task): completed with `run 20260726T003255.269079Z-2b3e9362f6a4-corpus-deep: 10242 records, clean=false` and non-zero exit (`[corpus] ERROR task failed`). Compared against the corpus run recorded at the prior commit (`e068dc04c359`, before this task's changes): that run recorded 1263 failed / 8926 passed / 53 unsupported; this run recorded 1262 failed / 8927 passed / 53 unsupported — statistically identical (one fewer failure, plausibly a flaky test), confirming the corpus's non-clean baseline is pre-existing and this purely-additive, unconsumed `turso_graph_ir` change did not alter corpus behavior.

## Commit

```
git add graph/ir/src/identity.rs graph/ir/src/role.rs graph/ir/src/lib.rs
git commit -S -m "graph/ir: add RoleId and the role definition model

Roles are the identity a native n-ary relation is built from. RoleTarget
keeps node labels and relationship types in distinct identity spaces so a
role that accepts a node label cannot silently accept the relationship type
with the same numeric value.

Tests: turso_graph_ir unit tests."
```

Commit SHA: `0678787100afbc4d26e0d45942fe85503820fe25`
Signature verified: `Good "git" signature for mwatts@users.noreply.github.com with ED25519 key SHA256:aHm6DQJW80HbWrpvSTASP8jYC3eBUGqXvFRpDh13MpY`

## Surprises / notes

- The `mise run corpus` task reports a non-zero/"ERROR" exit whenever the corpus isn't perfectly clean, which it already wasn't before this task (1263 pre-existing failures at HEAD~3). This is expected given the corpus's known-incomplete Cypher feature coverage (fulltext/hybrid search, shortestPath, degree functions, etc. — all unrelated to roles) and is not a regression introduced by this task. I compared failure counts against the immediately-prior recorded run to establish this rather than assuming it.
- No other surprises: the brief's code blocks were used verbatim, and both intentional "expected to fail" checkpoints (Steps 2 and 6) failed for exactly the stated reason before the corresponding definitions were added.
