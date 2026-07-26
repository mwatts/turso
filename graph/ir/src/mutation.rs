use crate::{
    Binding, BindingId, GraphId, LabelId, Plan, PropertyId, RelationshipTypeId, RoleBinding,
    SourceTableId, TypedExpression,
};

/// A bound graph mutation whose names and storage sources have been resolved.
#[derive(Clone, Debug, PartialEq)]
pub struct MutationRequest {
    pub graph: GraphId,
    pub input: Option<Plan>,
    pub operations: Vec<Mutation>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Mutation {
    CreateNode(CreateNode),
    CreateRelation(CreateRelation),
    SetProperty(SetProperty),
    SetLabels(SetLabels),
    ReplaceProperties(ReplaceProperties),
    ReplacePropertiesDynamic(ReplacePropertiesDynamic),
    RemoveProperty(RemoveProperty),
    Delete(DeleteEntity),
    MergeNode(MergeNode),
    MergeRelation(MergeRelation),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyValue {
    pub property: PropertyId,
    /// Conceptual owners used to resolve owner-specific physical mappings.
    pub semantic_types: Vec<String>,
    pub value: TypedExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateNode {
    pub binding: Binding,
    pub source: SourceTableId,
    pub labels: Vec<LabelId>,
    pub properties: Vec<PropertyValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateRelation {
    pub binding: Binding,
    pub source: SourceTableId,
    pub relationship_types: Vec<RelationshipTypeId>,
    pub properties: Vec<PropertyValue>,
    /// One entry per filled role, in the relation type's declaration order.
    /// A repeated player is legal; nothing here assumes distinct values.
    pub roles: Vec<RoleBinding>,
}

/// Resolves the physical source for a mutation target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MutationSource {
    /// The binding can only originate from one physical source.
    Static(SourceTableId),
    /// The binding's hidden source provenance selects the physical source.
    Binding(BindingId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetProperty {
    pub entity: BindingId,
    pub source: MutationSource,
    pub property: PropertyId,
    /// Conceptual owners used to resolve owner-specific physical mappings.
    pub semantic_types: Vec<String>,
    pub value: TypedExpression,
}

/// `SET n:Label1:Label2` — adds labels to an existing node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetLabels {
    pub entity: BindingId,
    pub source: MutationSource,
    pub labels: Vec<LabelId>,
}

/// `SET n = map` / `SET n += map` over a literal component map. `clear`
/// wipes every other payload property first (the `=` form).
#[derive(Clone, Debug, PartialEq)]
pub struct ReplaceProperties {
    pub entity: BindingId,
    pub source: MutationSource,
    /// Conceptual owners whose properties may be replaced or cleared.
    pub semantic_types: Vec<String>,
    pub entries: Vec<PropertyValue>,
    pub clear: bool,
}

/// `SET n = expr` / `SET n += expr` where the value is a map-shaped
/// expression evaluated at execution time (properties(m), a parameter).
/// Every payload column updates from the JSON value; `clear` nulls keys
/// the value omits, the merge form keeps them.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplacePropertiesDynamic {
    pub entity: BindingId,
    pub source: MutationSource,
    /// Conceptual owner types needed to validate dynamic keys before lowering
    /// them to physical columns. Empty retains schemaless behavior.
    pub semantic_types: Vec<String>,
    pub value: TypedExpression,
    pub clear: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoveProperty {
    pub entity: BindingId,
    pub source: MutationSource,
    pub property: PropertyId,
    /// Conceptual owners used to resolve owner-specific physical mappings.
    pub semantic_types: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeleteEntity {
    pub entity: BindingId,
    pub source: MutationSource,
    pub detach: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MergeNode {
    pub create: CreateNode,
    /// Applied only when the merge created the entity.
    pub on_create: Vec<Mutation>,
    /// Applied only when the merge matched an existing entity.
    pub on_match: Vec<Mutation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MergeRelation {
    pub create: CreateRelation,
    /// Applied only when the merge created the entity.
    pub on_create: Vec<Mutation>,
    /// Applied only when the merge matched an existing entity.
    pub on_match: Vec<Mutation>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Nullability, RoleId, ValueType};

    fn sample_create_relation() -> CreateRelation {
        let first = BindingId::new(1).unwrap();
        let second = BindingId::new(2).unwrap();
        let third = BindingId::new(4).unwrap();
        CreateRelation {
            binding: Binding::new(
                BindingId::new(3).unwrap(),
                "r",
                ValueType::Relationship,
                Nullability::NonNull,
            )
            .unwrap(),
            source: SourceTableId::new(1).unwrap(),
            relationship_types: vec![RelationshipTypeId::new(1).unwrap()],
            properties: Vec::new(),
            roles: vec![
                RoleBinding {
                    role: RoleId::new(1).unwrap(),
                    value: first,
                },
                RoleBinding {
                    role: RoleId::new(2).unwrap(),
                    value: second,
                },
                RoleBinding {
                    role: RoleId::new(3).unwrap(),
                    value: third,
                },
            ],
        }
    }

    #[test]
    fn a_created_relation_lists_its_role_bindings_in_declaration_order() {
        let create = sample_create_relation();
        assert_eq!(
            create.roles,
            vec![
                RoleBinding {
                    role: RoleId::new(1).unwrap(),
                    value: BindingId::new(1).unwrap()
                },
                RoleBinding {
                    role: RoleId::new(2).unwrap(),
                    value: BindingId::new(2).unwrap()
                },
                RoleBinding {
                    role: RoleId::new(3).unwrap(),
                    value: BindingId::new(4).unwrap()
                },
            ]
        );
    }

    #[test]
    fn a_create_relation_names_only_roles() {
        // Two ways to say who participates is one way too many: a writer that
        // read `from` while the binder filled `roles` would silently ignore
        // every role past the second.
        let create = sample_create_relation();
        assert_eq!(create.roles.len(), 3);
        // A length check alone survives a bug that fills the right number of
        // roles with the wrong players (the recurring defect class of this
        // plan) -- pin the exact (role, value) pairs too.
        assert_eq!(
            create.roles,
            vec![
                RoleBinding {
                    role: RoleId::new(1).unwrap(),
                    value: BindingId::new(1).unwrap()
                },
                RoleBinding {
                    role: RoleId::new(2).unwrap(),
                    value: BindingId::new(2).unwrap()
                },
                RoleBinding {
                    role: RoleId::new(3).unwrap(),
                    value: BindingId::new(4).unwrap()
                },
            ]
        );
    }

    #[test]
    fn a_role_binding_list_permits_the_same_player_twice() {
        // Repeated players are legal: a Match with the same team in the home
        // and away roles is a real thing to record, and nothing downstream may
        // assume role players are distinct.
        let player = BindingId::new(1).unwrap();
        let roles = [
            RoleBinding {
                role: RoleId::new(1).unwrap(),
                value: player,
            },
            RoleBinding {
                role: RoleId::new(2).unwrap(),
                value: player,
            },
        ];
        assert_eq!(roles.iter().filter(|role| role.value == player).count(), 2);
    }
}
