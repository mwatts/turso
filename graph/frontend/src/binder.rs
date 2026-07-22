use std::collections::HashMap;

use thiserror::Error;
use turso_graph_cypher as cypher;
use turso_graph_ir as ir;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogEntity {
    Node,
    Relationship,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProperty {
    pub id: ir::PropertyId,
    pub value_type: ir::ValueType,
    pub nullability: ir::Nullability,
}

/// Immutable name-resolution view captured for one graph prepare operation.
pub trait GraphCatalogSnapshot {
    fn node_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId>;
    fn relationship_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId>;
    fn relationship_sources(&self, graph: ir::GraphId) -> Vec<ir::SourceTableId> {
        self.relationship_source(graph).into_iter().collect()
    }
    fn label(&self, graph: ir::GraphId, name: &str) -> Option<ir::LabelId>;
    fn relationship_type(&self, graph: ir::GraphId, name: &str) -> Option<ir::RelationshipTypeId>;
    fn property(
        &self,
        graph: ir::GraphId,
        entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty>;
}

pub type ParameterTypes = HashMap<String, (ir::ValueType, ir::Nullability)>;

#[derive(Clone, Debug, PartialEq)]
pub struct BoundQuery {
    pub plan: ir::Plan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoundMutation {
    pub request: ir::MutationRequest,
    /// Pipeline stages after the initial mutation block, each introduced by
    /// a WITH clause.
    pub stages: Vec<MutationStage>,
    /// Trailing RETURN projections, bound against the final stage scope.
    pub returns: Vec<StageProjection>,
    pub returns_skip: Option<usize>,
    pub returns_limit: Option<usize>,
    /// ORDER BY over the returned rows: extra sort-key projections are
    /// appended after the visible columns; each entry is (column index,
    /// descending).
    pub returns_order: Vec<(usize, bool)>,
    /// Number of user-visible RETURN columns (sort keys follow).
    pub returns_visible: usize,
    pub returns_distinct: bool,
    /// Static types of the user-visible RETURN columns, in projection order.
    pub return_types: Vec<ir::ValueType>,
    /// Entity kind of every binding the pipeline can reference, letting the
    /// executor resolve relational layouts for projected entities.
    pub entity_kinds: std::collections::HashMap<ir::BindingId, CatalogEntity>,
}

/// One WITH-introduced pipeline stage of a mutation query.
#[derive(Clone, Debug, PartialEq)]
pub struct MutationStage {
    pub projections: Vec<StageProjection>,
    /// WITH ... WHERE predicate, bound against the stage's output scope.
    pub predicate: Option<ir::TypedExpression>,
    pub distinct: bool,
    pub items: Vec<StageItem>,
    /// WITH ... ORDER BY sort keys, bound against the stage's output scope
    /// and evaluated over the stage's projected rows (after DISTINCT).
    pub order: Vec<(ir::TypedExpression, bool)>,
    pub skip: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StageItem {
    Operation(ir::Mutation),
    Unwind {
        output: ir::BindingId,
        list: ir::TypedExpression,
    },
    /// MATCH after a mutation clause: joins each current row against the
    /// match results (correlated through internal reference parameters).
    /// Non-optional matches drop rows with no result; optional matches
    /// keep the row with null outputs.
    Match {
        plan: Box<ir::Plan>,
        outputs: Vec<ir::BindingId>,
        optional: bool,
    },
    /// FOREACH: run the nested items once per list element without changing
    /// the surrounding row set.
    Foreach {
        output: ir::BindingId,
        list: ir::TypedExpression,
        items: Vec<StageItem>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum StageProjection {
    Expression {
        output: ir::BindingId,
        expression: ir::TypedExpression,
    },
    Aggregate {
        output: ir::BindingId,
        function: ir::AggregateFunction,
        argument: Option<ir::TypedExpression>,
        distinct: bool,
    },
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BindError {
    #[error("graph has no {entity} source at byte {span_start}..{span_end}")]
    MissingSource {
        entity: &'static str,
        span_start: usize,
        span_end: usize,
    },
    #[error("duplicate variable `{name}` at byte {span_start}..{span_end}")]
    DuplicateVariable {
        name: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("unknown variable `{name}` at byte {span_start}..{span_end}")]
    UnknownVariable {
        name: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("unknown parameter `${name}` at byte {span_start}..{span_end}")]
    UnknownParameter {
        name: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("unknown label `{name}` at byte {span_start}..{span_end}")]
    UnknownLabel {
        name: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("unknown relationship type `{name}` at byte {span_start}..{span_end}")]
    UnknownRelationshipType {
        name: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("unknown property `{name}` at byte {span_start}..{span_end}")]
    UnknownProperty {
        name: String,
        span_start: usize,
        span_end: usize,
    },
    #[error("property access requires a node or relationship at byte {span_start}..{span_end}")]
    InvalidPropertyTarget { span_start: usize, span_end: usize },
    #[error(
        "{feature} is not supported in the initial graph slice at byte {span_start}..{span_end}"
    )]
    Unsupported {
        feature: &'static str,
        span_start: usize,
        span_end: usize,
    },
    #[error("graph IR invariant failed: {0}")]
    InvalidPlan(#[from] ir::PlanError),
    #[error("query produced no plan")]
    EmptyQuery,
    #[error("mutation query contains no mutation clauses")]
    EmptyMutation,
    #[error("graph query has too many bindings")]
    TooManyBindings,
    #[error("invalid relationship range {min}..{max} at byte {span_start}..{span_end}")]
    InvalidRelationshipRange {
        min: u32,
        max: u32,
        span_start: usize,
        span_end: usize,
    },
}

/// Resource cap applied to unbounded variable-length ranges (`[*]`,
/// `[*min..]`). This is not query semantics: the expansion marks such ranges
/// `unbounded`, and the traversal errors with a hops limit-exceeded when a
/// longer admissible path exists instead of silently truncating results.
const DEFAULT_UNBOUNDED_MAX_HOPS: u32 = 64;

pub fn bind(
    query: &cypher::Query,
    graph: ir::GraphId,
    catalog: &dyn GraphCatalogSnapshot,
    parameters: &ParameterTypes,
) -> Result<BoundQuery, BindError> {
    Binder::new(graph, catalog, parameters).bind_query(query)
}

pub fn bind_mutation(
    query: &cypher::Query,
    graph: ir::GraphId,
    catalog: &dyn GraphCatalogSnapshot,
    parameters: &ParameterTypes,
) -> Result<BoundMutation, BindError> {
    Binder::new(graph, catalog, parameters).bind_mutation_query(query)
}

#[derive(Clone)]
struct EntityBinding {
    kind: CatalogEntity,
    /// Label or relationship-type names declared where the entity was bound.
    names: Vec<String>,
}

/// How a named path's value can be produced.
#[derive(Clone, Debug)]
enum PathComposition {
    /// Fixed-length: the ordered node and relationship bindings.
    Fixed {
        nodes: Vec<ir::BindingId>,
        relationships: Vec<ir::BindingId>,
    },
    /// A single variable-length expansion materializes the path itself.
    Expanded(ir::BindingId),
    /// Mixed fixed/variable paths are not materializable yet.
    Unsupported,
}

struct Binder<'a> {
    graph: ir::GraphId,
    catalog: &'a dyn GraphCatalogSnapshot,
    parameters: &'a ParameterTypes,
    next_binding: u32,
    scope: Vec<ir::Binding>,
    entities: HashMap<ir::BindingId, EntityBinding>,
    plan: Option<ir::Plan>,
    list_scopes: std::cell::RefCell<Vec<(String, ir::ValueType)>>,
    /// Nested reduce() scopes as (accumulator name, element name).
    reduce_scopes: std::cell::RefCell<Vec<(String, String)>>,
    path_compositions: HashMap<String, PathComposition>,
}

impl<'a> Binder<'a> {
    fn new(
        graph: ir::GraphId,
        catalog: &'a dyn GraphCatalogSnapshot,
        parameters: &'a ParameterTypes,
    ) -> Self {
        Self {
            graph,
            catalog,
            parameters,
            next_binding: 1,
            scope: Vec::new(),
            entities: HashMap::new(),
            plan: None,
            list_scopes: std::cell::RefCell::new(Vec::new()),
            reduce_scopes: std::cell::RefCell::new(Vec::new()),
            path_compositions: HashMap::new(),
        }
    }

    fn bind_query(mut self, query: &cypher::Query) -> Result<BoundQuery, BindError> {
        let graph = self.graph;
        let catalog = self.catalog;
        let parameters = self.parameters;
        if query
            .unions
            .windows(2)
            .any(|pair| pair[0].all != pair[1].all)
        {
            return Err(at_unsupported(
                query.span,
                "mixing UNION and UNION ALL in one query",
            ));
        }
        self.bind_read_clauses(&query.clauses, query)?;
        let mut plan = self.plan.ok_or(BindError::EmptyQuery)?;
        plan = assemble_union_branches(plan, query, graph, catalog, parameters)?;
        Ok(BoundQuery { plan })
    }

    /// Binds a `CALL { ... }` scoped subquery: the inner query (including
    /// union branches) binds in isolation, and its result columns join the
    /// outer scope. Uncorrelated form only — inner clauses cannot see
    /// outer variables.
    fn bind_call_subquery(
        &mut self,
        inner: &cypher::Query,
        span: cypher::Span,
    ) -> Result<(), BindError> {
        if self.plan.is_some() || !self.scope.is_empty() {
            return Err(at_unsupported(span, "CALL subqueries after other clauses"));
        }
        let mut sub = Binder::new(self.graph, self.catalog, self.parameters);
        sub.next_binding = self.next_binding;
        sub.bind_read_clauses(&inner.clauses, inner)?;
        let entities = sub.entities.clone();
        let first_scope = sub.scope.clone();
        let next_binding = sub.next_binding;
        let plan = sub.plan.take().ok_or(BindError::EmptyQuery)?;
        let plan = assemble_union_branches(plan, inner, self.graph, self.catalog, self.parameters)?;
        self.next_binding = next_binding;
        // The subquery's RETURN columns become outer scope bindings under
        // their aliases (first-branch binding ids; union combines
        // positionally).
        for column in plan.result_shape().iter() {
            let binding = first_scope
                .iter()
                .find(|candidate| candidate.id() == column.binding())
                .ok_or(BindError::EmptyQuery)?;
            let renamed = ir::Binding::new(
                binding.id(),
                column.name(),
                binding.value_type().clone(),
                binding.nullability(),
            )?;
            if let Some(entity) = entities.get(&binding.id()) {
                self.entities.insert(binding.id(), entity.clone());
            }
            self.scope.push(renamed);
        }
        self.plan = Some(plan);
        Ok(())
    }

    fn bind_read_clauses(
        &mut self,
        clauses: &[cypher::Spanned<cypher::Clause>],
        query: &cypher::Query,
    ) -> Result<(), BindError> {
        for clause in clauses {
            match &clause.value {
                cypher::Clause::Match(clause) => {
                    self.bind_match(clause, clause_span(clause, query))?
                }
                cypher::Clause::Create(_)
                | cypher::Clause::Merge(_)
                | cypher::Clause::Set(_)
                | cypher::Clause::Remove(_)
                | cypher::Clause::Delete(_) => {
                    return Err(at_unsupported(
                        clause.span,
                        "mutation clauses in read queries",
                    ));
                }
                cypher::Clause::Unwind(clause) => self.bind_unwind(clause)?,
                cypher::Clause::With(clause) => self.bind_projection(clause, false)?,
                cypher::Clause::Return(clause) => self.bind_projection(clause, true)?,
                cypher::Clause::Foreach(_) => {
                    return Err(at_unsupported(
                        clause.span,
                        "mutation clauses in read queries",
                    ));
                }
                cypher::Clause::Call(value) => self.bind_call(value, clause.span)?,
                cypher::Clause::CallSubquery(inner) => {
                    self.bind_call_subquery(inner, clause.span)?
                }
            }
        }
        Ok(())
    }

    /// Minimal built-in procedure registry: introspection procedures over
    /// the label and relationship-type junctions.
    fn bind_call(
        &mut self,
        clause: &cypher::CallClause,
        span: cypher::Span,
    ) -> Result<(), BindError> {
        let (sentinel, default_yield) = match clause.name.value.to_ascii_lowercase().as_str() {
            "db.labels" => ("__cypher_all_labels", "label"),
            "db.relationshiptypes" => ("__cypher_all_relationship_types", "relationshipType"),
            _ => {
                return Err(at_unsupported(
                    clause.name.span,
                    "procedures outside the built-in registry",
                ));
            }
        };
        if !clause.arguments.is_empty() {
            return Err(at_unsupported(span, "procedure arguments"));
        }
        if clause.yields.len() > 1 {
            return Err(at_unsupported(span, "multi-column procedure YIELD"));
        }
        let name = clause
            .yields
            .first()
            .map(|item| item.value.clone())
            .unwrap_or_else(|| default_yield.to_owned());
        let list = ir::TypedExpression {
            expression: ir::Expression::Function {
                function: ir::FunctionName::new(sentinel).expect("static name"),
                arguments: Vec::new(),
            },
            value_type: ir::ValueType::List(Box::new(ir::ValueType::Text)),
            nullability: ir::Nullability::NonNull,
        };
        let output = ir::Binding::new(
            self.next_id()?,
            name,
            ir::ValueType::Text,
            ir::Nullability::NonNull,
        )?;
        let input = match self.plan.take() {
            Some(input) => input,
            None => ir::Plan::new(
                ir::PlanKind::Unit(ir::Unit),
                ir::Scope::default(),
                ir::ResultShape::default(),
            )?,
        };
        self.scope.push(output.clone());
        let scope = ir::Scope::new(self.scope.clone())?;
        // A bare CALL with no YIELD and no trailing clauses returns its
        // single column directly.
        let shape = ir::ResultShape::new(
            vec![ir::ResultColumn::new(output.id(), output.name())?],
            &scope,
        )?;
        self.plan = Some(ir::Plan::new(
            ir::PlanKind::Unwind(ir::Unwind {
                input: Box::new(input),
                list,
                output,
            }),
            scope,
            shape,
        )?);
        Ok(())
    }

    fn bind_mutation_query(mut self, query: &cypher::Query) -> Result<BoundMutation, BindError> {
        if let Some(branch) = query.unions.first() {
            return Err(at_unsupported(branch.span, "UNION in mutation queries"));
        }
        let mut operations = Vec::new();
        let mut stages: Vec<MutationStage> = Vec::new();
        let mut returns = Vec::new();
        let mut returns_skip = None;
        let mut returns_limit = None;
        let mut returns_order: Vec<(usize, bool)> = Vec::new();
        let mut returns_visible = 0;
        let mut returns_distinct = false;
        let mut deleted_variables: Vec<String> = Vec::new();
        let mut mutation_started = false;
        fn route(
            operations: &mut Vec<ir::Mutation>,
            stages: &mut [MutationStage],
            new_operations: Vec<ir::Mutation>,
        ) {
            if let Some(stage) = stages.last_mut() {
                stage
                    .items
                    .extend(new_operations.into_iter().map(StageItem::Operation));
            } else {
                operations.extend(new_operations);
            }
        }
        for clause in &query.clauses {
            match &clause.value {
                cypher::Clause::Match(value) => {
                    if mutation_started {
                        // The passthrough must snapshot scope before the
                        // match introduces its bindings.
                        if stages.is_empty() {
                            stages.push(self.passthrough_stage());
                        }
                        let item = self.bind_staged_match(value, clause.span)?;
                        stages
                            .last_mut()
                            .expect("stage pushed above")
                            .items
                            .push(item);
                    } else {
                        self.bind_match(value, clause.span)?;
                    }
                }
                cypher::Clause::Create(value) => {
                    mutation_started = true;
                    let mut new_operations = Vec::new();
                    for path in &value.paths {
                        self.bind_create_path(path, false, &mut new_operations)?;
                    }
                    route(&mut operations, &mut stages, new_operations);
                }
                cypher::Clause::Merge(value) => {
                    mutation_started = true;
                    let mut new_operations = Vec::new();
                    self.bind_create_path(&value.path, true, &mut new_operations)?;
                    self.attach_merge_actions(value, &mut new_operations)?;
                    route(&mut operations, &mut stages, new_operations);
                }
                cypher::Clause::Set(value) => {
                    mutation_started = true;
                    let mut new_operations = Vec::new();
                    self.bind_set(value, &mut new_operations)?;
                    route(&mut operations, &mut stages, new_operations);
                }
                cypher::Clause::Remove(value) => {
                    mutation_started = true;
                    let mut new_operations = Vec::new();
                    self.bind_remove(value, &mut new_operations)?;
                    route(&mut operations, &mut stages, new_operations);
                }
                cypher::Clause::Delete(value) => {
                    mutation_started = true;
                    let mut new_operations = Vec::new();
                    self.bind_delete(value, &mut new_operations)?;
                    route(&mut operations, &mut stages, new_operations);
                    deleted_variables.extend(
                        value
                            .variables
                            .iter()
                            .map(|variable| variable.value.clone()),
                    );
                }
                cypher::Clause::With(value) if !mutation_started => {
                    self.bind_projection(value, false)?;
                }
                cypher::Clause::With(value) if mutation_started => {
                    stages.push(self.bind_mutation_with(value)?);
                }
                cypher::Clause::Foreach(value) => {
                    mutation_started = true;
                    if stages.is_empty() {
                        stages.push(self.passthrough_stage());
                    }
                    let item = self.bind_foreach(value)?;
                    stages
                        .last_mut()
                        .expect("stage pushed above")
                        .items
                        .push(item);
                }
                cypher::Clause::Return(value)
                    if mutation_started
                        && std::ptr::eq(clause, query.clauses.last().expect("non-empty")) =>
                {
                    if value.predicate.is_some() {
                        return Err(at_unsupported(
                            clause.span,
                            "modifiers on a mutation RETURN clause",
                        ));
                    }
                    returns_distinct = value.distinct;
                    returns_skip = bind_constant_count(
                        value.skip.as_ref(),
                        "non-constant SKIP/LIMIT on a mutation RETURN clause",
                    )?;
                    returns_limit = bind_constant_count(
                        value.limit.as_ref(),
                        "non-constant SKIP/LIMIT on a mutation RETURN clause",
                    )?;
                    for item in &value.items {
                        let cypher::ProjectionItem::Expression { expression, .. } = item else {
                            return Err(at_unsupported(
                                clause.span,
                                "RETURN * after mutation clauses",
                            ));
                        };
                        if references_variable(expression, &deleted_variables) {
                            return Err(at_unsupported(
                                expression.span,
                                "returning deleted entities",
                            ));
                        }
                        let output = self.next_id()?;
                        if let Some((function, argument, distinct)) =
                            self.bind_aggregate_call(expression)?
                        {
                            returns.push(StageProjection::Aggregate {
                                output,
                                function,
                                argument,
                                distinct,
                            });
                        } else {
                            returns.push(StageProjection::Expression {
                                output,
                                expression: self.bind_expression(expression)?,
                            });
                        }
                    }
                    returns_visible = returns.len();
                    for sort in &value.order_by {
                        let output = self.next_id()?;
                        returns.push(StageProjection::Expression {
                            output,
                            expression: self.bind_expression(&sort.expression)?,
                        });
                        returns_order.push((returns.len() - 1, sort.descending));
                    }
                }
                cypher::Clause::Unwind(value) if !mutation_started => {
                    self.bind_unwind(value)?;
                }
                cypher::Clause::Unwind(value) => {
                    // Mid-pipeline UNWIND expands the row set in place. When
                    // no WITH has started a stage yet, open an implicit
                    // passthrough stage to carry it.
                    if stages.is_empty() {
                        stages.push(self.passthrough_stage());
                    }
                    let list = self.bind_expression(&value.expression)?;
                    let element_type = list_element_type(&list.value_type, value.expression.span)?;
                    let output = ir::Binding::new(
                        self.next_id()?,
                        value.alias.value.clone(),
                        element_type,
                        ir::Nullability::Nullable,
                    )?;
                    self.scope.push(output.clone());
                    stages
                        .last_mut()
                        .expect("stage pushed above")
                        .items
                        .push(StageItem::Unwind {
                            output: output.id(),
                            list,
                        });
                }
                cypher::Clause::Call(_) => {
                    return Err(at_unsupported(
                        clause.span,
                        "procedures in mutation queries",
                    ));
                }
                cypher::Clause::CallSubquery(inner) => {
                    if mutation_started {
                        return Err(at_unsupported(
                            clause.span,
                            "CALL subqueries after mutation clauses",
                        ));
                    }
                    self.bind_call_subquery(inner, clause.span)?;
                }
                cypher::Clause::With(_) | cypher::Clause::Return(_) => {
                    return Err(at_unsupported(
                        clause.span,
                        "projection clauses in mutation queries",
                    ));
                }
            }
        }
        if operations.is_empty() && stages.iter().all(|stage| stage.items.is_empty()) {
            return Err(BindError::EmptyMutation);
        }
        let entity_kinds = self
            .entities
            .iter()
            .map(|(id, entity)| (*id, entity.kind))
            .collect();
        let return_types = returns
            .iter()
            .take(returns_visible)
            .map(|projection| match projection {
                StageProjection::Expression { expression, .. } => expression.value_type.clone(),
                StageProjection::Aggregate {
                    function, argument, ..
                } => aggregate_value_type(function, argument),
            })
            .collect();
        Ok(BoundMutation {
            request: ir::MutationRequest {
                graph: self.graph,
                input: self.plan,
                operations,
            },
            stages,
            returns,
            returns_skip,
            returns_limit,
            returns_order,
            returns_visible,
            returns_distinct,
            return_types,
            entity_kinds,
        })
    }

    fn bind_foreach(&mut self, clause: &cypher::ForeachClause) -> Result<StageItem, BindError> {
        let list = self.bind_expression(&clause.list)?;
        let element_type = list_element_type(&list.value_type, clause.list.span)?;
        let output = ir::Binding::new(
            self.next_id()?,
            clause.variable.value.clone(),
            element_type,
            ir::Nullability::Nullable,
        )?;
        let scope_before = self.scope.len();
        self.scope.push(output.clone());
        let mut items = Vec::new();
        let bound = (|| {
            for inner in &clause.body {
                match &inner.value {
                    cypher::Clause::Create(value) => {
                        let mut operations = Vec::new();
                        for path in &value.paths {
                            self.bind_create_path(path, false, &mut operations)?;
                        }
                        items.extend(operations.into_iter().map(StageItem::Operation));
                    }
                    cypher::Clause::Merge(value) => {
                        let mut operations = Vec::new();
                        self.bind_create_path(&value.path, true, &mut operations)?;
                        self.attach_merge_actions(value, &mut operations)?;
                        items.extend(operations.into_iter().map(StageItem::Operation));
                    }
                    cypher::Clause::Set(value) => {
                        let mut operations = Vec::new();
                        self.bind_set(value, &mut operations)?;
                        items.extend(operations.into_iter().map(StageItem::Operation));
                    }
                    cypher::Clause::Remove(value) => {
                        let mut operations = Vec::new();
                        self.bind_remove(value, &mut operations)?;
                        items.extend(operations.into_iter().map(StageItem::Operation));
                    }
                    cypher::Clause::Delete(value) => {
                        let mut operations = Vec::new();
                        self.bind_delete(value, &mut operations)?;
                        items.extend(operations.into_iter().map(StageItem::Operation));
                    }
                    cypher::Clause::Foreach(value) => {
                        let item = self.bind_foreach(value)?;
                        items.push(item);
                    }
                    _ => {
                        return Err(at_unsupported(
                            inner.span,
                            "non-mutation clauses inside FOREACH",
                        ));
                    }
                }
            }
            Ok(())
        })();
        // The loop variable and any body-created bindings go out of scope.
        self.scope.truncate(scope_before);
        bound?;
        Ok(StageItem::Foreach {
            output: output.id(),
            list,
            items,
        })
    }

    /// A stage that forwards every current binding unchanged.
    /// Binds a MATCH that follows a mutation clause as a staged join: the
    /// pattern binds into a standalone plan whose rows cross-join the
    /// current row set at execution. Correlated patterns (variables already
    /// in scope) would need per-row execution and stay unsupported.
    /// Binds a MATCH that follows a mutation clause as a staged join: the
    /// pattern binds into a standalone plan run against each current row.
    /// Variables already in scope correlate by identity: their pattern
    /// occurrences rebind under fresh names filtered against internal
    /// reference parameters the executor supplies per row.
    fn bind_staged_match(
        &mut self,
        clause: &cypher::MatchClause,
        span: cypher::Span,
    ) -> Result<StageItem, BindError> {
        for path in &clause.paths {
            if path.variable.is_some() {
                return Err(at_unsupported(
                    span,
                    "named paths in MATCH after a mutation clause",
                ));
            }
        }
        // Correlated variables: pattern variables already bound in scope.
        let mut correlated: Vec<(String, ir::BindingId)> = Vec::new();
        for path in &clause.paths {
            let mut variables = Vec::new();
            if let Some(variable) = &path.start.variable {
                variables.push(variable);
            }
            for (relationship, node) in &path.steps {
                if let Some(variable) = &relationship.variable {
                    variables.push(variable);
                }
                if let Some(variable) = &node.variable {
                    variables.push(variable);
                }
            }
            for variable in variables {
                if correlated.iter().any(|(name, _)| name == &variable.value) {
                    continue;
                }
                if let Some(binding) = self
                    .scope
                    .iter()
                    .find(|binding| binding.name() == variable.value)
                {
                    correlated.push((variable.value.clone(), binding.id()));
                }
            }
        }
        let rename: std::collections::HashMap<String, String> = correlated
            .iter()
            .map(|(name, id)| (name.clone(), format!("__corr_{}_{name}", id.get())))
            .collect();
        let renamed = rename_match_clause(clause, &rename);
        let clause = if rename.is_empty() { clause } else { &renamed };
        // In-scope variables referenced without appearing in the pattern
        // have no column in the staged plan and stay unsupported.
        if let Some(predicate) = &clause.predicate {
            let outer_names: Vec<String> = self
                .scope
                .iter()
                .map(|binding| binding.name().to_owned())
                .collect();
            if names_referenced(predicate, &outer_names) {
                return Err(at_unsupported(
                    predicate.span,
                    "outer variable references in a staged MATCH predicate",
                ));
            }
        }

        let outer = self.plan.take();
        let before = self.scope.len();
        let optional = clause.optional;
        let inner = cypher::MatchClause {
            optional: false,
            paths: clause.paths.clone(),
            predicate: clause.predicate.clone(),
        };
        self.bind_match(&inner, span)?;
        // Correlate: each renamed binding must equal its outer identity,
        // delivered per row through an internal reference parameter.
        for (name, outer_id) in &correlated {
            let fresh_name = &rename[name];
            let fresh = self
                .scope
                .iter()
                .find(|binding| binding.name() == fresh_name)
                .cloned()
                .ok_or(BindError::EmptyQuery)?;
            let parameter = format!(
                "{}{}",
                crate::mutation::INTERNAL_PARAMETER_PREFIX,
                outer_id.get()
            );
            let predicate = ir::TypedExpression {
                expression: ir::Expression::Binary {
                    left: Box::new(ir::TypedExpression {
                        expression: ir::Expression::Binding(fresh.id()),
                        value_type: fresh.value_type().clone(),
                        nullability: fresh.nullability(),
                    }),
                    op: ir::BinaryOp::Equal,
                    right: Box::new(ir::TypedExpression {
                        expression: ir::Expression::Parameter(parameter),
                        value_type: ir::ValueType::Integer,
                        nullability: ir::Nullability::NonNull,
                    }),
                },
                value_type: ir::ValueType::Boolean,
                nullability: ir::Nullability::NonNull,
            };
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                input: Box::new(input),
                predicate,
            }))?;
        }
        let plan = self.plan.take().ok_or(BindError::EmptyQuery)?;
        self.plan = outer;
        let outputs = self.scope[before..].iter().map(ir::Binding::id).collect();
        Ok(StageItem::Match {
            plan: Box::new(plan),
            outputs,
            optional,
        })
    }

    fn passthrough_stage(&self) -> MutationStage {
        MutationStage {
            projections: self
                .scope
                .iter()
                .map(|binding| StageProjection::Expression {
                    output: binding.id(),
                    expression: ir::TypedExpression {
                        expression: ir::Expression::Binding(binding.id()),
                        value_type: binding.value_type().clone(),
                        nullability: binding.nullability(),
                    },
                })
                .collect(),
            predicate: None,
            distinct: false,
            items: Vec::new(),
            order: Vec::new(),
            skip: None,
            limit: None,
        }
    }

    fn bind_mutation_with(
        &mut self,
        clause: &cypher::ProjectionClause,
    ) -> Result<MutationStage, BindError> {
        let mut projections = Vec::new();
        let mut output_scope: Vec<ir::Binding> = Vec::new();
        let mut output_entities = HashMap::new();
        for item in &clause.items {
            match item {
                cypher::ProjectionItem::All(span) => {
                    for binding in &self.scope {
                        projections.push(StageProjection::Expression {
                            output: binding.id(),
                            expression: self.scope_binding_expression(binding, *span)?,
                        });
                        output_scope.push(binding.clone());
                        if let Some(entity) = self.entities.get(&binding.id()) {
                            output_entities.insert(binding.id(), entity.clone());
                        }
                    }
                }
                cypher::ProjectionItem::Expression { expression, alias } => {
                    if alias.is_none()
                        && !matches!(expression.value, cypher::Expression::Variable(_))
                    {
                        return Err(at_unsupported(
                            expression.span,
                            "unaliased WITH expressions",
                        ));
                    }
                    let name = alias
                        .as_ref()
                        .map(|alias| alias.value.clone())
                        .unwrap_or_else(|| projection_name(expression));
                    if output_scope.iter().any(|binding| binding.name() == name) {
                        let span = alias.as_ref().map_or(expression.span, |alias| alias.span);
                        return Err(BindError::DuplicateVariable {
                            name,
                            span_start: span.start,
                            span_end: span.end,
                        });
                    }
                    if let Some((function, argument, distinct)) =
                        self.bind_aggregate_call(expression)?
                    {
                        let value_type = match (&function, &argument) {
                            (ir::AggregateFunction::Count, _) => ir::ValueType::Integer,
                            (ir::AggregateFunction::Average, _) => ir::ValueType::Real,
                            (ir::AggregateFunction::Collect, Some(argument)) => {
                                ir::ValueType::List(Box::new(argument.value_type.clone()))
                            }
                            (_, Some(argument)) => argument.value_type.clone(),
                            (_, None) => ir::ValueType::Any,
                        };
                        let output = ir::Binding::new(
                            self.next_id()?,
                            name,
                            value_type,
                            ir::Nullability::Nullable,
                        )?;
                        projections.push(StageProjection::Aggregate {
                            output: output.id(),
                            function,
                            argument,
                            distinct,
                        });
                        output_scope.push(output);
                        continue;
                    }
                    let bound = self.bind_expression(expression)?;
                    let old_entity = match &expression.value {
                        cypher::Expression::Variable(name) => self
                            .scope
                            .iter()
                            .find(|binding| binding.name() == name)
                            .and_then(|binding| self.entities.get(&binding.id()).cloned()),
                        _ => None,
                    };
                    let output = ir::Binding::new(
                        self.next_id()?,
                        name,
                        bound.value_type.clone(),
                        bound.nullability,
                    )?;
                    if let Some(entity) = old_entity {
                        output_entities.insert(output.id(), entity);
                    }
                    projections.push(StageProjection::Expression {
                        output: output.id(),
                        expression: bound,
                    });
                    output_scope.push(output);
                }
            }
        }
        self.scope = output_scope;
        self.entities = output_entities;
        self.rematerialize_path_compositions();
        let predicate = clause
            .predicate
            .as_ref()
            .map(|predicate| self.bind_expression(predicate))
            .transpose()?;
        // ORDER BY, SKIP, and LIMIT are bound against the stage's own output
        // scope, so they see this WITH's aliases the same way a following
        // clause would (and lose access to anything the WITH didn't carry
        // forward, matching Cypher's WITH scoping).
        let order = clause
            .order_by
            .iter()
            .map(|sort| Ok((self.bind_expression(&sort.expression)?, sort.descending)))
            .collect::<Result<Vec<_>, BindError>>()?;
        let skip = bind_constant_count(
            clause.skip.as_ref(),
            "non-constant SKIP/LIMIT on a mutation WITH clause",
        )?;
        let limit = bind_constant_count(
            clause.limit.as_ref(),
            "non-constant SKIP/LIMIT on a mutation WITH clause",
        )?;
        Ok(MutationStage {
            projections,
            predicate,
            distinct: clause.distinct,
            items: Vec::new(),
            order,
            skip,
            limit,
        })
    }

    fn bind_create_path(
        &mut self,
        path: &cypher::PathPattern,
        merge: bool,
        operations: &mut Vec<ir::Mutation>,
    ) -> Result<(), BindError> {
        if merge {
            // openCypher: MERGE on a null property can never match nor
            // create (TCK Merge1 [17], Merge5 [29]).
            let null_property = path
                .start
                .properties
                .iter()
                .chain(path.steps.iter().flat_map(|(relationship, node)| {
                    relationship.properties.iter().chain(node.properties.iter())
                }))
                .find(|(_, value)| {
                    matches!(
                        value.value,
                        cypher::Expression::Literal(cypher::Literal::Null)
                    )
                });
            if let Some((_, value)) = null_property {
                return Err(at_unsupported(
                    value.span,
                    "MERGE with null property values",
                ));
            }
        }
        if let Some(variable) = &path.variable {
            // A path variable cannot rebind a name already in scope
            // (openCypher VariableTypeConflict); erroring here keeps the
            // scope invariant from tripping downstream.
            if self
                .scope
                .iter()
                .any(|binding| binding.name() == variable.value)
            {
                return Err(BindError::DuplicateVariable {
                    name: variable.value.clone(),
                    span_start: variable.span.start,
                    span_end: variable.span.end,
                });
            }
            // Register the path name so later clauses can reference it. The
            // composition (below) records how to rebuild the path value from
            // the node/relationship bindings created along the way, mirroring
            // the read-side `bind_match`/`bind_path`.
            let binding = ir::Binding::new(
                self.next_id()?,
                variable.value.clone(),
                ir::ValueType::Path,
                ir::Nullability::NonNull,
            )?;
            self.scope.push(binding);
        }
        let mut from = self.bind_created_node(&path.start, merge, operations)?;
        let mut path_nodes = vec![from];
        let mut path_relationships = Vec::new();
        for (relationship, node) in &path.steps {
            if relationship.range.is_some() {
                return Err(at_unsupported(
                    relationship.span,
                    "variable-length mutation relationships",
                ));
            }
            // Cypher defines MERGE over an undirected relationship: it may
            // match either direction and creates an outgoing one. Plain
            // CREATE still requires an explicit direction.
            if relationship.direction == cypher::Direction::Both && !merge {
                return Err(at_unsupported(
                    relationship.span,
                    "undirected relationship creation",
                ));
            }
            // openCypher requires exactly one relationship type when
            // creating or merging a relationship (TCK Merge5 [24]/[25]).
            if relationship.types.len() != 1 {
                return Err(at_unsupported(
                    relationship.span,
                    "relationship creation without exactly one type",
                ));
            }
            let to = self.bind_created_node(node, merge, operations)?;
            path_nodes.push(to);
            let source = self.relationship_source(relationship.span)?;
            let binding = self.new_entity_binding(
                relationship.variable.as_ref(),
                "_relationship",
                ir::ValueType::Relationship,
                ir::Nullability::NonNull,
                CatalogEntity::Relationship,
                relationship
                    .types
                    .iter()
                    .map(|name| name.value.clone())
                    .collect(),
                relationship.span,
            )?;
            path_relationships.push(binding.id());
            let relationship_types = relationship
                .types
                .iter()
                .map(|name| {
                    self.catalog
                        .relationship_type(self.graph, &name.value)
                        .ok_or_else(|| BindError::UnknownRelationshipType {
                            name: name.value.clone(),
                            span_start: name.span.start,
                            span_end: name.span.end,
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let properties = self
                .bind_mutation_properties(CatalogEntity::Relationship, &relationship.properties)?;
            let next_from = to;
            let (relationship_from, relationship_to) =
                if relationship.direction == cypher::Direction::Incoming {
                    (to, from)
                } else {
                    (from, to)
                };
            let create = ir::CreateRelationship {
                binding,
                source,
                from: relationship_from,
                to: relationship_to,
                direction: ir::Direction::Outgoing,
                relationship_types,
                properties,
            };
            operations.push(if merge {
                ir::Mutation::MergeRelationship(ir::MergeRelationship {
                    create,
                    on_create: Vec::new(),
                    on_match: Vec::new(),
                })
            } else {
                ir::Mutation::CreateRelationship(create)
            });
            from = next_from;
        }
        if let Some(variable) = &path.variable {
            self.path_compositions.insert(
                variable.value.clone(),
                PathComposition::Fixed {
                    nodes: path_nodes,
                    relationships: path_relationships,
                },
            );
        }
        Ok(())
    }

    /// Binds ON CREATE / ON MATCH SET actions and attaches them to the
    /// clause's deciding merge operation (the last one: for relationship
    /// merges, the relationship's existence decides matched-vs-created).
    fn attach_merge_actions(
        &mut self,
        clause: &cypher::MergeClause,
        operations: &mut [ir::Mutation],
    ) -> Result<(), BindError> {
        if clause.on_create.is_empty() && clause.on_match.is_empty() {
            return Ok(());
        }
        let on_create = clause
            .on_create
            .iter()
            .map(|item| self.bind_set_item(item))
            .collect::<Result<Vec<_>, _>>()?;
        let on_match = clause
            .on_match
            .iter()
            .map(|item| self.bind_set_item(item))
            .collect::<Result<Vec<_>, _>>()?;
        for operation in operations.iter_mut().rev() {
            match operation {
                ir::Mutation::MergeNode(merge) => {
                    merge.on_create = on_create;
                    merge.on_match = on_match;
                    return Ok(());
                }
                ir::Mutation::MergeRelationship(merge) => {
                    merge.on_create = on_create;
                    merge.on_match = on_match;
                    return Ok(());
                }
                _ => {}
            }
        }
        Err(at_unsupported(
            clause.path.span,
            "ON CREATE/ON MATCH without a merge operation",
        ))
    }

    fn bind_created_node(
        &mut self,
        node: &cypher::NodePattern,
        merge: bool,
        operations: &mut Vec<ir::Mutation>,
    ) -> Result<ir::BindingId, BindError> {
        if let Some(variable) = &node.variable {
            if let Some(binding) = self
                .scope
                .iter()
                .find(|binding| binding.name() == variable.value)
                .cloned()
            {
                if !node.labels.is_empty() || node.has_property_map {
                    return Err(at_unsupported(
                        node.span,
                        "labels or properties on an already-bound CREATE node",
                    ));
                }
                return Ok(binding.id());
            }
        }
        let source = self.node_source(node.span)?;
        let binding = self.new_entity_binding(
            node.variable.as_ref(),
            "_node",
            ir::ValueType::Node,
            ir::Nullability::NonNull,
            CatalogEntity::Node,
            node.labels
                .iter()
                .map(|label| label.value.clone())
                .collect(),
            node.span,
        )?;
        let create = ir::CreateNode {
            binding: binding.clone(),
            source,
            labels: self.resolve_labels(node)?,
            properties: self.bind_mutation_properties(CatalogEntity::Node, &node.properties)?,
        };
        operations.push(if merge {
            ir::Mutation::MergeNode(ir::MergeNode {
                create,
                on_create: Vec::new(),
                on_match: Vec::new(),
            })
        } else {
            ir::Mutation::CreateNode(create)
        });
        Ok(binding.id())
    }

    fn bind_set(
        &mut self,
        clause: &cypher::SetClause,
        operations: &mut Vec<ir::Mutation>,
    ) -> Result<(), BindError> {
        for item in &clause.items {
            let operation = self.bind_set_item(item)?;
            operations.push(operation);
        }
        Ok(())
    }

    fn bind_set_item(&mut self, item: &cypher::SetItem) -> Result<ir::Mutation, BindError> {
        match item {
            cypher::SetItem::Property { target, value } => {
                if contains_pattern_expression(value) {
                    return Err(at_unsupported(
                        value.span,
                        "patterns in a SET value expression",
                    ));
                }
                let (binding, kind, source) = self.resolve_mutation_target(target)?;
                let property = self.resolve_property(kind, &target.property)?;
                let bound = match &value.value {
                    // Map values bind against struct/union property targets;
                    // otherwise they store as JSON like any other map value.
                    cypher::Expression::Map(entries)
                        if matches!(
                            property.value_type,
                            ir::ValueType::Struct(_) | ir::ValueType::Union(_)
                        ) =>
                    {
                        self.bind_map_property(
                            &property.value_type,
                            property.nullability,
                            entries,
                            value.span,
                        )?
                    }
                    _ => self.bind_expression(value)?,
                };
                Ok(ir::Mutation::SetProperty(ir::SetProperty {
                    entity: binding,
                    source,
                    property: property.id,
                    value: bound,
                }))
            }
            cypher::SetItem::Labels { variable, labels } => {
                let (binding, kind) = self.resolve_set_variable(variable)?;
                if kind != CatalogEntity::Node {
                    return Err(at_unsupported(variable.span, "labels on a non-node entity"));
                }
                let labels = labels
                    .iter()
                    .map(|label| {
                        self.catalog.label(self.graph, &label.value).ok_or_else(|| {
                            BindError::UnknownLabel {
                                name: label.value.clone(),
                                span_start: label.span.start,
                                span_end: label.span.end,
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ir::Mutation::SetLabels(ir::SetLabels {
                    entity: binding,
                    labels,
                }))
            }
            cypher::SetItem::ReplaceEntity { variable, value }
            | cypher::SetItem::MergeEntity { variable, value } => {
                let clear = matches!(item, cypher::SetItem::ReplaceEntity { .. });
                let (binding, kind) = self.resolve_set_variable(variable)?;
                let source = self.entity_source(kind, variable.span)?;
                if let cypher::Expression::Map(entries) = &value.value {
                    let entries = self.bind_mutation_properties(kind, entries)?;
                    return Ok(ir::Mutation::ReplaceProperties(ir::ReplaceProperties {
                        entity: binding,
                        source,
                        entries,
                        clear,
                    }));
                }
                // Map-shaped expressions (properties(m), parameters) update
                // every payload column from the evaluated JSON value.
                let bound = self.bind_expression(value)?;
                if !matches!(bound.value_type, ir::ValueType::Map | ir::ValueType::Any) {
                    return Err(at_unsupported(
                        value.span,
                        "SET of a whole entity from a non-map value",
                    ));
                }
                Ok(ir::Mutation::ReplacePropertiesDynamic(
                    ir::ReplacePropertiesDynamic {
                        entity: binding,
                        source,
                        value: bound,
                        clear,
                    },
                ))
            }
        }
    }

    fn resolve_set_variable(
        &self,
        variable: &cypher::Spanned<String>,
    ) -> Result<(ir::BindingId, CatalogEntity), BindError> {
        let binding = self.resolve_binding(&variable.value, variable.span)?;
        let kind = self
            .entities
            .get(&binding.id())
            .ok_or(BindError::InvalidPropertyTarget {
                span_start: variable.span.start,
                span_end: variable.span.end,
            })?
            .kind;
        Ok((binding.id(), kind))
    }

    fn bind_remove(
        &self,
        clause: &cypher::RemoveClause,
        operations: &mut Vec<ir::Mutation>,
    ) -> Result<(), BindError> {
        for item in &clause.items {
            let (binding, kind, source) = self.resolve_mutation_target(item)?;
            let property = self.resolve_property(kind, &item.property)?;
            operations.push(ir::Mutation::RemoveProperty(ir::RemoveProperty {
                entity: binding,
                source,
                property: property.id,
            }));
        }
        Ok(())
    }

    fn bind_delete(
        &self,
        clause: &cypher::DeleteClause,
        operations: &mut Vec<ir::Mutation>,
    ) -> Result<(), BindError> {
        for variable in &clause.variables {
            let binding = self.resolve_binding(&variable.value, variable.span)?;
            let kind = self
                .entities
                .get(&binding.id())
                .ok_or(BindError::InvalidPropertyTarget {
                    span_start: variable.span.start,
                    span_end: variable.span.end,
                })?
                .kind;
            let source = self.entity_source(kind, variable.span)?;
            operations.push(ir::Mutation::Delete(ir::DeleteEntity {
                entity: binding.id(),
                source,
                detach: clause.detach,
            }));
        }
        Ok(())
    }

    fn bind_mutation_properties(
        &mut self,
        entity: CatalogEntity,
        properties: &[(cypher::Spanned<String>, cypher::Spanned<cypher::Expression>)],
    ) -> Result<Vec<ir::PropertyValue>, BindError> {
        properties
            .iter()
            .map(|(name, value)| {
                let resolved = self.resolve_property(entity, name)?;
                let bound_value = match &value.value {
                    cypher::Expression::Map(entries)
                        if matches!(
                            resolved.value_type,
                            ir::ValueType::Struct(_) | ir::ValueType::Union(_)
                        ) =>
                    {
                        self.bind_map_property(
                            &resolved.value_type,
                            resolved.nullability,
                            entries,
                            value.span,
                        )?
                    }
                    _ => self.bind_expression(value)?,
                };
                Ok(ir::PropertyValue {
                    property: resolved.id,
                    value: bound_value,
                })
            })
            .collect()
    }

    fn resolve_mutation_target(
        &self,
        target: &cypher::PropertyTarget,
    ) -> Result<(ir::BindingId, CatalogEntity, ir::SourceTableId), BindError> {
        let binding = self.resolve_binding(&target.variable.value, target.variable.span)?;
        let kind = self
            .entities
            .get(&binding.id())
            .ok_or(BindError::InvalidPropertyTarget {
                span_start: target.variable.span.start,
                span_end: target.variable.span.end,
            })?
            .kind;
        Ok((
            binding.id(),
            kind,
            self.entity_source(kind, target.variable.span)?,
        ))
    }

    fn entity_source(
        &self,
        kind: CatalogEntity,
        span: cypher::Span,
    ) -> Result<ir::SourceTableId, BindError> {
        match kind {
            CatalogEntity::Node => self.node_source(span),
            CatalogEntity::Relationship => self.relationship_source(span),
        }
    }

    fn node_source(&self, span: cypher::Span) -> Result<ir::SourceTableId, BindError> {
        self.catalog
            .node_source(self.graph)
            .ok_or(BindError::MissingSource {
                entity: "node",
                span_start: span.start,
                span_end: span.end,
            })
    }

    fn relationship_source(&self, span: cypher::Span) -> Result<ir::SourceTableId, BindError> {
        self.catalog
            .relationship_source(self.graph)
            .ok_or(BindError::MissingSource {
                entity: "relationship",
                span_start: span.start,
                span_end: span.end,
            })
    }

    fn bind_unwind(&mut self, clause: &cypher::UnwindClause) -> Result<(), BindError> {
        if self
            .scope
            .iter()
            .any(|binding| binding.name() == clause.alias.value)
        {
            return Err(BindError::DuplicateVariable {
                name: clause.alias.value.clone(),
                span_start: clause.alias.span.start,
                span_end: clause.alias.span.end,
            });
        }
        let list = self.bind_expression(&clause.expression)?;
        let value_type = match &list.value_type {
            ir::ValueType::List(element) => (**element).clone(),
            _ => ir::ValueType::Any,
        };
        let nullability = match &list.expression {
            ir::Expression::List(values)
                if values
                    .iter()
                    .all(|value| value.nullability == ir::Nullability::NonNull) =>
            {
                ir::Nullability::NonNull
            }
            _ => ir::Nullability::Nullable,
        };
        let output = ir::Binding::new(
            self.next_id()?,
            clause.alias.value.clone(),
            value_type,
            nullability,
        )?;
        let input = match self.plan.take() {
            Some(input) => input,
            None => ir::Plan::new(
                ir::PlanKind::Unit(ir::Unit),
                ir::Scope::default(),
                ir::ResultShape::default(),
            )?,
        };
        self.scope.push(output.clone());
        self.plan = Some(ir::Plan::new(
            ir::PlanKind::Unwind(ir::Unwind {
                input: Box::new(input),
                list,
                output,
            }),
            ir::Scope::new(self.scope.clone())?,
            ir::ResultShape::default(),
        )?);
        Ok(())
    }

    /// Selectivity score for a pattern node: a constant property filter is
    /// most selective (a single-row lookup), labels alone narrow a scan,
    /// and a bare node is least selective (a full scan).
    fn node_selectivity_score(node: &cypher::NodePattern) -> u8 {
        let has_constant_property = node
            .properties
            .iter()
            .any(|(_, value)| matches!(value.value, cypher::Expression::Literal(_)));
        if has_constant_property {
            2
        } else if !node.labels.is_empty() {
            1
        } else {
            0
        }
    }

    /// Whether `path` is safe to bind in reverse and the last node is a
    /// strictly better anchor than the first. See the MVP restrictions in
    /// the module doc for why named paths, variable-length hops, and
    /// bidirectional relationships are excluded: each has binding-order or
    /// direction semantics that a pure AST-level reversal must not disturb.
    ///
    /// Also requires every node and relationship variable in the path to be
    /// fresh (not already bound by an earlier clause or an earlier path in
    /// this same MATCH). `bind_start_node` treats an already-bound start
    /// specially: it resumes the existing plan instead of scanning, which is
    /// how OPTIONAL MATCH and same-clause re-matching correlate. Reversing
    /// such a path would move the already-bound variable into a step
    /// position instead, trading that continuation for a fresh scan plus an
    /// equality filter — a corpus regression (TCK Match7 [23] and similar
    /// `OPTIONAL MATCH (a)-->(b:Label)` shapes) showed this breaks the
    /// correlated LeftApply's null-preserving semantics.
    fn should_reverse_path(&self, path: &cypher::PathPattern) -> bool {
        if path.variable.is_some() || path.steps.is_empty() {
            return false;
        }
        if path
            .steps
            .iter()
            .any(|(relationship, _)| relationship.range.is_some())
        {
            return false;
        }
        if path
            .steps
            .iter()
            .any(|(relationship, _)| relationship.direction == cypher::Direction::Both)
        {
            return false;
        }
        let already_bound = |name: &str| self.scope.iter().any(|binding| binding.name() == name);
        if path
            .start
            .variable
            .as_ref()
            .is_some_and(|variable| already_bound(&variable.value))
        {
            return false;
        }
        if path.steps.iter().any(|(relationship, node)| {
            relationship
                .variable
                .as_ref()
                .is_some_and(|variable| already_bound(&variable.value))
                || node
                    .variable
                    .as_ref()
                    .is_some_and(|variable| already_bound(&variable.value))
        }) {
            return false;
        }
        let first_score = Self::node_selectivity_score(&path.start);
        let last_score = Self::node_selectivity_score(&path.steps[path.steps.len() - 1].1);
        last_score > first_score
    }

    /// Rebuilds `path` walked from its last node to its first, flipping each
    /// relationship's direction so the reversed pattern matches exactly the
    /// same rows (Cypher `MATCH` is declarative; only join order changes).
    /// Callers must have already checked [`Self::should_reverse_path`].
    fn reverse_path(path: &cypher::PathPattern) -> cypher::PathPattern {
        let mut node_chain = Vec::with_capacity(path.steps.len() + 1);
        node_chain.push(path.start.clone());
        node_chain.extend(path.steps.iter().map(|(_, node)| node.clone()));
        let flip_direction = |direction: cypher::Direction| match direction {
            cypher::Direction::Outgoing => cypher::Direction::Incoming,
            cypher::Direction::Incoming => cypher::Direction::Outgoing,
            cypher::Direction::Both => cypher::Direction::Both,
        };
        let new_start = node_chain
            .last()
            .expect("node_chain always has start plus one node per step")
            .clone();
        let new_steps = path
            .steps
            .iter()
            .enumerate()
            .rev()
            .map(|(index, (relationship, _))| {
                let mut relationship = relationship.clone();
                relationship.direction = flip_direction(relationship.direction);
                (relationship, node_chain[index].clone())
            })
            .collect();
        cypher::PathPattern {
            variable: path.variable.clone(),
            start: new_start,
            steps: new_steps,
            span: path.span,
        }
    }

    fn bind_match(
        &mut self,
        clause: &cypher::MatchClause,
        fallback: cypher::Span,
    ) -> Result<(), BindError> {
        if clause.optional && clause.paths.len() != 1 {
            return Err(at_unsupported(fallback, "multiple OPTIONAL MATCH paths"));
        }
        for path in &clause.paths {
            if let Some(variable) = &path.variable {
                // A path variable cannot rebind a name already in scope
                // (openCypher VariableTypeConflict).
                if self
                    .scope
                    .iter()
                    .any(|binding| binding.name() == variable.value)
                {
                    return Err(BindError::DuplicateVariable {
                        name: variable.value.clone(),
                        span_start: variable.span.start,
                        span_end: variable.span.end,
                    });
                }
                // Register the path name so later clauses can reference it.
                let binding = ir::Binding::new(
                    self.next_id()?,
                    variable.value.clone(),
                    ir::ValueType::Path,
                    ir::Nullability::NonNull,
                )?;
                self.scope.push(binding);
            }
            if clause.optional
                && path
                    .steps
                    .iter()
                    .any(|(relationship, _)| relationship.range.is_some())
            {
                return Err(at_unsupported(
                    path.span,
                    "optional variable-length relationships",
                ));
            }
        }
        let left = self.plan.clone();
        let old_ids: Vec<_> = self.scope.iter().map(ir::Binding::id).collect();
        for path in &clause.paths {
            let path_binding = path.variable.as_ref().and_then(|variable| {
                self.scope
                    .iter()
                    .find(|binding| binding.name() == variable.value)
                    .cloned()
            });
            // Anchor the join at whichever end is more selective: a scan
            // that starts from a constant-property lookup is far cheaper
            // than one that starts from a bare label scan and only applies
            // the selective filter after every hop.
            let reversed = self
                .should_reverse_path(path)
                .then(|| Self::reverse_path(path));
            let bound_path = reversed.as_ref().unwrap_or(path);
            let composition = self.bind_path(bound_path, path_binding.as_ref(), old_ids.len())?;
            if let Some(variable) = &path.variable {
                self.path_compositions
                    .insert(variable.value.clone(), composition);
            }
        }
        if let Some(predicate) = &clause.predicate {
            let predicate = self.bind_expression(predicate)?;
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                input: Box::new(input),
                predicate,
            }))?;
        }
        if clause.optional {
            // OPTIONAL MATCH as the first clause behaves like matching over
            // a single empty input row.
            let left = match left {
                Some(left) => left,
                None => ir::Plan::new(
                    ir::PlanKind::Unit(ir::Unit),
                    ir::Scope::default(),
                    ir::ResultShape::default(),
                )?,
            };
            let right = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.scope = self
                .scope
                .iter()
                .map(|binding| {
                    if old_ids.contains(&binding.id()) {
                        Ok(binding.clone())
                    } else {
                        ir::Binding::new(
                            binding.id(),
                            binding.name(),
                            binding.value_type().clone(),
                            ir::Nullability::Nullable,
                        )
                    }
                })
                .collect::<Result<_, _>>()?;
            let scope = ir::Scope::new(self.scope.clone())?;
            self.plan = Some(ir::Plan::new(
                ir::PlanKind::LeftApply(ir::LeftApply {
                    left: Box::new(left),
                    right: Box::new(right),
                    correlated: old_ids,
                }),
                scope,
                ir::ResultShape::default(),
            )?);
        }
        Ok(())
    }

    fn bind_path(
        &mut self,
        path: &cypher::PathPattern,
        path_binding: Option<&ir::Binding>,
        preexisting: usize,
    ) -> Result<PathComposition, BindError> {
        let start = self.bind_start_node(&path.start)?;
        let mut nodes = vec![start];
        let mut relationships = Vec::new();
        let mut variable_length = false;
        let expand_path =
            path_binding.filter(|_| path.steps.len() == 1 && path.steps[0].0.range.is_some());
        let mut from = start;
        let mut list_equality: Option<(ir::Binding, ir::Binding)> = None;
        for (relationship, node) in &path.steps {
            let mut relationship_list = None;
            if relationship.range.is_some() {
                variable_length = true;
                if let Some(variable) = &relationship.variable {
                    if let Some(existing) = self
                        .scope
                        .iter()
                        .find(|binding| binding.name() == variable.value)
                        .cloned()
                    {
                        // A bound list reused in a variable-length pattern
                        // constrains the traversal: materialize the walked
                        // relationship list anonymously and require it to
                        // equal the bound value (TCK Match4 [8]).
                        if !matches!(existing.value_type(), ir::ValueType::List(_)) {
                            return Err(BindError::DuplicateVariable {
                                name: variable.value.clone(),
                                span_start: variable.span.start,
                                span_end: variable.span.end,
                            });
                        }
                        let id = self.next_id()?;
                        let binding = ir::Binding::new(
                            id,
                            format!("__rellist_{}", id.get()),
                            ir::ValueType::List(Box::new(ir::ValueType::Relationship)),
                            ir::Nullability::Nullable,
                        )?;
                        relationship_list = Some(binding.clone());
                        list_equality = Some((binding, existing));
                    } else {
                        // A named variable-length relationship denotes the
                        // list of traversed relationships; the expansion
                        // materializes it as a grouped output while staying
                        // anonymous itself.
                        let binding = ir::Binding::new(
                            self.next_id()?,
                            variable.value.clone(),
                            ir::ValueType::List(Box::new(ir::ValueType::Relationship)),
                            ir::Nullability::Nullable,
                        )?;
                        self.scope.push(binding.clone());
                        relationship_list = Some(binding);
                    }
                }
            }
            if relationship.range.is_some() && !relationship.properties.is_empty() {
                return Err(at_unsupported(
                    relationship.span,
                    "variable-length relationship properties",
                ));
            }
            // A relationship variable bound by an EARLIER clause re-matched
            // here is an equality constraint (TCK Match3); reuse inside the
            // same clause keeps raising the duplicate-variable error.
            let reused_relationship = if relationship.range.is_none() {
                relationship.variable.as_ref().and_then(|variable| {
                    self.scope
                        .iter()
                        .position(|binding| binding.name() == variable.value)
                        .filter(|index| *index < preexisting)
                        .map(|index| self.scope[index].clone())
                })
            } else {
                None
            };
            if let Some(existing) = &reused_relationship {
                if self.entities.get(&existing.id()).map(|entity| entity.kind)
                    != Some(CatalogEntity::Relationship)
                {
                    return Err(at_unsupported(
                        relationship.span,
                        "reusing a non-relationship variable in a relationship pattern",
                    ));
                }
            }
            let relationship_binding = self.new_entity_binding(
                if relationship.range.is_some() || reused_relationship.is_some() {
                    None
                } else {
                    relationship.variable.as_ref()
                },
                "_relationship",
                ir::ValueType::Relationship,
                if relationship
                    .range
                    .as_ref()
                    .is_some_and(|range| range.value.min == Some(0))
                {
                    ir::Nullability::Nullable
                } else {
                    ir::Nullability::NonNull
                },
                CatalogEntity::Relationship,
                relationship
                    .types
                    .iter()
                    .map(|name| name.value.clone())
                    .collect(),
                relationship.span,
            )?;
            // A step node whose variable is already bound closes a cycle:
            // expand into an anonymous target and equate identities below.
            let reused = node.variable.as_ref().and_then(|variable| {
                self.scope
                    .iter()
                    .find(|binding| binding.name() == variable.value)
                    .cloned()
            });
            if let Some(existing) = &reused {
                if self.entities.get(&existing.id()).map(|entity| entity.kind)
                    != Some(CatalogEntity::Node)
                {
                    return Err(at_unsupported(
                        node.span,
                        "reusing a non-node variable in a node pattern",
                    ));
                }
            }
            let to = self.new_entity_binding(
                if reused.is_some() {
                    None
                } else {
                    node.variable.as_ref()
                },
                "_node",
                ir::ValueType::Node,
                ir::Nullability::NonNull,
                CatalogEntity::Node,
                node.labels
                    .iter()
                    .map(|label| label.value.clone())
                    .collect(),
                node.span,
            )?;
            let relationship_types = relationship
                .types
                .iter()
                .map(|name| {
                    self.catalog
                        .relationship_type(self.graph, &name.value)
                        .ok_or_else(|| BindError::UnknownRelationshipType {
                            name: name.value.clone(),
                            span_start: name.span.start,
                            span_end: name.span.end,
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let source =
                self.catalog
                    .relationship_source(self.graph)
                    .ok_or(BindError::MissingSource {
                        entity: "relationship",
                        span_start: relationship.span.start,
                        span_end: relationship.span.end,
                    })?;
            let target_node_source =
                self.catalog
                    .node_source(self.graph)
                    .ok_or(BindError::MissingSource {
                        entity: "node",
                        span_start: node.span.start,
                        span_end: node.span.end,
                    })?;
            let direction = match relationship.direction {
                cypher::Direction::Outgoing => ir::Direction::Outgoing,
                cypher::Direction::Incoming => ir::Direction::Incoming,
                cypher::Direction::Both => ir::Direction::Both,
            };
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            let scope = ir::Scope::new(self.scope.clone())?;
            // Cycle-closing without target properties folds the identity
            // equality into the relationship join (composite endpoint
            // indexes apply); targets with property maps still need the
            // node join, so they keep the post-expand filter.
            let bound_target = match (&reused, &relationship.range) {
                (Some(existing), None) if node.properties.is_empty() => Some(existing.id()),
                _ => None,
            };
            let kind = if let Some(range) = &relationship.range {
                let min_hops = range.value.min.unwrap_or(1);
                let unbounded = range.value.max.is_none();
                let max_hops = range
                    .value
                    .max
                    .unwrap_or_else(|| DEFAULT_UNBOUNDED_MAX_HOPS.max(min_hops));
                if min_hops > max_hops {
                    return Err(BindError::InvalidRelationshipRange {
                        min: min_hops,
                        max: max_hops,
                        span_start: range.span.start,
                        span_end: range.span.end,
                    });
                }
                ir::PlanKind::GraphExpand(ir::GraphExpand {
                    input: Box::new(input),
                    graph: self.graph,
                    relationship_source: source,
                    target_node_source,
                    from,
                    relationship: relationship_binding.clone(),
                    to: to.clone(),
                    direction,
                    relationship_types,
                    min_hops,
                    max_hops,
                    unbounded,
                    uniqueness: ir::PathUniqueness::Trail,
                    path_output: expand_path.cloned(),
                    relationship_list_output: relationship_list,
                })
            } else {
                ir::PlanKind::FixedExpand(ir::FixedExpand {
                    input: Box::new(input),
                    relationship_source: source,
                    target_node_source,
                    from,
                    relationship: relationship_binding.clone(),
                    to: to.clone(),
                    direction,
                    relationship_types,
                    bound_target,
                })
            };
            self.plan = Some(ir::Plan::new(kind, scope, ir::ResultShape::default())?);
            self.bind_properties(
                &relationship_binding,
                CatalogEntity::Relationship,
                &relationship.properties,
            )?;
            self.enforce_labels(to.id(), node)?;
            self.bind_properties(&to, CatalogEntity::Node, &node.properties)?;
            let equality = |left: &ir::Binding, right: &ir::Binding| ir::TypedExpression {
                expression: ir::Expression::Binary {
                    left: Box::new(ir::TypedExpression {
                        expression: ir::Expression::Binding(left.id()),
                        value_type: left.value_type().clone(),
                        nullability: left.nullability(),
                    }),
                    op: ir::BinaryOp::Equal,
                    right: Box::new(ir::TypedExpression {
                        expression: ir::Expression::Binding(right.id()),
                        value_type: right.value_type().clone(),
                        nullability: right.nullability(),
                    }),
                },
                value_type: ir::ValueType::Boolean,
                nullability: ir::Nullability::NonNull,
            };
            if let Some(existing) = &reused_relationship {
                let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
                self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                    input: Box::new(input),
                    predicate: equality(&relationship_binding, existing),
                }))?;
            }
            if let Some(existing) = reused {
                if bound_target.is_none() {
                    let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
                    self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                        input: Box::new(input),
                        predicate: equality(&to, &existing),
                    }))?;
                }
                from = existing.id();
            } else {
                from = to.id();
            }
            relationships.push(relationship_binding.id());
            nodes.push(from);
        }
        if let Some((materialized, bound)) = list_equality.take() {
            let side = |binding: &ir::Binding| ir::TypedExpression {
                expression: ir::Expression::Binding(binding.id()),
                value_type: binding.value_type().clone(),
                nullability: binding.nullability(),
            };
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                input: Box::new(input),
                predicate: ir::TypedExpression {
                    expression: ir::Expression::Binary {
                        left: Box::new(side(&materialized)),
                        op: ir::BinaryOp::Equal,
                        right: Box::new(side(&bound)),
                    },
                    value_type: ir::ValueType::Boolean,
                    nullability: ir::Nullability::NonNull,
                },
            }))?;
        }
        // Cypher relationship isomorphism: relationships within one MATCH
        // pattern bind pairwise-distinct edges (a co-membership pattern
        // must not reuse the anchor's own edge when both ends coincide).
        if !variable_length && relationships.len() > 1 {
            for i in 0..relationships.len() {
                for j in (i + 1)..relationships.len() {
                    let side = |id: ir::BindingId| ir::TypedExpression {
                        expression: ir::Expression::Binding(id),
                        value_type: ir::ValueType::Relationship,
                        nullability: ir::Nullability::NonNull,
                    };
                    let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
                    self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                        input: Box::new(input),
                        predicate: ir::TypedExpression {
                            expression: ir::Expression::Binary {
                                left: Box::new(side(relationships[i])),
                                op: ir::BinaryOp::NotEqual,
                                right: Box::new(side(relationships[j])),
                            },
                            value_type: ir::ValueType::Boolean,
                            nullability: ir::Nullability::NonNull,
                        },
                    }))?;
                }
            }
        }
        Ok(if !variable_length {
            PathComposition::Fixed {
                nodes,
                relationships,
            }
        } else if let Some(binding) = expand_path {
            PathComposition::Expanded(binding.id())
        } else {
            PathComposition::Unsupported
        })
    }

    fn bind_start_node(&mut self, node: &cypher::NodePattern) -> Result<ir::BindingId, BindError> {
        if let Some(variable) = &node.variable {
            if let Some(existing) = self
                .scope
                .iter()
                .find(|binding| binding.name() == variable.value)
                .cloned()
            {
                // openCypher raises VariableTypeConflict when a non-node
                // variable is reused in a node pattern (TCK Match1 [7]/[9]).
                if self.entities.get(&existing.id()).map(|entity| entity.kind)
                    != Some(CatalogEntity::Node)
                {
                    return Err(at_unsupported(
                        variable.span,
                        "reusing a non-node variable in a node pattern",
                    ));
                }
                self.enforce_labels(existing.id(), node)?;
                self.bind_properties(&existing, CatalogEntity::Node, &node.properties)?;
                return Ok(existing.id());
            }
        }
        let binding = self.new_entity_binding(
            node.variable.as_ref(),
            "_node",
            ir::ValueType::Node,
            ir::Nullability::NonNull,
            CatalogEntity::Node,
            node.labels
                .iter()
                .map(|label| label.value.clone())
                .collect(),
            node.span,
        )?;
        let labels = self.resolve_labels(node)?;
        let source = self
            .catalog
            .node_source(self.graph)
            .ok_or(BindError::MissingSource {
                entity: "node",
                span_start: node.span.start,
                span_end: node.span.end,
            })?;
        let scope = ir::Scope::new(self.scope.clone())?;
        let scan = ir::Plan::new(
            ir::PlanKind::NodeScan(ir::NodeScan {
                graph: self.graph,
                source,
                binding: binding.id(),
                labels,
            }),
            scope.clone(),
            ir::ResultShape::default(),
        )?;
        // A fresh scan while a plan already exists starts a disconnected
        // pattern: combine as a cartesian product instead of replacing it.
        self.plan = Some(match self.plan.take() {
            None => scan,
            Some(existing) => ir::Plan::new(
                ir::PlanKind::Join(ir::Join {
                    left: Box::new(existing),
                    right: Box::new(scan),
                }),
                scope,
                ir::ResultShape::default(),
            )?,
        });
        self.bind_properties(&binding, CatalogEntity::Node, &node.properties)?;
        Ok(binding.id())
    }

    fn bind_labels(&self, node: &cypher::NodePattern) -> Result<(), BindError> {
        self.resolve_labels(node).map(|_| ())
    }

    /// Enforces a node pattern's labels on an already-produced binding by
    /// filtering through the label junction — used for step nodes and
    /// reused variables, whose labels are not part of a NodeScan.
    fn enforce_labels(
        &mut self,
        binding: ir::BindingId,
        node: &cypher::NodePattern,
    ) -> Result<(), BindError> {
        self.bind_labels(node)?;
        for label in &node.labels {
            let predicate = ir::TypedExpression {
                expression: ir::Expression::Function {
                    function: ir::FunctionName::new("__cypher_has_label").expect("static name"),
                    arguments: vec![
                        ir::TypedExpression {
                            expression: ir::Expression::Binding(binding),
                            value_type: ir::ValueType::Node,
                            nullability: ir::Nullability::NonNull,
                        },
                        ir::TypedExpression {
                            expression: ir::Expression::Literal(ir::Literal::Text(
                                label.value.clone(),
                            )),
                            value_type: ir::ValueType::Text,
                            nullability: ir::Nullability::NonNull,
                        },
                    ],
                },
                value_type: ir::ValueType::Boolean,
                nullability: ir::Nullability::NonNull,
            };
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                input: Box::new(input),
                predicate,
            }))?;
        }
        Ok(())
    }

    fn resolve_labels(&self, node: &cypher::NodePattern) -> Result<Vec<ir::LabelId>, BindError> {
        node.labels
            .iter()
            .map(|label| {
                self.catalog.label(self.graph, &label.value).ok_or_else(|| {
                    BindError::UnknownLabel {
                        name: label.value.clone(),
                        span_start: label.span.start,
                        span_end: label.span.end,
                    }
                })
            })
            .collect()
    }

    fn bind_properties(
        &mut self,
        binding: &ir::Binding,
        entity: CatalogEntity,
        properties: &[(cypher::Spanned<String>, cypher::Spanned<cypher::Expression>)],
    ) -> Result<(), BindError> {
        for (name, value) in properties {
            let property = self.resolve_property(entity, name)?;
            let right = self.bind_expression(value)?;
            let left = ir::TypedExpression {
                expression: ir::Expression::Property {
                    entity: binding.id(),
                    property: property.id,
                    fields: Vec::new(),
                },
                value_type: property.value_type,
                nullability: property.nullability,
            };
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                input: Box::new(input),
                predicate: ir::TypedExpression {
                    expression: ir::Expression::Binary {
                        left: Box::new(left),
                        op: ir::BinaryOp::Equal,
                        right: Box::new(right),
                    },
                    value_type: ir::ValueType::Boolean,
                    nullability: ir::Nullability::NonNull,
                },
            }))?;
        }
        Ok(())
    }

    fn bind_projection(
        &mut self,
        clause: &cypher::ProjectionClause,
        is_return: bool,
    ) -> Result<(), BindError> {
        for item in &clause.items {
            if let cypher::ProjectionItem::Expression { expression, .. } = item {
                // openCypher forbids bare pattern expressions in RETURN and
                // WITH projections (TCK Pattern1 [22]/[23]).
                if matches!(
                    expression.value,
                    cypher::Expression::PatternPredicate { .. }
                ) {
                    return Err(at_unsupported(
                        expression.span,
                        "pattern expressions in projections",
                    ));
                }
            }
        }
        let mut input = match self.plan.take() {
            Some(input) => input,
            None => ir::Plan::new(
                ir::PlanKind::Unit(ir::Unit),
                ir::Scope::default(),
                ir::ResultShape::default(),
            )?,
        };
        // ORDER BY and WHERE see both projection aliases and pre-projection
        // variables: alias references substitute their source expressions
        // and everything binds in the pre-projection scope. Aliases of
        // aggregate items cannot substitute (no value before grouping).
        let mut alias_sources: HashMap<String, cypher::Expression> = HashMap::new();
        let shadowed: std::collections::HashSet<String> = self
            .scope
            .iter()
            .map(|binding| binding.name().to_owned())
            .collect();
        let mut has_aggregates = false;
        for item in &clause.items {
            if let cypher::ProjectionItem::Expression { expression, alias } = item {
                if contains_aggregate_call(&expression.value) {
                    has_aggregates = true;
                } else if let Some(alias) = alias {
                    alias_sources.insert(alias.value.clone(), expression.value.clone());
                }
            }
        }
        if let Some(predicate) = &clause.predicate {
            if !has_aggregates {
                let predicate = self.bind_expression(&substitute_variables(
                    predicate,
                    &alias_sources,
                    &shadowed,
                ))?;
                input = ir::Plan::new(
                    ir::PlanKind::Filter(ir::Filter {
                        input: Box::new(input),
                        predicate,
                    }),
                    ir::Scope::new(self.scope.clone())?,
                    ir::ResultShape::default(),
                )?;
            }
        }
        // Aggregating projections sort AFTER grouping, in the output scope
        // where aggregate aliases are real bindings (ORDER BY sum for
        // sum(..) AS sum); non-aggregating clauses keep the pre-projection
        // sort with alias substitution.
        let sort_keys = if has_aggregates {
            Vec::new()
        } else {
            clause
                .order_by
                .iter()
                .map(|item| {
                    Ok(ir::SortKey {
                        expression: self.bind_expression(&substitute_variables(
                            &item.expression,
                            &alias_sources,
                            &shadowed,
                        ))?,
                        direction: if item.descending {
                            ir::SortDirection::Descending
                        } else {
                            ir::SortDirection::Ascending
                        },
                        null_order: if item.descending {
                            ir::NullOrder::First
                        } else {
                            ir::NullOrder::Last
                        },
                    })
                })
                .collect::<Result<Vec<_>, BindError>>()?
        };
        if !sort_keys.is_empty() {
            input = ir::Plan::new(
                ir::PlanKind::Sort(ir::Sort {
                    input: Box::new(input),
                    keys: sort_keys,
                }),
                ir::Scope::new(self.scope.clone())?,
                ir::ResultShape::default(),
            )?;
        }
        // Final result column per item: either a stage-one binding passed
        // through, or a post-aggregate expression over hidden aggregation
        // outputs (`count(*) + 1`).
        enum ItemOutput {
            Direct(ir::Binding, cypher::Span),
            Compound {
                name: String,
                expression: cypher::Spanned<cypher::Expression>,
            },
        }
        let mut projections = Vec::new();
        let mut aggregations: Vec<ir::Aggregation> = Vec::new();
        let mut output_scope = Vec::new();
        let mut output_entities = HashMap::new();
        let mut item_outputs: Vec<ItemOutput> = Vec::new();
        let mut has_compound_aggregates = false;
        for item in &clause.items {
            match item {
                cypher::ProjectionItem::All(span) => {
                    for binding in &self.scope {
                        let expression = self.scope_binding_expression(binding, *span)?;
                        projections.push(ir::Projection {
                            output: binding.clone(),
                            expression,
                        });
                        output_scope.push(binding.clone());
                        item_outputs.push(ItemOutput::Direct(binding.clone(), *span));
                        if let Some(entity) = self.entities.get(&binding.id()) {
                            output_entities.insert(binding.id(), entity.clone());
                        }
                    }
                }
                cypher::ProjectionItem::Expression { expression, alias } => {
                    // openCypher: WITH items that are not plain variables
                    // must be aliased (TCK With4 [5]).
                    if !is_return
                        && alias.is_none()
                        && !matches!(expression.value, cypher::Expression::Variable(_))
                    {
                        return Err(at_unsupported(
                            expression.span,
                            "unaliased WITH expressions",
                        ));
                    }
                    let name = alias
                        .as_ref()
                        .map(|alias| alias.value.clone())
                        .unwrap_or_else(|| projection_name(expression));
                    if output_scope
                        .iter()
                        .any(|binding: &ir::Binding| binding.name() == name)
                    {
                        let span = alias.as_ref().map_or(expression.span, |alias| alias.span);
                        return Err(BindError::DuplicateVariable {
                            name,
                            span_start: span.start,
                            span_end: span.end,
                        });
                    }
                    if let Some((function, argument, distinct)) =
                        self.bind_aggregate_call(expression)?
                    {
                        let value_type = aggregate_value_type(&function, &argument);
                        let output = ir::Binding::new(
                            self.next_id()?,
                            name,
                            value_type,
                            ir::Nullability::Nullable,
                        )?;
                        aggregations.push(ir::Aggregation {
                            output: output.clone(),
                            function,
                            expression: argument,
                            distinct,
                        });
                        item_outputs.push(ItemOutput::Direct(output.clone(), expression.span));
                        output_scope.push(output);
                        continue;
                    }
                    if contains_aggregate_call(&expression.value) {
                        // Aggregates inside a larger expression: hoist each
                        // call into a hidden aggregation, bind the remainder
                        // after grouping.
                        let mut hidden = Vec::new();
                        let rewritten = self.extract_aggregate_calls(
                            expression,
                            &mut aggregations,
                            &mut hidden,
                        )?;
                        output_scope.extend(hidden);
                        item_outputs.push(ItemOutput::Compound {
                            name,
                            expression: rewritten,
                        });
                        has_compound_aggregates = true;
                        continue;
                    }
                    let bound = self.bind_expression(expression)?;
                    let old_entity = match &expression.value {
                        cypher::Expression::Variable(name) => self
                            .scope
                            .iter()
                            .find(|binding| binding.name() == name)
                            .and_then(|binding| self.entities.get(&binding.id()).cloned()),
                        _ => None,
                    };
                    let output = ir::Binding::new(
                        self.next_id()?,
                        name,
                        bound.value_type.clone(),
                        bound.nullability,
                    )?;
                    if let Some(entity) = old_entity {
                        output_entities.insert(output.id(), entity);
                    }
                    projections.push(ir::Projection {
                        output: output.clone(),
                        expression: bound,
                    });
                    item_outputs.push(ItemOutput::Direct(output.clone(), expression.span));
                    output_scope.push(output);
                }
            }
        }
        if has_compound_aggregates {
            // Stage one groups and aggregates (hidden outputs included in
            // its scope); stage two projects the final columns, binding
            // compound remainders in the post-aggregate scope where only
            // grouping keys and aggregation outputs are visible.
            let groupings = projections
                .into_iter()
                .map(|projection| ir::Grouping {
                    output: projection.output,
                    expression: projection.expression,
                })
                .collect();
            let aggregate_plan = ir::Plan::new(
                ir::PlanKind::Aggregate(ir::Aggregate {
                    input: Box::new(input),
                    groupings,
                    aggregations,
                }),
                ir::Scope::new(output_scope.clone())?,
                ir::ResultShape::default(),
            )?;
            self.scope = output_scope;
            self.entities = output_entities.clone();
            let mut final_projections = Vec::new();
            let mut final_scope = Vec::new();
            let mut final_entities = HashMap::new();
            for item in item_outputs {
                match item {
                    ItemOutput::Direct(binding, span) => {
                        let expression = self.scope_binding_expression(&binding, span)?;
                        final_projections.push(ir::Projection {
                            output: binding.clone(),
                            expression,
                        });
                        if let Some(entity) = output_entities.get(&binding.id()) {
                            final_entities.insert(binding.id(), entity.clone());
                        }
                        final_scope.push(binding);
                    }
                    ItemOutput::Compound { name, expression } => {
                        let bound = self.bind_expression(&expression)?;
                        let output = ir::Binding::new(
                            self.next_id()?,
                            name,
                            bound.value_type.clone(),
                            bound.nullability,
                        )?;
                        final_projections.push(ir::Projection {
                            output: output.clone(),
                            expression: bound,
                        });
                        final_scope.push(output);
                    }
                }
            }
            let scope = ir::Scope::new(final_scope.clone())?;
            let shape = if is_return {
                ir::ResultShape::new(
                    final_scope
                        .iter()
                        .map(|binding| ir::ResultColumn::new(binding.id(), binding.name()))
                        .collect::<Result<_, _>>()?,
                    &scope,
                )?
            } else {
                ir::ResultShape::default()
            };
            self.plan = Some(ir::Plan::new(
                ir::PlanKind::Project(ir::Project {
                    input: Box::new(aggregate_plan),
                    projections: final_projections,
                }),
                scope,
                shape,
            )?);
            self.scope = final_scope;
            self.entities = final_entities;
        } else {
            let scope = ir::Scope::new(output_scope.clone())?;
            let shape = if is_return {
                ir::ResultShape::new(
                    output_scope
                        .iter()
                        .map(|binding| ir::ResultColumn::new(binding.id(), binding.name()))
                        .collect::<Result<_, _>>()?,
                    &scope,
                )?
            } else {
                ir::ResultShape::default()
            };
            self.plan = Some(if aggregations.is_empty() {
                ir::Plan::new(
                    ir::PlanKind::Project(ir::Project {
                        input: Box::new(input),
                        projections,
                    }),
                    scope,
                    shape,
                )?
            } else {
                // Cypher implicit grouping: non-aggregated projection items
                // become the grouping keys.
                let groupings = projections
                    .into_iter()
                    .map(|projection| ir::Grouping {
                        output: projection.output,
                        expression: projection.expression,
                    })
                    .collect();
                ir::Plan::new(
                    ir::PlanKind::Aggregate(ir::Aggregate {
                        input: Box::new(input),
                        groupings,
                        aggregations,
                    }),
                    scope,
                    shape,
                )?
            });
            self.scope = output_scope;
            self.entities = output_entities;
        }
        self.rematerialize_path_compositions();
        // Aggregating projections sort after grouping in the output scope,
        // where aggregate aliases are real bindings.
        if has_aggregates && !clause.order_by.is_empty() {
            // A sort key naming an output alias binds directly; a key that
            // syntactically repeats a projection item's source expression
            // (ORDER BY a.name for a.name AS name) maps onto that output.
            // A sort key may repeat a projection item's source expression
            // (ORDER BY a.name for a.name AS name), possibly inside a larger
            // expression (ORDER BY a.name + 'C'); rewrite those occurrences
            // to the output alias so the key binds in the output scope.
            let alias_for = |expression: &cypher::Expression| -> Option<String> {
                clause.items.iter().find_map(|item| match item {
                    cypher::ProjectionItem::Expression {
                        expression: source,
                        alias,
                    } if expressions_match(&source.value, expression) => alias
                        .as_ref()
                        .map(|alias| alias.value.clone())
                        .or_else(|| Some(projection_name(source))),
                    _ => None,
                })
            };
            let keys = clause
                .order_by
                .iter()
                .map(|item| {
                    let rewritten = replace_projected_sources(&item.expression, &alias_for);
                    Ok(ir::SortKey {
                        expression: self.bind_expression(&rewritten)?,
                        direction: if item.descending {
                            ir::SortDirection::Descending
                        } else {
                            ir::SortDirection::Ascending
                        },
                        null_order: if item.descending {
                            ir::NullOrder::First
                        } else {
                            ir::NullOrder::Last
                        },
                    })
                })
                .collect::<Result<Vec<_>, BindError>>()?;
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            let result_shape = input.result_shape().clone();
            self.plan = Some(ir::Plan::new(
                ir::PlanKind::Sort(ir::Sort {
                    input: Box::new(input),
                    keys,
                }),
                ir::Scope::new(self.scope.clone())?,
                result_shape,
            )?);
        }
        // Aggregating projections filter after grouping (HAVING shape);
        // non-aggregating clauses already filtered pre-projection.
        if has_aggregates {
            if let Some(predicate) = &clause.predicate {
                let predicate = self.bind_expression(predicate)?;
                let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
                self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                    input: Box::new(input),
                    predicate,
                }))?;
            }
        }
        if clause.distinct {
            let keys = self
                .scope
                .iter()
                .map(|binding| ir::TypedExpression {
                    expression: ir::Expression::Binding(binding.id()),
                    value_type: binding.value_type().clone(),
                    nullability: binding.nullability(),
                })
                .collect();
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            let result_shape = input.result_shape().clone();
            self.plan = Some(ir::Plan::new(
                ir::PlanKind::Distinct(ir::Distinct {
                    input: Box::new(input),
                    keys,
                }),
                ir::Scope::new(self.scope.clone())?,
                result_shape,
            )?);
        }
        if let Some(skip) = &clause.skip {
            let count = self.bind_expression(skip)?;
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            let result_shape = input.result_shape().clone();
            self.plan = Some(ir::Plan::new(
                ir::PlanKind::Skip(ir::Skip {
                    input: Box::new(input),
                    count,
                }),
                ir::Scope::new(self.scope.clone())?,
                result_shape,
            )?);
        }
        if let Some(limit) = &clause.limit {
            let count = self.bind_expression(limit)?;
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            let result_shape = input.result_shape().clone();
            self.plan = Some(ir::Plan::new(
                ir::PlanKind::Limit(ir::Limit {
                    input: Box::new(input),
                    count,
                }),
                ir::Scope::new(self.scope.clone())?,
                result_shape,
            )?);
        }
        Ok(())
    }

    /// Rewrites every aggregate call inside a projection expression into a
    /// hidden aggregation output and returns the expression with each call
    /// replaced by a reference to its hidden binding. The remainder then
    /// binds after grouping (`count(*) + 1`, `2 * sum(x)`); non-aggregate
    /// variable references survive only when they are also grouping keys,
    /// otherwise post-aggregate binding fails loudly instead of silently
    /// mis-grouping.
    fn extract_aggregate_calls(
        &mut self,
        expression: &cypher::Spanned<cypher::Expression>,
        aggregations: &mut Vec<ir::Aggregation>,
        hidden: &mut Vec<ir::Binding>,
    ) -> Result<cypher::Spanned<cypher::Expression>, BindError> {
        if let Some((function, argument, distinct)) = self.bind_aggregate_call(expression)? {
            let value_type = aggregate_value_type(&function, &argument);
            let id = self.next_id()?;
            let name = format!("__turso_aggregate_{id}");
            let output = ir::Binding::new(id, name.clone(), value_type, ir::Nullability::Nullable)?;
            aggregations.push(ir::Aggregation {
                output: output.clone(),
                function,
                expression: argument,
                distinct,
            });
            hidden.push(output);
            return Ok(cypher::Spanned {
                value: cypher::Expression::Variable(name),
                span: expression.span,
            });
        }
        use cypher::Expression as E;
        let value = match &expression.value {
            E::Binary {
                left,
                operator,
                right,
            } => E::Binary {
                left: self
                    .extract_aggregate_calls(left, aggregations, hidden)
                    .map(Box::new)?,
                operator: *operator,
                right: self
                    .extract_aggregate_calls(right, aggregations, hidden)
                    .map(Box::new)?,
            },
            E::Unary { operator, operand } => E::Unary {
                operator: *operator,
                operand: self
                    .extract_aggregate_calls(operand, aggregations, hidden)
                    .map(Box::new)?,
            },
            E::Case {
                subject,
                branches,
                default,
            } => E::Case {
                subject: subject
                    .as_ref()
                    .map(|subject| {
                        self.extract_aggregate_calls(subject, aggregations, hidden)
                            .map(Box::new)
                    })
                    .transpose()?,
                branches: branches
                    .iter()
                    .map(|(when, then)| {
                        Ok((
                            self.extract_aggregate_calls(when, aggregations, hidden)?,
                            self.extract_aggregate_calls(then, aggregations, hidden)?,
                        ))
                    })
                    .collect::<Result<_, BindError>>()?,
                default: default
                    .as_ref()
                    .map(|default| {
                        self.extract_aggregate_calls(default, aggregations, hidden)
                            .map(Box::new)
                    })
                    .transpose()?,
            },
            E::Property { entity, name } => E::Property {
                entity: self
                    .extract_aggregate_calls(entity, aggregations, hidden)
                    .map(Box::new)?,
                name: name.clone(),
            },
            E::HasLabels { operand, labels } => E::HasLabels {
                operand: self
                    .extract_aggregate_calls(operand, aggregations, hidden)
                    .map(Box::new)?,
                labels: labels.clone(),
            },
            E::Index { base, index } => E::Index {
                base: self
                    .extract_aggregate_calls(base, aggregations, hidden)
                    .map(Box::new)?,
                index: self
                    .extract_aggregate_calls(index, aggregations, hidden)
                    .map(Box::new)?,
            },
            E::Slice { base, from, to } => E::Slice {
                base: self
                    .extract_aggregate_calls(base, aggregations, hidden)
                    .map(Box::new)?,
                from: from
                    .as_ref()
                    .map(|from| {
                        self.extract_aggregate_calls(from, aggregations, hidden)
                            .map(Box::new)
                    })
                    .transpose()?,
                to: to
                    .as_ref()
                    .map(|to| {
                        self.extract_aggregate_calls(to, aggregations, hidden)
                            .map(Box::new)
                    })
                    .transpose()?,
            },
            E::Cast { operand, type_name } => E::Cast {
                operand: self
                    .extract_aggregate_calls(operand, aggregations, hidden)
                    .map(Box::new)?,
                type_name: type_name.clone(),
            },
            E::Function {
                name,
                arguments,
                distinct,
                star,
            } => E::Function {
                name: name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| self.extract_aggregate_calls(argument, aggregations, hidden))
                    .collect::<Result<_, BindError>>()?,
                distinct: *distinct,
                star: *star,
            },
            E::List(items) => E::List(
                items
                    .iter()
                    .map(|item| self.extract_aggregate_calls(item, aggregations, hidden))
                    .collect::<Result<_, BindError>>()?,
            ),
            E::Map(entries) => E::Map(
                entries
                    .iter()
                    .map(|(key, value)| {
                        Ok((
                            key.clone(),
                            self.extract_aggregate_calls(value, aggregations, hidden)?,
                        ))
                    })
                    .collect::<Result<_, BindError>>()?,
            ),
            // Loop bodies and pattern subqueries have their own variable
            // scopes. Their input expressions still execute in the outer
            // projection scope, where aggregates must be hoisted.
            E::Quantifier {
                kind,
                variable,
                list,
                predicate,
            } => E::Quantifier {
                kind: *kind,
                variable: variable.clone(),
                list: self
                    .extract_aggregate_calls(list, aggregations, hidden)
                    .map(Box::new)?,
                predicate: predicate.clone(),
            },
            E::ListComprehension {
                variable,
                list,
                predicate,
                map,
            } => E::ListComprehension {
                variable: variable.clone(),
                list: self
                    .extract_aggregate_calls(list, aggregations, hidden)
                    .map(Box::new)?,
                predicate: predicate.clone(),
                map: map.clone(),
            },
            E::Literal(_)
            | E::Variable(_)
            | E::Parameter(_)
            | E::Reduce { .. }
            | E::PatternSubquery { .. }
            | E::PatternPredicate { .. } => expression.value.clone(),
        };
        Ok(cypher::Spanned {
            value,
            span: expression.span,
        })
    }

    /// Detects a projection item that is a single aggregate call and binds
    /// its argument. Returns None for non-aggregate expressions.
    fn bind_aggregate_call(
        &self,
        expression: &cypher::Spanned<cypher::Expression>,
    ) -> Result<Option<(ir::AggregateFunction, Option<ir::TypedExpression>, bool)>, BindError> {
        let cypher::Expression::Function {
            name,
            arguments,
            distinct,
            star,
        } = &expression.value
        else {
            return Ok(None);
        };
        let function = match name.value.to_ascii_lowercase().as_str() {
            "count" => ir::AggregateFunction::Count,
            "sum" => ir::AggregateFunction::Sum,
            "avg" => ir::AggregateFunction::Average,
            "min" => ir::AggregateFunction::Minimum,
            "max" => ir::AggregateFunction::Maximum,
            "collect" => ir::AggregateFunction::Collect,
            _ => return Ok(None),
        };
        if *star {
            if function != ir::AggregateFunction::Count {
                return Err(at_unsupported(
                    expression.span,
                    "star arguments outside count()",
                ));
            }
            return Ok(Some((function, None, *distinct)));
        }
        // openCypher: non-deterministic functions may not appear inside
        // aggregations (TCK Return6 [15]).
        if arguments.iter().any(|argument| {
            matches!(
                &argument.value,
                cypher::Expression::Function { name, .. }
                    if name.value.eq_ignore_ascii_case("rand")
            )
        }) {
            return Err(at_unsupported(
                expression.span,
                "non-deterministic expressions in aggregations",
            ));
        }
        let [argument] = arguments.as_slice() else {
            // Multi-argument min/max are SQL's scalar forms, not aggregates.
            if matches!(
                function,
                ir::AggregateFunction::Minimum | ir::AggregateFunction::Maximum
            ) {
                return Ok(None);
            }
            return Err(at_unsupported(
                expression.span,
                "aggregate calls without exactly one argument",
            ));
        };
        let argument = self.bind_expression(argument)?;
        Ok(Some((function, Some(argument), *distinct)))
    }

    fn bind_pattern_subquery(
        &self,
        count: bool,
        clause: &cypher::MatchClause,
        span: cypher::Span,
    ) -> Result<(ir::Expression, ir::ValueType, ir::Nullability), BindError> {
        let mut sub = Binder::new(self.graph, self.catalog, self.parameters);
        sub.bind_match(clause, span)?;
        let plan = sub.plan.ok_or(BindError::EmptyQuery)?;
        let mut correlations: Vec<(ir::BindingId, ir::BindingId)> = Vec::new();
        for path in &clause.paths {
            let mut names: Vec<&str> = Vec::new();
            if let Some(variable) = &path.variable {
                names.push(&variable.value);
            }
            if let Some(variable) = &path.start.variable {
                names.push(&variable.value);
            }
            for (relationship, node) in &path.steps {
                if let Some(variable) = &relationship.variable {
                    names.push(&variable.value);
                }
                if let Some(variable) = &node.variable {
                    names.push(&variable.value);
                }
            }
            for name in names {
                let Some(outer) = self.scope.iter().find(|binding| binding.name() == name) else {
                    continue;
                };
                let Some(inner) = plan.scope().resolve(name) else {
                    continue;
                };
                if !correlations
                    .iter()
                    .any(|(existing, _)| *existing == outer.id())
                {
                    correlations.push((outer.id(), inner.id()));
                }
            }
        }
        Ok((
            ir::Expression::PatternSubquery {
                count,
                plan: Box::new(plan),
                correlations,
            },
            if count {
                ir::ValueType::Integer
            } else {
                ir::ValueType::Boolean
            },
            ir::Nullability::NonNull,
        ))
    }

    fn bind_expression(
        &self,
        expression: &cypher::Spanned<cypher::Expression>,
    ) -> Result<ir::TypedExpression, BindError> {
        let (expression_ir, value_type, nullability) = match &expression.value {
            cypher::Expression::Literal(literal) => {
                let (literal, value_type, nullability) = bind_literal(literal);
                (ir::Expression::Literal(literal), value_type, nullability)
            }
            cypher::Expression::Variable(name) => {
                // reduce() variables shadow everything else; innermost wins.
                let reduce_hit = {
                    let scopes = self.reduce_scopes.borrow();
                    scopes
                        .iter()
                        .rposition(|(accumulator, element)| accumulator == name || element == name)
                        .map(|depth| (depth, scopes[depth].0 == *name))
                };
                if let Some((depth, is_accumulator)) = reduce_hit {
                    return Ok(ir::TypedExpression {
                        expression: if is_accumulator {
                            ir::Expression::ReduceAccumulator(depth)
                        } else {
                            ir::Expression::ReduceElement(depth)
                        },
                        value_type: ir::ValueType::Any,
                        nullability: ir::Nullability::Nullable,
                    });
                }
                let scope_hit = {
                    let scopes = self.list_scopes.borrow();
                    scopes
                        .iter()
                        .rposition(|(scope_name, _)| scope_name == name)
                        .map(|position| (position, scopes.len(), scopes[position].1.clone()))
                };
                if let Some(composition) = self.path_compositions.get(name) {
                    return match composition {
                        PathComposition::Fixed {
                            nodes,
                            relationships,
                        } => Ok(ir::TypedExpression {
                            expression: ir::Expression::PathValue {
                                nodes: nodes.clone(),
                                relationships: relationships.clone(),
                            },
                            value_type: ir::ValueType::Path,
                            nullability: ir::Nullability::NonNull,
                        }),
                        PathComposition::Expanded(binding) => Ok(ir::TypedExpression {
                            expression: ir::Expression::Binding(*binding),
                            value_type: ir::ValueType::Path,
                            nullability: ir::Nullability::NonNull,
                        }),
                        PathComposition::Unsupported => Err(at_unsupported(
                            expression.span,
                            "variable-length path values",
                        )),
                    };
                }
                if let Some((position, scope_count, element_type)) = scope_hit {
                    if position + 1 != scope_count {
                        return Err(at_unsupported(
                            expression.span,
                            "outer list-scope variables inside nested list scopes",
                        ));
                    }
                    (
                        ir::Expression::ListElement(position + 1),
                        element_type,
                        ir::Nullability::Nullable,
                    )
                } else {
                    let binding = self.resolve_binding(name, expression.span)?;
                    (
                        ir::Expression::Binding(binding.id()),
                        binding.value_type().clone(),
                        binding.nullability(),
                    )
                }
            }
            cypher::Expression::Parameter(name) => {
                let (value_type, nullability) =
                    self.parameters
                        .get(name)
                        .ok_or_else(|| BindError::UnknownParameter {
                            name: name.clone(),
                            span_start: expression.span.start,
                            span_end: expression.span.end,
                        })?;
                (
                    ir::Expression::Parameter(name.clone()),
                    value_type.clone(),
                    *nullability,
                )
            }
            cypher::Expression::Property { entity, name } => {
                // Component access on temporal values maps onto temporal_get.
                if is_temporal_unit(&name.value) {
                    let base = self.bind_expression(entity);
                    if let Ok(base) = &base {
                        if base.value_type == temporal_value_type() {
                            let unit = ir::TypedExpression {
                                expression: ir::Expression::Literal(ir::Literal::Text(
                                    name.value.clone(),
                                )),
                                value_type: ir::ValueType::Text,
                                nullability: ir::Nullability::NonNull,
                            };
                            let value_type = if matches!(name.value.as_str(), "timezone" | "offset")
                            {
                                ir::ValueType::Text
                            } else {
                                ir::ValueType::Integer
                            };
                            return Ok(sql_call(
                                "temporal_get",
                                vec![base.clone(), unit],
                                value_type,
                            ));
                        }
                    }
                }
                // Component access on duration values maps onto duration_get.
                if is_duration_unit(&name.value) {
                    let base = self.bind_expression(entity);
                    if let Ok(base) = &base {
                        if base.value_type == duration_value_type() {
                            let unit = ir::TypedExpression {
                                expression: ir::Expression::Literal(ir::Literal::Text(
                                    name.value.clone(),
                                )),
                                value_type: ir::ValueType::Text,
                                nullability: ir::Nullability::NonNull,
                            };
                            return Ok(sql_call(
                                "duration_get",
                                vec![base.clone(), unit],
                                ir::ValueType::Integer,
                            ));
                        }
                    }
                }
                let (root, field_chain) = flatten_property_chain(expression);
                // Quantifier/list-comprehension variables resolve through
                // bind_expression's list scopes, not the binding scope, so a
                // failed entity lookup falls to the generic member path.
                let entity_binding = match &root.value {
                    cypher::Expression::Variable(variable) => self
                        .resolve_binding(variable, root.span)
                        .ok()
                        .filter(|binding| self.entities.contains_key(&binding.id())),
                    _ => None,
                };
                let Some(binding) = entity_binding else {
                    // Member access over map-shaped values (map literals,
                    // UNWIND elements, parameters): json_extract per field,
                    // guarded so non-JSON values yield null, not an error.
                    let base = self.bind_expression(root)?;
                    if matches!(base.value_type, ir::ValueType::Map | ir::ValueType::Any) {
                        let mut current = base;
                        for field in &field_chain {
                            current = json_member_access(current, &field.value);
                        }
                        return Ok(current);
                    }
                    return Err(BindError::InvalidPropertyTarget {
                        span_start: root.span.start,
                        span_end: root.span.end,
                    });
                };
                let kind = self
                    .entities
                    .get(&binding.id())
                    .ok_or(BindError::InvalidPropertyTarget {
                        span_start: root.span.start,
                        span_end: root.span.end,
                    })?
                    .kind;
                let (property_name, nested_fields) = field_chain.split_first().expect(
                    "flatten_property_chain always yields at least the outer Property's name",
                );
                let property = self.resolve_property(kind, property_name)?;
                if nested_fields.len() > 2 {
                    return Err(at_unsupported(
                        expression.span,
                        "struct/union field access deeper than two levels",
                    ));
                }
                let mut value_type = property.value_type.clone();
                for field in nested_fields {
                    value_type = self.resolve_field(&value_type, field)?;
                }
                (
                    ir::Expression::Property {
                        entity: binding.id(),
                        property: property.id,
                        fields: nested_fields
                            .iter()
                            .map(|field| field.value.clone())
                            .collect(),
                    },
                    value_type,
                    nullable(binding.nullability(), property.nullability),
                )
            }
            cypher::Expression::Unary { operator, operand } => {
                let operand_span = operand.span;
                let operand = self.bind_expression(operand)?;
                if matches!(operator, cypher::UnaryOperator::Not)
                    && !boolean_compatible(&operand.value_type)
                {
                    return Err(at_unsupported(operand_span, "NOT on a non-boolean operand"));
                }
                let (op, nullability) = match operator {
                    cypher::UnaryOperator::Not => (ir::UnaryOp::Not, operand.nullability),
                    cypher::UnaryOperator::IsNull => {
                        (ir::UnaryOp::IsNull, ir::Nullability::NonNull)
                    }
                    cypher::UnaryOperator::IsNotNull => {
                        (ir::UnaryOp::IsNotNull, ir::Nullability::NonNull)
                    }
                };
                (
                    ir::Expression::Unary {
                        op,
                        expression: Box::new(operand),
                    },
                    ir::ValueType::Boolean,
                    nullability,
                )
            }
            cypher::Expression::PatternSubquery {
                count,
                paths,
                predicate,
            } => {
                let clause = cypher::MatchClause {
                    optional: false,
                    paths: paths.clone(),
                    predicate: predicate.as_deref().cloned(),
                };
                self.bind_pattern_subquery(*count, &clause, expression.span)?
            }
            cypher::Expression::HasLabels { operand, labels } => {
                let cypher::Expression::Variable(name) = &operand.value else {
                    return Err(at_unsupported(
                        operand.span,
                        "label predicates on non-variable expressions",
                    ));
                };
                if !self.scope.iter().any(|binding| binding.name() == *name) {
                    return Err(BindError::UnknownVariable {
                        name: name.clone(),
                        span_start: operand.span.start,
                        span_end: operand.span.end,
                    });
                }
                let clause = cypher::MatchClause {
                    optional: false,
                    paths: vec![cypher::PathPattern {
                        variable: None,
                        start: cypher::NodePattern {
                            variable: Some(cypher::Spanned::new(name.clone(), operand.span)),
                            labels: labels.clone(),
                            properties: Vec::new(),
                            has_property_map: false,
                            span: expression.span,
                        },
                        steps: Vec::new(),
                        span: expression.span,
                    }],
                    predicate: None,
                };
                self.bind_pattern_subquery(false, &clause, expression.span)?
            }
            cypher::Expression::PatternPredicate { path } => {
                // openCypher: a bare pattern predicate may only use already
                // bound variables (TCK Pattern1 [10]).
                let mut variables: Vec<&cypher::Spanned<String>> = Vec::new();
                variables.extend(&path.start.variable);
                for (relationship, node) in &path.steps {
                    variables.extend(&relationship.variable);
                    variables.extend(&node.variable);
                }
                for variable in variables {
                    if !self
                        .scope
                        .iter()
                        .any(|binding| binding.name() == variable.value)
                    {
                        return Err(BindError::UnknownVariable {
                            name: variable.value.clone(),
                            span_start: variable.span.start,
                            span_end: variable.span.end,
                        });
                    }
                }
                let clause = cypher::MatchClause {
                    optional: false,
                    paths: vec![path.as_ref().clone()],
                    predicate: None,
                };
                self.bind_pattern_subquery(false, &clause, expression.span)?
            }
            cypher::Expression::Index { base, index } => {
                let base_span = base.span;
                let base = self.bind_expression(base)?;
                let index = self.bind_expression(index)?;
                let element_type = match (&base.value_type, &index.value_type) {
                    (ir::ValueType::List(element), _) if numeric_compatible(&index.value_type) => {
                        (**element).clone()
                    }
                    (ir::ValueType::Any, _)
                        if numeric_compatible(&index.value_type)
                            || text_compatible(&index.value_type) =>
                    {
                        ir::ValueType::Any
                    }
                    _ => {
                        return Err(at_unsupported(
                            base_span,
                            "indexing this operand/key combination",
                        ));
                    }
                };
                (
                    ir::Expression::Index {
                        base: Box::new(base),
                        index: Box::new(index),
                    },
                    element_type,
                    ir::Nullability::Nullable,
                )
            }
            cypher::Expression::Slice { base, from, to } => {
                let base_span = base.span;
                let base = self.bind_expression(base)?;
                if !matches!(base.value_type, ir::ValueType::List(_) | ir::ValueType::Any) {
                    return Err(at_unsupported(base_span, "slicing a non-list operand"));
                }
                let value_type = base.value_type.clone();
                let from = from
                    .as_ref()
                    .map(|from| self.bind_expression(from))
                    .transpose()?
                    .map(Box::new);
                let to = to
                    .as_ref()
                    .map(|to| self.bind_expression(to))
                    .transpose()?
                    .map(Box::new);
                (
                    ir::Expression::Slice {
                        base: Box::new(base),
                        from,
                        to,
                    },
                    value_type,
                    ir::Nullability::Nullable,
                )
            }
            cypher::Expression::Cast { operand, type_name } => {
                let operand = self.bind_expression(operand)?;
                let lowered = type_name.value.to_ascii_lowercase();
                // AGE's universal and entity types cast as identity: the
                // value already carries the right representation here.
                if lowered == "vector" {
                    // ::vector constructs a core vector value.
                    return Ok(sql_call("vector32", vec![operand], ir::ValueType::Any));
                }
                if matches!(lowered.as_str(), "agtype" | "vertex" | "edge" | "path") {
                    return Ok(operand);
                }
                let target = match lowered.as_str() {
                    "integer" | "int" | "bigint" | "pg_bigint" | "smallint" => {
                        ir::ValueType::Integer
                    }
                    "float" | "float8" | "pg_float8" | "double" | "real" | "numeric" => {
                        ir::ValueType::Real
                    }
                    "text" | "string" | "varchar" | "cstring" => ir::ValueType::Text,
                    "bool" | "boolean" => ir::ValueType::Boolean,
                    _ => {
                        return Err(at_unsupported(type_name.span, "casts to this type name"));
                    }
                };
                (
                    ir::Expression::Cast {
                        expression: Box::new(operand),
                        target: target.clone(),
                    },
                    target,
                    ir::Nullability::Nullable,
                )
            }
            cypher::Expression::Quantifier {
                kind,
                variable,
                list,
                predicate,
            } => {
                let list_span = list.span;
                let list = self.bind_expression(list)?;
                let element_type = list_element_type(&list.value_type, list_span)?;
                self.list_scopes
                    .borrow_mut()
                    .push((variable.value.clone(), element_type));
                let depth = self.list_scopes.borrow().len();
                let predicate = self.bind_expression(predicate);
                self.list_scopes.borrow_mut().pop();
                let predicate = predicate?;
                if !boolean_compatible(&predicate.value_type) {
                    return Err(at_unsupported(
                        expression.span,
                        "quantifier predicates over non-boolean expressions",
                    ));
                }
                (
                    ir::Expression::Quantifier {
                        kind: match kind {
                            cypher::QuantifierKind::All => ir::QuantifierKind::All,
                            cypher::QuantifierKind::Any => ir::QuantifierKind::Any,
                            cypher::QuantifierKind::None => ir::QuantifierKind::None,
                            cypher::QuantifierKind::Single => ir::QuantifierKind::Single,
                        },
                        depth,
                        list: Box::new(list),
                        predicate: Box::new(predicate),
                    },
                    ir::ValueType::Boolean,
                    ir::Nullability::Nullable,
                )
            }
            cypher::Expression::Reduce {
                accumulator,
                initial,
                variable,
                list,
                expression: body,
            } => {
                let initial = self.bind_expression(initial)?;
                let list = self.bind_expression(list)?;
                let depth = self.reduce_scopes.borrow().len();
                self.reduce_scopes
                    .borrow_mut()
                    .push((accumulator.value.clone(), variable.value.clone()));
                let body = self.bind_expression(body);
                self.reduce_scopes.borrow_mut().pop();
                let body = body?;
                let value_type = initial.value_type.clone();
                (
                    ir::Expression::Reduce {
                        depth,
                        initial: Box::new(initial),
                        list: Box::new(list),
                        body: Box::new(body),
                    },
                    value_type,
                    ir::Nullability::Nullable,
                )
            }
            cypher::Expression::ListComprehension {
                variable,
                list,
                predicate,
                map,
            } => {
                let list_span = list.span;
                let list = self.bind_expression(list)?;
                let element_type = list_element_type(&list.value_type, list_span)?;
                self.list_scopes
                    .borrow_mut()
                    .push((variable.value.clone(), element_type.clone()));
                let depth = self.list_scopes.borrow().len();
                let bound = (|| {
                    let predicate = predicate
                        .as_ref()
                        .map(|predicate| self.bind_expression(predicate))
                        .transpose()?
                        .map(Box::new);
                    let map = map
                        .as_ref()
                        .map(|map| self.bind_expression(map))
                        .transpose()?
                        .map(Box::new);
                    Ok::<_, BindError>((predicate, map))
                })();
                self.list_scopes.borrow_mut().pop();
                let (predicate, map) = bound?;
                let element_result = map
                    .as_ref()
                    .map_or(element_type, |map| map.value_type.clone());
                (
                    ir::Expression::ListComprehension {
                        depth,
                        list: Box::new(list),
                        predicate,
                        map,
                    },
                    ir::ValueType::List(Box::new(element_result)),
                    ir::Nullability::Nullable,
                )
            }
            cypher::Expression::Case {
                subject,
                branches,
                default,
            } => {
                let subject = subject
                    .as_ref()
                    .map(|subject| self.bind_expression(subject))
                    .transpose()?
                    .map(Box::new);
                let branches = branches
                    .iter()
                    .map(|(condition, result)| {
                        Ok((
                            self.bind_expression(condition)?,
                            self.bind_expression(result)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, BindError>>()?;
                let default = default
                    .as_ref()
                    .map(|default| self.bind_expression(default))
                    .transpose()?
                    .map(Box::new);
                let mut result_types = branches
                    .iter()
                    .map(|(_, result)| &result.value_type)
                    .chain(default.iter().map(|default| &default.value_type));
                let value_type = match result_types.next() {
                    Some(first) if result_types.all(|next| next == first) => first.clone(),
                    _ => ir::ValueType::Any,
                };
                (
                    ir::Expression::Case {
                        subject,
                        branches,
                        default,
                    },
                    value_type,
                    ir::Nullability::Nullable,
                )
            }
            cypher::Expression::Binary {
                left,
                operator,
                right,
            } => {
                let right_span = right.span;
                let left = self.bind_expression(left)?;
                let right = self.bind_expression(right)?;
                if let Some(result) = duration_arithmetic(*operator, &left, &right) {
                    return Ok(result);
                }
                if let Some(result) = vector_operator(*operator, &left, &right) {
                    return Ok(result);
                }
                if let Some(result) = self.jsonb_operator(*operator, &left, &right) {
                    return Ok(result);
                }
                match operator {
                    cypher::BinaryOperator::In
                        if !matches!(
                            right.value_type,
                            ir::ValueType::List(_) | ir::ValueType::Any
                        ) =>
                    {
                        return Err(at_unsupported(
                            right_span,
                            "IN membership against a non-list operand",
                        ));
                    }
                    cypher::BinaryOperator::And
                    | cypher::BinaryOperator::Or
                    | cypher::BinaryOperator::Xor
                        if !boolean_compatible(&left.value_type)
                            || !boolean_compatible(&right.value_type) =>
                    {
                        return Err(at_unsupported(
                            right_span,
                            "boolean operators on non-boolean operands",
                        ));
                    }
                    cypher::BinaryOperator::StartsWith
                    | cypher::BinaryOperator::EndsWith
                    | cypher::BinaryOperator::Contains
                        if !text_compatible(&left.value_type)
                            || !text_compatible(&right.value_type) =>
                    {
                        return Err(at_unsupported(
                            right_span,
                            "string predicates on non-string operands",
                        ));
                    }
                    cypher::BinaryOperator::Subtract
                    | cypher::BinaryOperator::Multiply
                    | cypher::BinaryOperator::Divide
                    | cypher::BinaryOperator::Modulo
                    | cypher::BinaryOperator::Power
                        if !numeric_compatible(&left.value_type)
                            || !numeric_compatible(&right.value_type) =>
                    {
                        return Err(at_unsupported(
                            right_span,
                            "arithmetic on non-numeric operands",
                        ));
                    }
                    _ => {}
                }
                let value_type = binary_type(*operator, &left.value_type, &right.value_type);
                let nullability = nullable(left.nullability, right.nullability);
                (
                    ir::Expression::Binary {
                        left: Box::new(left),
                        op: bind_binary_operator(*operator),
                        right: Box::new(right),
                    },
                    value_type,
                    nullability,
                )
            }
            cypher::Expression::Function {
                name,
                arguments,
                distinct,
                star,
            } => {
                if *star {
                    return Err(at_unsupported(
                        expression.span,
                        "star arguments outside aggregating projections",
                    ));
                }
                if *distinct {
                    return Err(at_unsupported(
                        expression.span,
                        "DISTINCT function arguments",
                    ));
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| self.bind_expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                // labels(n) / type(r) resolve statically from the labels and
                // relationship types declared where the entity was bound.
                if let ("labels" | "type" | "label" | "properties" | "keys", [argument]) = (
                    name.value.to_ascii_lowercase().as_str(),
                    arguments.as_slice(),
                ) {
                    if let ir::Expression::Binding(id) = &argument.expression {
                        if let Some(entity) = self.entities.get(id) {
                            let lowered_name = name.value.to_ascii_lowercase();
                            if lowered_name == "properties" {
                                // Lowering enumerates the source's payload
                                // columns into a null-stripped JSON object.
                                return Ok(ir::TypedExpression {
                                    expression: ir::Expression::Function {
                                        function: ir::FunctionName::new("__cypher_properties")
                                            .expect("static name"),
                                        arguments: vec![argument.clone()],
                                    },
                                    value_type: ir::ValueType::Map,
                                    nullability: argument.nullability,
                                });
                            }
                            if lowered_name == "keys" {
                                // keys() reads an entity's property map, not
                                // its identity value; route through
                                // __cypher_properties first the same way the
                                // "properties" arm above does.
                                let properties = ir::TypedExpression {
                                    expression: ir::Expression::Function {
                                        function: ir::FunctionName::new("__cypher_properties")
                                            .expect("static name"),
                                        arguments: vec![argument.clone()],
                                    },
                                    value_type: ir::ValueType::Map,
                                    nullability: argument.nullability,
                                };
                                return Ok(ir::TypedExpression {
                                    expression: ir::Expression::Function {
                                        function: ir::FunctionName::new("__cypher_keys")
                                            .expect("static name"),
                                        arguments: vec![properties],
                                    },
                                    value_type: ir::ValueType::List(Box::new(ir::ValueType::Text)),
                                    nullability: argument.nullability,
                                });
                            }
                            if lowered_name == "labels" && entity.kind == CatalogEntity::Node {
                                // The label junction table is the source of
                                // truth; lowering resolves the sentinel.
                                return Ok(ir::TypedExpression {
                                    expression: ir::Expression::Function {
                                        function: ir::FunctionName::new("__cypher_labels")
                                            .expect("static name"),
                                        arguments: vec![argument.clone()],
                                    },
                                    value_type: ir::ValueType::List(Box::new(ir::ValueType::Text)),
                                    nullability: ir::Nullability::NonNull,
                                });
                            }
                            if lowered_name == "label" && entity.kind == CatalogEntity::Node {
                                return Ok(ir::TypedExpression {
                                    expression: ir::Expression::Function {
                                        function: ir::FunctionName::new("__cypher_label")
                                            .expect("static name"),
                                        arguments: vec![argument.clone()],
                                    },
                                    value_type: ir::ValueType::Text,
                                    nullability: ir::Nullability::Nullable,
                                });
                            }
                            let single_name = match lowered_name.as_str() {
                                "type" if entity.kind == CatalogEntity::Relationship => {
                                    entity.names.first()
                                }
                                _ => None,
                            };
                            if let (Some(single), 1) = (single_name, entity.names.len()) {
                                return Ok(ir::TypedExpression {
                                    expression: ir::Expression::Literal(ir::Literal::Text(
                                        single.clone(),
                                    )),
                                    value_type: ir::ValueType::Text,
                                    nullability: argument.nullability,
                                });
                            }
                        }
                    }
                    // Static resolution needs an entity record with declared
                    // names; bindings without one (undirected hops, values
                    // carried through WITH, NULL) resolve at runtime.
                    let lowered_name = name.value.to_ascii_lowercase();
                    if matches!(
                        argument.expression,
                        ir::Expression::Literal(ir::Literal::Null)
                    ) {
                        return Ok(ir::TypedExpression {
                            expression: ir::Expression::Literal(ir::Literal::Null),
                            value_type: ir::ValueType::Any,
                            nullability: ir::Nullability::Nullable,
                        });
                    }
                    match (lowered_name.as_str(), &argument.value_type) {
                        ("type", ir::ValueType::Relationship) => {
                            return Ok(ir::TypedExpression {
                                expression: ir::Expression::Function {
                                    function: ir::FunctionName::new("__cypher_relationship_type")
                                        .expect("static name"),
                                    arguments: vec![argument.clone()],
                                },
                                value_type: ir::ValueType::Text,
                                nullability: ir::Nullability::Nullable,
                            });
                        }
                        ("labels", ir::ValueType::Node) => {
                            return Ok(ir::TypedExpression {
                                expression: ir::Expression::Function {
                                    function: ir::FunctionName::new("__cypher_labels")
                                        .expect("static name"),
                                    arguments: vec![argument.clone()],
                                },
                                value_type: ir::ValueType::List(Box::new(ir::ValueType::Text)),
                                nullability: ir::Nullability::NonNull,
                            });
                        }
                        ("label", ir::ValueType::Node) => {
                            return Ok(ir::TypedExpression {
                                expression: ir::Expression::Function {
                                    function: ir::FunctionName::new("__cypher_label")
                                        .expect("static name"),
                                    arguments: vec![argument.clone()],
                                },
                                value_type: ir::ValueType::Text,
                                nullability: ir::Nullability::Nullable,
                            });
                        }
                        ("properties", ir::ValueType::Node | ir::ValueType::Relationship) => {
                            return Ok(ir::TypedExpression {
                                expression: ir::Expression::Function {
                                    function: ir::FunctionName::new("__cypher_properties")
                                        .expect("static name"),
                                    arguments: vec![argument.clone()],
                                },
                                value_type: ir::ValueType::Map,
                                nullability: argument.nullability,
                            });
                        }
                        ("properties", ir::ValueType::Map) => {
                            return Ok(argument.clone());
                        }
                        ("keys", ir::ValueType::Node | ir::ValueType::Relationship) => {
                            let properties = ir::TypedExpression {
                                expression: ir::Expression::Function {
                                    function: ir::FunctionName::new("__cypher_properties")
                                        .expect("static name"),
                                    arguments: vec![argument.clone()],
                                },
                                value_type: ir::ValueType::Map,
                                nullability: argument.nullability,
                            };
                            return Ok(ir::TypedExpression {
                                expression: ir::Expression::Function {
                                    function: ir::FunctionName::new("__cypher_keys")
                                        .expect("static name"),
                                    arguments: vec![properties],
                                },
                                value_type: ir::ValueType::List(Box::new(ir::ValueType::Text)),
                                nullability: argument.nullability,
                            });
                        }
                        _ => {}
                    }
                }
                if let ("reverse", [argument]) = (
                    name.value.to_ascii_lowercase().as_str(),
                    arguments.as_slice(),
                ) {
                    // List reversal reorders elements; core's reverse() is a
                    // string reverse and would flip the JSON text itself.
                    if matches!(argument.value_type, ir::ValueType::List(_)) {
                        return Ok(ir::TypedExpression {
                            expression: ir::Expression::Function {
                                function: ir::FunctionName::new("__cypher_list_reverse")
                                    .expect("static name"),
                                arguments: vec![argument.clone()],
                            },
                            value_type: argument.value_type.clone(),
                            nullability: argument.nullability,
                        });
                    }
                }
                if let ("nodes" | "relationships" | "length", [argument]) = (
                    name.value.to_ascii_lowercase().as_str(),
                    arguments.as_slice(),
                ) {
                    if argument.value_type == ir::ValueType::Path {
                        let lowered_name = name.value.to_ascii_lowercase();
                        let part = |field: &str, value_type: ir::ValueType| {
                            sql_call(
                                "json_extract",
                                vec![
                                    argument.clone(),
                                    ir::TypedExpression {
                                        expression: ir::Expression::Literal(ir::Literal::Text(
                                            field.to_owned(),
                                        )),
                                        value_type: ir::ValueType::Text,
                                        nullability: ir::Nullability::NonNull,
                                    },
                                ],
                                value_type,
                            )
                        };
                        return Ok(match lowered_name.as_str() {
                            "nodes" => part(
                                "$.nodes",
                                ir::ValueType::List(Box::new(ir::ValueType::Node)),
                            ),
                            "relationships" => part(
                                "$.relationships",
                                ir::ValueType::List(Box::new(ir::ValueType::Relationship)),
                            ),
                            _ => sql_call(
                                "json_array_length",
                                vec![part(
                                    "$.relationships",
                                    ir::ValueType::List(Box::new(ir::ValueType::Relationship)),
                                )],
                                ir::ValueType::Integer,
                            ),
                        });
                    }
                    if name.value.eq_ignore_ascii_case("length") {
                        // length() over lists and strings behaves like size().
                        return Ok(sql_call(
                            "__cypher_size",
                            vec![argument.clone()],
                            ir::ValueType::Integer,
                        ));
                    }
                }
                if let Some(temporal) = temporal_constructor(
                    &name.value.to_ascii_lowercase(),
                    &arguments,
                    expression.span,
                )? {
                    return Ok(temporal);
                }
                if let Some(duration) = duration_constructor(&name.value, &arguments) {
                    return Ok(duration);
                }
                if let Some(truncated) = temporal_truncate_call(&name.value, &arguments) {
                    return Ok(truncated);
                }
                if let Some(rewritten) =
                    rewrite_builtin_call(&name.value, &arguments, expression.span)?
                {
                    return Ok(rewritten);
                }
                let function = ir::FunctionName::new(name.value.clone())
                    .ok_or_else(|| at_unsupported(name.span, "empty function names"))?;
                let (value_type, nullability) = match crate::functions::lookup(function.as_str()) {
                    Some(signature) => {
                        if let Err(feature) =
                            crate::functions::validate_arguments(&signature, &arguments)
                        {
                            return Err(at_unsupported(expression.span, feature));
                        }
                        (
                            (signature.return_type)(&arguments),
                            ir::Nullability::Nullable,
                        )
                    }
                    None => (ir::ValueType::Any, ir::Nullability::Nullable),
                };
                (
                    ir::Expression::Function {
                        function,
                        arguments,
                    },
                    value_type,
                    nullability,
                )
            }
            cypher::Expression::List(values) => {
                let values = values
                    .iter()
                    .map(|value| self.bind_expression(value))
                    .collect::<Result<Vec<_>, _>>()?;
                let element_type = values
                    .first()
                    .map(|value| value.value_type.clone())
                    .filter(|first| values.iter().all(|value| &value.value_type == first))
                    .unwrap_or(ir::ValueType::Any);
                (
                    ir::Expression::List(values),
                    ir::ValueType::List(Box::new(element_type)),
                    ir::Nullability::NonNull,
                )
            }
            cypher::Expression::Map(entries) => {
                // General map values lower to JSON objects; struct/union
                // property targets still bind through bind_map_property.
                let entries = entries
                    .iter()
                    .map(|(key, value)| Ok((key.value.clone(), self.bind_expression(value)?)))
                    .collect::<Result<Vec<_>, BindError>>()?;
                (
                    ir::Expression::Map(entries),
                    ir::ValueType::Map,
                    ir::Nullability::NonNull,
                )
            }
        };
        Ok(ir::TypedExpression {
            expression: expression_ir,
            value_type,
            nullability,
        })
    }

    fn bind_map_property(
        &self,
        target: &ir::ValueType,
        nullability: ir::Nullability,
        entries: &[(cypher::Spanned<String>, cypher::Spanned<cypher::Expression>)],
        span: cypher::Span,
    ) -> Result<ir::TypedExpression, BindError> {
        match target {
            ir::ValueType::Struct(fields) => {
                if entries.len() != fields.len() {
                    return Err(at_unsupported(span, "struct literal field count mismatch"));
                }
                let mut bound = Vec::with_capacity(entries.len());
                for (name, value) in entries {
                    let field_type = fields
                        .iter()
                        .find(|(field_name, _)| field_name == &name.value)
                        .map(|(_, field_type)| field_type)
                        .ok_or_else(|| BindError::UnknownProperty {
                            name: name.value.clone(),
                            span_start: name.span.start,
                            span_end: name.span.end,
                        })?;
                    let bound_value = self.bind_expression(value)?;
                    if &bound_value.value_type != field_type {
                        return Err(at_unsupported(value.span, "struct field type mismatch"));
                    }
                    bound.push((name.value.clone(), bound_value));
                }
                Ok(ir::TypedExpression {
                    expression: ir::Expression::Map(bound),
                    value_type: target.clone(),
                    nullability,
                })
            }
            ir::ValueType::Union(variants) => {
                if entries.len() != 1 {
                    return Err(at_unsupported(
                        span,
                        "union literal must set exactly one variant",
                    ));
                }
                let (name, value) = &entries[0];
                let variant_type = variants
                    .iter()
                    .find(|(variant_name, _)| variant_name == &name.value)
                    .map(|(_, variant_type)| variant_type)
                    .ok_or_else(|| BindError::UnknownProperty {
                        name: name.value.clone(),
                        span_start: name.span.start,
                        span_end: name.span.end,
                    })?;
                let bound_value = self.bind_expression(value)?;
                if &bound_value.value_type != variant_type {
                    return Err(at_unsupported(value.span, "union variant type mismatch"));
                }
                Ok(ir::TypedExpression {
                    expression: ir::Expression::Map(vec![(name.value.clone(), bound_value)]),
                    value_type: target.clone(),
                    nullability,
                })
            }
            _ => Err(at_unsupported(
                span,
                "map literal outside a struct or union property",
            )),
        }
    }

    /// The projection expression for a scope binding carried through by `*`:
    /// named paths are values composed from their component bindings (or the
    /// column a previous projection materialized), never a bare reference to
    /// the path binding itself, which no plan column backs.
    fn scope_binding_expression(
        &self,
        binding: &ir::Binding,
        span: cypher::Span,
    ) -> Result<ir::TypedExpression, BindError> {
        match self.path_compositions.get(binding.name()) {
            Some(PathComposition::Fixed {
                nodes,
                relationships,
            }) => Ok(ir::TypedExpression {
                expression: ir::Expression::PathValue {
                    nodes: nodes.clone(),
                    relationships: relationships.clone(),
                },
                value_type: ir::ValueType::Path,
                nullability: ir::Nullability::NonNull,
            }),
            Some(PathComposition::Expanded(materialized)) => Ok(ir::TypedExpression {
                expression: ir::Expression::Binding(*materialized),
                value_type: ir::ValueType::Path,
                nullability: ir::Nullability::NonNull,
            }),
            Some(PathComposition::Unsupported) => {
                Err(at_unsupported(span, "variable-length path values"))
            }
            None => Ok(ir::TypedExpression {
                expression: ir::Expression::Binding(binding.id()),
                value_type: binding.value_type().clone(),
                nullability: binding.nullability(),
            }),
        }
    }

    /// A projection materializes every surviving path value as an output
    /// column and drops the component bindings, so compositions re-anchor on
    /// the output bindings and stale entries disappear.
    fn rematerialize_path_compositions(&mut self) {
        self.path_compositions = self
            .scope
            .iter()
            .filter(|binding| matches!(binding.value_type(), ir::ValueType::Path))
            .map(|binding| {
                (
                    binding.name().to_owned(),
                    PathComposition::Expanded(binding.id()),
                )
            })
            .collect();
    }

    fn resolve_binding(&self, name: &str, span: cypher::Span) -> Result<&ir::Binding, BindError> {
        self.scope
            .iter()
            .find(|binding| binding.name() == name)
            .ok_or_else(|| BindError::UnknownVariable {
                name: name.to_owned(),
                span_start: span.start,
                span_end: span.end,
            })
    }

    fn resolve_property(
        &self,
        entity: CatalogEntity,
        name: &cypher::Spanned<String>,
    ) -> Result<ResolvedProperty, BindError> {
        self.catalog
            .property(self.graph, entity, &name.value)
            .ok_or_else(|| BindError::UnknownProperty {
                name: name.value.clone(),
                span_start: name.span.start,
                span_end: name.span.end,
            })
    }

    fn resolve_field(
        &self,
        base_type: &ir::ValueType,
        name: &cypher::Spanned<String>,
    ) -> Result<ir::ValueType, BindError> {
        let fields: &[(String, ir::ValueType)] = match base_type {
            ir::ValueType::Struct(fields) => fields,
            ir::ValueType::Union(variants) => variants,
            _ => {
                return Err(BindError::InvalidPropertyTarget {
                    span_start: name.span.start,
                    span_end: name.span.end,
                })
            }
        };
        fields
            .iter()
            .find(|(field_name, _)| field_name == &name.value)
            .map(|(_, field_type)| field_type.clone())
            .ok_or_else(|| BindError::UnknownProperty {
                name: name.value.clone(),
                span_start: name.span.start,
                span_end: name.span.end,
            })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "entity binding construction keeps the declared metadata explicit"
    )]
    fn new_entity_binding(
        &mut self,
        variable: Option<&cypher::Spanned<String>>,
        anonymous_prefix: &str,
        value_type: ir::ValueType,
        nullability: ir::Nullability,
        kind: CatalogEntity,
        names: Vec<String>,
        span: cypher::Span,
    ) -> Result<ir::Binding, BindError> {
        if let Some(variable) = variable {
            if self
                .scope
                .iter()
                .any(|binding| binding.name() == variable.value)
            {
                return Err(BindError::DuplicateVariable {
                    name: variable.value.clone(),
                    span_start: variable.span.start,
                    span_end: variable.span.end,
                });
            }
        }
        let id = self.next_id()?;
        let name = variable
            .map(|variable| variable.value.clone())
            .unwrap_or_else(|| format!("{anonymous_prefix}_{}", id.get()));
        let binding =
            ir::Binding::new(id, name, value_type, nullability).map_err(BindError::InvalidPlan)?;
        self.scope.push(binding.clone());
        self.entities.insert(id, EntityBinding { kind, names });
        let _ = span;
        Ok(binding)
    }

    fn next_id(&mut self) -> Result<ir::BindingId, BindError> {
        let value = self.next_binding;
        self.next_binding = self
            .next_binding
            .checked_add(1)
            .ok_or(BindError::TooManyBindings)?;
        ir::BindingId::new(value).map_err(|_| BindError::TooManyBindings)
    }

    fn wrap_plan(&mut self, kind: ir::PlanKind) -> Result<(), BindError> {
        self.plan = Some(ir::Plan::new(
            kind,
            ir::Scope::new(self.scope.clone())?,
            ir::ResultShape::default(),
        )?);
        Ok(())
    }
}

/// Flattens a chain of Cypher `Property` nodes (`n.a.b.c`) into its root
/// expression and an ordered list of field names, root-to-leaf. For `n.a`,
/// returns `(n, [a])`; for `n.a.b`, returns `(n, [a, b])`.
fn flatten_property_chain(
    expression: &cypher::Spanned<cypher::Expression>,
) -> (
    &cypher::Spanned<cypher::Expression>,
    Vec<&cypher::Spanned<String>>,
) {
    match &expression.value {
        cypher::Expression::Property { entity, name } => {
            let (root, mut fields) = flatten_property_chain(entity);
            fields.push(name);
            (root, fields)
        }
        _ => (expression, Vec::new()),
    }
}

fn bind_literal(literal: &cypher::Literal) -> (ir::Literal, ir::ValueType, ir::Nullability) {
    match literal {
        cypher::Literal::Null => (
            ir::Literal::Null,
            ir::ValueType::Any,
            ir::Nullability::Nullable,
        ),
        cypher::Literal::Boolean(value) => (
            ir::Literal::Boolean(*value),
            ir::ValueType::Boolean,
            ir::Nullability::NonNull,
        ),
        cypher::Literal::Integer(value) => (
            ir::Literal::Integer(*value),
            ir::ValueType::Integer,
            ir::Nullability::NonNull,
        ),
        cypher::Literal::Real(value) => (
            ir::Literal::Real(*value),
            ir::ValueType::Real,
            ir::Nullability::NonNull,
        ),
        cypher::Literal::Text(value) => (
            ir::Literal::Text(value.clone()),
            ir::ValueType::Text,
            ir::Nullability::NonNull,
        ),
    }
}

/// Marker type for temporal values, which are stored as ISO-8601 text and
/// manipulated through the core time_* functions.
fn temporal_value_type() -> ir::ValueType {
    ir::ValueType::Custom {
        name: "cypher_temporal".to_owned(),
        base: Box::new(ir::ValueType::Text),
    }
}

/// Marker type for duration values, stored as canonical ISO-8601 text and
/// manipulated through the duration_* extension functions.
fn duration_value_type() -> ir::ValueType {
    ir::ValueType::Custom {
        name: "cypher_duration".to_owned(),
        base: Box::new(ir::ValueType::Text),
    }
}

fn is_duration_unit(name: &str) -> bool {
    matches!(
        name,
        "years"
            | "quarters"
            | "months"
            | "weeks"
            | "days"
            | "hours"
            | "minutes"
            | "seconds"
            | "milliseconds"
            | "microseconds"
            | "nanoseconds"
            | "monthsOfYear"
            | "minutesOfHour"
            | "secondsOfMinute"
            | "millisecondsOfSecond"
            | "microsecondsOfSecond"
            | "nanosecondsOfSecond"
    )
}

/// Builds a duration constructor over the duration_* extension functions.
/// Map components fold into the four stored fields (months, days, seconds,
/// nanoseconds) with bind-time arithmetic so expression values still work.
fn duration_constructor(
    name: &str,
    arguments: &[ir::TypedExpression],
) -> Option<ir::TypedExpression> {
    let finish = |mut value: ir::TypedExpression| {
        value.value_type = duration_value_type();
        Some(value)
    };
    match (name, arguments) {
        ("duration", [argument]) => match &argument.expression {
            ir::Expression::Map(entries) => {
                const GROUPS: [&[(&str, i64)]; 4] = [
                    &[("years", 12), ("quarters", 3), ("months", 1)],
                    &[("weeks", 7), ("days", 1)],
                    &[("hours", 3600), ("minutes", 60), ("seconds", 1)],
                    &[
                        ("milliseconds", 1_000_000),
                        ("microseconds", 1_000),
                        ("nanoseconds", 1),
                    ],
                ];
                let known = |key: &str| {
                    GROUPS
                        .iter()
                        .any(|group| group.iter().any(|(unit, _)| *unit == key))
                };
                if entries.iter().any(|(key, _)| !known(key)) {
                    return None;
                }
                let fields = GROUPS
                    .iter()
                    .map(|group| component_sum(entries, group))
                    .collect();
                finish(sql_call("duration_make", fields, ir::ValueType::Text))
            }
            _ if matches!(
                argument.value_type,
                ir::ValueType::Text | ir::ValueType::Any
            ) =>
            {
                finish(sql_call(
                    "duration_parse",
                    vec![argument.clone()],
                    ir::ValueType::Text,
                ))
            }
            _ => None,
        },
        (
            "duration.between" | "duration.inMonths" | "duration.inDays" | "duration.inSeconds",
            [start, end],
        ) => {
            let mode = match name {
                "duration.inMonths" => "months",
                "duration.inDays" => "days",
                "duration.inSeconds" => "seconds",
                _ => "between",
            };
            finish(sql_call(
                "duration_between",
                vec![
                    start.clone(),
                    end.clone(),
                    ir::TypedExpression {
                        expression: ir::Expression::Literal(ir::Literal::Text(mode.to_owned())),
                        value_type: ir::ValueType::Text,
                        nullability: ir::Nullability::NonNull,
                    },
                ],
                ir::ValueType::Text,
            ))
        }
        _ => None,
    }
}

/// Sums `value * scale` over the map entries present in `scales`, as a
/// bind-time integer expression (0 when no component is present).
fn component_sum(
    entries: &[(String, ir::TypedExpression)],
    scales: &[(&str, i64)],
) -> ir::TypedExpression {
    let integer_literal = |value: i64| ir::TypedExpression {
        expression: ir::Expression::Literal(ir::Literal::Integer(value)),
        value_type: ir::ValueType::Integer,
        nullability: ir::Nullability::NonNull,
    };
    let mut total: Option<ir::TypedExpression> = None;
    for (unit, scale) in scales {
        let Some((_, value)) = entries.iter().find(|(key, _)| key == unit) else {
            continue;
        };
        let scaled = if *scale == 1 {
            value.clone()
        } else {
            ir::TypedExpression {
                expression: ir::Expression::Binary {
                    left: Box::new(value.clone()),
                    op: ir::BinaryOp::Multiply,
                    right: Box::new(integer_literal(*scale)),
                },
                value_type: ir::ValueType::Integer,
                nullability: value.nullability,
            }
        };
        total = Some(match total {
            None => scaled,
            Some(current) => ir::TypedExpression {
                expression: ir::Expression::Binary {
                    left: Box::new(current),
                    op: ir::BinaryOp::Add,
                    right: Box::new(scaled),
                },
                value_type: ir::ValueType::Integer,
                nullability: ir::Nullability::Nullable,
            },
        });
    }
    total.unwrap_or_else(|| integer_literal(0))
}

impl Binder<'_> {
    /// jsonb operators map onto the frontend's jsonb_* extension
    /// functions; an entity operand means its property map.
    fn jsonb_operator(
        &self,
        operator: cypher::BinaryOperator,
        left: &ir::TypedExpression,
        right: &ir::TypedExpression,
    ) -> Option<ir::TypedExpression> {
        let (function, boolean, swap) = match operator {
            cypher::BinaryOperator::JsonGet => ("jsonb_get", false, false),
            cypher::BinaryOperator::JsonGetText => ("jsonb_get_text", false, false),
            cypher::BinaryOperator::JsonPath => ("jsonb_get_path", false, false),
            cypher::BinaryOperator::JsonPathText => ("jsonb_get_path", false, false),
            cypher::BinaryOperator::JsonExists => ("jsonb_exists", true, false),
            cypher::BinaryOperator::JsonExistsAny => ("jsonb_exists_any", true, false),
            cypher::BinaryOperator::JsonExistsAll => ("jsonb_exists_all", true, false),
            cypher::BinaryOperator::JsonContains => ("jsonb_contains", true, false),
            cypher::BinaryOperator::JsonContainedBy => ("jsonb_contains", true, true),
            _ => return None,
        };
        let materialize = |value: &ir::TypedExpression| {
            if let ir::Expression::Binding(id) = &value.expression {
                if self.entities.contains_key(id) {
                    return sql_call(
                        "__cypher_properties",
                        vec![value.clone()],
                        ir::ValueType::Map,
                    );
                }
            }
            value.clone()
        };
        let (first, second) = if swap {
            (materialize(right), materialize(left))
        } else {
            (materialize(left), materialize(right))
        };
        Some(sql_call(
            function,
            vec![first, second],
            if boolean {
                ir::ValueType::Boolean
            } else {
                ir::ValueType::Any
            },
        ))
    }
}

/// pgvector distance operators map onto core's vector functions; the
/// negative-inner-product operator negates vector_distance_dot.
fn vector_operator(
    operator: cypher::BinaryOperator,
    left: &ir::TypedExpression,
    right: &ir::TypedExpression,
) -> Option<ir::TypedExpression> {
    let function = match operator {
        cypher::BinaryOperator::VectorL2 => "vector_distance_l2",
        cypher::BinaryOperator::VectorCosine => "vector_distance_cos",
        cypher::BinaryOperator::VectorInnerProduct => "vector_distance_dot",
        _ => return None,
    };
    let call = sql_call(
        function,
        vec![left.clone(), right.clone()],
        ir::ValueType::Real,
    );
    Some(if operator == cypher::BinaryOperator::VectorInnerProduct {
        ir::TypedExpression {
            expression: ir::Expression::Unary {
                op: ir::UnaryOp::Negate,
                expression: Box::new(call),
            },
            value_type: ir::ValueType::Real,
            nullability: ir::Nullability::Nullable,
        }
    } else {
        call
    })
}

/// Rewrites datetime/duration arithmetic onto the duration extension
/// functions. Returns None when neither operand carries a temporal or
/// duration marker type.
fn duration_arithmetic(
    operator: cypher::BinaryOperator,
    left: &ir::TypedExpression,
    right: &ir::TypedExpression,
) -> Option<ir::TypedExpression> {
    let temporal = temporal_value_type();
    let duration = duration_value_type();
    let typed = |mut value: ir::TypedExpression, value_type: ir::ValueType| {
        value.value_type = value_type;
        Some(value)
    };
    match operator {
        cypher::BinaryOperator::Add => {
            if left.value_type == temporal && right.value_type == duration {
                typed(
                    sql_call(
                        "datetime_add_duration",
                        vec![left.clone(), right.clone()],
                        ir::ValueType::Text,
                    ),
                    temporal,
                )
            } else if left.value_type == duration && right.value_type == temporal {
                typed(
                    sql_call(
                        "datetime_add_duration",
                        vec![right.clone(), left.clone()],
                        ir::ValueType::Text,
                    ),
                    temporal,
                )
            } else if left.value_type == duration && right.value_type == duration {
                typed(
                    sql_call(
                        "duration_add",
                        vec![left.clone(), right.clone()],
                        ir::ValueType::Text,
                    ),
                    duration,
                )
            } else {
                None
            }
        }
        cypher::BinaryOperator::Subtract => {
            if left.value_type == temporal && right.value_type == duration {
                typed(
                    sql_call(
                        "datetime_sub_duration",
                        vec![left.clone(), right.clone()],
                        ir::ValueType::Text,
                    ),
                    temporal,
                )
            } else if left.value_type == duration && right.value_type == duration {
                let negated = sql_call("duration_neg", vec![right.clone()], ir::ValueType::Text);
                typed(
                    sql_call(
                        "duration_add",
                        vec![left.clone(), negated],
                        ir::ValueType::Text,
                    ),
                    duration,
                )
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Rewrites `<kind>.truncate(unit, value[, overrides])` onto the
/// temporal_truncate extension function.
fn temporal_truncate_call(
    name: &str,
    arguments: &[ir::TypedExpression],
) -> Option<ir::TypedExpression> {
    let kind = name.strip_suffix(".truncate")?;
    if !matches!(
        kind,
        "datetime" | "localdatetime" | "date" | "localtime" | "time"
    ) {
        return None;
    }
    let (unit, value, overrides) = match arguments {
        [unit, value] => (unit, value, None),
        [unit, value, overrides] => (unit, value, Some(overrides)),
        _ => return None,
    };
    let kind_literal = ir::TypedExpression {
        expression: ir::Expression::Literal(ir::Literal::Text(kind.to_owned())),
        value_type: ir::ValueType::Text,
        nullability: ir::Nullability::NonNull,
    };
    let mut call_arguments = vec![kind_literal, unit.clone(), value.clone()];
    if let Some(overrides) = overrides {
        call_arguments.push(overrides.clone());
    }
    let mut call = sql_call("temporal_truncate", call_arguments, ir::ValueType::Text);
    call.value_type = temporal_value_type();
    Some(call)
}

/// `base.field` over a map-shaped value: json_extract guarded by
/// json_valid so non-JSON bases produce null instead of a runtime error.
fn json_member_access(base: ir::TypedExpression, field: &str) -> ir::TypedExpression {
    let path = ir::TypedExpression {
        expression: ir::Expression::Literal(ir::Literal::Text(format!(
            "$.\"{}\"",
            field.replace('"', "\\\"")
        ))),
        value_type: ir::ValueType::Text,
        nullability: ir::Nullability::NonNull,
    };
    let valid = sql_call("json_valid", vec![base.clone()], ir::ValueType::Boolean);
    let extract = sql_call("json_extract", vec![base, path], ir::ValueType::Any);
    ir::TypedExpression {
        expression: ir::Expression::Case {
            subject: None,
            branches: vec![(valid, extract)],
            default: None,
        },
        value_type: ir::ValueType::Any,
        nullability: ir::Nullability::Nullable,
    }
}

fn is_temporal_unit(name: &str) -> bool {
    matches!(
        name,
        "year"
            | "month"
            | "day"
            | "week"
            | "weekYear"
            | "quarter"
            | "dayOfQuarter"
            | "ordinalDay"
            | "dayOfYear"
            | "weekday"
            | "dayOfWeek"
            | "hour"
            | "minute"
            | "second"
            | "millisecond"
            | "microsecond"
            | "nanosecond"
            | "timezone"
            | "offset"
            | "offsetMinutes"
            | "epochSeconds"
            | "epochMillis"
    )
}

fn sql_call(
    name: &str,
    arguments: Vec<ir::TypedExpression>,
    value_type: ir::ValueType,
) -> ir::TypedExpression {
    ir::TypedExpression {
        expression: ir::Expression::Function {
            function: ir::FunctionName::new(name).expect("static SQL function name"),
            arguments,
        },
        value_type,
        nullability: ir::Nullability::Nullable,
    }
}

/// Builds a temporal constructor over the temporal_* extension functions:
/// component maps lower to `temporal_make(kind, json)`, strings to
/// `temporal_parse(kind, text)`, and no arguments to `temporal_now(kind)`.
fn temporal_constructor(
    name: &str,
    arguments: &[ir::TypedExpression],
    span: cypher::Span,
) -> Result<Option<ir::TypedExpression>, BindError> {
    const COMPONENTS: [&str; 22] = [
        "year",
        "month",
        "day",
        "week",
        "dayOfWeek",
        "ordinalDay",
        "quarter",
        "dayOfQuarter",
        "hour",
        "minute",
        "second",
        "millisecond",
        "microsecond",
        "nanosecond",
        "timezone",
        "date",
        "time",
        "datetime",
        "epochSeconds",
        "epochMillis",
        "weekYear",
        "dayOfYear",
    ];
    let kind = match name {
        "datetime" | "localdatetime" | "date" | "localtime" | "time" => name,
        _ => return Ok(None),
    };
    let text_literal = |value: &str| ir::TypedExpression {
        expression: ir::Expression::Literal(ir::Literal::Text(value.to_owned())),
        value_type: ir::ValueType::Text,
        nullability: ir::Nullability::NonNull,
    };
    let finish = |value: ir::TypedExpression| {
        let mut value = value;
        value.value_type = temporal_value_type();
        Ok(Some(value))
    };
    match arguments {
        [] => finish(sql_call(
            "temporal_now",
            vec![text_literal(kind)],
            ir::ValueType::Text,
        )),
        [argument] => match &argument.expression {
            ir::Expression::Map(entries) => {
                if entries
                    .iter()
                    .any(|(key, _)| !COMPONENTS.contains(&key.as_str()))
                {
                    return Err(at_unsupported(
                        span,
                        "temporal constructor components outside the Cypher component set",
                    ));
                }
                finish(sql_call(
                    "temporal_make",
                    vec![text_literal(kind), argument.clone()],
                    ir::ValueType::Text,
                ))
            }
            _ if matches!(
                argument.value_type,
                ir::ValueType::Text | ir::ValueType::Any
            ) =>
            {
                finish(sql_call(
                    "temporal_parse",
                    vec![text_literal(kind), argument.clone()],
                    ir::ValueType::Text,
                ))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

/// Rewrites Cypher builtin calls onto existing IR forms (identity
/// passthrough, casts, index/slice sugar, renamed or sentinel SQL
/// functions). Returns None for names the generic path should handle.
fn rewrite_builtin_call(
    name: &str,
    arguments: &[ir::TypedExpression],
    span: cypher::Span,
) -> Result<Option<ir::TypedExpression>, BindError> {
    let sql_function =
        |sql: &str, args: &[ir::TypedExpression], value_type: ir::ValueType| ir::TypedExpression {
            expression: ir::Expression::Function {
                function: ir::FunctionName::new(sql).expect("static SQL function name"),
                arguments: args.to_vec(),
            },
            value_type,
            nullability: ir::Nullability::Nullable,
        };
    let cast = |argument: &ir::TypedExpression, target: ir::ValueType| ir::TypedExpression {
        expression: ir::Expression::Cast {
            expression: Box::new(argument.clone()),
            target: target.clone(),
        },
        value_type: target,
        nullability: ir::Nullability::Nullable,
    };
    let integer_literal = |value: i64| ir::TypedExpression {
        expression: ir::Expression::Literal(ir::Literal::Integer(value)),
        value_type: ir::ValueType::Integer,
        nullability: ir::Nullability::NonNull,
    };
    let index = |base: &ir::TypedExpression, at: i64| ir::TypedExpression {
        expression: ir::Expression::Index {
            base: Box::new(base.clone()),
            index: Box::new(integer_literal(at)),
        },
        value_type: match &base.value_type {
            ir::ValueType::List(element) => (**element).clone(),
            _ => ir::ValueType::Any,
        },
        nullability: ir::Nullability::Nullable,
    };
    Ok(Some(
        match (name.to_ascii_lowercase().as_str(), arguments) {
            ("id", [entity])
                if matches!(
                    entity.value_type,
                    ir::ValueType::Node | ir::ValueType::Relationship | ir::ValueType::Any
                ) =>
            {
                // The relational binding column is the entity identity.
                ir::TypedExpression {
                    expression: entity.expression.clone(),
                    value_type: ir::ValueType::Integer,
                    nullability: entity.nullability,
                }
            }
            ("cosine_distance", [_, _]) => {
                sql_function("vector_distance_cos", arguments, ir::ValueType::Real)
            }
            ("l2_distance", [_, _]) => {
                sql_function("vector_distance_l2", arguments, ir::ValueType::Real)
            }
            ("inner_product", [_, _]) => {
                sql_function("vector_distance_dot", arguments, ir::ValueType::Real)
            }
            ("toupper" | "touppercase", [_]) => {
                sql_function("upper", arguments, ir::ValueType::Text)
            }
            ("tolower" | "tolowercase", [_]) => {
                sql_function("lower", arguments, ir::ValueType::Text)
            }
            (conversion @ ("tostring" | "tointeger" | "tofloat" | "toboolean"), [argument])
                if !convertible_argument(conversion, &argument.value_type) =>
            {
                return Err(at_unsupported(span, "conversion from this value type"));
            }
            ("tostring", [argument]) => cast(argument, ir::ValueType::Text),
            ("tointeger", [argument]) => cast(argument, ir::ValueType::Integer),
            ("tofloat", [argument]) => cast(argument, ir::ValueType::Real),
            ("toboolean", [argument]) => cast(argument, ir::ValueType::Boolean),
            ("size", [argument]) => {
                // openCypher removed size() over pattern predicates (List6
                // [6]) and rejects it on paths (List6 [5]).
                if matches!(argument.expression, ir::Expression::PatternSubquery { .. })
                    || argument.value_type == ir::ValueType::Path
                {
                    return Err(at_unsupported(
                        span,
                        "size() over paths or pattern predicates",
                    ));
                }
                sql_function("__cypher_size", arguments, ir::ValueType::Integer)
            }
            ("range", [_, _] | [_, _, _]) => {
                // The TCK requires errors for non-integer arguments and a
                // zero step (List11 [4]/[5]).
                for argument in arguments {
                    if !matches!(
                        argument.value_type,
                        ir::ValueType::Integer | ir::ValueType::Any
                    ) {
                        return Err(at_unsupported(span, "range over non-integer arguments"));
                    }
                }
                if let Some(step) = arguments.get(2) {
                    if step.expression == ir::Expression::Literal(ir::Literal::Integer(0)) {
                        return Err(at_unsupported(span, "range with a zero step"));
                    }
                }
                sql_function(
                    "__cypher_range",
                    arguments,
                    ir::ValueType::List(Box::new(ir::ValueType::Integer)),
                )
            }
            ("keys", [_]) => sql_function(
                "__cypher_keys",
                arguments,
                ir::ValueType::List(Box::new(ir::ValueType::Text)),
            ),
            ("rand", []) => sql_function("__cypher_rand", arguments, ir::ValueType::Real),
            ("isempty", [_]) => sql_function("__cypher_isempty", arguments, ir::ValueType::Boolean),
            ("tofloatlist", [_]) => sql_function(
                "__cypher_list_real",
                arguments,
                ir::ValueType::List(Box::new(ir::ValueType::Real)),
            ),
            ("tointegerlist", [_]) => sql_function(
                "__cypher_list_integer",
                arguments,
                ir::ValueType::List(Box::new(ir::ValueType::Integer)),
            ),
            ("tostringlist", [_]) => sql_function(
                "__cypher_list_text",
                arguments,
                ir::ValueType::List(Box::new(ir::ValueType::Text)),
            ),
            ("tobooleanlist", [_]) => sql_function(
                "__cypher_list_boolean",
                arguments,
                ir::ValueType::List(Box::new(ir::ValueType::Boolean)),
            ),
            ("head", [base]) => index(base, 0),
            ("last", [base]) => index(base, -1),
            ("tail", [base]) => ir::TypedExpression {
                expression: ir::Expression::Slice {
                    base: Box::new(base.clone()),
                    from: Some(Box::new(integer_literal(1))),
                    to: None,
                },
                value_type: base.value_type.clone(),
                nullability: ir::Nullability::Nullable,
            },
            ("left", [text, count]) => sql_function(
                "substr",
                &[text.clone(), integer_literal(1), count.clone()],
                ir::ValueType::Text,
            ),
            ("right", [text, count]) => {
                let negated = ir::TypedExpression {
                    expression: ir::Expression::Binary {
                        left: Box::new(integer_literal(0)),
                        op: ir::BinaryOp::Subtract,
                        right: Box::new(count.clone()),
                    },
                    value_type: ir::ValueType::Integer,
                    nullability: count.nullability,
                };
                sql_function("substr", &[text.clone(), negated], ir::ValueType::Text)
            }
            _ => return Ok(None),
        },
    ))
}

/// Shallow scan for references to the named variables: direct uses,
/// property chains, and function arguments — the forms a mutation RETURN
/// can produce over a deleted entity.
fn references_variable(expression: &cypher::Spanned<cypher::Expression>, names: &[String]) -> bool {
    match &expression.value {
        // Returning a deleted entity's value is legal Cypher (a snapshot),
        // but asking for its metadata is a runtime error (TCK Return2 [16]).
        cypher::Expression::Function {
            name, arguments, ..
        } if matches!(
            name.value.to_ascii_lowercase().as_str(),
            "labels" | "type" | "properties"
        ) =>
        {
            arguments.iter().any(|argument| {
                matches!(
                    &argument.value,
                    cypher::Expression::Variable(variable)
                        if names.iter().any(|deleted| deleted == variable)
                )
            })
        }
        cypher::Expression::Function { arguments, .. } => arguments
            .iter()
            .any(|argument| references_variable(argument, names)),
        cypher::Expression::Unary { operand, .. } => references_variable(operand, names),
        cypher::Expression::Binary { left, right, .. } => {
            references_variable(left, names) || references_variable(right, names)
        }
        _ => false,
    }
}

/// True when any bare `Variable` in the expression names one of `names`,
/// recursing through every syntactic form.
fn names_referenced(expression: &cypher::Spanned<cypher::Expression>, names: &[String]) -> bool {
    let sub = |value: &cypher::Spanned<cypher::Expression>| names_referenced(value, names);
    match &expression.value {
        cypher::Expression::Variable(name) => names.iter().any(|candidate| candidate == name),
        cypher::Expression::Property { entity, .. } => sub(entity),
        cypher::Expression::Function { arguments, .. } => arguments.iter().any(sub),
        cypher::Expression::Unary { operand, .. } => sub(operand),
        cypher::Expression::Binary { left, right, .. } => sub(left) || sub(right),
        cypher::Expression::Case {
            subject,
            branches,
            default,
        } => {
            subject.as_deref().is_some_and(sub)
                || branches.iter().any(|(when, then)| sub(when) || sub(then))
                || default.as_deref().is_some_and(sub)
        }
        cypher::Expression::Index { base, index } => sub(base) || sub(index),
        cypher::Expression::Slice { base, from, to } => {
            sub(base) || from.as_deref().is_some_and(sub) || to.as_deref().is_some_and(sub)
        }
        cypher::Expression::Cast { operand, .. } => sub(operand),
        cypher::Expression::List(values) => values.iter().any(sub),
        cypher::Expression::Map(entries) => entries.iter().any(|(_, value)| sub(value)),
        cypher::Expression::Quantifier {
            variable,
            list,
            predicate,
            ..
        } => {
            // The loop variable shadows an outer name inside the predicate.
            let inner: Vec<String> = names
                .iter()
                .filter(|name| **name != variable.value)
                .cloned()
                .collect();
            sub(list) || names_referenced(predicate, &inner)
        }
        cypher::Expression::ListComprehension {
            list,
            predicate,
            map,
            ..
        } => sub(list) || predicate.as_deref().is_some_and(sub) || map.as_deref().is_some_and(sub),
        _ => false,
    }
}

/// Renames variable occurrences in a MATCH clause per `rename`: pattern
/// variables, property-map values, and the predicate.
fn rename_match_clause(
    clause: &cypher::MatchClause,
    rename: &std::collections::HashMap<String, String>,
) -> cypher::MatchClause {
    let rename_name = |name: &cypher::Spanned<String>| {
        cypher::Spanned::new(
            rename
                .get(&name.value)
                .cloned()
                .unwrap_or_else(|| name.value.clone()),
            name.span,
        )
    };
    let rename_properties =
        |properties: &[(cypher::Spanned<String>, cypher::Spanned<cypher::Expression>)]| {
            properties
                .iter()
                .map(|(key, value)| (key.clone(), rename_expression(value, rename)))
                .collect()
        };
    let rename_node = |node: &cypher::NodePattern| cypher::NodePattern {
        variable: node.variable.as_ref().map(rename_name),
        labels: node.labels.clone(),
        properties: rename_properties(&node.properties),
        has_property_map: node.has_property_map,
        span: node.span,
    };
    cypher::MatchClause {
        optional: clause.optional,
        paths: clause
            .paths
            .iter()
            .map(|path| cypher::PathPattern {
                variable: path.variable.clone(),
                start: rename_node(&path.start),
                steps: path
                    .steps
                    .iter()
                    .map(|(relationship, node)| {
                        (
                            cypher::RelationshipPattern {
                                variable: relationship.variable.as_ref().map(rename_name),
                                types: relationship.types.clone(),
                                direction: relationship.direction,
                                range: relationship.range.clone(),
                                properties: rename_properties(&relationship.properties),
                                span: relationship.span,
                            },
                            rename_node(node),
                        )
                    })
                    .collect(),
                span: path.span,
            })
            .collect(),
        predicate: clause
            .predicate
            .as_ref()
            .map(|predicate| rename_expression(predicate, rename)),
    }
}

/// Replaces bare `Variable` occurrences with their aliased source
/// expressions (projection aliases visible to ORDER BY/WHERE), skipping
/// names shadowed by quantifier or list-comprehension loop variables.
/// Span-insensitive structural equality for the expression shapes that occur
/// as post-aggregation sort keys: variables, property chains, literals, and
/// (aggregate) function calls. Spanned's derived PartialEq compares byte
/// spans, so it can never equate a sort key with a projection source.
fn expressions_match(a: &cypher::Expression, b: &cypher::Expression) -> bool {
    use cypher::Expression as E;
    match (a, b) {
        (E::Variable(x), E::Variable(y)) => x == y,
        (E::Parameter(x), E::Parameter(y)) => x == y,
        (E::Literal(x), E::Literal(y)) => x == y,
        (
            E::Property {
                entity: entity_a,
                name: name_a,
            },
            E::Property {
                entity: entity_b,
                name: name_b,
            },
        ) => name_a.value == name_b.value && expressions_match(&entity_a.value, &entity_b.value),
        (
            E::Function {
                name: name_a,
                arguments: arguments_a,
                distinct: distinct_a,
                star: star_a,
            },
            E::Function {
                name: name_b,
                arguments: arguments_b,
                distinct: distinct_b,
                star: star_b,
            },
        ) => {
            name_a.value.eq_ignore_ascii_case(&name_b.value)
                && distinct_a == distinct_b
                && star_a == star_b
                && arguments_a.len() == arguments_b.len()
                && arguments_a
                    .iter()
                    .zip(arguments_b)
                    .all(|(x, y)| expressions_match(&x.value, &y.value))
        }
        _ => false,
    }
}

/// Rewrite every sub-expression for which `alias_for` yields an output alias
/// into a bare variable reference to that alias, top-down, so post-aggregation
/// sort keys built over projection sources bind in the output scope.
fn replace_projected_sources(
    expression: &cypher::Spanned<cypher::Expression>,
    alias_for: &dyn Fn(&cypher::Expression) -> Option<String>,
) -> cypher::Spanned<cypher::Expression> {
    use cypher::Expression as E;
    if let Some(alias) = alias_for(&expression.value) {
        return cypher::Spanned::new(E::Variable(alias), expression.span);
    }
    let sub = |value: &cypher::Spanned<E>| Box::new(replace_projected_sources(value, alias_for));
    let sub_opt = |value: &Option<Box<cypher::Spanned<E>>>| value.as_deref().map(sub);
    let value = match &expression.value {
        E::Property { entity, name } => E::Property {
            entity: sub(entity),
            name: name.clone(),
        },
        E::Binary {
            left,
            operator,
            right,
        } => E::Binary {
            left: sub(left),
            operator: *operator,
            right: sub(right),
        },
        E::Unary { operator, operand } => E::Unary {
            operator: *operator,
            operand: sub(operand),
        },
        E::Function {
            name,
            arguments,
            distinct,
            star,
        } => E::Function {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| replace_projected_sources(argument, alias_for))
                .collect(),
            distinct: *distinct,
            star: *star,
        },
        E::Case {
            subject,
            branches,
            default,
        } => E::Case {
            subject: sub_opt(subject),
            branches: branches
                .iter()
                .map(|(when, then)| {
                    (
                        replace_projected_sources(when, alias_for),
                        replace_projected_sources(then, alias_for),
                    )
                })
                .collect(),
            default: sub_opt(default),
        },
        E::Index { base, index } => E::Index {
            base: sub(base),
            index: sub(index),
        },
        E::Slice { base, from, to } => E::Slice {
            base: sub(base),
            from: sub_opt(from),
            to: sub_opt(to),
        },
        E::Cast { operand, type_name } => E::Cast {
            operand: sub(operand),
            type_name: type_name.clone(),
        },
        E::List(values) => E::List(
            values
                .iter()
                .map(|value| replace_projected_sources(value, alias_for))
                .collect(),
        ),
        E::Map(entries) => E::Map(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), replace_projected_sources(value, alias_for)))
                .collect(),
        ),
        other => other.clone(),
    };
    cypher::Spanned::new(value, expression.span)
}

fn substitute_variables(
    expression: &cypher::Spanned<cypher::Expression>,
    sources: &HashMap<String, cypher::Expression>,
    shadowed: &std::collections::HashSet<String>,
) -> cypher::Spanned<cypher::Expression> {
    if sources.is_empty() {
        return expression.clone();
    }
    use cypher::Expression as E;
    let sub = |value: &cypher::Spanned<E>| Box::new(substitute_variables(value, sources, shadowed));
    let sub_opt = |value: &Option<Box<cypher::Spanned<E>>>| value.as_deref().map(sub);
    let value = match &expression.value {
        E::Variable(name) => match sources.get(name) {
            Some(source) => source.clone(),
            None => expression.value.clone(),
        },
        E::Property { entity, name } => E::Property {
            // A property base whose alias shadows a pre-projection variable
            // (RETURN n.name AS n ORDER BY n.name) keeps the original
            // entity resolution; non-shadowing aliases substitute normally.
            entity: match &entity.value {
                E::Variable(base) if shadowed.contains(base) => entity.clone(),
                _ => sub(entity),
            },
            name: name.clone(),
        },
        E::Binary {
            left,
            operator,
            right,
        } => E::Binary {
            left: sub(left),
            operator: *operator,
            right: sub(right),
        },
        E::Unary { operator, operand } => E::Unary {
            operator: *operator,
            operand: sub(operand),
        },
        E::Function {
            name,
            arguments,
            distinct,
            star,
        } => E::Function {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| substitute_variables(argument, sources, shadowed))
                .collect(),
            distinct: *distinct,
            star: *star,
        },
        E::Case {
            subject,
            branches,
            default,
        } => E::Case {
            subject: sub_opt(subject),
            branches: branches
                .iter()
                .map(|(when, then)| {
                    (
                        substitute_variables(when, sources, shadowed),
                        substitute_variables(then, sources, shadowed),
                    )
                })
                .collect(),
            default: sub_opt(default),
        },
        E::Index { base, index } => E::Index {
            base: sub(base),
            index: sub(index),
        },
        E::Slice { base, from, to } => E::Slice {
            base: sub(base),
            from: sub_opt(from),
            to: sub_opt(to),
        },
        E::Cast { operand, type_name } => E::Cast {
            operand: sub(operand),
            type_name: type_name.clone(),
        },
        E::List(values) => E::List(
            values
                .iter()
                .map(|value| substitute_variables(value, sources, shadowed))
                .collect(),
        ),
        E::Map(entries) => E::Map(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), substitute_variables(value, sources, shadowed)))
                .collect(),
        ),
        E::Quantifier {
            kind,
            variable,
            list,
            predicate,
        } => {
            let shadowed = sources.contains_key(&variable.value);
            E::Quantifier {
                kind: *kind,
                variable: variable.clone(),
                list: sub(list),
                predicate: if shadowed {
                    predicate.clone()
                } else {
                    sub(predicate)
                },
            }
        }
        E::ListComprehension {
            variable,
            list,
            predicate,
            map,
        } => {
            let shadowed = sources.contains_key(&variable.value);
            E::ListComprehension {
                variable: variable.clone(),
                list: sub(list),
                predicate: if shadowed {
                    predicate.clone()
                } else {
                    sub_opt(predicate)
                },
                map: if shadowed { map.clone() } else { sub_opt(map) },
            }
        }
        other => other.clone(),
    };
    cypher::Spanned::new(value, expression.span)
}

/// Renames bare `Variable` occurrences in an expression, skipping names
/// shadowed by quantifier or list-comprehension loop variables.
fn rename_expression(
    expression: &cypher::Spanned<cypher::Expression>,
    rename: &std::collections::HashMap<String, String>,
) -> cypher::Spanned<cypher::Expression> {
    use cypher::Expression as E;
    let sub = |value: &cypher::Spanned<E>| Box::new(rename_expression(value, rename));
    let sub_opt = |value: &Option<Box<cypher::Spanned<E>>>| value.as_deref().map(sub);
    let value = match &expression.value {
        E::Variable(name) => E::Variable(rename.get(name).cloned().unwrap_or_else(|| name.clone())),
        E::Property { entity, name } => E::Property {
            entity: sub(entity),
            name: name.clone(),
        },
        E::Binary {
            left,
            operator,
            right,
        } => E::Binary {
            left: sub(left),
            operator: *operator,
            right: sub(right),
        },
        E::Unary { operator, operand } => E::Unary {
            operator: *operator,
            operand: sub(operand),
        },
        E::Function {
            name,
            arguments,
            distinct,
            star,
        } => E::Function {
            name: name.clone(),
            arguments: arguments
                .iter()
                .map(|argument| rename_expression(argument, rename))
                .collect(),
            distinct: *distinct,
            star: *star,
        },
        E::Case {
            subject,
            branches,
            default,
        } => E::Case {
            subject: sub_opt(subject),
            branches: branches
                .iter()
                .map(|(when, then)| {
                    (
                        rename_expression(when, rename),
                        rename_expression(then, rename),
                    )
                })
                .collect(),
            default: sub_opt(default),
        },
        E::Index { base, index } => E::Index {
            base: sub(base),
            index: sub(index),
        },
        E::Slice { base, from, to } => E::Slice {
            base: sub(base),
            from: sub_opt(from),
            to: sub_opt(to),
        },
        E::Cast { operand, type_name } => E::Cast {
            operand: sub(operand),
            type_name: type_name.clone(),
        },
        E::List(values) => E::List(
            values
                .iter()
                .map(|value| rename_expression(value, rename))
                .collect(),
        ),
        E::Map(entries) => E::Map(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), rename_expression(value, rename)))
                .collect(),
        ),
        E::Quantifier {
            kind,
            variable,
            list,
            predicate,
        } => {
            let shadowed = rename.contains_key(&variable.value);
            E::Quantifier {
                kind: *kind,
                variable: variable.clone(),
                list: sub(list),
                predicate: if shadowed {
                    predicate.clone()
                } else {
                    sub(predicate)
                },
            }
        }
        E::ListComprehension {
            variable,
            list,
            predicate,
            map,
        } => {
            let shadowed = rename.contains_key(&variable.value);
            E::ListComprehension {
                variable: variable.clone(),
                list: sub(list),
                predicate: if shadowed {
                    predicate.clone()
                } else {
                    sub_opt(predicate)
                },
                map: if shadowed { map.clone() } else { sub_opt(map) },
            }
        }
        other => other.clone(),
    };
    cypher::Spanned::new(value, expression.span)
}

