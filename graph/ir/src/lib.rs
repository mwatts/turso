//! Turso-owned graph identities, catalog contracts, and bound query IR.
//!
//! This crate is the dependency root for graph language and runtime crates. It
//! must not expose parser, storage-engine, PostgreSQL, or donor implementation
//! types.

#![forbid(unsafe_code)]

mod error;
mod expression;
mod identity;
mod mutation;
mod plan;
mod role;
mod scope;
mod semantics;

pub use error::{InvalidId, PlanError};
pub use expression::{
    BinaryOp, Expression, FunctionName, Literal, NullOrder, QuantifierKind, SortDirection,
    TypedExpression, UnaryOp, ValueType, VectorKind,
};
pub use identity::{
    BindingId, GraphId, LabelId, NodeId, PropertyId, RelationshipId, RelationshipTypeId, RoleId,
    SourceTableId,
};
pub use mutation::{
    CreateNode, CreateRelation, DeleteEntity, MergeNode, MergeRelation, Mutation, MutationRequest,
    MutationSource, PropertyValue, RemoveProperty, ReplaceProperties, ReplacePropertiesDynamic,
    SetLabels, SetProperty, SetRoles,
};
pub use plan::{
    Aggregate, AggregateFunction, Aggregation, Distinct, Filter, GraphExpand, Grouping, Join,
    LeftApply, Limit, NodeScan, PathUniqueness, Plan, PlanKind, ProcedureCall, ProcedureIdentity,
    ProcedureOutput, Project, Projection, RoleExpand, Skip, Sort, SortKey, Union, Unit, Unwind,
};
pub use role::{RoleBinding, RoleCardinality, RoleDef, RoleTarget};
pub use scope::{Binding, Nullability, ResultColumn, ResultShape, Scope};
pub use semantics::{
    semantic_profile_digest, Duplicates, LabelListOrder, NullComparison, NullSort, RowOrder,
    SemanticProfile, WriteClassification, SEMANTIC_PROFILE, SEMANTIC_PROFILE_VERSION,
};
