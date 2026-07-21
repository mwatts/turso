use std::ops::{Deref, DerefMut};

use turso_graph_ir::ValueType;

/// Prepared Cypher read statement: the core prepared statement plus the
/// query's static result-column types in projection order.
///
/// Booleans reach storage as integers, so callers that need to render Cypher
/// values faithfully must consult [`Statement::result_types`]. EXPLAIN forms
/// report an empty slice: their output shape belongs to core's
/// `EXPLAIN QUERY PLAN`, not the Cypher projection.
pub struct Statement {
    inner: turso_core::Statement,
    result_types: Vec<ValueType>,
}

impl Statement {
    pub(crate) fn new(inner: turso_core::Statement, result_types: Vec<ValueType>) -> Self {
        Self {
            inner,
            result_types,
        }
    }

    pub fn result_types(&self) -> &[ValueType] {
        &self.result_types
    }

    pub fn into_inner(self) -> turso_core::Statement {
        self.inner
    }
}

impl Deref for Statement {
    type Target = turso_core::Statement;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Statement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