/// True when the expression nests a bare pattern (subquery or predicate)
/// in a value position; openCypher only allows patterns in boolean
/// contexts, and SET right-hand sides must reject them (TCK Pattern1).
fn contains_pattern_expression(expression: &cypher::Spanned<cypher::Expression>) -> bool {
    match &expression.value {
        cypher::Expression::PatternSubquery { .. }
        | cypher::Expression::PatternPredicate { .. } => true,
        cypher::Expression::Function { arguments, .. } => {
            arguments.iter().any(contains_pattern_expression)
        }
        cypher::Expression::Unary { operand, .. } => contains_pattern_expression(operand),
        cypher::Expression::Binary { left, right, .. } => {
            contains_pattern_expression(left) || contains_pattern_expression(right)
        }
        cypher::Expression::Property { entity, .. } => contains_pattern_expression(entity),
        cypher::Expression::Index { base, index } => {
            contains_pattern_expression(base) || contains_pattern_expression(index)
        }
        cypher::Expression::Slice { base, from, to } => {
            contains_pattern_expression(base)
                || from.as_deref().is_some_and(contains_pattern_expression)
                || to.as_deref().is_some_and(contains_pattern_expression)
        }
        cypher::Expression::Cast { operand, .. } => contains_pattern_expression(operand),
        cypher::Expression::List(values) => values.iter().any(contains_pattern_expression),
        cypher::Expression::Map(entries) => entries
            .iter()
            .any(|(_, value)| contains_pattern_expression(value)),
        _ => false,
    }
}

