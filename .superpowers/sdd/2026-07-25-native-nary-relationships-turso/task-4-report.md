# Task 4 report: role-shaped relationship table layout

Commit: `4905a4fd4054e3b43aac5a8c477d4b9b2e0ce0a0` (signed, on `feature/graph-nary`)

## Brief-assumption verifications (done before writing any code)

1. **`entry.spill_table(role)` / `entry.single_valued_roles()`** — both already
   exist on `RegisteredRelationshipSource` in `graph/frontend/src/catalog.rs`
   (lines 136-145), landed in Task 2. No addition needed; concern #1 in the
   dispatch prompt does not apply.
2. **`structural_columns()` duplication** — see ruling below.
3. **`git add -A`** — not used. Commit stages exactly 8 source files by name;
   `graph/test-results/REPORT.md` and `graph/test-results/runs.jsonl` are left
   modified but unstaged in the working tree.
4. **Construction-site list drift** — the brief's Step 5 list covered test
   literal *construction* sites but missed several real production
   *consumption* sites that read `.start_column`/`.end_column` for SQL
   generation. Found by `rg -n "start_column|end_column"` across
   `graph/frontend` and cross-checked against `#[cfg(test)] mod tests`
   boundaries in each file. Full list below.

## Files changed (all 8, all committed)

- `graph/frontend/src/lowering.rs`
  - Replaced `RelationshipTableLayout` (`start_column`/`end_column` fields)
    with `RelationshipRoleLayout` + role-shaped `RelationshipTableLayout`
    (`roles: Vec<RelationshipRoleLayout>`), plus `role()` and
    `structural_columns()` inherent methods, exactly per the brief's Step 3
    code.
  - Production consumer `lower_fixed_expand` (Cypher `-[r]->` hop lowering,
    ~line 1454-1538): added `start_column`/`end_column` locals derived from
    `relationship.roles[0]`/`roles[1]` (declaration order), replacing all
    `relationship.start_column`/`end_column` field reads in the direction/join
    SQL-building match arms.
  - Production consumer for `startNode()`/`endNode()` Cypher functions
    (~line 2462): same `roles[0]`/`roles[1]` indexing.
  - Test fixture `EndpointCatalog::relationship_layout` (`mod tests`):
    replaced the two-field literal with a two-element `roles` vec using
    `ir::RoleId::new(1)`/`new(2)`, `ir::RoleCardinality::One`, `spill_table:
    None`.
- `graph/frontend/src/schema_catalog.rs`
  - Import: added `RelationshipRoleLayout` alongside `RelationshipTableLayout`.
  - `relationship_layout`: now builds `roles` directly from
    `entry.roles`, computing `spill_table` via `entry.spill_table(role)` for
    `Many` roles, `None` for `One` — matches brief Step 4 exactly.
  - `payload_columns`: relationship arm now calls `self.relationship_layout(source)`
    and uses `layout.structural_columns()` instead of re-deriving the
    structural set from `entry` (see ruling below) — a deliberate deviation
    from the brief's literal Step 4 snippet.
  - Test: added `binary_relationship_catalog()` helper (fresh connection,
    `people`/`friendships` tables, `RelationshipSourceRegistration::binary(...)`)
    and the test `a_relationship_layout_exposes_roles_and_excludes_them_from_payload`,
    both exactly as specified in Step 1.
