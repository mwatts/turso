use std::sync::{Arc, OnceLock};

use turso_core::{FrontendCompiler, FrontendId, LimboError, Result};
use turso_graph_ir as ir;
use turso_parser::ast;

use crate::{
    bind, lower_relational, GraphCatalogSnapshot, ParameterTypes, RelationalCatalogSnapshot,
};

const GRAPH_FRONTEND_NAME: &str = "graph-cypher";

/// Catalog view required to compile Cypher all the way to Turso's SQL AST.
pub trait GraphCompilationCatalog:
    GraphCatalogSnapshot + RelationalCatalogSnapshot + Send + Sync + 'static
{
}

impl<T> GraphCompilationCatalog for T where
    T: GraphCatalogSnapshot + RelationalCatalogSnapshot + Send + Sync + 'static
{
}

/// Stateless connection-local compiler for the Cypher frontend.
pub struct GraphCompiler {
    graph: ir::GraphId,
    catalog: Arc<dyn GraphCompilationCatalog>,
    parameters: ParameterTypes,
}

impl GraphCompiler {
    pub fn new(
        graph: ir::GraphId,
        catalog: Arc<dyn GraphCompilationCatalog>,
        parameters: ParameterTypes,
    ) -> Self {
        Self {
            graph,
            catalog,
            parameters,
        }
    }
}

impl FrontendCompiler for GraphCompiler {
    fn compile(&self, source: &str) -> Result<(Option<ast::Cmd>, usize)> {
        let query = turso_graph_cypher::parse(source)
            .map_err(|error| LimboError::ParseError(error.to_string()))?;
        let bound = bind(&query, self.graph, self.catalog.as_ref(), &self.parameters)
            .map_err(|error| LimboError::ParseError(error.to_string()))?;
        let statement = lower_relational(&bound.plan, self.catalog.as_ref())
            .map_err(|error| LimboError::ParseError(error.to_string()))?;
        Ok((Some(ast::Cmd::Stmt(statement)), source.len()))
    }
}

pub fn graph_frontend_id() -> FrontendId {
    static ID: OnceLock<FrontendId> = OnceLock::new();
    ID.get_or_init(|| {
        FrontendId::new(GRAPH_FRONTEND_NAME).expect("static graph frontend id must be non-empty")
    })
    .clone()
}
