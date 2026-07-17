//! Turso-owned graph identities, catalog contracts, and bound query IR.
//!
//! This crate is the dependency root for graph language and runtime crates. It
//! must not expose parser, storage-engine, PostgreSQL, or donor implementation
//! types.

#![forbid(unsafe_code)]

mod error;
mod expression;
mod identity;
mod plan;
mod scope;

pub use error::{InvalidId, PlanError};
pub use expression::{
    BinaryOp, Expression, FunctionName, Literal, NullOrder, SortDirection, TypedExpression,
    UnaryOp, ValueType,
};
pub use identity::{
    BindingId, GraphId, LabelId, NodeId, PropertyId, RelationshipId, RelationshipTypeId,
    SourceTableId,
};
pub use plan::{
    Aggregate, AggregateFunction, Aggregation, Distinct, Filter, FixedExpand, Grouping, LeftApply,
    Limit, NodeScan, Plan, PlanKind, Project, Projection, Skip, Sort, SortKey, Union, Unwind,
};
pub use scope::{Binding, Direction, Nullability, ResultColumn, ResultShape, Scope};
