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
