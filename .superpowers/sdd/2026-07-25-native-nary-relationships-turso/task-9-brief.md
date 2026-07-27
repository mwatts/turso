### Task 9: Add roles to the create-relationship IR alongside `from`/`to`

Expand half of the mutation migration.

**Files:**
- Modify: `graph/ir/src/mutation.rs:40-70` (`CreateRelationship`)
- Modify: `graph/frontend/src/binder.rs:1472-1605`
- Test: `graph/ir/src/mutation.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `ir::RoleBinding` (Task 1), `GraphCatalogSnapshot::relationship_roles` (Task 8).
- Produces: `pub roles: Vec<RoleBinding>` on `CreateRelationship`, populated from `from`/`to` and authoritative from Task 10 onward.

- [ ] **Step 1: Write the failing test**

In `graph/ir/src/mutation.rs`'s test module:

```rust
    #[test]
    fn a_created_relationship_lists_its_role_bindings_in_declaration_order() {
        let create = sample_create_relationship();
        assert_eq!(
            create.roles,
            vec![
                RoleBinding { role: RoleId::new(1).unwrap(), value: BindingId::new(1).unwrap() },
                RoleBinding { role: RoleId::new(2).unwrap(), value: BindingId::new(2).unwrap() },
            ]
        );
    }

    #[test]
    fn a_role_binding_list_permits_the_same_player_twice() {
        // Repeated players are legal: a Match with the same team in the home
        // and away roles is a real thing to record, and nothing downstream may
        // assume role players are distinct.
        let player = BindingId::new(1).unwrap();
        let roles = vec![
            RoleBinding { role: RoleId::new(1).unwrap(), value: player },
            RoleBinding { role: RoleId::new(2).unwrap(), value: player },
        ];
        assert_eq!(roles.iter().filter(|role| role.value == player).count(), 2);
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_ir --lib mutation::`
Expected: FAIL to compile with `no field roles on type CreateRelationship`.

- [ ] **Step 3: Add the field**

In `graph/ir/src/mutation.rs`, after `direction`:

```rust
    /// One entry per filled role, in the relation type's declaration order.
    /// A repeated player is legal; nothing here assumes distinct values.
    pub roles: Vec<RoleBinding>,
```

- [ ] **Step 4: Populate it in the binder**

At `binder.rs:1586`, where `ir::CreateRelationship` is constructed:

```rust
            roles: vec![
                ir::RoleBinding { role: start_role, value: from },
                ir::RoleBinding { role: end_role, value: to },
            ],
```

with `start_role`/`end_role` read from `self.catalog.relationship_roles(relationship_type)`.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_ir -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/ir: carry role bindings on relationship creates

Expand half of the create migration. The binder fills roles from the from
and to bindings it already resolves, so both representations agree before
the writer switches over.

Tests: mutation unit tests, including that a repeated player across two
roles is representable; corpus at 8,926."
```

---

