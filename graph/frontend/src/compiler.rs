#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use parking_lot::{Mutex, RwLock};
use turso_core::{FrontendCompilation, FrontendCompiler, FrontendId, LimboError, Result};
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
    fn semantic_property_for_key(
        &self,
        source: ir::SourceTableId,
        type_names: &[String],
        key: &str,
    ) -> Option<Option<(String, ir::ValueType, String)>> {
        RelationalCatalogSnapshot::semantic_property_for_key(self, source, type_names, key)
    }

    fn semantic_property_for_id(
        &self,
        source: ir::SourceTableId,
        type_names: &[String],
        property: ir::PropertyId,
    ) -> Option<Option<(String, ir::ValueType, String)>> {
        RelationalCatalogSnapshot::semantic_property_for_id(self, source, type_names, property)
    }

    fn semantic_properties(
        &self,
        source: ir::SourceTableId,
        type_names: &[String],
    ) -> Option<Vec<(ir::PropertyId, String, ir::ValueType, String)>> {
        RelationalCatalogSnapshot::semantic_properties(self, source, type_names)
    }
}

impl<T> GraphCompilationCatalog for T where
    T: GraphCatalogSnapshot + RelationalCatalogSnapshot + Send + Sync + 'static
{
}

/// One parse/bind/lower of a Cypher source, cached on [`GraphCompiler`] so
/// prepare can recover result types without a second bind.
#[derive(Clone)]
pub(crate) struct CompileOutcome {
    pub(crate) source: String,
    pub(crate) cmd: ast::Cmd,
    pub(crate) result_types: Vec<ir::ValueType>,
    pub(crate) needs_snapshot: bool,
}

/// Connection-local compiler for the Cypher frontend.
///
/// Caches the most recent compile outcome by exact source text so
/// `prepare_frontend` and session result-type recovery share one pass.
pub struct GraphCompiler {
    graph: ir::GraphId,
    catalog: SharedGraphCatalog,
    parameters: ParameterTypes,
    last: Mutex<Option<CompileOutcome>>,
    /// Cache-miss compile count; unit tests assert prepare shares one pass.
    #[cfg(test)]
    compile_misses: AtomicUsize,
}

pub(crate) type SharedGraphCatalog = Arc<RwLock<Arc<dyn GraphCompilationCatalog>>>;

impl GraphCompiler {
    pub fn new(
        graph: ir::GraphId,
        catalog: Arc<dyn GraphCompilationCatalog>,
        parameters: ParameterTypes,
    ) -> Self {
        Self::with_shared(graph, Arc::new(RwLock::new(catalog)), parameters)
    }

    pub(crate) fn with_shared(
        graph: ir::GraphId,
        catalog: SharedGraphCatalog,
        parameters: ParameterTypes,
    ) -> Self {
        Self {
            graph,
            catalog,
            parameters,
            last: Mutex::new(None),
            #[cfg(test)]
            compile_misses: AtomicUsize::new(0),
        }
    }

    /// Parse, bind, and lower once. Cache by exact source string.
    pub(crate) fn compile_outcome(&self, source: &str) -> Result<CompileOutcome> {
        if let Some(cached) = self
            .last
            .lock()
            .as_ref()
            .filter(|outcome| outcome.source == source)
            .cloned()
        {
            return Ok(cached);
        }

        #[cfg(test)]
        self.compile_misses.fetch_add(1, Ordering::SeqCst);

        let catalog = self.catalog.read().clone();
        let query = turso_graph_cypher::parse(source)
            .map_err(|error| LimboError::ParseError(error.to_string()))?;
        let needs_snapshot = query_needs_traversal_snapshot(&query);
        let bound = bind(&query, self.graph, catalog.as_ref(), &self.parameters)
            .map_err(|error| LimboError::ParseError(error.to_string()))?;
        let scope = bound.plan.scope();
        let result_types = bound
            .plan
            .result_shape()
            .iter()
            .map(|column| {
                scope
                    .get(column.binding())
                    .map(|binding| binding.value_type().clone())
                    .unwrap_or(ir::ValueType::Any)
            })
            .collect();
        let statement = lower_relational(&bound.plan, catalog.as_ref())
            .map_err(|error| LimboError::ParseError(error.to_string()))?;
        let outcome = CompileOutcome {
            source: source.to_owned(),
            cmd: ast::Cmd::Stmt(statement),
            result_types,
            needs_snapshot,
        };
        *self.last.lock() = Some(outcome.clone());
        Ok(outcome)
    }

    /// Result column types for `source` when it is the last cached compile.
    pub(crate) fn take_result_types_for(&self, source: &str) -> Option<Vec<ir::ValueType>> {
        self.last
            .lock()
            .as_ref()
            .filter(|outcome| outcome.source == source)
            .map(|outcome| outcome.result_types.clone())
    }

    /// Drop the cached compile (catalog generation change or explicit invalidation).
    pub(crate) fn clear_last_compile(&self) {
        *self.last.lock() = None;
    }

    #[cfg(test)]
    pub(crate) fn compile_misses(&self) -> usize {
        self.compile_misses.load(Ordering::SeqCst)
    }
}

impl FrontendCompiler for GraphCompiler {
    fn compile(&self, source: &str) -> Result<FrontendCompilation> {
        let outcome = self.compile_outcome(source)?;
        Ok(FrontendCompilation {
            prerequisites: Vec::new(),
            cmd: Some(outcome.cmd),
            consumed: source.len(),
        })
    }
}

