### Task 1: `RoleId` identity and the role model

**Files:**
- Modify: `graph/ir/src/identity.rs:59-92`
- Create: `graph/ir/src/role.rs`
- Modify: `graph/ir/src/lib.rs:9-41`
- Test: `graph/ir/src/identity.rs` (existing `mod tests`), `graph/ir/src/role.rs` (new `mod tests`)

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  - `turso_graph_ir::RoleId` — non-zero `u32` identity with `RoleId::new(u32) -> Result<Self, InvalidId>` and `RoleId::get(self) -> u32`.
  - `turso_graph_ir::RoleTarget` — `enum { Node(LabelId), Relation(RelationshipTypeId) }`.
  - `turso_graph_ir::RoleCardinality` — `enum { One, Many }`.
  - `turso_graph_ir::RoleDef { role: RoleId, name: String, target_types: Vec<RoleTarget>, optional: bool, cardinality: RoleCardinality }`.
  - `turso_graph_ir::RoleBinding { role: RoleId, value: BindingId }`.

- [ ] **Step 1: Write the failing identity test**

In `graph/ir/src/identity.rs`, inside `mod tests`, extend
`all_public_identities_reject_zero` with a `RoleId` line:

```rust
        assert!(RoleId::new(0).is_err());
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p turso_graph_ir all_public_identities_reject_zero`
Expected: FAIL to compile with `cannot find type RoleId in this scope`.

- [ ] **Step 3: Define `RoleId`**

In `graph/ir/src/identity.rs`, after the `PropertyId` line (`identity.rs:65`):

```rust
define_u32_id!(RoleId, "role");
```

- [ ] **Step 4: Run it to verify it passes**

Run: `cargo test -p turso_graph_ir all_public_identities_reject_zero`
Expected: PASS.

- [ ] **Step 5: Write the failing role-model test**

Create `graph/ir/src/role.rs` containing only its test module for now:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LabelId, RelationshipTypeId};

    #[test]
    fn a_role_target_keeps_node_and_relation_identity_spaces_distinct() {
        // A node label and a relationship type may share the numeric value 1.
        // Flattening them into one integer would make a role that accepts
        // Person silently accept the relationship type numbered 1.
        let node = RoleTarget::Node(LabelId::new(1).unwrap());
        let relation = RoleTarget::Relation(RelationshipTypeId::new(1).unwrap());
        assert_ne!(node, relation);
    }

    #[test]
    fn a_role_definition_carries_optionality_cardinality_and_targets() {
        let scribe = RoleDef {
            role: RoleId::new(1).unwrap(),
            name: "scribe".to_owned(),
            target_types: vec![RoleTarget::Node(LabelId::new(7).unwrap())],
            optional: false,
            cardinality: RoleCardinality::One,
        };
        assert!(!scribe.optional);
        assert_eq!(scribe.cardinality, RoleCardinality::One);
        assert_eq!(scribe.target_types.len(), 1);
    }

    #[test]
    fn empty_target_types_mean_unconstrained_not_uninhabited() {
        // An empty list is the schemaless default. Reading it as "no player is
        // allowed" would make every schemaless create fail.
        let any = RoleDef {
            role: RoleId::new(2).unwrap(),
            name: "start".to_owned(),
            target_types: Vec::new(),
            optional: false,
            cardinality: RoleCardinality::One,
        };
        assert!(any.accepts_any_target());
    }
}
```

- [ ] **Step 6: Wire the module and run to verify it fails**

Add to `graph/ir/src/lib.rs` beside the other `mod` lines:

```rust
mod role;
```

Run: `cargo test -p turso_graph_ir --lib role::`
Expected: FAIL to compile with `cannot find type RoleTarget in this scope`.

- [ ] **Step 7: Write the role model**

Prepend to `graph/ir/src/role.rs`, above the test module:

```rust
use crate::{BindingId, LabelId, RelationshipTypeId, RoleId};

/// What a player of a role may be.
///
/// A role player is either a node of some label or a relation of some type.
/// The two identity spaces stay distinct rather than being flattened: label 1
/// and relationship type 1 are different things, and a role that accepts one
/// must not accept the other.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RoleTarget {
    Node(LabelId),
    Relation(RelationshipTypeId),
}

/// How many players one role may hold in one relation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RoleCardinality {
    /// Exactly one player, stored in an indexed endpoint column on the
    /// relation table.
    One,
    /// Any number of players, stored in a per-role spill table.
    Many,
}

/// One named role of one relationship type. Roles are local to their relation
/// type: there are no global role interfaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleDef {
    pub role: RoleId,
    pub name: String,
    /// What a player of this role may be. Empty means unconstrained.
    pub target_types: Vec<RoleTarget>,
    pub optional: bool,
    pub cardinality: RoleCardinality,
}

impl RoleDef {
    /// True when this role constrains nothing. Schemaless roles are always
    /// unconstrained; a semantic role usually is not.
    pub fn accepts_any_target(&self) -> bool {
        self.target_types.is_empty()
    }
}

/// One role filled by one player in a mutation.
///
/// Repeated players are legal: the same `value` may appear under two different
/// `role`s of one relation, and nothing downstream assumes players are
/// distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoleBinding {
    pub role: RoleId,
    pub value: BindingId,
}
```

- [ ] **Step 8: Re-export and run to verify it passes**

In `graph/ir/src/lib.rs`, add `RoleId` to the `identity` re-export list and add
a new re-export line:

```rust
pub use identity::{
    BindingId, GraphId, LabelId, NodeId, PropertyId, RelationshipId, RelationshipTypeId, RoleId,
    SourceTableId,
};
pub use role::{RoleBinding, RoleCardinality, RoleDef, RoleTarget};
```

Run: `cargo test -p turso_graph_ir`
Expected: PASS.

- [ ] **Step 9: Gate and commit**

```bash
cargo fmt
cargo clippy --workspace --all-features --all-targets -- --deny=warnings
cargo test -p turso_graph_ir
mise run corpus
git add graph/ir/src/identity.rs graph/ir/src/role.rs graph/ir/src/lib.rs
git commit -S -m "graph/ir: add RoleId and the role definition model

Roles are the identity a native n-ary relation is built from. RoleTarget
keeps node labels and relationship types in distinct identity spaces so a
role that accepts a node label cannot silently accept the relationship type
with the same numeric value.

Tests: turso_graph_ir unit tests."
```

---

