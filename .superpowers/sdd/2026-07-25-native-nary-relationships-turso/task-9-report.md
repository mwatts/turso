# Task 9 Report: Add roles to the create-relationship IR alongside `from`/`to`

Status: **DONE**

## Round 2 (review fix)

Review came back spec-compliant but quality-not-approved: **no test asserted
binder-level `CreateRelationship.roles` correctness**. The reviewer proved
this by sabotage: swapping the role/value pairing in `bind_create_path`, and
emptying `roles` entirely, both left `cargo test -p turso_graph_ir -p
turso_graph_frontend` at 301 passed / 0 failed. Root cause:
`binds_created_path_to_stable_sources_and_endpoints` never asserted
`.roles`; the `mutation.rs` tests build a sample struct rather than
exercising the binder; the `schema_catalog.rs` test covers the catalog
projection, not the binder's use of it. Role/value mispairing is the
recurring defect class on this plan (Tasks 4, 5, 6, 7), and Tasks 10/11 are
about to make `roles` authoritative and delete `from`/`to`, so this gap
mattered.

Added `binder::tests::binds_created_path_role_bindings_pair_the_physical_role_with_its_resolved_value`
(`graph/frontend/src/binder.rs`, alongside
`binds_created_path_to_stable_sources_and_endpoints`): binds the same real
CREATE statement and asserts `relationship.roles` equals the exact ordered
`vec![RoleBinding { role: RoleId(1), value: first.binding.id() },
RoleBinding { role: RoleId(2), value: second.binding.id() }]` — the full
pairing, not a length or set check.

### Red/green proof (both sabotages reproduced and reverted)

1. Swapped `value: relationship_from`/`value: relationship_to` in the
   `roles: vec![...]` literal at the `ir::CreateRelationship` construction
   site. `cargo test -p turso_graph_ir -p turso_graph_frontend`:
   ```
   thread 'binder::tests::binds_created_path_role_bindings_pair_the_physical_role_with_its_resolved_value' panicked at graph/frontend/src/binder.rs:7265:9:
   assertion `left == right` failed
     left: [RoleBinding { role: RoleId(1), value: BindingId(2) }, RoleBinding { role: RoleId(2), value: BindingId(1) }]
    right: [RoleBinding { role: RoleId(1), value: BindingId(1) }, RoleBinding { role: RoleId(2), value: BindingId(2) }]
   test result: FAILED. 153 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
   ```
   Restored; confirmed green.
2. Replaced `roles: vec![...]` with `roles: Vec::new()`. Same command:
   ```
   thread 'binder::tests::binds_created_path_role_bindings_pair_the_physical_role_with_its_resolved_value' panicked at graph/frontend/src/binder.rs:7256:9:
   assertion `left == right` failed
     left: []
    right: [RoleBinding { role: RoleId(1), value: BindingId(1) }, RoleBinding { role: RoleId(2), value: BindingId(2) }]
   test result: FAILED. 153 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
   ```
   Restored; confirmed green.

Both sabotages failed exactly the new test and nothing else (152 unrelated
passes, +1 new test = 153, in both runs).

### Gates (round 2)

- `cargo fmt` — clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — 0 errors (10 residual warnings are the same pre-existing `ar` C-toolchain
  build-script noise from `limbo_sqlite_test_ext`, unrelated to Rust code).
- `cargo test -p turso_graph_ir -p turso_graph_frontend` — **302 passed, 3
  ignored, 0 failed** (up from 301; the one new test).
- `mise run corpus` — **skipped**. This round's diff is test-module-only
  (`graph/frontend/src/binder.rs`, `+41` lines, confirmed via `git diff
  a976f2fd5 -- <the 4 task files>` before amending); no non-test code
  changed since the already-verified corpus run, so it cannot move the
  recorded 8926/10242 result.

Minor note recorded by the coordinator's own ledger, not fixed here: the
`graph` parameter added to `relationship_roles`/`relationship_role` reverses
Task 8's stated "no `graph` parameter" decision. Judged technically required
by the generic trait-default projection, blast radius fully contained.
Stands as-is.

## Summary

Expand half of the mutation migration. `CreateRelationship` now carries
`roles: Vec<RoleBinding>` alongside `from`/`to`, populated by the binder from
the same bindings it already resolves, in both semantic and schemaless mode.