pub fn graph_frontend_id() -> FrontendId {
    static ID: OnceLock<FrontendId> = OnceLock::new();
    ID.get_or_init(|| {
        FrontendId::new(GRAPH_FRONTEND_NAME).expect("static graph frontend id must be non-empty")
    })
    .clone()
}

/// Variable-length relationship expansions need a traversal snapshot.
pub(crate) fn query_needs_traversal_snapshot(query: &turso_graph_cypher::Query) -> bool {
    let clause_needs =
        |clause: &turso_graph_cypher::Spanned<turso_graph_cypher::Clause>| match &clause.value {
            turso_graph_cypher::Clause::Match(value) => value.paths.iter().any(|path| {
                path.steps
                    .iter()
                    .any(|(relationship, _)| relationship.range.is_some())
            }),
            _ => false,
        };
    query.clauses.iter().any(clause_needs)
        || query
            .unions
            .iter()
            .any(|branch| branch.clauses.iter().any(clause_needs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CatalogEntity, GraphCatalogSnapshot, NodeTableLayout, RelationalCatalogSnapshot,
        RelationshipRoleLayout, RelationshipTableLayout, ResolvedProperty,
    };
    use turso_graph_ir::{
        GraphId, LabelId, Nullability, PropertyId, RelationshipTypeId, RoleCardinality, RoleId,
        SourceTableId, ValueType,
    };

    struct Catalog;

    impl GraphCatalogSnapshot for Catalog {
        fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
            SourceTableId::new(1).ok()
        }

        fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
            SourceTableId::new(2).ok()
        }

        fn label(&self, _graph: GraphId, name: &str) -> Option<LabelId> {
            (name == "Person").then(|| LabelId::new(1).unwrap())
        }

        fn relationship_type(&self, _graph: GraphId, name: &str) -> Option<RelationshipTypeId> {
            (name == "KNOWS").then(|| RelationshipTypeId::new(1).unwrap())
        }

        fn property(
            &self,
            _graph: GraphId,
            entity: CatalogEntity,
            name: &str,
        ) -> Option<ResolvedProperty> {
            let (id, value_type) = match (entity, name) {
                (CatalogEntity::Node, "id") => (1, ValueType::Integer),
                (CatalogEntity::Node, "name") => (2, ValueType::Text),
                _ => return None,
            };
            Some(ResolvedProperty {
                id: PropertyId::new(id).unwrap(),
                value_type,
                nullability: Nullability::Nullable,
            })
        }

        fn relationship_source_roles(
            &self,
            source: SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            self.relationship_layout(source)
        }
    }

    impl RelationalCatalogSnapshot for Catalog {
        fn node_layout(&self, source: SourceTableId) -> Option<NodeTableLayout> {
            (source.get() == 1).then(|| NodeTableLayout {
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            })
        }

        fn relationship_layout(&self, source: SourceTableId) -> Option<RelationshipTableLayout> {
            (source.get() == 2).then(|| RelationshipTableLayout {
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                roles: vec![
                    RelationshipRoleLayout {
                        role: RoleId::new(1).unwrap(),
                        name: "start".to_owned(),
                        column: "src".to_owned(),
                        cardinality: RoleCardinality::One,
                        spill_table: None,
                    },
                    RelationshipRoleLayout {
                        role: RoleId::new(2).unwrap(),
                        name: "end".to_owned(),
                        column: "dst".to_owned(),
                        cardinality: RoleCardinality::One,
                        spill_table: None,
                    },
                ],
            })
        }

        fn property_column(&self, _source: SourceTableId, property: PropertyId) -> Option<String> {
            match property.get() {
                1 => Some("id".to_owned()),
                2 => Some("name".to_owned()),
                _ => None,
            }
        }
    }

    fn compiler() -> GraphCompiler {
        GraphCompiler::new(
            GraphId::new(1).unwrap(),
            Arc::new(Catalog),
            ParameterTypes::new(),
        )
    }

    #[test]
    fn compile_outcome_reuses_cache_for_same_source() {
        let compiler = compiler();
        let source = "MATCH (n:Person) RETURN n.name AS name";
        let first = compiler.compile_outcome(source).expect("first compile");
        let second = compiler.compile_outcome(source).expect("cached compile");
        assert_eq!(compiler.compile_misses(), 1, "second call must not re-bind");
        assert_eq!(first.result_types, second.result_types);
        assert_eq!(first.cmd.to_string(), second.cmd.to_string());
        assert!(!first.needs_snapshot);
        assert_eq!(
            compiler.take_result_types_for(source).as_deref(),
            Some(first.result_types.as_slice())
        );
    }

    #[test]
    fn clear_last_compile_drops_cache() {
        let compiler = compiler();
        let source = "MATCH (n:Person) RETURN n.name AS name";
        compiler.compile_outcome(source).expect("compile");
        assert!(compiler.take_result_types_for(source).is_some());
        assert_eq!(compiler.compile_misses(), 1);
        compiler.clear_last_compile();
        assert!(compiler.take_result_types_for(source).is_none());

        compiler.compile_outcome(source).expect("recompile");
        assert_eq!(compiler.compile_misses(), 2, "cleared cache must re-bind");
    }

    #[test]
    fn variable_length_match_needs_snapshot() {
        let compiler = compiler();
        let outcome = compiler
            .compile_outcome("MATCH (a:Person)-[:KNOWS*1..2]->(b) RETURN b.name")
            .expect("compile");
        assert!(outcome.needs_snapshot);
    }
}