- `graph/frontend/src/lib.rs` — added `RelationshipRoleLayout` to the
  `pub use lowering::{...}` re-export list (needed by every consumer, internal
  and external, since it's a brand-new public type).
- `graph/frontend/src/mutation.rs`
  - `execute_operation`'s `SET n = map` clear-columns path (~line 1419-1425):
    previously hand-rolled `identity + start_column + end_column`; now calls
    `catalog.relationship_layout(source)` and `.structural_columns()` when the
    source is a relationship, falling back to `vec![layout.identity.clone()]`
    for nodes — collapses a second independent structural-columns definition
    into the single one on `RelationshipTableLayout`.
  - `insert_relationship` (CREATE relationship, ~line 1874-1879):
    `layout.start_column`/`end_column` → `layout.roles[0].column.clone()` /
    `roles[1].column.clone()` (declaration order; `CreateRelationship` IR is
    inherently a two-role `from`/`to` construct).
  - `delete_entity` (DETACH DELETE cleanup, ~line 2027-2036):
    `relationship.start_column`/`end_column` → `relationship.roles[0].column`
    / `roles[1].column`. This branch is only reached when
    `catalog.relationship_endpoint_sources` resolves a `(start_source,
    end_source)` pair, i.e. already binary-gated by the caller — not a new
    arity check.
  - Test fixture (`mod tests`, ~line 2373-2390): `RelationshipRoleLayout`
    import added; literal converted to the two-element `roles` vec.
- `graph/frontend/src/session.rs` — test fixture only (`mod tests`, `Catalog`
  impl of `relationship_layout`): converted to the `roles` vec; import updated.
- `graph/frontend/src/graph_expand.rs` — test fixture only (`mod tests`,
  `CompilerCatalog`): converted; imports updated (`RelationshipRoleLayout`
  from `crate`, `RoleCardinality`/`RoleId` from `turso_graph_ir`).
- `graph/frontend/src/compiler.rs` — test fixture only (`mod tests`,
  `Catalog`): converted; same import additions.
- `graph/frontend/tests/fixed_pattern_fixtures.rs` — external test file,
  `Catalog::relationship_layout`: converted; imports updated
  (`turso_graph_frontend::RelationshipRoleLayout`,
  `turso_graph_ir::{RoleCardinality, RoleId}`).

Not touched (verified, no change needed):
- `graph/testkit/src/dynamic_catalog.rs` — its `relationship_layout` just
  delegates to `self.inner.relationship_layout(source)`; no literal
  construction, nothing to update.
- `graph/frontend/tests/fixture.rs`, `graph/frontend/tests/dialect_alignment.rs`
  — both drive `SchemaCatalog` directly (no hand-built
  `RelationshipTableLayout`), so they pick up the new shape automatically
  through `schema_catalog.rs`'s changes.
- `graph/frontend/src/dialect.rs` — its `turso_graphs` vtab SQL has
  `start_column`/`end_column` as *virtual-table column names* backed by a
  subquery against the `RELATIONSHIP_ROLES_TABLE`, unrelated to the
  `RelationshipTableLayout` Rust struct. Out of scope for this task (already
  fixed for the DB-catalog-column removal in an earlier task per
  `progress.md`).

## Full construction/consumption-site list actually updated

Test-literal *construction* (Step 5's intent):
`lowering.rs` (`EndpointCatalog`), `mutation.rs` (`Catalog`), `session.rs`
(`Catalog`), `graph_expand.rs` (`CompilerCatalog`), `compiler.rs` (`Catalog`),
`fixed_pattern_fixtures.rs` (`Catalog`).

Production field-*consumption* (not in the brief's list, found via `rg` +
compile errors, confirmed via `#[cfg(test)]` boundary checks):
`lowering.rs::lower_fixed_expand` (Expand join SQL), `lowering.rs` (`startNode()`/
`endNode()` lowering), `mutation.rs::execute_operation` (SET-clear structural
set), `mutation.rs::insert_relationship` (CREATE), `mutation.rs::delete_entity`
(DETACH DELETE cleanup).

All of these are inherently two-role code paths (Cypher's `-[r]->` pattern
syntax, `CreateRelationship`/`DeleteEntity` IR nodes), so indexing
`roles[0]`/`roles[1]` in declaration order is a positional lookup baked into
what those IR nodes can express today, not a runtime `if roles.len() == 2`
branch on the catalog's relation shape.

## Ruling: `structural_columns()` duplication (dispatch concern #2)

The brief's Step 4 snippet re-derives `identity_column + single_valued_roles()`
inline in `payload_columns` from `entry` (`RegisteredRelationshipSource`),
independently of the just-introduced `RelationshipTableLayout::structural_columns()`.
That's two definitions of "which columns are structural" that could drift.

`payload_columns` and `relationship_layout` are both methods in the same
`impl RelationalCatalogSnapshot for SchemaCatalog` block, so `payload_columns`
can call `self.relationship_layout(source)` directly — no contortion. I did
that instead of the brief's literal snippet:

```rust
} else if let Some(layout) = self.relationship_layout(source) {
    (layout.table.clone(), layout.structural_columns())
} else {
```

This removes the second call to `relationship_source_entry` in that branch
entirely and leaves exactly one definition of the structural-column set
(`RelationshipTableLayout::structural_columns`). I additionally found and
collapsed a *third*, pre-existing hand-rolled definition in
`mutation.rs::execute_operation`'s `SET n = map` clear-columns path (which
pushed `identity_column` then `start_column`/`end_column` by hand); it now
calls `catalog.relationship_layout(source)` + `.structural_columns()` the same
way.

`RegisteredRelationshipSource::single_valued_roles()` is left in place
(catalog.rs) — it's still legitimate public API surface (part of Task 2's
deliverable), just no longer called from `payload_columns`.