/// Statically valid argument types for the to*() conversions (TCK
/// TypeConversion1-4 "fail on invalid types"). Any stays permitted.
fn convertible_argument(conversion: &str, value_type: &ir::ValueType) -> bool {
    match value_type {
        ir::ValueType::Any => true,
        ir::ValueType::Custom { .. } => conversion == "tostring",
        ir::ValueType::Boolean => matches!(conversion, "tostring" | "tointeger" | "toboolean"),
        ir::ValueType::Integer => true,
        ir::ValueType::Real => matches!(conversion, "tostring" | "tointeger" | "tofloat"),
        ir::ValueType::Text => true,
        _ => false,
    }
}

fn list_element_type(
    value_type: &ir::ValueType,
    span: cypher::Span,
) -> Result<ir::ValueType, BindError> {
    match value_type {
        ir::ValueType::List(element) => Ok((**element).clone()),
        ir::ValueType::Any => Ok(ir::ValueType::Any),
        _ => Err(at_unsupported(span, "list scans over non-list expressions")),
    }
}

fn boolean_compatible(value_type: &ir::ValueType) -> bool {
    matches!(value_type, ir::ValueType::Boolean | ir::ValueType::Any)
}

fn text_compatible(value_type: &ir::ValueType) -> bool {
    matches!(value_type, ir::ValueType::Text | ir::ValueType::Any)
}

