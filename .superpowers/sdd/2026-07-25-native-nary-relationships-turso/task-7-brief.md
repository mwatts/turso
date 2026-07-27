### Task 7: Delete `Direction` and rename to `RoleExpand`

**Files:**
- Modify: `graph/ir/src/scope.rs:12-30` (delete `Direction`), `graph/ir/src/plan.rs`, `graph/ir/src/lib.rs`
- Modify: `graph/frontend/src/binder.rs`, `graph/frontend/src/lowering.rs`, `graph/frontend/src/graph_expand.rs:120-150`
- Modify: `graph/cypher/src/parser.rs:470-490` (desugar at the AST boundary)
- Test: `graph/ir/src/plan.rs`, `graph/frontend/tests/desugaring_golden.rs` (create)

**Interfaces:**
- Consumes: everything from Tasks 5 and 6.
- Produces:
  - `ir::RoleExpand` replacing `ir::FixedExpand`, with `direction` gone.
  - `ir::PlanKind::RoleExpand` replacing `PlanKind::FixedExpand`.
  - `ir::Direction` no longer exists. `turso_cypher::ast::Direction` survives — it is a parser-level spelling, and the binder is where it dies.

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/tests/desugaring_golden.rs`:

```rust
//! Arrow syntax is sugar over roles. If the two forms ever bind to different
//! IR, then a "binary" query and its role-form equivalent can disagree at
//! runtime, and the claim that binary is a layout of the role model is false.

mod fixture;

use fixture::{bind_fixture, first_role_expand};

#[test]
fn arrow_and_role_forms_of_the_same_pattern_bind_identically() {
    let arrow = bind_fixture("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b");
    let roles = bind_fixture(
        "MATCH (a:Person), (b:Person), [r:KNOWS](start: a, end: b) RETURN b",
    );
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}

#[test]
fn the_reversed_arrow_is_the_reversed_role_pair() {
    let arrow = bind_fixture("MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b");
    let roles = bind_fixture(
        "MATCH (a:Person), (b:Person), [r:KNOWS](end: a, start: b) RETURN b",
    );
    assert_eq!(first_role_expand(&arrow), first_role_expand(&roles));
}
```

This test depends on the standalone role pattern, which lands in Task 12. Mark
both `#[ignore = "standalone role pattern lands in Task 12"]` now and remove the
attribute in Task 13's final step.

Add the non-ignored rename assertion to `graph/ir/src/plan.rs`'s test module:

```rust
    #[test]
    fn a_role_expand_names_its_roles_and_no_direction() {
        // Direction is a parser spelling, not a plan concept. A plan that still
        // carried it would give two sources of truth for which way a traversal
        // runs.
        let expand = sample_role_expand();
        assert_eq!(expand.role_pair(), (RoleId::new(1).unwrap(), RoleId::new(2).unwrap()));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_ir --lib plan::`
Expected: FAIL to compile with `cannot find function sample_role_expand`.

- [ ] **Step 3: Delete the direction field and rename the struct**

- In `graph/ir/src/plan.rs`: rename `FixedExpand` → `RoleExpand`, rename the
  `PlanKind::FixedExpand` variant → `PlanKind::RoleExpand`, delete `pub direction: Direction`
  from both `RoleExpand` and `GraphExpand`.
- In `graph/frontend/src/lowering.rs`: rename `lower_fixed_expand` →
  `lower_role_expand` (Tasks 14 and 15 refer to it by the new name).
- In `graph/frontend/tests/fixed_pattern_fixtures.rs`: rename the Task 5 helper
  `first_fixed_expand` → `first_role_expand` and move it into
  `graph/frontend/tests/fixture.rs`, because `desugaring_golden.rs` uses it too.
- In `graph/ir/src/scope.rs`: delete the `Direction` enum (lines 12-30).
- In `graph/ir/src/lib.rs`: drop `Direction` from the `scope` re-export, rename
  `FixedExpand` → `RoleExpand` in the `plan` re-export.
- Add `fn sample_role_expand() -> RoleExpand` to `plan.rs`'s test module,
  constructing every field with `SourceTableId::new(1)`-style literals.

- [ ] **Step 4: Desugar in the parser walker, not the binder**

In `graph/cypher/src/parser.rs`, leave `ast::Direction` as-is: it is the
grammar's spelling of an arrow. In `graph/frontend/src/binder.rs`, replace the
`ir::Direction` construction from Task 5 with a direct match on the AST:

```rust
        let (from_role, to_role, symmetric) = match pattern.direction {
            ast::Direction::Outgoing => (start_role, end_role, false),
            ast::Direction::Incoming => (end_role, start_role, false),
            // Undirected only stays one expand when both roles target the same
            // node source; otherwise the caller has already split it into a
            // union of two directed branches.
            ast::Direction::Both => (start_role, end_role, true),
        };
```

and delete every remaining `use turso_graph_ir::Direction` and
`ir::Direction::` reference across `binder.rs`, `lowering.rs`, `graph_expand.rs`,
`snapshot.rs`, and the test fixtures.

- [ ] **Step 5: Replace the vtab direction column**

In `graph/frontend/src/graph_expand.rs`, replace `fn direction(value: &Value)`
with role-name columns:

```rust
/// The vtab receives role names rather than a direction word, because a
/// relation with more than two roles has no direction to name.
fn role_name(value: &Value) -> Result<String, ExpandError> {
    match value {
        Value::Text(text) => Ok(text.as_str().to_ascii_lowercase()),
        other => Err(ExpandError::InvalidInput {
            column: "role",
            got: format!("{other:?}"),
        }),
    }
}
```

and change the two input columns previously carrying `direction` to
`from_role` and `to_role`, keeping `INPUT_COLUMN_COUNT` at 14 by replacing the
single direction column and adding one — update the constant to 15 and the
column-name table alongside it.

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_cypher`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/ir: delete Direction and rename FixedExpand to RoleExpand

Direction was a second source of truth for which way a traversal runs. It
survives only as the parser's spelling of an arrow, desugared into a role
pair at the binder boundary; no plan, lowering, or runtime type mentions it.

The expand vtab takes from_role/to_role names instead of a direction word,
because a relation with more than two roles has no direction to name.

Tests: plan unit tests; corpus at 8,926; cypherbench at baseline."
```

---

