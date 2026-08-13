#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use parking_lot::{Mutex, RwLock};
use turso_core::{FrontendCompilation, FrontendCompiler, FrontendId, LimboError, Result};
use turso_graph_ir as ir;
use turso_parser::ast;

use crate::{
    ExpandLowerOptions, GraphCatalogSnapshot, ParameterTypes, RelationalCatalogSnapshot, bind,
    lower_relational_with_options,
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
    /// Path-search budgets embedded into `__tdb_int_g_expand` SQL at lower.
    traversal_limits: Mutex<turso_graph_runtime::TraversalLimits>,
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
            traversal_limits: Mutex::new(turso_graph_runtime::TraversalLimits::default()),
            last: Mutex::new(None),
            #[cfg(test)]
            compile_misses: AtomicUsize::new(0),
        }
    }

    /// Update expand path-search budgets used on the next compile miss.
    ///
    /// Clears the last compile cache so a prepared source re-lowers with the
    /// new budgets.
    pub fn set_traversal_limits(&self, limits: turso_graph_runtime::TraversalLimits) {
        *self.traversal_limits.lock() = limits;
        self.clear_last_compile();
    }

    pub fn traversal_limits(&self) -> turso_graph_runtime::TraversalLimits {
        *self.traversal_limits.lock()
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
        let options = ExpandLowerOptions {
            traversal_limits: self.traversal_limits(),
            result_path_cap: result_path_cap(&bound.plan),
        };
        let statement = lower_relational_with_options(&bound.plan, catalog.as_ref(), options)
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

/// Literal LIMIT (+ SKIP) on the outermost result, if both are constant.
///
/// Used only to cap expand `max_paths` so a `LIMIT 10` query need not enumerate
/// the full path budget. Non-literal counts return `None` (no cap from LIMIT).
fn result_path_cap(plan: &ir::Plan) -> Option<u64> {
    match plan.kind() {
        ir::PlanKind::Limit(limit) => {
            let lim = literal_nonneg_count(&limit.count)?;
            let skip = skip_before_limit(&limit.input).unwrap_or(0);
            Some(lim.saturating_add(skip).max(1))
        }
        ir::PlanKind::Project(project) => result_path_cap(&project.input),
        ir::PlanKind::Sort(sort) => result_path_cap(&sort.input),
        ir::PlanKind::Distinct(distinct) => result_path_cap(&distinct.input),
        ir::PlanKind::Skip(skip) => result_path_cap(&skip.input),
        _ => None,
    }
}

fn literal_nonneg_count(expression: &ir::TypedExpression) -> Option<u64> {
    match &expression.expression {
        ir::Expression::Literal(ir::Literal::Integer(value)) if *value >= 0 => Some(*value as u64),
        _ => None,
    }
}

fn skip_before_limit(plan: &ir::Plan) -> Option<u64> {
    match plan.kind() {
        ir::PlanKind::Skip(skip) => literal_nonneg_count(&skip.count),
        ir::PlanKind::Sort(sort) => skip_before_limit(&sort.input),
        ir::PlanKind::Project(project) => skip_before_limit(&project.input),
        ir::PlanKind::Distinct(distinct) => skip_before_limit(&distinct.input),
        _ => None,
    }
}

/// Variable-length relationship expansions need a traversal snapshot.
pub(crate) fn query_needs_traversal_snapshot(query: &turso_graph_cypher::Query) -> bool {
    let clause_needs =
        |clause: &turso_graph_cypher::Spanned<turso_graph_cypher::Clause>| match &clause.value {
            turso_graph_cypher::Clause::Match(value) => {
                value.paths.elements.iter().any(|element| {
                    // A role pattern's grammar has no hop range at all (Task 12
                    // rejects one as a parse error), so it can never need a
                    // traversal snapshot.
                    let turso_graph_cypher::PatternElement::Path(path) = element else {
                        return false;
                    };
                    path.steps
                        .iter()
                        .any(|(relationship, _)| relationship.range.is_some())
                })
            }
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

    #[test]
    fn expand_sql_embeds_session_path_budget() {
        let compiler = compiler();
        let mut limits = turso_graph_runtime::TraversalLimits::default();
        limits.max_paths = 50_000;
        limits.max_node_visits = 12_345;
        compiler.set_traversal_limits(limits);
        let outcome = compiler
            .compile_outcome("MATCH (a:Person)-[:KNOWS*1..2]->(b) RETURN b.name")
            .expect("compile");
        let sql = outcome.cmd.to_string();
        assert!(
            sql.contains("12345"),
            "session max_node_visits should appear in expand args: {sql}"
        );
        assert!(
            sql.contains("50000"),
            "session max_paths should appear in expand args: {sql}"
        );
    }

    #[test]
    fn expand_sql_caps_max_paths_with_outer_limit() {
        let compiler = compiler();
        let mut limits = turso_graph_runtime::TraversalLimits::default();
        limits.max_paths = 50_000;
        compiler.set_traversal_limits(limits);
        let outcome = compiler
            .compile_outcome(
                "MATCH (a:Person)-[:KNOWS*1..2]->(b) RETURN b.name ORDER BY b.name LIMIT 7",
            )
            .expect("compile");
        let sql = outcome.cmd.to_string();
        // Expand arg order ends: uniqueness, visits, edge_visits, max_paths, work, memory.
        // max_paths must be min(50000, 7) = 7, not the session 50000.
        assert!(
            !sql.contains("50000"),
            "outer LIMIT should replace the session max_paths in expand args: {sql}"
        );
        assert!(
            sql.contains(", 7, 20000000,") || sql.contains(", 7, 20000000)"),
            "expected max_paths=7 before default max_work: {sql}"
        );
    }
}
