### Task 5: Add the role pair to the expand IR alongside `direction`

This is the expand half of the expand/contract pair. The IR grows role fields;
`direction` stays and stays authoritative. Nothing changes behaviourally.

**Files:**
- Modify: `graph/ir/src/plan.rs:44-70` (`FixedExpand`), `:72-110` (`GraphExpand`)
- Modify: `graph/frontend/src/binder.rs:2700-2825`
- Test: `graph/ir/src/plan.rs` (existing `mod tests`), `graph/frontend/tests/fixed_pattern_fixtures.rs`

**Interfaces:**
- Consumes: `turso_graph_ir::RoleId` from Task 1.
- Produces: on both `FixedExpand` and `GraphExpand`:
  - `pub from_role: RoleId`
  - `pub to_role: RoleId`
  - `pub symmetric: bool`
  - `FixedExpand::role_pair(&self) -> (RoleId, RoleId)`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/tests/fixed_pattern_fixtures.rs`:

```rust
#[test]
fn an_outgoing_expand_binds_the_start_to_end_role_pair() {
    // The role pair must agree with the direction it is replacing, or the
    // contract half of this migration silently reverses every traversal.
    let plan = bind_fixture("MATCH (a:Person)-[r:KNOWS]->(b:Person) RETURN b");
    let expand = first_fixed_expand(&plan);
    assert_eq!(expand.from_role.get(), 1, "role 1 is `start`");
    assert_eq!(expand.to_role.get(), 2, "role 2 is `end`");
    assert!(!expand.symmetric);
}

#[test]
fn an_incoming_expand_reverses_the_role_pair_rather_than_flagging_it() {
    let plan = bind_fixture("MATCH (a:Person)<-[r:KNOWS]-(b:Person) RETURN b");
    let expand = first_fixed_expand(&plan);
    assert_eq!(expand.role_pair(), (role(2), role(1)));
    assert!(!expand.symmetric);
}

#[test]
fn an_undirected_same_source_expand_is_the_symmetric_pair() {
    // Today's Direction::Both. The binder only emits it when both endpoints
    // come from one node source; otherwise it unions two directed branches,
    // and this test would find two expands rather than a symmetric one.
    let plan = bind_fixture("MATCH (a:Person)-[r:KNOWS]-(b:Person) RETURN b");
    let expand = first_fixed_expand(&plan);
    assert_eq!(expand.role_pair(), (role(1), role(2)));
    assert!(expand.symmetric, "an undirected pattern matches the pair in both orders");
}
```

Add the helpers `fn role(value: u32) -> RoleId { RoleId::new(value).unwrap() }` and
`fn first_fixed_expand(plan: &ir::Plan) -> &ir::FixedExpand` (a depth-first walk
of `PlanKind` returning the first `FixedExpand`) next to the existing fixture
helpers in that file.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --test fixed_pattern_fixtures role_pair`
Expected: FAIL to compile with `no field from_role on type &FixedExpand`.

- [ ] **Step 3: Add the fields**

In `graph/ir/src/plan.rs`, add to both `FixedExpand` and `GraphExpand`, directly
after `direction`:

```rust
    /// Role the traversal leaves the source binding through.
    pub from_role: RoleId,
    /// Role the traversal enters the target binding through.
    pub to_role: RoleId,
    /// Also match the reversed pair. This is what an undirected pattern means
    /// when both endpoints share a node source; a plain ordered pair cannot
    /// say it.
    pub symmetric: bool,
```

and on `FixedExpand`:

```rust
impl FixedExpand {
    pub fn role_pair(&self) -> (RoleId, RoleId) {
        (self.from_role, self.to_role)
    }
}
```

Import `RoleId` at the top of `plan.rs`.

- [ ] **Step 4: Populate them in the binder**

In `graph/frontend/src/binder.rs`, at each `ir::FixedExpand`/`ir::GraphExpand`
construction site, derive the pair from the direction that is already computed:

```rust
        let (from_role, to_role, symmetric) = match direction {
            ir::Direction::Outgoing => (start_role, end_role, false),
            ir::Direction::Incoming => (end_role, start_role, false),
            ir::Direction::Both => (start_role, end_role, true),
        };
```

where `start_role` and `end_role` come from the relationship source's role list:

```rust
        let roles = self
            .catalog
            .relationship_source_roles(relationship_source)
            .ok_or(BindError::UnknownRelationshipSource { .. })?;
        let start_role = roles[0].role;
        let end_role = roles[1].role;
```

Name it `relationship_source_roles`, keyed by `SourceTableId`. Task 8 adds a
separate `relationship_roles` keyed by `RelationshipTypeId` returning
`SemanticRole`; the two must not share a name.

Add `fn relationship_source_roles(&self, source: ir::SourceTableId) -> Option<Vec<RegisteredRelationshipRole>>`
to the `GraphCatalogSnapshot` trait (`binder.rs:55-138`) and implement it for
`SchemaCatalog` by delegating to `relationship_layout`, and for the test
snapshots in `graph/frontend/tests/fixture.rs` and
`graph/frontend/src/session.rs` by returning the two synthesized roles.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p turso_graph_frontend --test fixed_pattern_fixtures`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir -p turso_graph_frontend
mise run corpus
git add -A
git commit -S -m "graph/ir: carry the role pair on expands beside direction

Expand half of the direction migration. The binder derives (from_role,
to_role, symmetric) from the direction it already computes, so the two
representations agree everywhere before any consumer switches over.
Direction stays authoritative until lowering moves.

Tests: fixed-pattern fixtures assert outgoing, incoming, and undirected
patterns produce the pair that matches their direction; corpus at 8,926."
```

---

