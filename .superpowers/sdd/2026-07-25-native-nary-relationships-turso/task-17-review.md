# Task 17 Review — traversal runtime: Direction → role-pair adjacency

Reviewed: brief + CONTROLLER CORRECTIONS A-J, implementer report (commit `d72ccdc6a`), diff `aa053d8a2..d72ccdc6a` (18 files, +1087/-444).

## Verdict 1 — Spec compliance: ❌ (conditional fail)

Every individually-checked correction (A, B, C's mechanics, D, I, J) is satisfied as written. But Correction C's real intent — replacing the two `role_by_name("start")`/`role_by_name("end")` `.expect()` calls with a general role-pair edge-construction pass — is only half-generalized: the One/One pass is fully general (all ordered pairs of single-valued roles), but the Many-role pass only ever emits `(One, Many)` pairs, never `(Many, Many)`. A relation with two `Many` roles registers successfully, its snapshot builds without error, and traversal over that relation's own role pair returns silently empty — a hard-coded-shape gap in a path the brief and corrections both frame as "must be general," verified empirically (see Critical-1). Everything else in the spec is met.

## Verdict 2 — Task quality: Not approved as final; approved pending Critical-1

Sabotage discipline, test hygiene on the covered paths, and adherence to the corrections (esp. B, I, J, the tautology-avoidance in J) are all solid — see S1-S6 below and Important/Minor notes. The blocking issue is Critical-1: a constructible schema shape silently returns zero rows with no test anywhere catching it. That is a correctness defect per the brief's own stated bar ("silently-no-rows on a constructible schema would be a correctness defect, not a scope limit"), not a documented scope limit, so the work is not done until it is either rejected at registration/build time or actually wired up and tested.

---

## Findings

### Critical

**C1. `(Many, Many)` role pairs are constructible and silently produce zero edges — confirmed by direct experiment, not just S5.**

- `graph/frontend/src/catalog.rs`: `register_graph_in_transaction`/`install_role_pair_indexes` (~L388-460, ~L900-934) only special-case `RoleCardinality::One` roles; nothing rejects a `RelationshipSourceRegistration` with two (or more) `Many` roles. No count/cardinality-shape validation exists anywhere in `catalog.rs`, `semantic.rs`, `binder.rs`, or `mutation.rs` (checked via targeted `rg` across all four for "Many role", "at most one", "TooManyMany", cardinality-count errors — none found).
- `graph/frontend/src/snapshot.rs` (this diff, ~L730-780 per report): the Many-role spill pass only ever pairs each `Many` role against the `single_valued_roles` (i.e., `One` roles); two `Many` roles in the same source are never paired with each other, in either direction.
- I built a minimal fixture with a relationship source declaring exactly two `Many`-cardinality roles (`authors`, `editors`, no `One` roles at all), registered it, inserted one relationship row plus two players per role into both spill tables, and refreshed the snapshot in-process (temporary test added to `graph/frontend/src/snapshot.rs`, run, then reverted — tree left clean). Results:
  - Registration: **succeeds** (`registration_result.is_ok() = true`).
  - Snapshot refresh: **succeeds**, `Ok(Published { .. })`, no `OrphanSpillRow` or any other error.
  - `snapshot.graph().edge_count()` for this relationship: **0**. `node_count()`: 4 (all four players loaded as nodes, zero edges built).
  - `Graph::resolve_pairs(&[], authors_role, editors_role, false)`: **`[]`** — no relationship type is registered under that role pair at all.
  - `Graph::neighbors(node_10, &[])`: **`[]`** — silent empty result, no error surfaced anywhere in the call chain.
- This is exactly the failure mode the brief warned against: `csr.rs::neighbor_cursor`'s own doc comment states "A triple absent from the graph ... contributes an empty lane rather than an error" — a deliberate, correct design for *unknown* pairs at a *known-absent* edge, but here it silently swallows a pair that the schema itself declares should exist and have data.
- No test in the diff or in `graph/frontend/tests/nary_relations.rs` exercises a `(Many, Many)` shape at all (the existing `witnessed_session` fixture and `a_many_valued_role_holds_several_players_in_one_relation` test use exactly one `Many` role alongside two `One` roles, and only assert a raw spill-table row count — see Important-1).
- **Judgment**: the report's framing ("Correction C's wording covers only One-One and One-Many pairs, so Many-Many is out of scope") does not hold up against the brief's own explicit fallback question, which this experiment answers directly: the shape is constructible today, and traversal over it is silently wrong (empty), not refused. This is plan-mandated territory to flag, not a defect to just note in passing — the brief itself set the bar this fails ("Silently-no-rows on a constructible schema would be a correctness defect, not a scope limit"), so this is a **Critical**, not a Minor/deferred item.
- **Resolution options** (not prescribing which; coordinator's call): (a) reject `Many`+`Many` at registration time with a clear catalog error (smallest fix, matches "binary is a layout, not a kind" only insofar as it's an explicit, named restriction rather than a silent gap), or (b) extend the snapshot builder's Many-role pass to also pair every `Many` role against every other `Many` role (both directions), matching the generality the One/One pass already has, plus a traversal-level test (not just a row-count test) proving it.

### Important

**I1. The only `Many`-role test in the suite never exercises graph traversal — it would not have caught C1 or S5.**
`graph/frontend/tests/nary_relations.rs::a_many_valued_role_holds_several_players_in_one_relation` (~L175-212) asserts only `SELECT count(*) FROM relationships__witness` = 2 — a raw spill-table row count. It never runs a `MATCH`/`GraphExpand` traversal through the `witness` role. Combined with S5 (below), this means the entire `(One, Many)` snapshot-join pass this diff added has zero traversal-level coverage; only a table-population check. Given Correction C explicitly calls out the `(One, Many)` pass as one of the two required passes, this is a real coverage gap independent of C1, and should get at least one test that performs a graph traversal (not just a row count) across a `Many` role.

**I2. `semantic_constraints.rs` hard-codes `role_by_name("start")`/`role_by_name("end")` — pre-existing, and confirmed unreached by this diff's traversal path (coordinator's question).**
- Confirmed via `grep -n "semantic_constraints.rs" review-aa053d8a2..d72ccdc6a.diff` → **no matches**. The file (lines ~128-141 `SemanticEndpoint` enum with `Start`/`End` variants, and the `role_by_name("start")`/`role_by_name("end")` calls at ~1486/1493) is entirely untouched by Task 17; it predates this task (semantic-roles work).
- Traced its call graph: `rows_for_registration`, `validate_runtime` (~L302), `validate_state` (~L343) are the public entry points, invoked from `binder.rs`/`schema_catalog.rs` at DDL/mutation time to validate participation/degree (MIN/MAX) cardinality constraints on relationship endpoints — a write-side/schema-validation subsystem, structurally separate from the read-side traversal runtime this diff modifies (`csr.rs`, `traversal.rs`, `shortest.rs`, `snapshot.rs`). No caller in this diff's changed files reaches into `semantic_constraints.rs`, and nothing in `semantic_constraints.rs` calls into the new role-pair adjacency machinery.
- **Conclusion**: pre-existing, and not reached by the new n-ary traversal path in this diff. Not a Task 17 defect. It is worth flagging for the whole-branch review, since `SemanticEndpoint::{Start,End}` is itself a binary-shaped enum that will need attention whenever cardinality-constraint validation is generalized to arbitrary roles — but that is out of scope here and at most a Minor/deferred item, exactly as the coordinator's own fallback framing anticipated.

