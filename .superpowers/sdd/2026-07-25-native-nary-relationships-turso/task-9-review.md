# Task 9 Review: Add roles to the create-relationship IR alongside `from`/`to`

Commit reviewed: `a976f2fd5` (range `37c4829d7..a976f2fd5`)

## 1. Spec compliance

Brief steps, in order:

- **Step 1 (failing tests in `mutation.rs`)**: ✅ present verbatim (one array-literal
  tweak for clippy, no behavior change), both tests read back as expected.
- **Step 2 (verify fail)**: not independently re-verifiable (field now exists), but
  plausible given Step 3.
- **Step 3 (`roles: Vec<RoleBinding>` field on `CreateRelationship`)**: ✅ present at
  `graph/ir/src/mutation.rs:50-52`, doc comment states the repeated-player invariant.
- **Step 4 (populate in binder from `catalog.relationship_roles`)**: ✅ done, but by a
  different and better path than the brief's literal text — via
  `relationship_role(graph, ty, "start"/"end")` (name lookup) rather than indexing a
  `Vec` returned by `relationship_roles`. This is a positive deviation: it can't
  silently regress to `roles[0]`/`roles[1]`, and it required closing the Task 8 gap
  (schemaless `relationship_roles` returned `Vec::new()` unconditionally) to work at
  all. The controller's ruling — close the gap generically via a physical-layout
  projection rather than a literal two-role synthesis — is exactly what shipped.
  Verified by sabotage: reverting the trait-default projection to `Vec::new()`
  reproduces exactly the 12 named failures the report describes
  (`MissingRelationshipRole { role: "start", ... }` at the same test names);
  reverting only `SchemaCatalog`'s override reproduces exactly 1 (its own direct
  unit test), confirming the fixtures that don't go through `SchemaCatalog` rely on
  the trait default, not on `SchemaCatalog`.
- **Step 5 (test pass)**: ✅ confirmed independently: `cargo test -p turso_graph_ir -p
  turso_graph_frontend` → 301 passed, 3 ignored, 0 failed, matching the report.
- **Step 6 (gate/commit)**: `cargo fmt --check` and `cargo clippy -p turso_graph_ir -p
  turso_graph_frontend -p turso_graph_testkit --all-features --all-targets
  --deny=warnings` both clean on the touched crates (2 unrelated clippy errors exist
  workspace-wide in `core/mvcc` and `core/vdbe`, pre-existing, untouched by this
  diff, out of scope). `mise run corpus` not re-run per controller instruction; its
  recorded result was independently verified by the controller already.

Global constraints:

- **"Binary is a layout, not a kind"**: ✅ no `if roles.len() == 2`, no `is_binary`,
  no hard-coded fast path in the general role machinery (`GraphCatalogSnapshot`
  trait default, `SchemaCatalog` override, `SemanticRole::from`). Verified by
  sabotage: replacing the generic projection with a positional
  `["start","end"][i]` name assignment turns the new
  `schemaless_relationship_roles_project_the_physical_role_layout` test red
  (`RoleId(1) != RoleId(2)`), so the genericity is actually tested, not just
  claimed. The remaining literal `"start"`/`"end"` strings in `bind_create_path`
  (both the pre-existing read-side check and the new lookups) are the two
  syntactic endpoints of Cypher's binary `()-[]->()`  pattern, resolved by name
  against whatever the catalog reports — not a position-based or arity-assuming
  shortcut, and explicitly documented as debt Task 11 removes.
- **Role identity is `RoleId`, never a vector index**: ✅ in the generic
  machinery. `roles[0]`/`roles[1]` occurrences are confined to test code asserting
  a specific fixture's known declared order, not resolution logic.
- **Physical/semantic layers share `RoleId`, no re-derivation**: ✅ confirmed by
  reading `impl From<RelationshipRoleLayout> for SemanticRole`
  (`graph/frontend/src/semantic.rs:147`): `role: role.role` — reused directly.
- **Repeated role player legal**: ✅ unaffected; `mutation.rs`'s existing test covers
  it, untouched by this diff.
- **Every behavior change needs a test that fails without it**: ❌ **not fully met** —
  see Critical finding below.

Net: brief mechanically satisfied, and the Task 8 gap closure follows the
controller's ruling faithfully and generically. One correctness-relevant test gap
found (below) keeps this from an unqualified pass.

**Verdict: ✅ (spec compliant)** — the brief's literal steps and the global
constraints as stated are satisfied. The gap found below is a test-quality issue
under "every behavior change needs a test," not a spec deviation, and is called out
under Task quality.