fn numeric_compatible(value_type: &ir::ValueType) -> bool {
    matches!(
        value_type,
        ir::ValueType::Integer | ir::ValueType::Real | ir::ValueType::Any
    )
}

fn bind_binary_operator(operator: cypher::BinaryOperator) -> ir::BinaryOp {
    match operator {
        cypher::BinaryOperator::Or => ir::BinaryOp::Or,
        cypher::BinaryOperator::Xor => ir::BinaryOp::Xor,
        cypher::BinaryOperator::And => ir::BinaryOp::And,
        cypher::BinaryOperator::Equal => ir::BinaryOp::Equal,
        cypher::BinaryOperator::NotEqual => ir::BinaryOp::NotEqual,
        cypher::BinaryOperator::Less => ir::BinaryOp::Less,
        cypher::BinaryOperator::LessOrEqual => ir::BinaryOp::LessOrEqual,
        cypher::BinaryOperator::Greater => ir::BinaryOp::Greater,
        cypher::BinaryOperator::GreaterOrEqual => ir::BinaryOp::GreaterOrEqual,
        cypher::BinaryOperator::In => ir::BinaryOp::In,
        cypher::BinaryOperator::StartsWith => ir::BinaryOp::StartsWith,
        cypher::BinaryOperator::EndsWith => ir::BinaryOp::EndsWith,
        cypher::BinaryOperator::Contains => ir::BinaryOp::Contains,
        cypher::BinaryOperator::Add => ir::BinaryOp::Add,
        cypher::BinaryOperator::Subtract => ir::BinaryOp::Subtract,
        cypher::BinaryOperator::Multiply => ir::BinaryOp::Multiply,
        cypher::BinaryOperator::Divide => ir::BinaryOp::Divide,
        cypher::BinaryOperator::Modulo => ir::BinaryOp::Modulo,
        // Vector and jsonb operators are intercepted before generic binding.
        cypher::BinaryOperator::VectorL2
        | cypher::BinaryOperator::VectorCosine
        | cypher::BinaryOperator::VectorInnerProduct
        | cypher::BinaryOperator::JsonGet
        | cypher::BinaryOperator::JsonGetText
        | cypher::BinaryOperator::JsonPath
        | cypher::BinaryOperator::JsonPathText
        | cypher::BinaryOperator::JsonExists
        | cypher::BinaryOperator::JsonExistsAny
        | cypher::BinaryOperator::JsonExistsAll
        | cypher::BinaryOperator::JsonContains
        | cypher::BinaryOperator::JsonContainedBy => ir::BinaryOp::Subtract,
        cypher::BinaryOperator::Power => ir::BinaryOp::Power,
    }
}

