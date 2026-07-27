### Task 4: Role-shaped relationship table layout

**Files:**
- Modify: `graph/frontend/src/lowering.rs:14-40` (`RelationshipTableLayout`, `RelationalCatalogSnapshot`)
- Modify: `graph/frontend/src/schema_catalog.rs:762-770` (`relationship_layout`), `:828-842` (`payload_columns`)
- Test: `graph/frontend/src/schema_catalog.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `RegisteredRelationshipRole` from Task 2.
- Produces:
  - `RelationshipRoleLayout { role: ir::RoleId, name: String, column: String, cardinality: RoleCardinality, spill_table: Option<String> }`
  - `RelationshipTableLayout { table: String, identity_column: String, roles: Vec<RelationshipRoleLayout> }`
  - `RelationshipTableLayout::role(&self, role: ir::RoleId) -> Option<&RelationshipRoleLayout>`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/src/schema_catalog.rs`, inside `mod tests`:

```rust
    #[test]
    fn a_relationship_layout_exposes_roles_and_excludes_them_from_payload() {
        // Payload columns are everything that is not structural. A role column
        // that leaked into the payload would be readable as a property and
        // writable by SET, which would corrupt the relation's participation.
        let (catalog, source) = binary_relationship_catalog();
        let layout = catalog
            .relationship_layout(source)
            .expect("relationship layout");
        assert_eq!(layout.roles.len(), 2);
        assert_eq!(layout.roles[0].name, "start");
        assert_eq!(layout.roles[0].column, "src");
        assert!(layout.roles[0].spill_table.is_none());
        assert_eq!(
            layout.role(layout.roles[1].role).map(|role| role.column.as_str()),
            Some("dst")
        );

        let payload = catalog.payload_columns(source).expect("payload columns");
        assert!(
            payload.iter().all(|(logical, _)| logical != "src" && logical != "dst"),
            "role columns must not appear as payload properties: {payload:?}"
        );
    }
```

Add a `binary_relationship_catalog()` helper next to the existing test helpers
in that module, returning the `SchemaCatalog` and the relationship
`SourceTableId` built from `RelationshipSourceRegistration::binary("KNOWS",
"friendships", "id", "src", "dst", "Person", "Person")`, reusing the existing
registration helper at `schema_catalog.rs:1016`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib schema_catalog::`
Expected: FAIL to compile with `struct RelationshipTableLayout has no field named roles`.

- [ ] **Step 3: Replace the layout type**

In `graph/frontend/src/lowering.rs`, replace `RelationshipTableLayout`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipRoleLayout {
    pub role: ir::RoleId,
    pub name: String,
    /// Endpoint column on the relation table. Empty for `Many` roles.
    pub column: String,
    pub cardinality: ir::RoleCardinality,
    /// Set for `Many` roles: `<table>__<role>(relation_id, node_id)`.
    pub spill_table: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipTableLayout {
    pub table: String,
    pub identity_column: String,
    /// Declaration order. A two-role relation is `[start, end]`.
    pub roles: Vec<RelationshipRoleLayout>,
}

impl RelationshipTableLayout {
    pub fn role(&self, role: ir::RoleId) -> Option<&RelationshipRoleLayout> {
        self.roles.iter().find(|entry| entry.role == role)
    }

    /// Columns that carry participation rather than payload.
    pub fn structural_columns(&self) -> Vec<String> {
        let mut columns = vec![self.identity_column.clone()];
        columns.extend(
            self.roles
                .iter()
                .filter(|role| role.cardinality == ir::RoleCardinality::One)
                .map(|role| role.column.clone()),
        );
        columns
    }
}
```

- [ ] **Step 4: Build the layout from the registered roles**

In `graph/frontend/src/schema_catalog.rs`, replace `relationship_layout`:

```rust
    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout> {
        let entry = self.relationship_source_entry(source)?;
        Some(RelationshipTableLayout {
            table: entry.table.clone(),
            identity_column: entry.identity_column.clone(),
            roles: entry
                .roles
                .iter()
                .map(|role| RelationshipRoleLayout {
                    role: role.role,
                    name: role.name.clone(),
                    column: role.column.clone(),
                    cardinality: role.cardinality,
                    spill_table: match role.cardinality {
                        ir::RoleCardinality::One => None,
                        ir::RoleCardinality::Many => Some(entry.spill_table(role)),
                    },
                })
                .collect(),
        })
    }
```

and in `payload_columns` replace the relationship arm's `structural` vector with
the layout's:

```rust
        } else if let Some(entry) = self.relationship_source_entry(source) {
            let mut structural = vec![entry.identity_column.clone()];
            structural.extend(
                entry
                    .single_valued_roles()
                    .map(|role| role.column.clone()),
            );
            (entry.table.clone(), structural)
        } else {
```

Import `RelationshipRoleLayout` alongside `RelationshipTableLayout` at
`schema_catalog.rs:13`.

- [ ] **Step 5: Update the remaining layout construction sites**

`RelationshipTableLayout` literals appear in test and fixture code at
`graph/frontend/src/graph_expand.rs:777`, `graph/frontend/src/session.rs`
(`use` at `:483`), `graph/frontend/tests/fixture.rs`,
`graph/frontend/tests/dialect_alignment.rs`, and
`graph/frontend/tests/fixed_pattern_fixtures.rs`. Replace each
`start_column`/`end_column` pair with a two-element `roles` vector using
`ir::RoleId::new(1)`/`ir::RoleId::new(2)` and `RoleCardinality::One`.

- [ ] **Step 6: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/frontend: make the relationship layout role-shaped

RelationshipTableLayout carries the relation's roles instead of a start and
end column, and structural_columns derives the payload exclusion set from
them, so a role column can never be read as a property or written by SET.

Tests: schema_catalog unit test asserts role exposure and payload exclusion;
corpus at 8,926."
```

---

