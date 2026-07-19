use crate::{
    Binding, BindingId, Direction, GraphId, LabelId, Plan, PropertyId, RelationshipTypeId,
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
    CreateRelationship(CreateRelationship),
    SetProperty(SetProperty),
    SetLabels(SetLabels),
    ReplaceProperties(ReplaceProperties),
    RemoveProperty(RemoveProperty),
    Delete(DeleteEntity),
    MergeNode(MergeNode),
    MergeRelationship(MergeRelationship),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyValue {
    pub property: PropertyId,
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
pub struct CreateRelationship {
    pub binding: Binding,
    pub source: SourceTableId,
    pub from: BindingId,
    pub to: BindingId,
    pub direction: Direction,
    pub relationship_types: Vec<RelationshipTypeId>,
    pub properties: Vec<PropertyValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetProperty {
    pub entity: BindingId,
    pub source: SourceTableId,
    pub property: PropertyId,
    pub value: TypedExpression,
}

/// `SET n:Label1:Label2` — adds labels to an existing node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetLabels {
    pub entity: BindingId,
    pub labels: Vec<LabelId>,
}

/// `SET n = map` / `SET n += map` over a literal component map. `clear`
/// wipes every other payload property first (the `=` form).
#[derive(Clone, Debug, PartialEq)]
pub struct ReplaceProperties {
    pub entity: BindingId,
    pub source: SourceTableId,
    pub entries: Vec<PropertyValue>,
    pub clear: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoveProperty {
    pub entity: BindingId,
    pub source: SourceTableId,
    pub property: PropertyId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeleteEntity {
    pub entity: BindingId,
    pub source: SourceTableId,
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
pub struct MergeRelationship {
    pub create: CreateRelationship,
    /// Applied only when the merge created the entity.
    pub on_create: Vec<Mutation>,
    /// Applied only when the merge matched an existing entity.
    pub on_match: Vec<Mutation>,
}