fn binary_type(
    operator: cypher::BinaryOperator,
    left: &ir::ValueType,
    right: &ir::ValueType,
) -> ir::ValueType {
    match operator {
        cypher::BinaryOperator::Or
        | cypher::BinaryOperator::And
        | cypher::BinaryOperator::Equal
        | cypher::BinaryOperator::NotEqual
        | cypher::BinaryOperator::Less
        | cypher::BinaryOperator::LessOrEqual
        | cypher::BinaryOperator::Greater
        | cypher::BinaryOperator::GreaterOrEqual
        | cypher::BinaryOperator::In
        | cypher::BinaryOperator::Xor
        | cypher::BinaryOperator::StartsWith
        | cypher::BinaryOperator::EndsWith
        | cypher::BinaryOperator::Contains => ir::ValueType::Boolean,
        cypher::BinaryOperator::Power => ir::ValueType::Real,
        _ if left == &ir::ValueType::Real || right == &ir::ValueType::Real => ir::ValueType::Real,
        _ if left == &ir::ValueType::Integer && right == &ir::ValueType::Integer => {
            ir::ValueType::Integer
        }
        _ => ir::ValueType::Any,
    }
}

fn nullable(left: ir::Nullability, right: ir::Nullability) -> ir::Nullability {
    if left == ir::Nullability::Nullable || right == ir::Nullability::Nullable {
        ir::Nullability::Nullable
    } else {
        ir::Nullability::NonNull
    }
}