This also closes the gap left open by Task 8: Task 8's brief required
schemaless mode to synthesize role registrations from the physical
relationship layout, but the implementation shipped without that, and its
review accepted the omission because the only consumer at the time
(`bind_create_path`'s role-target-type check) tolerated an empty role list
via a `continue`-on-`None` branch. Task 9's Step 4 is a second consumer that
needs a real `RoleId`, not a tolerated `None`, so the gap became a hard
failure: `relationship_role(ty, "start"/"end")` returned `None` for every
schemaless relationship create/merge, because `SchemaCatalog::relationship_roles`
returns `Vec::new()` whenever `self.semantic` is `None` (schema_catalog.rs,
now around line 634), which is the state of every schemaless graph in
production (`session.rs:220`) and every test fixture. A prior attempt at this
task correctly diagnosed this and stopped rather than special-case it
unilaterally (12 previously-passing tests went red under the brief's Step 4
as originally written).

## The fix, and why it stays generic

The ruling was: project the relation's *actual physical role list* into the
catalog's role view, rather than hard-coding "two roles named `start` and
`end`". Hard-coding binary arity into the catalog is exactly what this whole
plan removes — "binary is a layout, not a kind" is the plan's central
invariant, and a role's identity is its `RoleId`, never a vector index or a
literal name synthesized out of nowhere.

Concretely:

1. **`graph/frontend/src/semantic.rs`**: added
   `impl From<lowering::RelationshipRoleLayout> for SemanticRole`. It reuses
   the physical `RoleId` directly (never re-derives one), sets `targets:
   Vec::new()` (schemaless imposes no target-type constraint — the existing
   "unconstrained" convention), and `optional: false` (`RelationshipRoleLayout`
   carries no such flag; a physical registration requires every declared
   role to be filled).

2. **`graph/frontend/src/binder.rs`** (`GraphCatalogSnapshot` trait):
   - `relationship_roles`/`relationship_role` gained a `graph: ir::GraphId`
     parameter (previously only `relationship_roles` was missing it, while
     every sibling catalog-lookup method — `relationship_type`,
     `relationship_source_for_type`, `property`, ... — already takes one;
     this was an oversight, not a signature I was asked to preserve).
   - The **trait default** for `relationship_roles` no longer returns
     `Vec::new()` unconditionally. It now projects the physical layout
     generically for *any* catalog: `relationship_source_for_type(graph, ty)`
     → `relationship_source_roles(source)` → map each
     `RelationshipRoleLayout` through the new `From` impl. Every
     `GraphCatalogSnapshot` implementer already had to supply
     `relationship_source_roles` for the read-side hop lookup (Task 5/6), so
     this default now works for all of them without touching their bodies:
     the three hand-rolled test `Catalog` fixtures in `binder.rs`,
     `mutation.rs`, `session.rs` (the ones that hit the 12 failures),
     `graph/testkit/src/dynamic_catalog.rs`'s `DynamicCatalog` (delegates
     `relationship_source`/`relationship_source_roles` to its inner
     `SchemaCatalog` and does not override `relationship_roles` itself, so
     it now inherits the corrected default), and the fixture Catalogs under
     `graph/frontend/tests/*.rs`.
   - Updated the 3 call sites in `bind_create_path` to pass `self.graph`.