### Minor

**M1. `MissingEndpoint.role` type change (`&'static str` → `String`) is a reasonable, unavoidable consequence of role-by-name generalization, not flagged as a problem** — noted only because it's a public-error-shape change; call sites all update correctly (no dangling `&'static str` assumptions found).

**M2. `TraversalRequest::outgoing` convenience constructor was deleted** (doc comment: "Goes away with direction once Task 17"). Confirmed this was pre-announced in the removed constructor's own doc comment (not a surprise deletion) and no caller in the diff or the wider tree still references it — clean removal, not a regression.

---

## S1-S6 sabotage results

- **S1** (`if roles.len() == 2 { <binary short-circuit> }` in the adjacency-keying path): **RED.** Central invariant is verified — nothing routes around role-pair keying for the 2-role case.
- **S2** (swap `from_role`/`to_role` at a construction/lookup site): **RED.** Reversed adjacency is caught.
- **S3** (`resolve_path_algorithm` ignores its new `arity` argument): **RED.** The `RolePairRequired` test goes red as required.
- **S4** (arity-2 case forced to require a role pair, inverse of S1): **RED**, and heavily so — both the new arity-2 tests and Correction I's `debug_assert_eq!` pins in `shortest.rs` catch it independently.
- **S5** (delete the `Many`-role spill-table join pass in `snapshot.rs`): **NOT RED anywhere** — 0 test failures across `turso_graph_frontend`/`turso_graph_runtime` with the entire pass removed. This is the direct evidence behind Critical-1/Important-1: the `(One, Many)` path this diff added has no traversal-level test verifying its existence, let alone its correctness.
- **S6** (corrupt the rendered `relationship_arity` field in `graph/ir/src/semantics.rs`): **RED.** The pinned digest test catches it (produces a different, wrong digest).

All six sabotage edits were reverted (`git checkout -- <path>`) immediately after observing each result; the working tree was confirmed clean (`git status --short` empty) after each revert and again at the end of this review, including after the additional C1 experiment (temporary test added to and removed from `graph/frontend/src/snapshot.rs`).