## TDD: red before green

1. Wrote the test (`schema_catalog.rs`) and the new struct/impl changes
   together (since the structural investigation surfaced both at once), then
   used `git stash push -- graph/frontend/src/lowering.rs graph/frontend/src/lib.rs`
   to revert *only* the struct/export change while keeping the new test and
   the new `relationship_layout`/`payload_columns` bodies.
2. Ran `cargo test -p turso_graph_frontend --lib schema_catalog::` — compile
   failure, confirmed the exact expected shape:
   ```
   error[E0609]: no field `roles` on type `lowering::RelationshipTableLayout`
      --> graph/frontend/src/schema_catalog.rs:1387:27
        available fields are: `table`, `identity_column`, `start_column`, `end_column`
   ```
   (plus cascading errors for `.role()`/`.structural_columns()` not existing).
3. `git stash pop` restored the implementation; reran — green (see below).
4. Additionally sabotage-tested `structural_columns()` (temporarily hard-coded
   it to return only `[identity_column]`, dropping the role-column filter) and
   reran the new test alone:
   ```
   thread '...a_relationship_layout_exposes_roles_and_excludes_them_from_payload' panicked
   role columns must not appear as payload properties: [("src", "src"), ("dst", "dst")]
   ```
   Confirms the test is not tautological — it actually catches a payload leak.
   Reverted the sabotage; full suite passed again afterward (byte-identical
   diff stat before/after: `92 ++++++++++++++++++++++++++++++++----------` on
   `lowering.rs`, unchanged).

## Test commands and output

```
$ cargo test -p turso_graph_frontend --lib schema_catalog::
cargo test: 8 passed, 135 filtered out (1 suite, 0.02s)

$ cargo test -p turso_graph_frontend
cargo test: 266 passed (11 suites, 0.63s/0.65s across reruns)

$ cargo test -p turso_graph_frontend --lib schema_catalog::tests::a_relationship_layout_exposes_roles_and_excludes_them_from_payload
cargo test: 1 passed, 142 filtered out (1 suite, 0.01s)
```

## Gates

- `cargo fmt` — applied; `cargo fmt -- --check` clean afterward.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — exit 0, 10 warnings (all pre-existing `ar`-toolchain `limbo_sqlite_test_ext`
  build-script noise, not from this change). Ran in exactly the required
  workspace form; did not use `-p turso_graph_frontend` per the dispatch note.
  (Aside: plain `cargo build --workspace --all-features --all-targets` hits an
  unrelated pre-existing `#[global_allocator]` conflict between `py-turso` and
  `turso` when *every* target is linked together — not part of the specified
  gate, and clippy — which doesn't link binaries — doesn't hit it. Not
  investigated further as it's outside this task's scope and outside the
  documented gate command.)