3. **`graph/frontend/src/schema_catalog.rs`** (`SchemaCatalog`'s override):
   kept the semantic branch exactly as it was (plus a `graph != self.graph.id`
   guard, matching every sibling method in this impl block, which the
   pre-existing code lacked only because the method didn't take `graph` yet).
   For the `self.semantic.is_none()` branch, added the identical physical
   projection the trait default now applies (`relationship_source_for_type`
   + `relationship_source_roles`, mapped through `SemanticRole::from`) —
   duplicated locally rather than trying to call the trait default from
   inside an override, which Rust has no syntax for.

4. **`graph/ir/src/mutation.rs`** and the `bind_create_path` construction
   site in `binder.rs`: unchanged from the prior attempt, which was correct
   as far as it went (`roles: Vec<RoleBinding>` field + doc comment; `roles:
   vec![RoleBinding { role: start_role, value: ... }, ...]` at the
   `ir::CreateRelationship` literal, with `start_role`/`end_role` resolved
   by name via `relationship_role`, never by position).

No production behavior changes for schema'd or schemaless relationships:
schemaless mode's physical layout literally *is* `[start, end]` today, so
`relationship_roles` reports the same two roles it always implicitly
represented in `from`/`to` — just now also exposed as typed `RoleBinding`s.
The projection is written generically (no `if roles.len() == 2`, no
`is_binary`, no hard-coded role names) so it keeps working unchanged once
Task 14 lands many-valued roles with a different physical role count.

## Tests

- `graph/ir/src/mutation.rs` (brief's Step 1, verbatim, from the prior
  attempt): `a_created_relationship_lists_its_role_bindings_in_declaration_order`,
  `a_role_binding_list_permits_the_same_player_twice`. The second was changed
  from `vec![...]` to a `[...]` array literal to satisfy
  `clippy::useless_vec` (it's only ever `.iter()`'d) — a lint-only change,
  no behavior difference.
- `graph/frontend/src/schema_catalog.rs`: added
  `schemaless_relationship_roles_project_the_physical_role_layout`, built on
  the existing `reversed_binary_relationship_catalog()` fixture (roles
  registered `[end, start]`, deliberately not start-then-end) so the
  assertion can't pass by positional coincidence. It asserts the projected
  `SemanticRole`s for a schemaless relationship type carry the *same*
  `RoleId`s as the physical `RelationshipTableLayout`, have empty `targets`,
  and are not `optional`.

### Red/green proof (sabotage, not reasoning)

- Reverted `SchemaCatalog::relationship_roles`'s schemaless branch to
  `Vec::new()`: `schemaless_relationship_roles_project_the_physical_role_layout`
  went red (`FAILED. 0 passed; 1 failed`). Restored; back to green.
- Reverted the `GraphCatalogSnapshot::relationship_roles` trait default to
  `Vec::new()`: `cargo test -p turso_graph_ir -p turso_graph_frontend`
  reproduced the exact same 12 failures the prior attempt found (same test
  names, same `BindError::MissingRelationshipRole` panic at
  `binder.rs:binds_created_path_to_stable_sources_and_endpoints`). Restored;
  back to green.

Both proofs used direct edits, not `git stash`/checkout.

## Gates

- `cargo fmt` — clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — 0 errors. (10 residual "warnings" printed are `ar: illegal option -- D`
  build-script noise from `limbo_sqlite_test_ext`'s C toolchain invocation,
  unrelated to any Rust lint and pre-existing; clippy itself reported 0
  errors, 0 warnings on our crates.)
- `cargo test -p turso_graph_ir -p turso_graph_frontend` — **301 passed, 3
  ignored, 0 failed** (up from 300 passed before this task's new test).
- `mise run corpus` (release build, run in full) — per-suite results
  against the plan's stated stable baselines:

  | suite | passed | not-passed | baseline |
  |---|---|---|---|
  | age-deep | 3042 | 553 | 3042/553 — match |
  | cqlite-deep | 113 | 11 | 113/11 — match |
  | grafeo-deep | 277 | 95 | 277/95 — match |
  | sparrowdb-deep | 2164 | 61 | 2164/61 — match |
  | tck-deep | 3330 | 596 | 3330–3332 passed — match (the one suite documented to vary) |
  | **total** | **8926** | **1263** (+53 unsupported) | **8926/10242 — exact match** |

  Every suite sits exactly at (or, for `tck-deep`, within the documented
  variance of) baseline. No regression anywhere. The run's own exit status
  is `clean=false` / nonzero — that reflects the corpus's own convention of
  treating any non-`Passed` outcome (including the many long-standing
  baseline `failed`/`unsupported` rows) as "not clean," not a regression
  signal; the per-suite counts are the actual regression check, and they
  match exactly.
- `mise run cypherbench-sample` — not run. It is not in this task's brief
  gate list (only `mise run corpus` is), and I did not fabricate a result
  for it.

`graph/test-results/runs.jsonl` and `REPORT.md` were updated by the corpus
run as a side effect; per instructions these are left uncommitted for
separate commit. `graph/test-results/history.jsonl` is gitignored and was
not touched in git's view.

## Files changed

- `graph/ir/src/mutation.rs` — `roles: Vec<RoleBinding>` field (prior
  attempt's work, kept) + its two unit tests (one array-literal fix for
  clippy).
- `graph/frontend/src/binder.rs` — `roles` population in `bind_create_path`
  (prior attempt's work, kept); `GraphCatalogSnapshot::relationship_roles`/
  `relationship_role` gained a `graph` parameter and the trait default now
  projects the physical role layout instead of returning nothing; the 3
  `relationship_role(...)` call sites updated to pass `self.graph`.
- `graph/frontend/src/schema_catalog.rs` — `SchemaCatalog::relationship_roles`
  updated to match the new trait signature, with the schemaless branch now
  projecting the physical layout; added
  `schemaless_relationship_roles_project_the_physical_role_layout`.
- `graph/frontend/src/semantic.rs` — added
  `impl From<RelationshipRoleLayout> for SemanticRole`.

## Commit

`git commit -S` on `graph/ir/src/mutation.rs`, `graph/frontend/src/binder.rs`,
`graph/frontend/src/schema_catalog.rs`, `graph/frontend/src/semantic.rs` only
(not `graph/test-results/*`).
