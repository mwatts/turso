//! Cypher source text, parser, source AST, spans, and diagnostics.
//!
//! Parser-specific and adapted donor types stay inside this crate. Consumers
//! cross the boundary through Turso-owned graph IR contracts.

#![forbid(unsafe_code)]

mod ast;
mod parser;

pub use ast::{
    BinaryOperator, Clause, CreateClause, DeleteClause, Direction, Expression, Literal,
    MatchClause, MergeClause, NodePattern, PathPattern, ProjectionClause, ProjectionItem,
    PropertyTarget, Query, RelationshipPattern, RelationshipRange, RemoveClause, SetClause,
    SetItem, SortItem, Span, Spanned, UnwindClause,
};
pub use parser::{ParseError, parse};