- `cargo test -p turso_graph_frontend` — 266 passed, 0 failed.
- `mise run corpus` — **8,926 / 10,242 passed** (1,316 failed). Suite
  breakdown: `age-deep` 3042/553, `cqlite-deep` 113/11, `grafeo-deep` 277/95,
  `sparrowdb-deep` 2164/61, `tck-deep` 3330/596. This is an **exact** per-suite
  match with the recorded baseline runs at commits `e068dc04c` and
  `0678787100af` (both also 8926/1316 with identical suite breakdown per
  `graph/test-results/runs.jsonl`), i.e. parity with the documented
  ">= 8926" gate floor and the known `tck.expressions.temporal.temporal10.scenario-12`
  flaky-test noise, not a regression.

## Commit

```
4905a4fd4054e3b43aac5a8c477d4b9b2e0ce0a0 (signed)
graph/frontend: make the relationship layout role-shaped
```

Staged explicitly (no `git add -A`):
`graph/frontend/src/{lib.rs,lowering.rs,schema_catalog.rs,mutation.rs,session.rs,graph_expand.rs,compiler.rs}`,
`graph/frontend/tests/fixed_pattern_fixtures.rs`.

`graph/test-results/REPORT.md` and `graph/test-results/runs.jsonl` are
modified in the working tree (from the corpus run) but were left unstaged and
uncommitted, per instruction.

## Fix round 1

Review finding (Important): `relationship_layout` copies `entry.roles` in
storage order, and four production sites indexed `roles[0]`/`roles[1]`
positionally to get `start`/`end`. `RelationshipSourceRegistration::binary()`
is currently the only constructor and always declares `[start, end]`, so this
was safe by coincidence, not by construction — `validate_registration_names`
does not require `start` to precede `end`, and `relationship_endpoint_sources`
gates all four sites by name, order-agnostically. A future construction path
declaring `end` before `start` would silently invert direction with no
compile error.

