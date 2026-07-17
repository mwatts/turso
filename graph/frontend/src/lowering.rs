use thiserror::Error;
use turso_graph_ir::Plan;
use turso_parser::ast;

use crate::RegisteredGraph;

#[derive(Debug, Error)]
pub enum LowerError {
    #[error("relational graph lowering is not implemented")]
    NotImplemented,
}

/// Lower a bound fixed-pattern graph plan into Turso's public SQL AST.
///
/// This intentionally returns an AST rather than planner or VDBE internals.
pub fn lower_relational(_plan: &Plan, _graph: &RegisteredGraph) -> Result<ast::Stmt, LowerError> {
    Err(LowerError::NotImplemented)
}
