# Task 8 Review: Semantic roles

## Spec Compliance

- ✅ `SemanticRole { role: ir::RoleId, name, targets: Vec<ir::RoleTarget>, optional, cardinality }` matches the brief exactly (semantic.rs:462-480).
- ✅ `SemanticRelationshipType { name, source, roles: Vec<SemanticRoleRegistration>, properties }` replaces `start`/`end`; `EndpointConstraint` removed from `lib.rs` exports (lib.rs:204-211).
- ✅ `SEMANTIC_ROLE_TABLE` DDL replaces `SEMANTIC_ENDPOINTS_TABLE`, one row per (role, target), `target_kind CHECK(... IN ('node','relation'))` discriminator present (semantic.rs:1349-1362... schema block around the `CREATE TABLE` for the role table). Deviation from the brief's literal snippet: adds `graph_id` and keys by `role_id` instead of `ordinal` — correctly justified: every sibling semantic table already carries `graph_id` (multi-graph catalog), and `role_id` is the physical `RoleId` reused directly rather than re-derived, which is *more* correct than the brief's own sketch (see RoleId-identity verification below). Not a defect.
- ✅ `GraphCatalogSnapshot::relationship_roles(ty) -> Vec<SemanticRole>` and `relationship_role(ty, name) -> Option<SemanticRole>` added to the trait with a sensible default (binder.rs:32-54), replacing `relationship_endpoints`. The report calls the missing `graph: GraphId` parameter a "documented deviation from the brief" — it is not a deviation at all; the brief's own Step 5 code block (task-8-brief.md:142-151) already omits that parameter. Minor report-accuracy nit only, not a code issue.
- ✅ `BindError::RoleTargetTypeViolation` replaces `InvalidEndpointType` (binder.rs:74-87) and is genuinely reachable and tested — verified by reading the updated CREATE-relationship binder loop (binder.rs:1582-1600) and the pre-existing tests updated to assert on it (`semantic_relationship_merge_enforces_types_endpoints_and_idempotency`, `ambiguous_matched_bindings_cannot_create_semantic_relationship_endpoints`, `endpoint_validation_covers_both_directions` in tests/semantic_schema.rs). No `unwrap`/`expect`/`panic!` introduced on this path.
- ✅ Two parked defects assigned to this task are genuinely fixed:
  - (a) The `.expect("binary relationship source has a start/end role")` panics are gone from `semantic.rs` (grep confirms zero matches for that literal string; the remaining `.expect()`s in the file are pre-existing invariant asserts guarded by prior validation in the same function, consistent with the file's existing style — not user-triggerable).
  - (b) `check_owned_columns`'s independently-derived `[identity, start.column, end.column]` set is replaced by `source.single_valued_roles().map(|r| r.column)` (semantic.rs:1047-1052), which generalizes to every single-valued role, not just start/end.
- ✅ Two brief-mandated tests exist and are genuine, not vacuous — verified below by sabotage.
- ⚠️ Cannot fully verify from diff alone: whether any other crate outside `graph/frontend` still references the removed `EndpointConstraint`/`relationship_endpoints` API (report claims a clean `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_graph_cypher` run with 357+17 passing; I did not re-run the other three crates — only `turso_graph_frontend` was in scope for sabotage checks).

### Findings (missing/extra/misunderstood)

1. **Missing test coverage, not missing code** — override item 3 ("a role with no target rows is still recovered ... verify the join actually exists"). The join *does* exist and is correct as shipped (`load_roles` in semantic.rs iterates the physical `source.roles` as the left side and falls back to an empty-target `SemanticRole` when no persisted rows match). But I sabotaged the `None` arm to `continue` (turning it into an inner join, silently dropping unconstrained roles) and ran the **entire** `turso_graph_frontend` test suite: `cargo test -p turso_graph_frontend` → **278 passed, 3 ignored, 0 failed** — i.e. nothing catches a silently-dropped unconstrained role. This is a real, verified gap. Reported as Important below.
2. **Schemaless synthesis deviates from the brief's literal Step 5** — the brief says schemaless mode should synthesize two required, single-valued, empty-target `start`/`end` roles; the implementer instead kept the trait default (`Vec::new()`, no roles at all) and argues behavioral equivalence. I independently verified: `relationship_role`/`relationship_roles` has exactly one consumer in the whole crate (binder.rs:1586, the CREATE-relationship type check), and at that call site `None` (role absent) and `Some(role-with-empty-targets)` both take the same `continue` branch — so the outcome is provably identical today. This is a legitimate, low-risk simplification, but it is a literal deviation from the brief's stated mechanism and would silently stop being equivalent if a second consumer (e.g. `required_roles()` in a schemaless MERGE check) is added later without revisiting this decision. Minor.

## Strengths

- The `RoleId`-identity design is stronger than what the brief/override anticipated: `load_roles` never re-derives a `RoleId` from `ordinal`/`role_id` at all — it always copies `physical_role.role` verbatim from the physical registration, using the persisted `role_id` column purely as a join key. This makes drift between the semantic and physical `RoleId` structurally impossible, not just tested-to-be-absent. Confirmed by sabotage (below).
- `target_kind` discriminator is real, persisted, and used symmetrically on write (`resolve_target_id`, semantic.rs:900-913) and read (`load_roles`'s `"node"`/`"relation"` match arms). Confirmed by sabotage.
- `check_owned_columns`'s fix genuinely generalizes beyond the two hardcoded `start`/`end` roles it used to special-case, and does so via a field that is a direct 1:1 copy-through from `RegisteredRelationshipSource` into `RelationshipTableLayout` (schema_catalog.rs:759-778), so the two "structural columns" computations can't diverge even though they're not literally one function.
- Honest TDD disclosure with an actual, verified sabotage-and-restore proof (I independently re-ran it and it holds — see below).
- No `roles.len() == 2` / `is_binary` special-casing anywhere in the diff; binary is genuinely just a 2-role instance of the general path (`SemanticRelationshipType::binary()` is a plain convenience constructor, not a runtime branch).
- Corpus per-suite table cross-checked directly against `graph/test-results/runs.jsonl`'s `20260726T064743.471434Z-3c7dccf9b09d-corpus-deep` row: age-deep 3042/3042, cqlite-deep 113/113, grafeo-deep 277/277, sparrowdb-deep 2164/2164, tck-deep 3330 (within band), total passed=8926/10242 — all figures match exactly.

## Sabotage verification performed (all restored; tree confirmed clean via `git diff`/`git status` before and after each)

| # | Sabotage | Result |
|---|---|---|
| 1 | `load_roles`'s `"relation" =>` arm misread as `RoleTarget::Node` | `a_role_may_target_a_relationship_type` FAILS: `cited must accept a relation player, got [Node(...)]` |
| 2 | write path: `role.cardinality.as_str()` forced to always `"one"` | `a_semantic_role_carries_targets_optionality_and_cardinality` FAILS: `left: One, right: Many` |
| 3 | write path: `role.optional` forced to always `false` | same test FAILS: `assertion failed: witnesses.optional` |
| 4 | `load_roles`'s `Some(group)` arm: `role: physical_role.role` → `ir::RoleId::new(physical_role.role.get() + 1)` | `semantic_role_id_matches_the_physical_role_id` FAILS: `left: RoleId(2), right: RoleId(1)` |
| 5 | `load_roles`'s `None` arm (unconstrained-role recovery) → `continue` (inner join) | **Entire crate passes**: `cargo test -p turso_graph_frontend` → 278 passed, 0 failed. No test catches a silently dropped unconstrained role. |

Item 4 differs slightly from the override's suggested sabotage (there is no "derive from ordinal (0-based)" logic to sabotage — the code never re-derives `RoleId` from `ordinal` at all, it copies the physical value directly, which is a stronger design). I sabotaged the actual mechanism that would need to fail for the invariant to be at risk, and it bites correctly.

## Issues

### Critical (Must Fix)
None.

### Important (Should Fix)

- **semantic.rs `load_roles` unconstrained-role recovery path has zero test coverage.** Verified by sabotage #5 above: turning the physical-roles-left-join into an inner join (silently dropping any role with no persisted target rows) passes all 278 tests in `turso_graph_frontend`. The brief explicitly calls this out ("a role with no target rows is still recovered ... this is why loading joins the physical roles as the left side") and the override explicitly flags a missing test here as a data-correctness bug. The shipped code is correct; the invariant is unverified and could regress silently. Add a test registering a relationship type with an unconstrained role (empty `targets`) and asserting `relationship_type.roles.len()` / `role(name)` still surfaces it after a reload.
- **`check_owned_columns` generalization (parked defect (b)) has no regression test proving the *new* behavior.** The existing `StructuralColumn` tests (tests/semantic_schema.rs:915, 922) only exercise the pre-existing binary start/end case, which passed before this task's fix too. No test uses `TERNARY_SCHEMA` (or similar) to prove a third role's structural column (e.g. `folio_id`) is now protected from a colliding `SemanticProperty`. Given this was one of two defects explicitly assigned to this task, its actual fix — generalizing beyond start/end — should have direct coverage; recommend adding one assertion to `a_semantic_role_carries_targets_optionality_and_cardinality`'s fixture family or a new small test registering a property mapped to `scribe_id`/`folio_id` and expecting `StructuralColumn`.

### Minor (Nice to Have)

- Positional `.roles[0]`/`.roles[1]` indexing appears four times in `graph/frontend/tests/semantic_schema.rs` (lines ~2511-2512, 2630, 4428, 4669), all mutating a `SemanticRelationshipType::binary(...)`-built registration before re-registering it in a fragment/cardinality test. It's test-only scaffolding relying on `.binary()`'s documented `[start, end]` order, not production resolution logic, but it is literally the pattern the plan calls out as the recurring defect class. Cleaner as `schema.relationship_types[0].role_mut("start").targets = ...` (or equivalent) if such an accessor is added later.
- Report's claim that omitting the `graph: GraphId` parameter on `relationship_roles`/`relationship_role` is "a documented deviation from the brief" is inaccurate — the brief's own Step 5 snippet already specifies no such parameter. Not a code issue, just a self-report imprecision.
- Report's description of `single_valued_roles()`/`structural_columns()` as "the single source of truth" is a bit strong — they are two separate method implementations of the identical `cardinality == One` predicate on two different types (`RegisteredRelationshipSource` vs `RelationshipTableLayout`). They cannot currently diverge because `RelationshipTableLayout` is built as a direct field-copy projection of `RegisteredRelationshipSource` (schema_catalog.rs:759-778), but that's a provable invariant, not literal code sharing.

## Assessment

**Task quality:** Needs fixes (two Important test-coverage gaps; no Critical defects, spec compliance is otherwise clean).

**Reasoning:** The production code correctly implements the n-ary semantic role model — the `RoleId`-identity, `target_kind` discriminator, optionality/cardinality round-trip, and unconstrained-role recovery are all real and, where I sabotaged them, genuinely caught by the two new tests. However, two of the four correctness-critical mechanisms this task was responsible for (the empty-target left-join recovery, and the generalized `check_owned_columns` fix) have zero direct test coverage — both are data-correctness-critical paths that could silently regress. Given the plan's own emphasis on relation-as-player and n-ary correctness as the central invariant of this migration, these gaps should be closed before merge, even though the underlying fixes are sound as written.
