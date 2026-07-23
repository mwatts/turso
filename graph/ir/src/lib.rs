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
mod scope;

pub use error::{InvalidId, PlanError};
pub use expression::{
    BinaryOp, Expression, FunctionName, Literal, NullOrder, QuantifierKind, SortDirection,
    TypedExpression, UnaryOp, ValueType, VectorKind,
};
pub use identity::{
    BindingId, GraphId, LabelId, NodeId, PropertyId, RelationshipId, RelationshipTypeId,
    SourceTableId,
};
pub use mutation::{
    CreateNode, CreateRelationship, DeleteEntity, MergeNode, MergeRelationship, Mutation,
    MutationRequest, MutationSource, PropertyValue, RemoveProperty, ReplaceProperties,
    ReplacePropertiesDynamic, SetLabels, SetProperty,
};
pub use plan::{
    Aggregate, AggregateFunction, Aggregation, Distinct, Filter, FixedExpand, GraphExpand,
    Grouping, Join, LeftApply, Limit, NodeScan, PathUniqueness, Plan, PlanKind, ProcedureCall,
    ProcedureIdentity, ProcedureOutput, Project, Projection, Skip, Sort, SortKey, Union, Unit,
    Unwind,
};
pub use scope::{Binding, Direction, Nullability, ResultColumn, ResultShape, Scope};
