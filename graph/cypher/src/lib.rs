//! Cypher source text, parser, source AST, spans, and diagnostics.
//!
//! Parser-specific and adapted donor types stay inside this crate. Consumers
//! cross the boundary through Turso-owned graph IR contracts.

#![forbid(unsafe_code)]

mod ast;
mod parser;

pub use ast::{
    BinaryOperator, CallClause, Clause, ColumnDecl, CreateClause, DeleteClause, Direction,
    Expression, ForeachClause, GraphDdl, Literal, MatchClause, MergeClause, NodeDecl, NodePattern,
    PathPattern, Pattern, PatternElement, ProjectionClause, ProjectionItem, PropertyTarget,
    QuantifierKind, Query, RelationDecl, RelationshipPattern, RelationshipRange, RemoveClause,
    RoleArgument, RoleDecl, RolePattern, SetClause, SetItem, SortItem, Span, Spanned,
    UnaryOperator, UnionBranch, UnwindClause,
};
pub use parser::{parse, parse_ddl, ParseError};
