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
    #[error("graph query has too many bindings")]
    TooManyBindings,
}

pub fn bind(
    query: &cypher::Query,
    graph: ir::GraphId,
    catalog: &dyn GraphCatalogSnapshot,
    parameters: &ParameterTypes,
) -> Result<BoundQuery, BindError> {
    Binder::new(graph, catalog, parameters).bind_query(query)
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
        }
    }

    fn bind_query(mut self, query: &cypher::Query) -> Result<BoundQuery, BindError> {
        for clause in &query.clauses {
            match &clause.value {
                cypher::Clause::Match(clause) => {
                    self.bind_match(clause, clause_span(clause, query))?
                }
                cypher::Clause::With(clause) => self.bind_projection(clause, false)?,
                cypher::Clause::Return(clause) => self.bind_projection(clause, true)?,
            }
        }
        Ok(BoundQuery {
            plan: self.plan.ok_or(BindError::EmptyQuery)?,
        })
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
            if relationship.range.is_some() {
                return Err(at_unsupported(
                    relationship.span,
                    "variable-length relationships",
                ));
            }
            let relationship_binding = self.new_entity_binding(
                relationship.variable.as_ref(),
                "_relationship",
                ir::ValueType::Relationship,
                CatalogEntity::Relationship,
                relationship.span,
            )?;
            let to = self.new_entity_binding(
                node.variable.as_ref(),
                "_node",
                ir::ValueType::Node,
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
            let direction = match relationship.direction {
                cypher::Direction::Outgoing => ir::Direction::Outgoing,
                cypher::Direction::Incoming => ir::Direction::Incoming,
                cypher::Direction::Both => ir::Direction::Both,
            };
            let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
            let scope = ir::Scope::new(self.scope.clone())?;
            self.plan = Some(ir::Plan::new(
                ir::PlanKind::FixedExpand(ir::FixedExpand {
                    input: Box::new(input),
                    relationship_source: source,
                    from,
                    relationship: relationship_binding.clone(),
                    to: to.clone(),
                    direction,
                    relationship_types,
                }),
                scope,
                ir::ResultShape::default(),
            )?);
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
        let input = self.plan.take().ok_or(BindError::EmptyQuery)?;
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
        Ok(())
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
                let binding = self.resolve_binding(name, expression.span)?;
                (
                    ir::Expression::Binding(binding.id()),
                    binding.value_type().clone(),
                    binding.nullability(),
                )
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
                let cypher::Expression::Variable(variable) = &entity.value else {
                    return Err(BindError::InvalidPropertyTarget {
                        span_start: entity.span.start,
                        span_end: entity.span.end,
                    });
                };
                let binding = self.resolve_binding(variable, entity.span)?;
                let kind = self
                    .entities
                    .get(&binding.id())
                    .ok_or(BindError::InvalidPropertyTarget {
                        span_start: entity.span.start,
                        span_end: entity.span.end,
                    })?
                    .kind;
                let property = self.resolve_property(kind, name)?;
                (
                    ir::Expression::Property {
                        entity: binding.id(),
                        property: property.id,
                    },
                    property.value_type,
                    nullable(binding.nullability(), property.nullability),
                )
            }
            cypher::Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.bind_expression(left)?;
                let right = self.bind_expression(right)?;
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
                (
                    ir::Expression::Function {
                        function,
                        arguments,
                    },
                    ir::ValueType::Any,
                    ir::Nullability::Nullable,
                )
            }
        };
        Ok(ir::TypedExpression {
            expression: expression_ir,
            value_type,
            nullability,
        })
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

    fn new_entity_binding(
        &mut self,
        variable: Option<&cypher::Spanned<String>>,
        anonymous_prefix: &str,
        value_type: ir::ValueType,
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
        let binding = ir::Binding::new(id, name, value_type, ir::Nullability::NonNull)
            .map_err(BindError::InvalidPlan)?;
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

fn bind_binary_operator(operator: cypher::BinaryOperator) -> ir::BinaryOp {
    match operator {
        cypher::BinaryOperator::Or => ir::BinaryOp::Or,
        cypher::BinaryOperator::And => ir::BinaryOp::And,
        cypher::BinaryOperator::Equal => ir::BinaryOp::Equal,
        cypher::BinaryOperator::NotEqual => ir::BinaryOp::NotEqual,
        cypher::BinaryOperator::Less => ir::BinaryOp::Less,
        cypher::BinaryOperator::LessOrEqual => ir::BinaryOp::LessOrEqual,
        cypher::BinaryOperator::Greater => ir::BinaryOp::Greater,
        cypher::BinaryOperator::GreaterOrEqual => ir::BinaryOp::GreaterOrEqual,
        cypher::BinaryOperator::Add => ir::BinaryOp::Add,
        cypher::BinaryOperator::Subtract => ir::BinaryOp::Subtract,
        cypher::BinaryOperator::Multiply => ir::BinaryOp::Multiply,
        cypher::BinaryOperator::Divide => ir::BinaryOp::Divide,
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
        | cypher::BinaryOperator::GreaterOrEqual => ir::ValueType::Boolean,
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
        cypher::Expression::Property { name, .. } => name.value.clone(),
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
    fn rejects_variable_length_relationships_before_planning() {
        let error = bind_text(
            "MATCH (p:Person)-[:KNOWS*1..3]->(friend) RETURN friend",
            ParameterTypes::new(),
        )
        .expect_err("first slice is fixed-hop only");
        assert!(matches!(
            error,
            BindError::Unsupported {
                feature: "variable-length relationships",
                ..
            }
        ));
    }
}
