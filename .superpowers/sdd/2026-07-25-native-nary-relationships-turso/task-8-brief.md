### Task 8: Semantic roles

Semantic mode currently stores allowed endpoint node types as
`EndpointConstraint { start, end }`. That becomes a per-role target-type list
carrying optionality and cardinality.

**Files:**
- Modify: `graph/frontend/src/semantic.rs` — table list, `EndpointConstraint` → `SemanticRole`, `SemanticRelationshipType`
- Modify: `graph/frontend/src/binder.rs:55-138` (trait), `:225-416` (`BindError`)
- Test: `graph/frontend/src/semantic.rs` (existing `mod tests`)

**Interfaces:**
- Consumes: `ir::{RoleCardinality, RoleTarget, RoleId}` (Task 1), `RegisteredRelationshipRole` (Task 2).
- Produces:
  - `SemanticRole { role: ir::RoleId, name: String, targets: Vec<ir::RoleTarget>, optional: bool, cardinality: ir::RoleCardinality }`
  - `SemanticRelationshipType { name, source, roles: Vec<SemanticRole>, properties }` — `start`/`end` gone.
  - `SEMANTIC_ROLE_TABLE: &str = "__turso_internal_graph_semantic_roles"` replacing `__turso_internal_graph_semantic_endpoints`.
  - `GraphCatalogSnapshot::relationship_role(&self, ty: ir::RelationshipTypeId, name: &str) -> Option<SemanticRole>`
  - `GraphCatalogSnapshot::relationship_roles(&self, ty: ir::RelationshipTypeId) -> Vec<SemanticRole>`

- [ ] **Step 1: Write the failing test**

In `graph/frontend/src/semantic.rs`, inside `mod tests`:

```rust
    #[test]
    fn a_semantic_role_carries_targets_optionality_and_cardinality() {
        let connection = connection();
        install_semantic_schema(&connection, TERNARY_SCHEMA).expect("install schema");
        let catalog = load_semantic_catalog(&connection, "scriptorium").expect("load catalog");

        let transcription = catalog
            .relationship_type("Transcription")
            .expect("Transcription type");
        assert_eq!(transcription.roles.len(), 3);

        let scribe = transcription.role("scribe").expect("scribe role");
        assert!(!scribe.optional);
        assert_eq!(scribe.cardinality, ir::RoleCardinality::One);
        assert_eq!(scribe.targets.len(), 1, "scribe accepts Person only");

        let witnesses = transcription.role("witness").expect("witness role");
        assert!(witnesses.optional);
        assert_eq!(witnesses.cardinality, ir::RoleCardinality::Many);
    }

    #[test]
    fn a_role_may_target_a_relationship_type() {
        // Relation-as-player: a role whose player is itself a relation. A
        // target list that could only hold node labels would make this
        // unrepresentable.
        let connection = connection();
        install_semantic_schema(&connection, CITATION_SCHEMA).expect("install schema");
        let catalog = load_semantic_catalog(&connection, "scriptorium").expect("load catalog");

        let cites = catalog.relationship_type("Citation").expect("Citation type");
        let cited = cites.role("cited").expect("cited role");
        assert!(
            cited
                .targets
                .iter()
                .any(|target| matches!(target, ir::RoleTarget::Relation(_))),
            "cited must accept a relation player, got {:?}",
            cited.targets
        );
    }
```

Add `TERNARY_SCHEMA` and `CITATION_SCHEMA` beside the existing schema fixtures in
that module, declaring `Transcription` with roles `scribe`/`text`/`folio` (plus
an optional many-valued `witness`) and `Citation` with a `cited` role targeting
`Transcription`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p turso_graph_frontend --lib semantic::`
Expected: FAIL to compile with `no field roles on type SemanticRelationshipType`.

- [ ] **Step 3: Replace the endpoint constraint with roles**

In `graph/frontend/src/semantic.rs`:

```rust
pub(crate) const SEMANTIC_ROLE_TABLE: &str = "__turso_internal_graph_semantic_roles";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticRole {
    pub role: ir::RoleId,
    pub name: String,
    /// What a player may be. Empty means unconstrained.
    pub targets: Vec<ir::RoleTarget>,
    pub optional: bool,
    pub cardinality: ir::RoleCardinality,
}

impl SemanticRelationshipType {
    pub fn role(&self, name: &str) -> Option<&SemanticRole> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }

    pub fn required_roles(&self) -> impl Iterator<Item = &SemanticRole> {
        self.roles.iter().filter(|role| !role.optional)
    }
}
```

with `SemanticRelationshipType { name: String, source: ir::SourceTableId, roles: Vec<SemanticRole>, properties: Vec<SemanticProperty> }`.

- [ ] **Step 4: Replace the endpoint table**

Replace the `__turso_internal_graph_semantic_endpoints` DDL with:

```rust
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {SEMANTIC_ROLE_TABLE}(\
            type_id INTEGER NOT NULL, ordinal INTEGER NOT NULL, \
            name TEXT NOT NULL COLLATE NOCASE, \
            optional INTEGER NOT NULL CHECK(optional IN (0, 1)), \
            cardinality TEXT NOT NULL CHECK(cardinality IN ('one', 'many')), \
            target_kind TEXT NOT NULL CHECK(target_kind IN ('node', 'relation')), \
            target_id INTEGER NOT NULL, \
            PRIMARY KEY(type_id, ordinal, target_kind, target_id))"
    ))?;
```

One row per (role, target). A role with an empty target list gets no rows and is
recovered from the physical registration's role list, which is why loading joins
the physical roles as the left side.

Write and read it with the same `target_kind` discriminator mapping to
`ir::RoleTarget::Node` / `ir::RoleTarget::Relation`. **Do not** collapse the two
kinds into one integer space — a label and a relationship type may share a value.

- [ ] **Step 5: Expose roles on the binder's catalog trait**

Add to `GraphCatalogSnapshot`, replacing `relationship_endpoints` and
`relationship_endpoint_sources`:

```rust
    /// Roles of a relationship type in declaration order. Empty when the type
    /// is unknown.
    fn relationship_roles(&self, ty: ir::RelationshipTypeId) -> Vec<SemanticRole>;

    fn relationship_role(&self, ty: ir::RelationshipTypeId, name: &str) -> Option<SemanticRole> {
        self.relationship_roles(ty)
            .into_iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }
```

Schemaless mode implements it by synthesizing two required single-valued roles
named `start` and `end` with empty target lists, from the physical registration.

Replace `BindError::InvalidEndpointType` with:

```rust
    #[error("role `{role}` of relationship type `{relationship_type}` does not accept {found}")]
    RoleTargetTypeViolation {
        relationship_type: String,
        role: String,
        found: String,
        span_start: usize,
        span_end: usize,
    },
```

and update the existing endpoint-type check to build it from the role's targets.

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
git commit -S -m "graph/semantic: store roles instead of endpoint constraints

A relationship type declares named roles carrying target types, optionality,
and cardinality. Targets keep node labels and relationship types in distinct
kinds so relation-as-player is representable and a label cannot be mistaken
for the relationship type sharing its number.

Schemaless mode synthesizes two required start/end roles with no target
constraint, so the two modes present one shape to the binder.

Tests: semantic unit tests over a ternary type with an optional many-valued
role and over a role targeting a relationship type; corpus at 8,926."
```

---