/// The result type of an aggregate call, shared by visible and hidden
/// aggregation outputs.
fn aggregate_value_type(
    function: &ir::AggregateFunction,
    argument: &Option<ir::TypedExpression>,
) -> ir::ValueType {
    match (function, argument) {
        (ir::AggregateFunction::Count, _) => ir::ValueType::Integer,
        (ir::AggregateFunction::Average, _) => ir::ValueType::Real,
        (ir::AggregateFunction::Collect, Some(argument)) => {
            ir::ValueType::List(Box::new(argument.value_type.clone()))
        }
        (_, Some(argument)) => argument.value_type.clone(),
        (_, None) => ir::ValueType::Any,
    }
}

/// True when the expression tree contains an aggregate call in a position
/// this binder hoists (see `extract_aggregate_calls` for the scope rules).
/// A bare top-level call is also "contained".
fn contains_aggregate_call(expression: &cypher::Expression) -> bool {
    use cypher::Expression as E;
    let spanned =
        |child: &cypher::Spanned<cypher::Expression>| contains_aggregate_call(&child.value);
    match expression {
        E::Function {
            name,
            arguments,
            star,
            ..
        } => {
            let aggregate = matches!(
                name.value.to_ascii_lowercase().as_str(),
                "count" | "sum" | "avg" | "min" | "max" | "collect"
            ) && (*star || arguments.len() == 1);
            aggregate || arguments.iter().any(spanned)
        }
        E::Binary { left, right, .. } => spanned(left) || spanned(right),
        E::Unary { operand, .. } => spanned(operand),
        E::Case {
            subject,
            branches,
            default,
        } => {
            subject.as_deref().is_some_and(spanned)
                || branches
                    .iter()
                    .any(|(when, then)| spanned(when) || spanned(then))
                || default.as_deref().is_some_and(spanned)
        }
        E::Property { entity, .. } => spanned(entity),
        E::HasLabels { operand, .. } => spanned(operand),
        E::Index { base, index } => spanned(base) || spanned(index),
        E::Slice { base, from, to } => {
            spanned(base)
                || from.as_deref().is_some_and(spanned)
                || to.as_deref().is_some_and(spanned)
        }
        E::Cast { operand, .. } => spanned(operand),
        E::List(items) => items.iter().any(spanned),
        E::Map(entries) => entries.iter().any(|(_, value)| spanned(value)),
        E::Quantifier { list, .. } | E::ListComprehension { list, .. } => spanned(list),
        E::Literal(_)
        | E::Variable(_)
        | E::Parameter(_)
        | E::Reduce { .. }
        | E::PatternSubquery { .. }
        | E::PatternPredicate { .. } => false,
    }
}