**Fix**: added `RelationshipTableLayout::start_role()` / `end_role()` in
`graph/frontend/src/lowering.rs`, backed by a private `role_by_name` (same
`eq_ignore_ascii_case` matching as `catalog.rs`'s `role_by_name`). Switched all
four sites to them instead of positional indexing:

- `graph/frontend/src/lowering.rs::lower_fixed_expand` — `start_column`/
  `end_column` locals now come from `.start_role()`/`.end_role()`.
- `graph/frontend/src/lowering.rs` (`startNode()`/`endNode()` lowering) —
  resolves the role via `.start_role()`/`.end_role()` before reading `.column`.
- `graph/frontend/src/mutation.rs::insert_relationship` — the two-tuple
  `(column, value)` pairs for `CREATE (a)-[r]->(b)` now come from
  `.start_role()`/`.end_role()`.
- `graph/frontend/src/mutation.rs::delete_entity` — the DETACH DELETE
  cleanup predicates now resolve the role by name before reading `.column`.

All four call sites already had a `LowerError::MissingSource`/equivalent
error path available at that point, so the `Option` from `start_role()`/
`end_role()` is surfaced through the existing error type rather than a new
one or a panic. No validation was added requiring `start` to precede `end` at
registration — binary stays a layout, not a specially-validated kind.

**No added validation, no arity branch**: confirmed the diff touches only the
name-based lookup and its four call sites; no `if roles.len() == 2` or
registration-order check was introduced anywhere.

**Test discipline (red before green)**: sabotaged `start_role()`/`end_role()`
back to `self.roles.first()` / `self.roles.get(1)` (the positional bug),
confirmed both new tests failed for the stated reason:

```
thread 'lowering::tests::start_end_role_lookup_is_name_based_not_positional' panicked
  left: "(SELECT ep.\"end node\" FROM ...)"
 right: "(SELECT ep.\"start node\" FROM ...)"

thread 'schema_catalog::tests::a_relationship_with_end_declared_before_start_resolves_endpoints_by_name' panicked
  left: Some("person_a")
 right: Some("person_b")
```

then restored the real implementation and reran — both pass (`cargo test -p
turso_graph_frontend --lib -- start_end_role_lookup_is_name_based_not_positional
a_relationship_with_end_declared_before_start_resolves_endpoints_by_name`:
2 passed).

**Tests added**:

- `graph/frontend/src/schema_catalog.rs`: `reversed_binary_relationship_catalog()`
  helper (hand-built `RelationshipSourceRegistration` — `binary()` can't
  express this — declaring `end` before `start` over an `acquaintances`
  table with distinct `person_a`/`person_b` columns) and the test
  `a_relationship_with_end_declared_before_start_resolves_endpoints_by_name`,
  which asserts `layout.roles[0].name == "end"` (proving declaration order is
  really reversed), then asserts `start_role()`/`end_role()` resolve to the
  correct columns regardless, and finally binds+lowers a real
  `MATCH (a:Person)-[:KNOWS]->(b:Person)` query through the production
  `SchemaCatalog` to confirm the pipeline doesn't error on reversed
  declaration order (this last part is a sanity check only — it does not by
  itself distinguish correct from inverted SQL, since `ast::Stmt` returned by
  `lower_relational` has no `Display`/round-trip-to-SQL-text impl to assert
  against string content).
- `graph/frontend/src/lowering.rs`: `ReversedEndpointCatalog` fixture (same
  layout as the existing `EndpointCatalog` but `end` declared first) and the
  test `start_end_role_lookup_is_name_based_not_positional`, which is the one
  that actually inspects generated SQL text: asserts `startNode()` still
  produces `SELECT ep."start node" FROM ...` and `endNode()` still produces
  `SELECT ep."end node" FROM ...`, i.e. that a hop through this catalog
  lowers with the endpoints the right way round, matching (not swapped
  relative to) `endpoint_functions_use_quoted_relationship_layout_columns`'s
  assertions on the non-reversed `EndpointCatalog`.

## Fix round 1 gates

- `cargo fmt` — applied; `cargo fmt -- --check` clean.
- `cargo clippy --workspace --all-features --all-targets -- --deny=warnings`
  — exit 0, 10 warnings (same pre-existing `ar`-toolchain baseline).
- `cargo test -p turso_graph_frontend` — 268 passed, 0 failed (266 + 2 new).
- `mise run corpus` — ran twice at commit `4905a4fd4054` (working tree with
  the fix round changes): 8927/10242 and 8928/10242 passed. Since these
  numbers differ from the earlier baseline runs recorded in `runs.jsonl`
  (8926/1316) by more than the usual ±1 and touch production lowering/
  mutation paths, did a controlled comparison instead of trusting the
  aggregate: `git stash`'d the fix-round changes back to the as-committed
  `4905a4fd4054` state, reran `mise run corpus` (8927/1315, matching the
  ±1 flaky-test band on its own), captured the full failing-test-ID list,
  restored the fix-round changes (`git stash pop`), and diffed the
  failing-ID lists between the pre-fix-round run and the earlier fix-round
  run (8928/1314). The full-line diff shows exactly one line of difference:
  `tck.expressions.temporal.temporal10.scenario-12.examples-1-row-2` failed
  in the pre-fix-round run and passed in the fix-round run. This is a
  sub-case of the documented flaky scenario
  (`tck.expressions.temporal.temporal10.scenario-12`) — floating-point/
  timing jitter in temporal arithmetic, unrelated to relationship roles —
  and is the only test that changed state between the two runs. No other
  test differs. This is within the documented noise band and not a
  regression.

## Fix round 1 commit

Commit created on top of `4905a4fd4054e3b43aac5a8c477d4b9b2e0ce0a0`, signed,
explicit file list (`graph/frontend/src/lowering.rs`,
`graph/frontend/src/mutation.rs`, `graph/frontend/src/schema_catalog.rs`) —
no `git add -A`; `graph/test-results/*` left uncommitted.
