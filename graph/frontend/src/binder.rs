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

#[derive(Clone, Copy)]
struct EntityBinding {
    kind: CatalogEntity,
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
        Ok(BoundQuery { plan })
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
            }
        }
        Ok(())
    }

    fn bind_mutation_query(mut self, query: &cypher::Query) -> Result<BoundMutation, BindError> {
        if let Some(branch) = query.unions.first() {
            return Err(at_unsupported(branch.span, "UNION in mutation queries"));
        }
        let mut operations = Vec::new();
        let mut mutation_started = false;
        for clause in &query.clauses {
            match &clause.value {
                cypher::Clause::Match(value) => {
                    if mutation_started {
                        return Err(at_unsupported(clause.span, "MATCH after a mutation clause"));
                    }
                    self.bind_match(value, clause.span)?;
                }
                cypher::Clause::Create(value) => {
                    mutation_started = true;
                    for path in &value.paths {
                        self.bind_create_path(path, false, &mut operations)?;
                    }
                }
                cypher::Clause::Merge(value) => {
                    mutation_started = true;
                    self.bind_create_path(&value.path, true, &mut operations)?;
                }
                cypher::Clause::Set(value) => {
                    mutation_started = true;
                    self.bind_set(value, &mut operations)?;
                }
                cypher::Clause::Remove(value) => {
                    mutation_started = true;
                    self.bind_remove(value, &mut operations)?;
                }
                cypher::Clause::Delete(value) => {
                    mutation_started = true;
                    self.bind_delete(value, &mut operations)?;
                }
                cypher::Clause::Unwind(_) | cypher::Clause::With(_) | cypher::Clause::Return(_) => {
                    return Err(at_unsupported(
                        clause.span,
                        "projection clauses in mutation queries",
                    ));
                }
            }
        }
        if operations.is_empty() {
            return Err(BindError::EmptyMutation);
        }
        Ok(BoundMutation {
            request: ir::MutationRequest {
                graph: self.graph,
                input: self.plan,
                operations,
            },
        })
    }

    fn bind_create_path(
        &mut self,
        path: &cypher::PathPattern,
        merge: bool,
        operations: &mut Vec<ir::Mutation>,
    ) -> Result<(), BindError> {
        if let Some(variable) = &path.variable {
            // Register the path name so later clauses can reference it; the
            // path value itself is not materialized in the initial slice.
            let binding = ir::Binding::new(
                self.next_id()?,
                variable.value.clone(),
                ir::ValueType::Path,
                ir::Nullability::NonNull,
            )?;
            self.scope.push(binding);
        }
        let mut from = self.bind_created_node(&path.start, merge, operations)?;
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
            let to = self.bind_created_node(node, merge, operations)?;
            let source = self.relationship_source(relationship.span)?;
            let binding = self.new_entity_binding(
                relationship.variable.as_ref(),
                "_relationship",
                ir::ValueType::Relationship,
                ir::Nullability::NonNull,
                CatalogEntity::Relationship,
                relationship.span,
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
                ir::Mutation::MergeRelationship(ir::MergeRelationship { create })
            } else {
                ir::Mutation::CreateRelationship(create)
            });
            from = next_from;
        }
        Ok(())
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
                if !node.labels.is_empty() || !node.properties.is_empty() {
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
            node.span,
        )?;
        let create = ir::CreateNode {
            binding: binding.clone(),
            source,
            labels: self.resolve_labels(node)?,
            properties: self.bind_mutation_properties(CatalogEntity::Node, &node.properties)?,
        };
        operations.push(if merge {
            ir::Mutation::MergeNode(ir::MergeNode { create })
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
            let (binding, kind, source) = self.resolve_mutation_target(&item.target)?;
            let property = self.resolve_property(kind, &item.target.property)?;
            operations.push(ir::Mutation::SetProperty(ir::SetProperty {
                entity: binding,
                source,
                property: property.id,
                value: self.bind_expression(&item.value)?,
            }));
        }
        Ok(())
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
                    cypher::Expression::Map(entries) => self.bind_map_property(
                        &resolved.value_type,
                        resolved.nullability,
                        entries,
                        value.span,
                    )?,
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

    fn bind_match(
        &mut self,
        clause: &cypher::MatchClause,
        fallback: cypher::Span,
    ) -> Result<(), BindError> {
        if clause.paths.len() != 1 {
            return Err(at_unsupported(fallback, "multiple path patterns"));
        }
        let path = &clause.paths[0];
        if path.variable.is_some() {
            return Err(at_unsupported(path.span, "named paths"));
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
        let left = self.plan.clone();
        let old_ids: Vec<_> = self.scope.iter().map(ir::Binding::id).collect();
        self.bind_path(path)?;
        if let Some(predicate) = &clause.predicate {
            let predicate = self.bind_expression(predicate)?;
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                input: Box::new(input),
                predicate,
            }))?;
        }
        if clause.optional {
            let left =
                left.ok_or_else(|| at_unsupported(path.span, "OPTIONAL MATCH without an input"))?;
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

    fn bind_path(&mut self, path: &cypher::PathPattern) -> Result<(), BindError> {
        let start = self.bind_start_node(&path.start)?;
        let mut from = start;
        for (relationship, node) in &path.steps {
            if relationship.range.is_some() && relationship.variable.is_some() {
                return Err(at_unsupported(
                    relationship.span,
                    "named variable-length relationships",
                ));
            }
            if relationship.range.is_some() && !relationship.properties.is_empty() {
                return Err(at_unsupported(
                    relationship.span,
                    "variable-length relationship properties",
                ));
            }
            let relationship_binding = self.new_entity_binding(
                relationship.variable.as_ref(),
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
                relationship.span,
            )?;
            let to = self.new_entity_binding(
                node.variable.as_ref(),
                "_node",
                ir::ValueType::Node,
                ir::Nullability::NonNull,
                CatalogEntity::Node,
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
            let kind = if let Some(range) = &relationship.range {
                let min_hops = range.value.min.unwrap_or(1);
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
                    uniqueness: ir::PathUniqueness::Trail,
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
                })
            };
            self.plan = Some(ir::Plan::new(kind, scope, ir::ResultShape::default())?);
            self.bind_properties(
                &relationship_binding,
                CatalogEntity::Relationship,
                &relationship.properties,
            )?;
            self.bind_labels(node)?;
            self.bind_properties(&to, CatalogEntity::Node, &node.properties)?;
            from = to.id();
        }
        Ok(())
    }

    fn bind_start_node(&mut self, node: &cypher::NodePattern) -> Result<ir::BindingId, BindError> {
        if let Some(variable) = &node.variable {
            if let Some(existing) = self
                .scope
                .iter()
                .find(|binding| binding.name() == variable.value)
                .cloned()
            {
                self.bind_labels(node)?;
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
        self.plan = Some(ir::Plan::new(
            ir::PlanKind::NodeScan(ir::NodeScan {
                graph: self.graph,
                source,
                binding: binding.id(),
                labels,
            }),
            scope,
            ir::ResultShape::default(),
        )?);
        self.bind_properties(&binding, CatalogEntity::Node, &node.properties)?;
        Ok(binding.id())
    }

    fn bind_labels(&self, node: &cypher::NodePattern) -> Result<(), BindError> {
        self.resolve_labels(node).map(|_| ())
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
        let sort_keys = clause
            .order_by
            .iter()
            .map(|item| {
                Ok(ir::SortKey {
                    expression: self.bind_expression(&item.expression)?,
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
        let mut projections = Vec::new();
        let mut output_scope = Vec::new();
        let mut output_entities = HashMap::new();
        for item in &clause.items {
            match item {
                cypher::ProjectionItem::All(_) => {
                    for binding in &self.scope {
                        let expression = ir::TypedExpression {
                            expression: ir::Expression::Binding(binding.id()),
                            value_type: binding.value_type().clone(),
                            nullability: binding.nullability(),
                        };
                        projections.push(ir::Projection {
                            output: binding.clone(),
                            expression,
                        });
                        output_scope.push(binding.clone());
                        if let Some(entity) = self.entities.get(&binding.id()) {
                            output_entities.insert(binding.id(), *entity);
                        }
                    }
                }
                cypher::ProjectionItem::Expression { expression, alias } => {
                    let bound = self.bind_expression(expression)?;
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
                    let old_entity = match &expression.value {
                        cypher::Expression::Variable(name) => self
                            .scope
                            .iter()
                            .find(|binding| binding.name() == name)
                            .and_then(|binding| self.entities.get(&binding.id()).copied()),
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
                    output_scope.push(output);
                }
            }
        }
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
        self.plan = Some(ir::Plan::new(
            ir::PlanKind::Project(ir::Project {
                input: Box::new(input),
                projections,
            }),
            scope,
            shape,
        )?);
        self.scope = output_scope;
        self.entities = output_entities;
        if let Some(predicate) = &clause.predicate {
            let predicate = self.bind_expression(predicate)?;
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            self.wrap_plan(ir::PlanKind::Filter(ir::Filter {
                input: Box::new(input),
                predicate,
            }))?;
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
                let scope_hit = {
                    let scopes = self.list_scopes.borrow();
                    scopes
                        .iter()
                        .rposition(|(scope_name, _)| scope_name == name)
                        .map(|position| (position, scopes.len(), scopes[position].1.clone()))
                };
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
            cypher::Expression::Property { .. } => {
                let (root, field_chain) = flatten_property_chain(expression);
                let cypher::Expression::Variable(variable) = &root.value else {
                    return Err(BindError::InvalidPropertyTarget {
                        span_start: root.span.start,
                        span_end: root.span.end,
                    });
                };
                let binding = self.resolve_binding(variable, root.span)?;
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
                let target = match type_name.value.to_ascii_lowercase().as_str() {
                    "integer" | "int" | "bigint" | "smallint" => ir::ValueType::Integer,
                    "float" | "float8" | "pg_float8" | "double" | "real" | "numeric" => {
                        ir::ValueType::Real
                    }
                    "text" | "string" | "varchar" => ir::ValueType::Text,
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
            } => {
                if *distinct {
                    return Err(at_unsupported(
                        expression.span,
                        "DISTINCT function arguments",
                    ));
                }
                let function = ir::FunctionName::new(name.value.clone())
                    .ok_or_else(|| at_unsupported(name.span, "empty function names"))?;
                let arguments = arguments
                    .iter()
                    .map(|argument| self.bind_expression(argument))
                    .collect::<Result<Vec<_>, _>>()?;
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
            cypher::Expression::Map(_) => {
                return Err(at_unsupported(
                    expression.span,
                    "map literal outside a property assignment",
                ));
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

    fn new_entity_binding(
        &mut self,
        variable: Option<&cypher::Spanned<String>>,
        anonymous_prefix: &str,
        value_type: ir::ValueType,
        nullability: ir::Nullability,
        kind: CatalogEntity,
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
        self.entities.insert(id, EntityBinding { kind });
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

fn at_unsupported(span: cypher::Span, feature: &'static str) -> BindError {
    BindError::Unsupported {
        feature,
        span_start: span.start,
        span_end: span.end,
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
        assert!(matches!(
            bind_mutation_text("CREATE (n:Person {id: 1}) RETURN n"),
            Err(BindError::Unsupported { .. })
        ));
        assert!(matches!(
            bind_mutation_text("CREATE (:Person {id: 1}) MATCH (n) DELETE n"),
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

        let error = bind_text(
            "MATCH (p:Person)-[rels:KNOWS*1..3]->(friend) RETURN friend",
            ParameterTypes::new(),
        )
        .expect_err("named relationship list semantics are not implemented");
        assert!(matches!(
            error,
            BindError::Unsupported {
                feature: "named variable-length relationships",
                ..
            }
        ));
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