fn projection_name(expression: &cypher::Spanned<cypher::Expression>) -> String {
    match &expression.value {
        cypher::Expression::Variable(name) => name.clone(),
        cypher::Expression::Property { entity, name } => match &entity.value {
            cypher::Expression::Variable(entity) => format!("{entity}.{}", name.value),
            _ => name.value.clone(),
        },
        _ => format!("expression_{}", expression.span.start),
    }
}

fn clause_span(_clause: &cypher::MatchClause, query: &cypher::Query) -> cypher::Span {
    query.span
}

/// Folds a query's UNION branches onto `plan`, each branch bound by a
/// fresh binder (branch column names combine positionally).
fn assemble_union_branches(
    mut plan: ir::Plan,
    query: &cypher::Query,
    graph: ir::GraphId,
    catalog: &dyn GraphCatalogSnapshot,
    parameters: &ParameterTypes,
) -> Result<ir::Plan, BindError> {
    for branch in &query.unions {
        let mut branch_binder = Binder::new(graph, catalog, parameters);
        branch_binder.bind_read_clauses(&branch.clauses, query)?;
        let branch_plan = branch_binder.plan.ok_or(BindError::EmptyQuery)?;
        let names = |plan: &ir::Plan| {
            plan.result_shape()
                .iter()
                .map(|column| column.name().to_owned())
                .collect::<Vec<_>>()
        };
        if names(&plan) != names(&branch_plan) {
            return Err(at_unsupported(
                branch.span,
                "UNION branches with different result columns",
            ));
        }
        let scope = plan.scope().clone();
        let result_shape = plan.result_shape().clone();
        plan = ir::Plan::new(
            ir::PlanKind::Union(ir::Union::new(vec![plan, branch_plan], branch.all)?),
            scope,
            result_shape,
        )?;
    }
    Ok(plan)
}