## 2. Task quality

### Findings

- **Critical — no test asserts binder-level `CreateRelationship.roles` pairing
  correctness.** Sabotage: in `bind_create_path`'s `ir::CreateRelationship`
  literal, (a) swapping which `RoleId` pairs with which endpoint
  (`start_role`↔`relationship_to`, `end_role`↔`relationship_from`) and (b)
  replacing the whole `roles: vec![...]` with `roles: Vec::new()` **both leave
  `cargo test -p turso_graph_ir -p turso_graph_frontend` at 301 passed, 0
  failed**. The existing binder test that exercises this exact code path
  (`binds_created_path_to_stable_sources_and_endpoints`) only asserts
  `.source`/`.from`/`.to`, never `.roles`; it would pass unchanged even if
  `.roles` were empty or wrong. The `mutation.rs` unit tests construct a
  `CreateRelationship` by hand (never call the binder) and the
  `schema_catalog.rs` test asserts the catalog's role *projection*, not the
  binder's *use* of it. So the specific new behavior this task was scoped to
  deliver — the binder populating `.roles` — has zero coverage that would go
  red if it silently regressed to empty, swapped, or positionally-assigned. Both
  sabotage edits were confirmed and reverted; working tree is restored (`git
  diff --stat` shows only the pre-existing, untouched `graph/test-results/*`
  changes).
- **Minor — Task 8 decision reversal is justified and its blast radius is
  contained, but undocumented as a formal decision.** Adding `graph:
  ir::GraphId` to `relationship_roles`/`relationship_role` reverses Task 8's
  explicit "no `graph` parameter" call. It's technically necessary — the
  generic default must call `relationship_source_for_type(graph, ty)`, which
  requires `graph` — and consistent with every sibling catalog method already
  taking `graph` first. Confirmed via grep that `GraphCatalogSnapshot` has no
  implementors outside `graph/`, and all 3 production call sites plus the 1
  trait-internal call were updated; nothing missed. This is correct, just not
  narrated in the brief/report as "reversing a documented prior decision"
  beyond the report's own context section — acceptable since the controller
  already flagged and accepted the tradeoff.
- **Verified, no defect — `graph/testkit/src/dynamic_catalog.rs` and all
  hand-rolled `Catalog`/`BinaryCatalog` test fixtures.** None of them override
  `relationship_roles`/`relationship_role` (grep confirms); all inherit the
  corrected trait default. `DynamicCatalog` also inherits
  `relationship_source_for_type`'s default (`self.relationship_source(graph)`),
  which is correct for it specifically because it is documented and designed
  to back exactly one relationship table regardless of type — not a silent
  wrong-default trap. `cargo build -p turso_graph_testkit` compiles clean.
- **Minor — YAGNI check: no scope creep.** The extra files touched
  (`schema_catalog.rs`, `semantic.rs`) beyond the brief's listed
  `mutation.rs`/`binder.rs` are the minimum needed to close the Task 8 gap the
  controller ordered closed generically; no speculative abstraction, no
  unrelated refactors. `cargo fmt` clean on touched crates.

### Recommendation

**Not approved as-is** — the Critical finding is a real, demonstrated hole in
regression coverage for the exact behavior this task exists to add (binder-level
role-binding population), squarely inside "every behavior change needs a test that
fails without it." Recommend a follow-up test added to `binder.rs`'s test module
(alongside `binds_created_path_to_stable_sources_and_endpoints`, or as a sibling)
that binds a CREATE against the existing `reversed`-style fixture (or a
`Catalog` with roles registered out of `start`-then-`end` order) and asserts
`relationship.roles` contains the correct `RoleBinding { role, value }` pairs by
identity, not just that bind succeeds. This is a small, targeted addition, not a
reason to redo the task's substantive fix, which is otherwise sound and generic.

## Sabotage log (all reverted; working tree confirmed clean afterward)

1. `SchemaCatalog::relationship_roles` schemaless branch → `Vec::new()`:
   1 test red (`schemaless_relationship_roles_project_the_physical_role_layout`).
2. `GraphCatalogSnapshot::relationship_roles` trait default → `Vec::new()`:
   12 tests red, same names/panic as report.
3. Trait default's projection → positional `["start","end"][i]` naming instead
   of `role.name`: 1 test red (`RoleId` mismatch), confirming genericity is tested.
4. `bind_create_path`'s `roles: vec![...]` → role/value pairing swapped: 0 tests
   red.
5. `bind_create_path`'s `roles: vec![...]` → `Vec::new()`: 0 tests red.

(4) and (5) are the basis for the Critical finding.
