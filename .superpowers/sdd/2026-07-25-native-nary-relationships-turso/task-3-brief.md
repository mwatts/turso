### Task 3: Fail loudly on a pre-role catalog

**Files:**
- Modify: `graph/frontend/src/catalog.rs:79-119` (`CatalogError`), `:178-197` (`load_registered_graph`)
- Test: `graph/frontend/src/catalog.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `RELATIONSHIP_ROLES_TABLE` from Task 2.
- Produces: `CatalogError::IncompatibleGraphLayout { detail: String }`.

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn a_catalog_predating_roles_fails_at_open_and_names_the_fresh_start_policy() {
        // Fresh start: there is no legacy reader and no migration. Opening a
        // pre-role catalog must say so rather than reporting a confusing
        // "invalid catalog value" from a missing column.
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");
        // Simulate the pre-role layout: the roles table did not exist.
        execute_internal(
            &connection,
            format!("DROP TABLE {RELATIONSHIP_ROLES_TABLE}"),
        )
        .expect("drop roles table");

        let error = load_registered_graph(&connection, "social").expect_err("pre-role catalog");
        let message = error.to_string();
        assert!(
            matches!(error, CatalogError::IncompatibleGraphLayout { .. }),
            "expected IncompatibleGraphLayout, got {message}"
        );
        assert!(
            message.contains("no migration"),
            "the error must name the fresh-start policy, got {message}"
        );
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib catalog::a_catalog_predating_roles`
Expected: FAIL to compile with `no variant named IncompatibleGraphLayout`.

- [ ] **Step 3: Add the variant**

In `CatalogError`:

```rust
    #[error("graph catalog predates native relationship roles ({detail}); this build reads only role-shaped catalogs and there is no migration, so the graph must be created fresh")]
    IncompatibleGraphLayout { detail: String },
```

- [ ] **Step 4: Detect the old layout at open**

In `load_registered_graph`, immediately after `ensure_catalog_exists`:

```rust
    if query_rows(
        connection,
        &format!(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name = {}",
            sql_string(RELATIONSHIP_ROLES_TABLE)
        ),
    )?
    .is_empty()
    {
        return Err(CatalogError::IncompatibleGraphLayout {
            detail: format!("{RELATIONSHIP_ROLES_TABLE} is absent"),
        });
    }
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --lib catalog::`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add graph/frontend/src/catalog.rs
git commit -S -m "graph/catalog: reject pre-role catalogs at open

There is no legacy reader and no migration path. A catalog without the
relationship-roles table must fail with a message that names the
fresh-start policy, not with an incidental invalid-value error from a
missing column.

Tests: catalog unit test drops the roles table and asserts the error text."
```

---

