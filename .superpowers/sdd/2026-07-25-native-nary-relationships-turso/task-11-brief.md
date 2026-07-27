### Task 11: Delete `from`/`to`/`direction` and rename to `CreateRelation`

**Files:**
- Modify: `graph/ir/src/mutation.rs`, `graph/ir/src/lib.rs`
- Modify: `graph/frontend/src/binder.rs:1472-1605`, `graph/frontend/src/mutation.rs`
- Test: `graph/ir/src/mutation.rs`

**Interfaces:**
- Produces: `ir::CreateRelation { binding, source, relationship_types, roles, properties }`; `ir::Mutation::CreateRelation`; `ir::MergeRelation { create: CreateRelation, on_create, on_match }`. `CreateRelationship`, `MergeRelationship`, and the `from`/`to`/`direction` fields no longer exist.

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn a_create_relation_names_only_roles() {
        // Two ways to say who participates is one way too many: a writer that
        // read `from` while the binder filled `roles` would silently ignore
        // every role past the second.
        let create = sample_create_relation();
        assert_eq!(create.roles.len(), 3);
    }
```

with `sample_create_relation()` building a three-role create.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_ir --lib mutation::`
Expected: FAIL to compile with `cannot find type CreateRelation`.

- [ ] **Step 3: Delete and rename**

- `graph/ir/src/mutation.rs`: rename `CreateRelationship` → `CreateRelation` and
  `MergeRelationship` → `MergeRelation`; delete `pub from`, `pub to`, `pub direction`;
  rename the `Mutation::CreateRelationship`/`MergeRelationship` variants.
- `graph/ir/src/lib.rs`: update the re-exports.
- `graph/frontend/src/binder.rs`: drop the `from`/`to`/`direction` initializers;
  keep the endpoint resolution that produces the role bindings.
- `graph/frontend/src/mutation.rs`: update the match arms and any remaining
  `create.from` / `create.to` reads.
- `graph/frontend/tests/fixture.rs`: update `Session::execute_create_relation`
  (added in Task 10) to build an `ir::CreateRelation`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_ir -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend -p turso_graph_runtime -p turso_cypher
mise run corpus
mise run cypherbench-sample
git add -A
git commit -S -m "graph/ir: rename CreateRelationship to CreateRelation and drop endpoints

Roles are now the only statement of who participates in a created relation.
Keeping from/to alongside them would let a writer that read the old fields
silently ignore every role past the second.

Tests: mutation unit tests; corpus at 8,926; cypherbench at baseline."
```

---

