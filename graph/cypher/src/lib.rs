//! Cypher source text, parser, source AST, spans, and diagnostics.
//!
//! Parser-specific and adapted donor types stay inside this crate. Consumers
//! cross the boundary through Turso-owned graph IR contracts.

#![forbid(unsafe_code)]

mod ast;
mod parser;

pub use ast::{
    BinaryOperator, Clause, Direction, Expression, Literal, MatchClause, NodePattern, PathPattern,
    ProjectionClause, ProjectionItem, Query, RelationshipPattern, RelationshipRange, Span, Spanned,
};
pub use parser::{parse, ParseError};