fn at_unsupported(span: cypher::Span, feature: &'static str) -> BindError {
    BindError::Unsupported {
        feature,
        span_start: span.start,
        span_end: span.end,
    }
}

/// Binds a SKIP/LIMIT expression that must be a non-negative integer
/// literal: mutation pipelines execute row counts in Rust rather than
/// lowering them into a SQL plan, so a runtime-only bound (a parameter or
/// computed expression) has nowhere to be evaluated yet.
fn bind_constant_count(
    expression: Option<&cypher::Spanned<cypher::Expression>>,
    context: &'static str,
) -> Result<Option<usize>, BindError> {
    match expression {
        None => Ok(None),
        Some(expression) => match &expression.value {
            cypher::Expression::Literal(cypher::Literal::Integer(value)) if *value >= 0 => {
                Ok(Some(*value as usize))
            }
            _ => Err(at_unsupported(expression.span, context)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Catalog;

    impl GraphCatalogSnapshot for Catalog {
        fn node_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
            ir::SourceTableId::new(10).ok()
        }

        fn relationship_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
            ir::SourceTableId::new(11).ok()
        }

        fn label(&self, _graph: ir::GraphId, name: &str) -> Option<ir::LabelId> {
            (name == "Person").then(|| ir::LabelId::new(20).expect("non-zero"))
        }

        fn relationship_type(
            &self,
            _graph: ir::GraphId,
            name: &str,
        ) -> Option<ir::RelationshipTypeId> {
            (name == "KNOWS").then(|| ir::RelationshipTypeId::new(30).expect("non-zero"))
        }

        fn property(
            &self,
            _graph: ir::GraphId,
            entity: CatalogEntity,
            name: &str,
        ) -> Option<ResolvedProperty> {
            let (id, value_type, nullability) = match (entity, name) {
                (CatalogEntity::Node, "id") => {
                    (40, ir::ValueType::Integer, ir::Nullability::NonNull)
                }
                (CatalogEntity::Node, "name") => {
                    (41, ir::ValueType::Text, ir::Nullability::Nullable)
                }
                (CatalogEntity::Relationship, "since") => {
                    (42, ir::ValueType::Integer, ir::Nullability::Nullable)
                }
                (CatalogEntity::Node, "location") => (
                    43,
                    ir::ValueType::Struct(vec![
                        ("x".to_owned(), ir::ValueType::Integer),
                        ("y".to_owned(), ir::ValueType::Integer),
                    ]),
                    ir::Nullability::Nullable,
                ),
                _ => return None,
            };
            Some(ResolvedProperty {
                id: ir::PropertyId::new(id).expect("non-zero"),
                value_type,
                nullability,
            })
        }
    }

    fn bind_text(source: &str, parameters: ParameterTypes) -> Result<BoundQuery, BindError> {
        let query = cypher::parse(source).expect("fixture must parse");
        bind(
            &query,
            ir::GraphId::new(1).expect("non-zero"),
            &Catalog,
            &parameters,
        )
    }

    fn bind_mutation_text(source: &str) -> Result<BoundMutation, BindError> {
        let query = cypher::parse(source).expect("fixture must parse");
        bind_mutation(
            &query,
            ir::GraphId::new(1).expect("non-zero"),
            &Catalog,
            &ParameterTypes::new(),
        )
    }

    #[test]
    fn binds_created_path_to_stable_sources_and_endpoints() {
        let bound = bind_mutation_text(
            "CREATE (a:Person {id: 1, name: 'Ada'})-[r:KNOWS {since: 2020}]->(b:Person {id: 2})",
        )
        .expect("mutation should bind");
        assert!(bound.request.input.is_none());
        assert_eq!(bound.request.operations.len(), 3);
        let ir::Mutation::CreateNode(first) = &bound.request.operations[0] else {
            panic!("expected first node creation")
        };
        let ir::Mutation::CreateRelationship(relationship) = &bound.request.operations[2] else {
            panic!("expected relationship creation")
        };
        assert_eq!(first.source, ir::SourceTableId::new(10).unwrap());
        assert_eq!(relationship.source, ir::SourceTableId::new(11).unwrap());
        assert_eq!(relationship.from, first.binding.id());
        let ir::Mutation::CreateNode(second) = &bound.request.operations[1] else {
            panic!("expected second node creation")
        };
        assert_eq!(relationship.to, second.binding.id());
    }

    #[test]
    fn binds_match_updates_and_detach_delete_against_the_match_input() {
        let bound = bind_mutation_text(
            "MATCH (n:Person {id: 1}) SET n.name = 'Grace' REMOVE n.name DETACH DELETE n",
        )
        .expect("mutation should bind");
        assert!(bound.request.input.is_some());
        assert!(matches!(
            bound.request.operations.as_slice(),
            [
                ir::Mutation::SetProperty(_),
                ir::Mutation::RemoveProperty(_),
                ir::Mutation::Delete(_)
            ]
        ));
    }

    #[test]
    fn mutation_binding_rejects_read_projection_and_unknown_targets() {
        assert!(matches!(
            bind_mutation_text("MATCH (n) SET missing.name = 'x'"),
            Err(BindError::UnknownVariable { .. })
        ));
        let bound = bind_mutation_text("CREATE (n:Person {id: 1}) RETURN n")
            .expect("trailing RETURN after a mutation should bind");
        assert_eq!(bound.returns.len(), 1);
        let staged = bind_mutation_text("CREATE (:Person {id: 1}) WITH 1 AS x RETURN x")
            .expect("WITH pipelines after mutations should bind");
        assert_eq!(staged.stages.len(), 1);
        assert_eq!(staged.returns.len(), 1);
        let staged_match = bind_mutation_text("CREATE (:Person {id: 1}) MATCH (n) DELETE n")
            .expect("uncorrelated MATCH after a mutation should bind as a stage");
        assert!(staged_match.stages.iter().any(|stage| stage
            .items
            .iter()
            .any(|item| matches!(item, StageItem::Match { .. }))));
        // Correlated re-matching of a mutation binding executes per row.
        let correlated = bind_mutation_text("CREATE (a:Person {id: 1}) MATCH (a) DELETE a")
            .expect("correlated MATCH after a mutation should bind");
        assert!(correlated.stages.iter().any(|stage| {
            stage
                .items
                .iter()
                .any(|item| matches!(item, StageItem::Match { .. }))
        }));
    }

    #[test]
    fn mutation_with_binds_order_by_skip_and_limit() {
        // The WITH here follows a staged (post-mutation) MATCH, so it binds
        // through `bind_mutation_with` rather than the read projection path
        // that handles a WITH appearing before the first mutating clause.
        let bound = bind_mutation_text(
            "CREATE (:Person {id: 1}) MATCH (n:Person) \
             WITH n ORDER BY n.id DESC SKIP 1 LIMIT 1 SET n.name = 'Z'",
        )
        .expect("WITH ORDER BY/SKIP/LIMIT should bind in a mutation stage");
        let stage = bound.stages.last().expect("staged MATCH and WITH stages");
        assert_eq!(stage.order.len(), 1);
        assert!(stage.order[0].1, "DESC should record descending = true");
        assert_eq!(stage.skip, Some(1));
        assert_eq!(stage.limit, Some(1));
        assert!(matches!(
            bind_mutation_text(
                "CREATE (:Person {id: 1}) MATCH (n:Person) WITH n LIMIT $n SET n.name = 'Z'"
            ),
            Err(BindError::Unsupported { .. })
        ));
    }

    #[test]
    fn resolves_names_parameters_types_and_result_shape() {
        let parameters = HashMap::from([(
            "id".to_owned(),
            (ir::ValueType::Integer, ir::Nullability::NonNull),
        )]);
        let bound = bind_text(
            "MATCH (p:Person {id: $id})-[r:KNOWS {since: 2020}]->(friend) RETURN friend.name AS name",
            parameters,
        )
        .expect("query should bind");
        assert_eq!(bound.plan.result_shape().len(), 1);
        let output = bound.plan.scope().resolve("name").expect("projected name");
        assert_eq!(output.value_type(), &ir::ValueType::Text);
        assert_eq!(output.nullability(), ir::Nullability::Nullable);
    }

    #[test]
    fn standalone_projection_uses_a_single_unit_row() {
        let bound = bind_text("RETURN 1 AS value", ParameterTypes::new())
            .expect("standalone projection should bind");
        let ir::PlanKind::Project(project) = bound.plan.kind() else {
            panic!("expected projection")
        };
        assert!(matches!(project.input.kind(), ir::PlanKind::Unit(_)));
    }

    #[test]
    fn with_replaces_the_visible_scope() {
        let error = bind_text(
            "MATCH (p:Person) WITH p AS person RETURN p",
            ParameterTypes::new(),
        )
        .expect_err("old name must leave scope");
        assert!(matches!(error, BindError::UnknownVariable { name, .. } if name == "p"));
    }

    #[test]
    fn optional_match_makes_introduced_bindings_nullable() {
        let bound = bind_text(
            "MATCH (p:Person) OPTIONAL MATCH (p)-[r:KNOWS]->(friend) RETURN friend",
            ParameterTypes::new(),
        )
        .expect("optional match should bind");
        let friend = bound.plan.scope().resolve("friend").expect("friend output");
        assert_eq!(friend.nullability(), ir::Nullability::Nullable);
    }

    /// Descends single-input plan nodes to the innermost `NodeScan`'s
    /// binding, so a test can assert which variable ended up as the join
    /// anchor without hardcoding the exact operator tree shape.
    fn innermost_node_scan_binding(plan: &ir::Plan) -> ir::BindingId {
        match plan.kind() {
            ir::PlanKind::NodeScan(scan) => scan.binding,
            ir::PlanKind::FixedExpand(expand) => innermost_node_scan_binding(&expand.input),
            ir::PlanKind::GraphExpand(expand) => innermost_node_scan_binding(&expand.input),
            ir::PlanKind::Filter(filter) => innermost_node_scan_binding(&filter.input),
            ir::PlanKind::Project(project) => innermost_node_scan_binding(&project.input),
            ir::PlanKind::Aggregate(aggregate) => innermost_node_scan_binding(&aggregate.input),
            ir::PlanKind::Distinct(distinct) => innermost_node_scan_binding(&distinct.input),
            ir::PlanKind::Sort(sort) => innermost_node_scan_binding(&sort.input),
            ir::PlanKind::Skip(skip) => innermost_node_scan_binding(&skip.input),
            ir::PlanKind::Limit(limit) => innermost_node_scan_binding(&limit.input),
            ir::PlanKind::Unwind(unwind) => innermost_node_scan_binding(&unwind.input),
            other => panic!("no single-input descent to a NodeScan from {other:?}"),
        }
    }

    /// Binds a single MATCH clause in isolation and returns the binder's raw
    /// scope and plan. RETURN/WITH projections allocate fresh output
    /// `BindingId`s for every projected column (see `bind_projection`), so
    /// comparing a final `BoundQuery`'s scope against internal `NodeScan`
    /// bindings never matches; binding the MATCH alone keeps the original
    /// ids so a test can check which variable the binder anchored on.
    fn bind_match_only(source: &str) -> (Vec<ir::Binding>, ir::Plan) {
        let query = cypher::parse(source).expect("fixture must parse");
        let cypher::Clause::Match(match_clause) = &query.clauses[0].value else {
            panic!("fixture must start with a MATCH clause");
        };
        let parameters = ParameterTypes::new();
        let mut binder = Binder::new(
            ir::GraphId::new(1).expect("non-zero"),
            &Catalog,
            &parameters,
        );
        binder
            .bind_match(match_clause, query.span)
            .expect("match should bind");
        (
            binder.scope,
            binder.plan.expect("match must produce a plan"),
        )
    }

    fn binding_id(scope: &[ir::Binding], name: &str) -> ir::BindingId {
        scope
            .iter()
            .find(|binding| binding.name() == name)
            .unwrap_or_else(|| panic!("{name} must be in scope"))
            .id()
    }

    #[test]
    fn anchors_pattern_binding_at_the_most_selective_node() {
        // n is a bare label scan, k carries a constant property filter: the
        // binder should reverse the path and anchor the join at k instead
        // of walking left-to-right from n and filtering by id last.
        let (scope, plan) =
            bind_match_only("MATCH (n:Person)-[:KNOWS]->(m:Person)<-[:KNOWS]-(k:Person {id: 1})");
        assert_eq!(innermost_node_scan_binding(&plan), binding_id(&scope, "k"));
    }

    #[test]
    fn does_not_reverse_when_the_first_node_is_already_more_selective() {
        // Mirror image of the above: n now carries the constant filter, so
        // the existing left-to-right order is already optimal and must be
        // left alone.
        let (scope, plan) =
            bind_match_only("MATCH (n:Person {id: 1})-[:KNOWS]->(m:Person)<-[:KNOWS]-(k:Person)");
        assert_eq!(innermost_node_scan_binding(&plan), binding_id(&scope, "n"));
    }

    #[test]
    fn does_not_reverse_a_named_path_or_variable_length_pattern() {
        // Named paths build their PathValue node/relationship lists in
        // binding order; variable-length hops have their own materialized
        // path_output machinery. Both must keep first-to-last binding order
        // even when the last node would otherwise look more selective.
        let (scope, plan) = bind_match_only(
            "MATCH p = (n:Person)-[:KNOWS]->(m:Person)<-[:KNOWS]-(k:Person {id: 1})",
        );
        assert_eq!(innermost_node_scan_binding(&plan), binding_id(&scope, "n"));

        let (scope, plan) = bind_match_only(
            "MATCH (n:Person)-[:KNOWS*1..3]->(m:Person)<-[:KNOWS]-(k:Person {id: 1})",
        );
        assert_eq!(innermost_node_scan_binding(&plan), binding_id(&scope, "n"));
    }

    #[test]
    fn does_not_reverse_an_optional_match_continuing_from_an_earlier_binding() {
        // Regression (TCK Match7 [23] and similar `OPTIONAL MATCH
        // (a)-->(b:Label)` shapes in the corpus): `a` is already bound by
        // the preceding MATCH, so the OPTIONAL MATCH path must keep
        // resuming from `a`'s existing plan rather than reversing to scan
        // `b` and demoting `a` to a step-target equality filter — that
        // reversal broke the correlated LeftApply's null-preserving
        // semantics (duplicated matched rows, dropped the unmatched null
        // row).
        let query = cypher::parse("MATCH (a:Person) OPTIONAL MATCH (a)-[:KNOWS]->(b:Person)")
            .expect("fixture must parse");
        let parameters = ParameterTypes::new();
        let mut binder = Binder::new(
            ir::GraphId::new(1).expect("non-zero"),
            &Catalog,
            &parameters,
        );
        binder
            .bind_read_clauses(&query.clauses, &query)
            .expect("query should bind");
        let a_id = binding_id(&binder.scope, "a");
        let plan = binder.plan.expect("query must produce a plan");
        let ir::PlanKind::LeftApply(left_apply) = plan.kind() else {
            panic!("expected OPTIONAL MATCH to bind as a LeftApply");
        };
        // If the path had been reversed, the right side's innermost scan
        // would be a fresh NodeScan for `b`, not a continuation of `a`.
        assert_eq!(innermost_node_scan_binding(&left_apply.right), a_id);
    }

    #[test]
    fn rejects_duplicate_variables_with_their_source_name() {
        let error = bind_text(
            "MATCH (p:Person)-[p:KNOWS]->(friend) RETURN friend",
            ParameterTypes::new(),
        )
        .expect_err("relationship cannot shadow node");
        assert!(matches!(error, BindError::DuplicateVariable { name, .. } if name == "p"));
    }

    #[test]
    fn reports_unknown_catalog_and_parameter_names_as_typed_errors() {
        let cases = [
            ("MATCH (p:Missing) RETURN p", "label"),
            (
                "MATCH (p:Person)-[:MISSING]->(friend) RETURN friend",
                "relationship type",
            ),
            ("MATCH (p:Person) RETURN p.missing", "property"),
            ("MATCH (p:Person {id: $missing}) RETURN p", "parameter"),
        ];
        for (source, expected) in cases {
            let error = bind_text(source, ParameterTypes::new()).expect_err(expected);
            let matches = match error {
                BindError::UnknownLabel { .. } => expected == "label",
                BindError::UnknownRelationshipType { .. } => expected == "relationship type",
                BindError::UnknownProperty { .. } => expected == "property",
                BindError::UnknownParameter { .. } => expected == "parameter",
                _ => false,
            };
            assert!(matches, "expected {expected} error");
        }
    }

    #[test]
    fn binds_bounded_anonymous_variable_length_relationships() {
        let bound = bind_text(
            "MATCH (p:Person)-[:KNOWS*1..3]->(friend) RETURN friend",
            ParameterTypes::new(),
        )
        .expect("bounded expansion should bind");
        let ir::PlanKind::Project(project) = bound.plan.kind() else {
            panic!("expected projection");
        };
        let ir::PlanKind::GraphExpand(expand) = project.input.kind() else {
            panic!("expected graph expansion");
        };
        assert_eq!((expand.min_hops, expand.max_hops), (1, 3));
        assert_eq!(expand.uniqueness, ir::PathUniqueness::Trail);

        let named = bind_text(
            "MATCH (p:Person)-[rels:KNOWS*1..3]->(friend) RETURN friend",
            ParameterTypes::new(),
        )
        .expect("a named variable-length relationship registers a list binding");
        assert!(matches!(named.plan.kind(), ir::PlanKind::Project(_)));
    }

    #[test]
    fn binds_map_literal_to_struct_mutation_property() {
        let bound = bind_mutation_text("CREATE (:Person {location: {x: 1, y: 2}})")
            .expect("mutation should bind");
        let ir::Mutation::CreateNode(node) = &bound.request.operations[0] else {
            panic!("expected node creation")
        };
        assert_eq!(node.properties.len(), 1);
        let property = &node.properties[0];
        assert_eq!(
            property.property,
            ir::PropertyId::new(43).expect("non-zero")
        );
        match &property.value.expression {
            ir::Expression::Map(entries) => {
                assert_eq!(entries.len(), 2);
                assert_eq!(entries[0].0, "x");
                assert!(matches!(
                    entries[0].1.expression,
                    ir::Expression::Literal(ir::Literal::Integer(1))
                ));
                assert_eq!(entries[1].0, "y");
                assert!(matches!(
                    entries[1].1.expression,
                    ir::Expression::Literal(ir::Literal::Integer(2))
                ));
            }
            other => panic!("expected Map expression, got {other:?}"),
        }
    }

    #[test]
    fn binds_nested_struct_field_access() {
        let bound = bind_text("MATCH (n) RETURN n.location.x", ParameterTypes::new())
            .expect("query should bind");
        let ir::PlanKind::Project(project) = bound.plan.kind() else {
            panic!("expected projection");
        };
        assert_eq!(project.projections.len(), 1);
        match &project.projections[0].expression.expression {
            ir::Expression::Property {
                property, fields, ..
            } => {
                assert_eq!(*property, ir::PropertyId::new(43).expect("non-zero"));
                assert_eq!(fields, &vec!["x".to_owned()]);
            }
            other => panic!("expected Property expression, got {other:?}"),
        }
        assert_eq!(
            project.projections[0].expression.value_type,
            ir::ValueType::Integer
        );
    }

    /// `IN` list membership must bind as a boolean binary containment so
    /// WHERE predicates and projections over list literals type-check
    /// downstream instead of failing at the parser.
    #[test]
    fn binds_in_membership_as_boolean_binary() {
        let bound = bind_text(
            "MATCH (n) RETURN n.name IN ['A', 'B'] AS r",
            ParameterTypes::new(),
        )
        .expect("query should bind");
        let ir::PlanKind::Project(project) = bound.plan.kind() else {
            panic!("expected projection");
        };
        match &project.projections[0].expression.expression {
            ir::Expression::Binary { op, right, .. } => {
                assert_eq!(*op, ir::BinaryOp::In);
                assert!(matches!(
                    right.expression,
                    ir::Expression::List(ref values) if values.len() == 2
                ));
            }
            other => panic!("expected Binary expression, got {other:?}"),
        }
        assert_eq!(
            project.projections[0].expression.value_type,
            ir::ValueType::Boolean
        );
    }

    #[test]
    fn rejects_field_access_deeper_than_two_levels() {
        let error = bind_text("MATCH (n) RETURN n.location.x.y.z", ParameterTypes::new())
            .expect_err("chain deeper than two levels must be rejected");
        assert!(matches!(error, BindError::Unsupported { .. }));
    }
}
