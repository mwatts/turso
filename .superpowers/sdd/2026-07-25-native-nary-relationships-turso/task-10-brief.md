### Task 10: Write relations from their role bindings

**Files:**
- Modify: `graph/frontend/src/mutation.rs:1832-1884` (`insert_relationship`)
- Test: `graph/frontend/tests/nary_relations.rs` (create)

**Interfaces:**
- Consumes: `CreateRelationship::roles` (Task 9), `RelationshipTableLayout::role` (Task 4).
- Produces: no new public API; `insert_relationship` now derives its fixed columns from the role bindings.

- [ ] **Step 1: Write the failing test**

Create `graph/frontend/tests/nary_relations.rs`:

```rust
//! Native n-ary relations, end to end. Everything here would be expressible
//! only by reification under the old binary model, so each test names the
//! thing reification loses.

mod fixture;

use fixture::{ternary_session, Session};

#[test]
fn a_three_role_relation_writes_one_row_with_three_endpoint_columns() {
    // Reification would write a node plus three edges and lose the fact that
    // the three players are one assertion.
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}), (f:Folio {id: 3}) \
         CREATE [x:Transcription {year: 1387}](scribe: p, text: t, folio: f)",
    );
    let rows = session.sql("SELECT scribe, txt, folio, year FROM transcriptions");
    assert_eq!(rows.len(), 1, "one relation, one row");
    assert_eq!(rows[0], vec!["1", "2", "3", "1387"]);
}

#[test]
fn the_same_player_may_fill_two_roles_of_one_relation() {
    let session = ternary_session();
    session.run(
        "MATCH (p:Person {id: 1}), (t:Text {id: 2}) \
         CREATE [x:Transcription](scribe: p, text: t, folio: p)",
    );
    // The old binary writer had no way to express this without two rows.
    let rows = session.sql("SELECT scribe, folio FROM transcriptions");
    assert_eq!(rows[0], vec!["1", "1"]);
}
```

Add `ternary_session()` to `graph/frontend/tests/fixture.rs`, registering the
`Person`/`Text`/`Folio` node sources and the three-role `Transcription`
relationship source from Task 2's test, plus `Session::run`/`Session::sql`
helpers if that file does not already have them.

This test uses the standalone role pattern from Task 12; mark it
`#[ignore = "surface syntax lands in Task 12"]` and remove the attribute at the
end of Task 13. Add one non-ignored test that exercises the writer through the
IR directly:

```rust
#[test]
fn the_writer_places_each_role_player_in_its_own_column() {
    let session = ternary_session();
    session.execute_create_relation(
        "Transcription",
        &[("scribe", 1), ("text", 2), ("folio", 3)],
        &[("year", "1387")],
    );
    assert_eq!(
        session.sql("SELECT scribe, txt, folio FROM transcriptions")[0],
        vec!["1", "2", "3"]
    );
}
```

`Session::execute_create_relation` builds an `ir::CreateRelationship` with the
named roles resolved through the layout and runs it, bypassing the parser.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: FAIL — the writer places only two players, so `folio` is NULL.

- [ ] **Step 3: Derive the fixed columns from the roles**

In `graph/frontend/src/mutation.rs`, replace the two-element `fixed` slice in
`insert_relationship`:

```rust
    let mut fixed = Vec::with_capacity(create.roles.len());
    let mut spilled = Vec::new();
    for binding in &create.roles {
        let role = layout
            .role(binding.role)
            .ok_or(MutationError::UnknownRole { role: binding.role })?;
        let player = self.resolve_binding_value(binding.value)?;
        match role.cardinality {
            ir::RoleCardinality::One => fixed.push((role.column.clone(), player)),
            // A many-valued role has no column on the relation table; its
            // players land in the spill table after the relation row exists
            // and has an identity to point at.
            ir::RoleCardinality::Many => spilled.push((role.clone(), player)),
        }
    }
    let relation_id = self.insert_entity(&layout.table, &layout.identity_column, &fixed, properties)?;
```

Spill inserts land in Task 14; leave `spilled` unused here with an explicit
assertion rather than a silent drop:

```rust
    assert!(
        spilled.is_empty(),
        "many-valued roles are written in a later step; a Many role must not reach here yet"
    );
```

Add to `MutationError`:

```rust
    #[error("relation has no role {role:?}")]
    UnknownRole { role: ir::RoleId },
    #[error("a role player must resolve to an integer identity")]
    NonIntegerPlayer,
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test nary_relations`
Expected: PASS for the non-ignored test.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/mutation: write relations from their role bindings

insert_relationship derives its fixed column list from the create's role
bindings instead of a start and end pair, so a relation with any number of
single-valued roles writes one row with one column per role.

Many-valued roles are collected and asserted absent for now; their spill
writes land with the spill tables.

Tests: nary_relations writer test over a three-role relation; corpus at 8,926."
```

---

