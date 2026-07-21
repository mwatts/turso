use std::collections::HashMap;

use thiserror::Error;
use turso_graph_ir as ir;
use turso_parser::ast;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeTableLayout {
    pub table: String,
    pub identity_column: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipTableLayout {
    pub table: String,
    pub identity_column: String,
    pub start_column: String,
    pub end_column: String,
}

/// Physical relational names resolved from stable graph catalog identities.
pub trait RelationalCatalogSnapshot {
    fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout>;
    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout>;
    fn property_column(
        &self,
        source: ir::SourceTableId,
        property: ir::PropertyId,
    ) -> Option<String>;
    /// Whether the property's column stores jsonb blobs (declared JSONB).
    /// Lowering renders such columns through json() so every consumer sees
    /// canonical JSON text, exactly as with text-encoded properties.
    fn property_column_is_jsonb(
        &self,
        _source: ir::SourceTableId,
        _property: ir::PropertyId,
    ) -> bool {
        false
    }
    /// Junction table recording each node's labels, when the graph has one.
    fn labels_table(&self) -> Option<String> {
        None
    }
    /// Human-readable name of a label identity, when known.
    fn label_name(&self, _label: ir::LabelId) -> Option<String> {
        None
    }
    /// Junction table recording each relationship's type, when present.
    fn relationship_types_table(&self) -> Option<String> {
        None
    }
    /// Human-readable name of a relationship-type identity, when known.
    fn relationship_type_name(&self, _relationship_type: ir::RelationshipTypeId) -> Option<String> {
        None
    }
    /// Payload properties of a source as (cypher name, physical column)
    /// pairs — every column except identity/endpoint columns. Enables
    /// whole-entity property reads (`properties(n)`).
    fn payload_columns(&self, _source: ir::SourceTableId) -> Option<Vec<(String, String)>> {
        None
    }
}

#[derive(Debug, Error)]
pub enum LowerError {
    #[error("missing relational layout for graph source {0}")]
    MissingSource(ir::SourceTableId),
    #[error("missing relational column for property {property} on source {source_id}")]
    MissingProperty {
        source_id: ir::SourceTableId,
        property: ir::PropertyId,
    },
    #[error("binding {0} has no relational source")]
    MissingBinding(ir::BindingId),
    #[error("unsupported relational graph operator: {0}")]
    UnsupportedOperator(&'static str),
    #[error("invalid resolved function or parameter name: {0}")]
    InvalidName(String),
    #[error("generated relational SQL did not parse: {0}")]
    InvalidGeneratedSql(#[from] turso_core::LimboError),
    #[error("generated relational SQL contained no statement")]
    EmptyGeneratedSql,
}

#[derive(Clone, Copy)]
enum EntityKind {
    Node,
    Relationship,
}

#[derive(Clone)]
struct BindingLayout {
    source: ir::SourceTableId,
    kind: EntityKind,
    /// Property ids whose values ride along as materialized columns
    /// (named by `property_column_ref`) in this plan node's output.
    properties: std::collections::BTreeSet<u32>,
}

struct Lowered {
    sql: String,
    bindings: HashMap<ir::BindingId, BindingLayout>,
}

pub(crate) struct LoweredMutationInput {
    pub(crate) sql: String,
    bindings: HashMap<ir::BindingId, BindingLayout>,
}

#[derive(Clone, Copy)]
pub(crate) enum MutationEntityKind {
    Node,
    Relationship,
}

pub(crate) fn lower_mutation_input(
    plan: &ir::Plan,
    catalog: &dyn RelationalCatalogSnapshot,
) -> Result<LoweredMutationInput, LowerError> {
    let mut wanted = WantedProperties::new();
    collect_wanted(plan, &mut wanted);
    let lowered = lower_plan(plan, catalog, false, &wanted)?;
    Ok(LoweredMutationInput {
        sql: lowered.sql,
        bindings: lowered.bindings,
    })
}

pub(crate) fn unit_mutation_input() -> LoweredMutationInput {
    LoweredMutationInput {
        sql: "SELECT 1 AS __unit".to_owned(),
        bindings: HashMap::new(),
    }
}

pub(crate) fn lower_mutation_expression(
    expression: &ir::TypedExpression,
    input: &LoweredMutationInput,
    catalog: &dyn RelationalCatalogSnapshot,
    references: &HashMap<ir::BindingId, String>,
    additional_bindings: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
) -> Result<String, LowerError> {
    let mut bindings = input.bindings.clone();
    bindings.extend(additional_bindings.iter().map(|(binding, (source, kind))| {
        (
            *binding,
            BindingLayout {
                source: *source,
                kind: match kind {
                    MutationEntityKind::Node => EntityKind::Node,
                    MutationEntityKind::Relationship => EntityKind::Relationship,
                },
                properties: Default::default(),
            },
        )
    }));
    lower_expression_with_references(expression, &bindings, catalog, "q", references)
}

pub(crate) fn mutation_rows_sql(
    input: &LoweredMutationInput,
    bindings: &[ir::BindingId],
) -> String {
    let columns = bindings
        .iter()
        .map(|binding| format!("q.{}", binding_column(*binding)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("SELECT {columns} FROM ({}) AS q", input.sql)
}

pub(crate) fn quoted_identifier(identifier: &str) -> String {
    quote_identifier(identifier)
}

/// Lower a bound fixed-pattern graph plan into Turso's public SQL AST.
///
/// Generated SQL is parsed immediately, making the AST the only value that
/// crosses into Turso preparation. No planner or VDBE internals are built here.
pub fn lower_relational(
    plan: &ir::Plan,
    catalog: &dyn RelationalCatalogSnapshot,
) -> Result<ast::Stmt, LowerError> {
    let mut wanted = WantedProperties::new();
    collect_wanted(plan, &mut wanted);
    let lowered = lower_plan(plan, catalog, false, &wanted)?;
    let sql = if plan.result_shape().is_empty() {
        lowered.sql
    } else {
        let columns = plan
            .result_shape()
            .iter()
            .map(|column| {
                format!(
                    "q.{} AS {}",
                    binding_column(column.binding()),
                    quote_identifier(column.name())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!("SELECT {columns} FROM ({}) AS q", lowered.sql)
    };
    if std::env::var_os("TURSO_GRAPH_DEBUG_SQL").is_some() {
        eprintln!("SQL: {sql}");
    }
    let (command, _) = turso_core::dialect::sqlite::parse(&sql)?;
    match command {
        Some(ast::Cmd::Stmt(statement)) => Ok(statement),
        _ => Err(LowerError::EmptyGeneratedSql),
    }
}

/// Properties each binding's consumers actually read, collected from every
/// expression in the plan. Scans and expands materialize these as extra
/// columns so property access lowers to a column reference instead of a
/// correlated subquery per occurrence.
type WantedProperties = HashMap<ir::BindingId, std::collections::BTreeSet<u32>>;

fn property_column_ref(binding: ir::BindingId, property: ir::PropertyId) -> String {
    format!("{}_p{}", binding_column(binding), property.get())
}

/// Extra SELECT columns materializing a binding's wanted properties from
/// `alias`, recording availability on the layout. Properties without a
/// physical column fall back to subquery lowering.
fn materialize_properties(
    wanted: &WantedProperties,
    binding: ir::BindingId,
    source: ir::SourceTableId,
    alias: &str,
    catalog: &dyn RelationalCatalogSnapshot,
    layout: &mut BindingLayout,
) -> String {
    let Some(properties) = wanted.get(&binding) else {
        return String::new();
    };
    let mut columns = String::new();
    for property in properties {
        let Some(id) = ir::PropertyId::new(*property).ok() else {
            continue;
        };
        let Some(column) = catalog.property_column(source, id) else {
            continue;
        };
        columns.push_str(&format!(
            ", {alias}.{} AS {}",
            quote_identifier(&column),
            property_column_ref(binding, id)
        ));
        layout.properties.insert(*property);
    }
    columns
}

fn collect_wanted(plan: &ir::Plan, wanted: &mut WantedProperties) {
    let mut expressions: Vec<&ir::TypedExpression> = Vec::new();
    match plan.kind() {
        ir::PlanKind::Unit(_) | ir::PlanKind::NodeScan(_) => {}
        ir::PlanKind::FixedExpand(expand) => collect_wanted(&expand.input, wanted),
        ir::PlanKind::GraphExpand(expand) => collect_wanted(&expand.input, wanted),
        ir::PlanKind::Filter(filter) => {
            expressions.push(&filter.predicate);
            collect_wanted(&filter.input, wanted);
        }
        ir::PlanKind::Project(project) => {
            expressions.extend(project.projections.iter().map(|p| &p.expression));
            collect_wanted(&project.input, wanted);
        }
        ir::PlanKind::Aggregate(aggregate) => {
            expressions.extend(aggregate.groupings.iter().map(|g| &g.expression));
            expressions.extend(
                aggregate
                    .aggregations
                    .iter()
                    .filter_map(|a| a.expression.as_ref()),
            );
            collect_wanted(&aggregate.input, wanted);
        }
        ir::PlanKind::Distinct(distinct) => {
            expressions.extend(distinct.keys.iter());
            collect_wanted(&distinct.input, wanted);
        }
        ir::PlanKind::Sort(sort) => {
            expressions.extend(sort.keys.iter().map(|k| &k.expression));
            collect_wanted(&sort.input, wanted);
        }
        ir::PlanKind::Skip(skip) => {
            expressions.push(&skip.count);
            collect_wanted(&skip.input, wanted);
        }
        ir::PlanKind::Limit(limit) => {
            expressions.push(&limit.count);
            collect_wanted(&limit.input, wanted);
        }
        ir::PlanKind::LeftApply(apply) => {
            collect_wanted(&apply.left, wanted);
            collect_wanted(&apply.right, wanted);
        }
        ir::PlanKind::Unwind(unwind) => {
            expressions.push(&unwind.list);
            collect_wanted(&unwind.input, wanted);
        }
        ir::PlanKind::Join(join) => {
            collect_wanted(&join.left, wanted);
            collect_wanted(&join.right, wanted);
        }
        ir::PlanKind::Union(union) => {
            for input in union.inputs() {
                collect_wanted(input, wanted);
            }
        }
    }
    for expression in expressions {
        collect_expression_wanted(expression, wanted);
    }
}

fn collect_expression_wanted(expression: &ir::TypedExpression, wanted: &mut WantedProperties) {
    let mut stack = vec![&expression.expression];
    while let Some(current) = stack.pop() {
        match current {
            ir::Expression::Property {
                entity,
                property,
                fields,
            } if fields.is_empty() => {
                wanted.entry(*entity).or_default().insert(property.get());
            }
            _ => {}
        }
        for child in expression_children(current) {
            stack.push(child);
        }
    }
}

fn expression_children<'a>(expression: &'a ir::Expression) -> Vec<&'a ir::Expression> {
    let mut children: Vec<&'a ir::Expression> = Vec::new();
    let mut push = |value: &'a ir::TypedExpression| {
        children.push(&value.expression);
    };
    match expression {
        ir::Expression::Binary { left, right, .. } => {
            push(left);
            push(right);
        }
        ir::Expression::Unary { expression, .. } => push(expression),
        ir::Expression::Function { arguments, .. } => arguments.iter().for_each(&mut push),
        ir::Expression::Case {
            subject,
            branches,
            default,
        } => {
            if let Some(subject) = subject {
                push(subject);
            }
            for (when, then) in branches {
                push(when);
                push(then);
            }
            if let Some(default) = default {
                push(default);
            }
        }
        ir::Expression::List(values) => values.iter().for_each(&mut push),
        ir::Expression::Map(entries) => entries.iter().for_each(|(_, value)| push(value)),
        ir::Expression::Index { base, index } => {
            push(base);
            push(index);
        }
        ir::Expression::Slice { base, from, to } => {
            push(base);
            from.as_deref().map(&mut push);
            to.as_deref().map(&mut push);
        }
        ir::Expression::Cast { expression, .. } => push(expression),
        ir::Expression::Quantifier {
            list, predicate, ..
        } => {
            push(list);
            push(predicate);
        }
        ir::Expression::ListComprehension {
            list,
            predicate,
            map,
            ..
        } => {
            push(list);
            predicate.as_deref().map(&mut push);
            map.as_deref().map(&mut push);
        }
        // PathValue carries binding ids, not sub-expressions.
        ir::Expression::PathValue { .. } => {}
        _ => {}
    }
    children
}

fn lower_plan(
    plan: &ir::Plan,
    catalog: &dyn RelationalCatalogSnapshot,
    optional: bool,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    match plan.kind() {
        ir::PlanKind::Unit(_) => Ok(Lowered {
            sql: "SELECT 1 AS __unit".to_owned(),
            bindings: HashMap::new(),
        }),
        ir::PlanKind::NodeScan(scan) => lower_node_scan(scan, catalog, wanted),
        ir::PlanKind::FixedExpand(expand) => {
            lower_fixed_expand(expand, catalog, optional, &[], wanted, None)
        }
        ir::PlanKind::GraphExpand(expand) => lower_graph_expand(expand, catalog, wanted),
        ir::PlanKind::Filter(filter) => {
            let input = lower_plan(&filter.input, catalog, optional, wanted)?;
            let predicate = lower_expression(&filter.predicate, &input.bindings, catalog, "q")?;
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q WHERE {predicate}", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Project(project) => lower_project(project, catalog, optional, wanted),
        ir::PlanKind::Distinct(distinct) => {
            let input = lower_plan(&distinct.input, catalog, optional, wanted)?;
            Ok(Lowered {
                sql: format!("SELECT DISTINCT q.* FROM ({}) AS q", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::LeftApply(apply) => {
            lower_optional_chain(&apply.right, Some(&apply.left), catalog, wanted)
        }
        ir::PlanKind::Aggregate(aggregate) => {
            let input = lower_plan(&aggregate.input, catalog, optional, wanted)?;
            let mut selects = Vec::new();
            let mut groups = Vec::new();
            let mut bindings = HashMap::new();
            for grouping in &aggregate.groupings {
                let sql = lower_expression(&grouping.expression, &input.bindings, catalog, "q")?;
                selects.push(format!(
                    "({sql}) AS {}",
                    binding_column(grouping.output.id())
                ));
                groups.push(format!("({sql})"));
                // Entity groupings keep their relational layout addressable
                // for later property access.
                if let ir::Expression::Binding(source_binding) = &grouping.expression.expression {
                    if let Some(layout) = input.bindings.get(source_binding) {
                        let mut layout = layout.clone();
                        layout.properties.clear();
                        bindings.insert(grouping.output.id(), layout);
                    }
                }
            }
            for aggregation in &aggregate.aggregations {
                let argument = aggregation
                    .expression
                    .as_ref()
                    .map(|expression| lower_expression(expression, &input.bindings, catalog, "q"))
                    .transpose()?;
                let distinct = if aggregation.distinct {
                    "DISTINCT "
                } else {
                    ""
                };
                let call = match (&aggregation.function, &argument) {
                    (ir::AggregateFunction::Count, None) => "count(*)".to_owned(),
                    (ir::AggregateFunction::Count, Some(argument)) => {
                        format!("count({distinct}({argument}))")
                    }
                    (ir::AggregateFunction::Sum, Some(argument)) => {
                        format!("sum({distinct}({argument}))")
                    }
                    (ir::AggregateFunction::Average, Some(argument)) => {
                        format!("avg({distinct}({argument}))")
                    }
                    (ir::AggregateFunction::Minimum, Some(argument)) => {
                        format!("min({distinct}({argument}))")
                    }
                    (ir::AggregateFunction::Maximum, Some(argument)) => {
                        format!("max({distinct}({argument}))")
                    }
                    (ir::AggregateFunction::Collect, Some(argument)) => {
                        format!("json_group_array({distinct}({argument}))")
                    }
                    _ => {
                        return Err(LowerError::UnsupportedOperator(
                            "aggregate call without an argument",
                        ));
                    }
                };
                selects.push(format!(
                    "{call} AS {}",
                    binding_column(aggregation.output.id())
                ));
            }
            let group_by = if groups.is_empty() {
                String::new()
            } else {
                format!(" GROUP BY {}", groups.join(", "))
            };
            Ok(Lowered {
                sql: format!(
                    "SELECT {} FROM ({}) AS q{group_by}",
                    selects.join(", "),
                    input.sql
                ),
                bindings,
            })
        }
        ir::PlanKind::Sort(sort) => {
            let input = lower_plan(&sort.input, catalog, optional, wanted)?;
            let ordering = lower_ordering(&sort.keys, &input.bindings, catalog)?;
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q ORDER BY {ordering}", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Skip(skip) => {
            let input = lower_plan(&skip.input, catalog, optional, wanted)?;
            let count = lower_expression(&skip.count, &input.bindings, catalog, "q")?;
            Ok(Lowered {
                sql: format!(
                    "SELECT q.* FROM ({}) AS q LIMIT -1 OFFSET {count}",
                    input.sql
                ),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Limit(limit) => {
            let input = lower_plan(&limit.input, catalog, optional, wanted)?;
            let count = lower_expression(&limit.count, &input.bindings, catalog, "q")?;
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q LIMIT {count}", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Unwind(unwind) => {
            let input = lower_plan(&unwind.input, catalog, optional, wanted)?;
            let list = lower_expression(&unwind.list, &input.bindings, catalog, "q")?;
            Ok(Lowered {
                sql: format!(
                    "SELECT q.*, j.value AS {} FROM ({}) AS q JOIN json_each({list}) AS j",
                    binding_column(unwind.output.id()),
                    input.sql
                ),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Join(join) => {
            let left = lower_plan(&join.left, catalog, optional, wanted)?;
            let right = lower_plan(&join.right, catalog, optional, wanted)?;
            let mut bindings = left.bindings;
            bindings.extend(right.bindings);
            Ok(Lowered {
                sql: format!(
                    "SELECT l.*, r.* FROM ({}) AS l JOIN ({}) AS r",
                    left.sql, right.sql
                ),
                bindings,
            })
        }
        ir::PlanKind::Union(union) => {
            let mut parts = Vec::new();
            let mut bindings = None;
            let no_pushdown = WantedProperties::new();
            for input in union.inputs() {
                let lowered = lower_plan(input, catalog, optional, &no_pushdown)?;
                // Branch column names differ (per-branch binding ids); SQL
                // set operators combine positionally and the first branch's
                // names win, matching this Union node's scope.
                parts.push(format!("SELECT q.* FROM ({}) AS q", lowered.sql));
                if bindings.is_none() {
                    bindings = Some(lowered.bindings);
                }
            }
            let separator = if union.is_all() {
                " UNION ALL "
            } else {
                " UNION "
            };
            Ok(Lowered {
                sql: parts.join(separator),
                bindings: bindings.unwrap_or_default(),
            })
        }
    }
}

fn lower_graph_expand(
    expand: &ir::GraphExpand,
    catalog: &dyn RelationalCatalogSnapshot,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    let input = lower_plan(&expand.input, catalog, false, wanted)?;
    let from = input
        .bindings
        .get(&expand.from)
        .cloned()
        .ok_or(LowerError::MissingBinding(expand.from))?;
    let relationship = catalog
        .relationship_layout(expand.relationship_source)
        .ok_or(LowerError::MissingSource(expand.relationship_source))?;
    let target = catalog
        .node_layout(expand.target_node_source)
        .ok_or(LowerError::MissingSource(expand.target_node_source))?;
    let direction = match expand.direction {
        ir::Direction::Outgoing => "outgoing",
        ir::Direction::Incoming => "incoming",
        ir::Direction::Both => "both",
    };
    let uniqueness = match expand.uniqueness {
        ir::PathUniqueness::Walk => "walk",
        ir::PathUniqueness::Trail => "trail",
        ir::PathUniqueness::Path => "path",
    };
    let relationship_types = expand
        .relationship_types
        .iter()
        .map(|relationship_type| relationship_type.get().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let limits = turso_graph_runtime::TraversalLimits::default();
    let mut bindings = input.bindings.clone();
    bindings.insert(
        expand.relationship.id(),
        BindingLayout {
            source: expand.relationship_source,
            kind: EntityKind::Relationship,
            properties: Default::default(),
        },
    );
    bindings.insert(
        expand.to.id(),
        BindingLayout {
            source: expand.target_node_source,
            kind: EntityKind::Node,
            properties: Default::default(),
        },
    );
    if expand.path_output.is_some() || expand.relationship_list_output.is_some() {
        // Materialize each traversed path: group the expansion's per-step
        // rows by path identity, building the {nodes, relationships} value
        // and/or the relationship-identity list alongside the terminal
        // node/relationship columns.
        let group_columns = expand
            .input
            .scope()
            .iter()
            // Path bindings live in scope without a materialized column;
            // grouping on them would reference columns the input never
            // produced. List bindings are materialized (projection outputs
            // and earlier expansions' grouped list columns) and must pass
            // through — a bound relationship list is compared against the
            // expansion's own list right above this grouping.
            .filter(|binding| !matches!(binding.value_type(), ir::ValueType::Path))
            .flat_map(|binding| {
                // The binding's identity column plus any pushed-down
                // property columns riding along in the input (all
                // functionally dependent on the identity, so grouping on
                // them is dedup-neutral).
                let mut columns = vec![format!("q.{}", binding_column(binding.id()))];
                if let Some(layout) = input.bindings.get(&binding.id()) {
                    for property in &layout.properties {
                        if let Ok(id) = ir::PropertyId::new(*property) {
                            columns.push(format!("q.{}", property_column_ref(binding.id(), id)));
                        }
                    }
                }
                columns
            })
            .collect::<Vec<_>>()
            .join(", ");
        let inner_select = if group_columns.is_empty() {
            String::new()
        } else {
            format!("{group_columns}, ")
        };
        let mut aggregates = Vec::new();
        if let Some(path_output) = &expand.path_output {
            aggregates.push(format!(
                "json_object('nodes', json_group_array(gx.node_identity), \
                 'relationships', json_group_array(gx.relationship_identity) \
                 FILTER (WHERE gx.path_position > 0)) AS {}",
                binding_column(path_output.id()),
            ));
        }
        if let Some(list_output) = &expand.relationship_list_output {
            aggregates.push(format!(
                "json_group_array(gx.relationship_identity) \
                 FILTER (WHERE gx.path_position > 0) AS {}",
                binding_column(list_output.id()),
            ));
        }
        let aggregates = aggregates.join(", ");
        return Ok(Lowered {
            sql: format!(
                "SELECT g.*, r.{} AS {}, n.{} AS {} \
                 FROM (SELECT {}{aggregates}, \
                 max(CASE WHEN gx.is_terminal = 1 THEN gx.node_identity END) AS __gx_node, \
                 max(CASE WHEN gx.is_terminal = 1 THEN gx.relationship_identity END) AS __gx_rel \
                 FROM ({}) AS q \
                 JOIN __turso_graph_expand({}, {}, q.{}, '{}', '{}', {}, {}, '{}', {}, {}, {}, {}, {}) AS gx \
                 GROUP BY {}gx.path_id) AS g \
                 JOIN {} AS n ON n.{} = g.__gx_node \
                 LEFT JOIN {} AS r ON r.{} = g.__gx_rel",
                quote_identifier(&relationship.identity_column),
                binding_column(expand.relationship.id()),
                quote_identifier(&target.identity_column),
                binding_column(expand.to.id()),
                inner_select,
                input.sql,
                expand.graph.get(),
                from.source.get(),
                binding_column(expand.from),
                direction,
                relationship_types,
                expand.min_hops,
                expand.max_hops,
                uniqueness,
                limits.max_node_visits,
                limits.max_edge_visits,
                limits.max_paths,
                limits.max_work,
                limits.max_memory_bytes,
                inner_select,
                quote_identifier(&target.table),
                quote_identifier(&target.identity_column),
                quote_identifier(&relationship.table),
                quote_identifier(&relationship.identity_column),
            ),
            bindings,
        });
    }
    Ok(Lowered {
        sql: format!(
            "SELECT q.*, r.{} AS {}, n.{} AS {} \
             FROM ({}) AS q \
             JOIN __turso_graph_expand({}, {}, q.{}, '{}', '{}', {}, {}, '{}', {}, {}, {}, {}, {}) AS gx \
             JOIN {} AS n ON gx.is_terminal = 1 AND gx.node_source_id = {} AND n.{} = gx.node_identity \
             LEFT JOIN {} AS r ON gx.relationship_source_id = {} AND r.{} = gx.relationship_identity",
            quote_identifier(&relationship.identity_column),
            binding_column(expand.relationship.id()),
            quote_identifier(&target.identity_column),
            binding_column(expand.to.id()),
            input.sql,
            expand.graph.get(),
            from.source.get(),
            binding_column(expand.from),
            direction,
            relationship_types,
            expand.min_hops,
            expand.max_hops,
            uniqueness,
            limits.max_node_visits,
            limits.max_edge_visits,
            limits.max_paths,
            limits.max_work,
            limits.max_memory_bytes,
            quote_identifier(&target.table),
            expand.target_node_source.get(),
            quote_identifier(&target.identity_column),
            quote_identifier(&relationship.table),
            expand.relationship_source.get(),
            quote_identifier(&relationship.identity_column),
        ),
        bindings,
    })
}

/// Lowers a LeftApply's right side: expands and filters ABOVE `boundary`
/// (the mandatory left plan embedded as the chain's input) belong to the
/// OPTIONAL MATCH and lower as LEFT JOINs with their predicates folded
/// into the join condition; the boundary subtree and everything below it
/// is mandatory and lowers normally.
fn lower_optional_chain(
    plan: &ir::Plan,
    boundary: Option<&ir::Plan>,
    catalog: &dyn RelationalCatalogSnapshot,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    if boundary.is_some_and(|boundary| plan == boundary) {
        return lower_plan(plan, catalog, false, wanted);
    }
    let original = plan;
    let mut current = plan;
    let mut predicates = Vec::new();
    while let ir::PlanKind::Filter(filter) = current.kind() {
        if boundary.is_some_and(|boundary| current == boundary) {
            return lower_plan(original, catalog, false, wanted);
        }
        predicates.push(&filter.predicate);
        current = &filter.input;
    }
    if boundary.is_some_and(|boundary| current == boundary) {
        // Only filters sat above the boundary: they are optional-pattern
        // predicates over the mandatory plan; treat them as a plain filter
        // (an optional pattern with no expansion adds no bindings).
        return lower_plan(original, catalog, false, wanted);
    }
    match current.kind() {
        ir::PlanKind::FixedExpand(expand) => {
            lower_fixed_expand(expand, catalog, true, &predicates, wanted, boundary)
        }
        _ => lower_plan(original, catalog, false, wanted),
    }
}

fn lower_project(
    project: &ir::Project,
    catalog: &dyn RelationalCatalogSnapshot,
    optional: bool,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    let (input, sort_keys) = match project.input.kind() {
        ir::PlanKind::Sort(sort) => (
            lower_plan(&sort.input, catalog, optional, wanted)?,
            Some(sort.keys.as_slice()),
        ),
        _ => (lower_plan(&project.input, catalog, optional, wanted)?, None),
    };
    let mut bindings = HashMap::new();
    let columns = project
        .projections
        .iter()
        .map(|projection| {
            if let ir::Expression::Binding(input_binding) = projection.expression.expression {
                if let Some(layout) = input.bindings.get(&input_binding) {
                    let mut layout = layout.clone();
                    layout.properties.clear();
                    bindings.insert(projection.output.id(), layout);
                }
            }
            let expression =
                lower_expression(&projection.expression, &input.bindings, catalog, "q")?;
            Ok(format!(
                "{expression} AS {}",
                binding_column(projection.output.id())
            ))
        })
        .collect::<Result<Vec<_>, LowerError>>()?
        .join(", ");
    let ordering = sort_keys
        .map(|keys| lower_ordering(keys, &input.bindings, catalog))
        .transpose()?
        .map(|ordering| format!(" ORDER BY {ordering}"))
        .unwrap_or_default();
    Ok(Lowered {
        sql: format!("SELECT {columns} FROM ({}) AS q{ordering}", input.sql),
        bindings,
    })
}

fn lower_ordering(
    keys: &[ir::SortKey],
    bindings: &HashMap<ir::BindingId, BindingLayout>,
    catalog: &dyn RelationalCatalogSnapshot,
) -> Result<String, LowerError> {
    keys.iter()
        .map(|key| {
            let expression = lower_expression(&key.expression, bindings, catalog, "q")?;
            let direction = match key.direction {
                ir::SortDirection::Ascending => "ASC",
                ir::SortDirection::Descending => "DESC",
            };
            let null_order = match key.null_order {
                ir::NullOrder::First => "NULLS FIRST",
                ir::NullOrder::Last => "NULLS LAST",
            };
            Ok(format!("{expression} {direction} {null_order}"))
        })
        .collect::<Result<Vec<_>, LowerError>>()
        .map(|items| items.join(", "))
}

fn lower_node_scan(
    scan: &ir::NodeScan,
    catalog: &dyn RelationalCatalogSnapshot,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    let layout = catalog
        .node_layout(scan.source)
        .ok_or(LowerError::MissingSource(scan.source))?;
    let mut bindings = HashMap::new();
    let mut binding_layout = BindingLayout {
        source: scan.source,
        kind: EntityKind::Node,
        properties: Default::default(),
    };
    let extra = materialize_properties(
        wanted,
        scan.binding,
        scan.source,
        "n",
        catalog,
        &mut binding_layout,
    );
    bindings.insert(scan.binding, binding_layout);
    let mut sql = format!(
        "SELECT n.{} AS {}{extra} FROM {} AS n",
        quote_identifier(&layout.identity_column),
        binding_column(scan.binding),
        quote_identifier(&layout.table)
    );
    // Filter labeled scans through the node-label junction when available.
    // Joins (each label yields at most one junction row per node) keep the
    // scan shape simple for downstream traversal joins.
    if let Some(labels_table) = catalog.labels_table() {
        for (index, label) in scan.labels.iter().enumerate() {
            if let Some(name) = catalog.label_name(*label) {
                sql.push_str(&format!(
                    " JOIN {} AS lbl{index} ON lbl{index}.node_id = n.{} AND lbl{index}.label = '{}'",
                    quote_identifier(&labels_table),
                    quote_identifier(&layout.identity_column),
                    name.replace('\'', "''")
                ));
            }
        }
    }
    Ok(Lowered { sql, bindings })
}

fn lower_fixed_expand(
    expand: &ir::FixedExpand,
    catalog: &dyn RelationalCatalogSnapshot,
    optional: bool,
    join_predicates: &[&ir::TypedExpression],
    wanted: &WantedProperties,
    boundary: Option<&ir::Plan>,
) -> Result<Lowered, LowerError> {
    let input = if optional {
        lower_optional_chain(&expand.input, boundary, catalog, wanted)?
    } else {
        lower_plan(&expand.input, catalog, false, wanted)?
    };
    let relationship = catalog
        .relationship_layout(expand.relationship_source)
        .ok_or(LowerError::MissingSource(expand.relationship_source))?;
    let target = catalog
        .node_layout(expand.target_node_source)
        .ok_or(LowerError::MissingSource(expand.target_node_source))?;
    let join = if optional { "LEFT JOIN" } else { "JOIN" };
    let from = format!("q.{}", binding_column(expand.from));
    let relationship_alias = "r";
    let target_alias = "n";
    // A cycle-closing target equates to an existing binding: both endpoint
    // conditions go on the relationship join (composite endpoint indexes
    // apply) and the node table is not re-joined.
    let bound_reference = expand
        .bound_target
        .map(|binding| format!("q.{}", binding_column(binding)));
    let (relationship_on, mut node_on) = match (&bound_reference, expand.direction) {
        (Some(bound), ir::Direction::Outgoing) => (
            format!(
                "{relationship_alias}.{} = {from} AND {relationship_alias}.{} = {bound}",
                quote_identifier(&relationship.start_column),
                quote_identifier(&relationship.end_column)
            ),
            String::new(),
        ),
        (Some(bound), ir::Direction::Incoming) => (
            format!(
                "{relationship_alias}.{} = {from} AND {relationship_alias}.{} = {bound}",
                quote_identifier(&relationship.end_column),
                quote_identifier(&relationship.start_column)
            ),
            String::new(),
        ),
        (Some(bound), ir::Direction::Both) => (
            format!(
                "(({relationship_alias}.{start} = {from} AND {relationship_alias}.{end} = {bound})                  OR ({relationship_alias}.{end} = {from} AND {relationship_alias}.{start} = {bound}))",
                start = quote_identifier(&relationship.start_column),
                end = quote_identifier(&relationship.end_column)
            ),
            String::new(),
        ),
        (None, _) => match expand.direction {
        ir::Direction::Outgoing => (
            format!(
                "{relationship_alias}.{} = {from}",
                quote_identifier(&relationship.start_column)
            ),
            format!(
                "{target_alias}.{} = {relationship_alias}.{}",
                quote_identifier(&target.identity_column),
                quote_identifier(&relationship.end_column)
            ),
        ),
        ir::Direction::Incoming => (
            format!(
                "{relationship_alias}.{} = {from}",
                quote_identifier(&relationship.end_column)
            ),
            format!(
                "{target_alias}.{} = {relationship_alias}.{}",
                quote_identifier(&target.identity_column),
                quote_identifier(&relationship.start_column)
            ),
        ),
        ir::Direction::Both => (
            format!(
                "({relationship_alias}.{} = {from} OR {relationship_alias}.{} = {from})",
                quote_identifier(&relationship.start_column),
                quote_identifier(&relationship.end_column)
            ),
            format!(
                "{target_alias}.{} = CASE WHEN {relationship_alias}.{} = {from} \
                 THEN {relationship_alias}.{} ELSE {relationship_alias}.{} END",
                quote_identifier(&target.identity_column),
                quote_identifier(&relationship.start_column),
                quote_identifier(&relationship.end_column),
                quote_identifier(&relationship.start_column)
            ),
        ),
        },
    };
    // Filter typed hops through the relationship-type junction; recorded
    // types are authoritative, untyped rows only match untyped patterns.
    let mut relationship_on = if expand.relationship_types.is_empty() {
        relationship_on
    } else if let Some(types_table) = catalog.relationship_types_table() {
        let names = expand
            .relationship_types
            .iter()
            .filter_map(|relationship_type| catalog.relationship_type_name(*relationship_type))
            .map(|name| format!("'{}'", name.replace('\'', "''")))
            .collect::<Vec<_>>();
        if names.is_empty() {
            relationship_on
        } else {
            format!(
                "({relationship_on}) AND EXISTS (SELECT 1 FROM {} AS jt \
                 WHERE jt.relationship_id = {relationship_alias}.{} \
                 AND jt.type IN ({}))",
                quote_identifier(&types_table),
                quote_identifier(&relationship.identity_column),
                names.join(", ")
            )
        }
    } else {
        relationship_on
    };
    let mut bindings = input.bindings;
    bindings.insert(
        expand.relationship.id(),
        BindingLayout {
            source: expand.relationship_source,
            kind: EntityKind::Relationship,
            properties: Default::default(),
        },
    );
    bindings.insert(
        expand.to.id(),
        BindingLayout {
            source: expand.target_node_source,
            kind: EntityKind::Node,
            properties: Default::default(),
        },
    );
    // Optional expands keep the subquery fallback: LEFT JOIN column
    // nullability for the relationship depends on the target match.
    let mut extra = String::new();
    if !optional {
        let mut relationship_layout = bindings
            .get(&expand.relationship.id())
            .cloned()
            .expect("inserted above");
        extra.push_str(&materialize_properties(
            wanted,
            expand.relationship.id(),
            expand.relationship_source,
            relationship_alias,
            catalog,
            &mut relationship_layout,
        ));
        // A cycle-closing target has no node alias to materialize from;
        // downstream references go through the pre-bound variable instead.
        if bound_reference.is_none() {
            let mut to_layout = bindings
                .get(&expand.to.id())
                .cloned()
                .expect("inserted above");
            extra.push_str(&materialize_properties(
                wanted,
                expand.to.id(),
                expand.target_node_source,
                target_alias,
                catalog,
                &mut to_layout,
            ));
            bindings.insert(expand.to.id(), to_layout);
        }
        bindings.insert(expand.relationship.id(), relationship_layout);
    }
    if !join_predicates.is_empty() {
        let references = HashMap::from([
            (
                expand.relationship.id(),
                format!(
                    "{relationship_alias}.{}",
                    quote_identifier(&relationship.identity_column)
                ),
            ),
            (
                expand.to.id(),
                match &bound_reference {
                    Some(bound) => bound.clone(),
                    None => format!(
                        "{target_alias}.{}",
                        quote_identifier(&target.identity_column)
                    ),
                },
            ),
        ]);
        let predicates = join_predicates
            .iter()
            .map(|predicate| {
                lower_expression_with_references(predicate, &bindings, catalog, "q", &references)
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(" AND ");
        if node_on.is_empty() {
            relationship_on = format!("({relationship_on}) AND ({predicates})");
        } else {
            node_on = format!("({node_on}) AND ({predicates})");
        }
    }
    let null_probe = match &bound_reference {
        Some(_) => format!(
            "{relationship_alias}.{}",
            quote_identifier(&relationship.identity_column)
        ),
        None => format!(
            "{target_alias}.{}",
            quote_identifier(&target.identity_column)
        ),
    };
    let relationship_identity = if optional {
        format!(
            "CASE WHEN {null_probe} IS NULL THEN NULL ELSE {relationship_alias}.{} END",
            quote_identifier(&relationship.identity_column)
        )
    } else {
        format!(
            "{relationship_alias}.{}",
            quote_identifier(&relationship.identity_column)
        )
    };
    let target_value = match &bound_reference {
        Some(bound) if optional => {
            format!("CASE WHEN {null_probe} IS NULL THEN NULL ELSE {bound} END")
        }
        Some(bound) => bound.clone(),
        None => format!(
            "{target_alias}.{}",
            quote_identifier(&target.identity_column)
        ),
    };
    let node_join = match &bound_reference {
        Some(_) => String::new(),
        None => format!(
            " {join} {} AS {target_alias} ON {node_on}",
            quote_identifier(&target.table)
        ),
    };
    Ok(Lowered {
        sql: format!(
            "SELECT q.*, {relationship_identity} AS {}, {target_value} AS {}{extra} \
             FROM ({}) AS q {join} {} AS {relationship_alias} ON {relationship_on}{node_join}",
            binding_column(expand.relationship.id()),
            binding_column(expand.to.id()),
            input.sql,
            quote_identifier(&relationship.table),
        ),
        bindings,
    })
}

fn lower_expression(
    expression: &ir::TypedExpression,
    bindings: &HashMap<ir::BindingId, BindingLayout>,
    catalog: &dyn RelationalCatalogSnapshot,
    input_alias: &str,
) -> Result<String, LowerError> {
    lower_expression_with_references(expression, bindings, catalog, input_alias, &HashMap::new())
}

fn lower_expression_with_references(
    expression: &ir::TypedExpression,
    bindings: &HashMap<ir::BindingId, BindingLayout>,
    catalog: &dyn RelationalCatalogSnapshot,
    input_alias: &str,
    references: &HashMap<ir::BindingId, String>,
) -> Result<String, LowerError> {
    match &expression.expression {
        ir::Expression::Literal(literal) => Ok(lower_literal(literal)),
        ir::Expression::Binding(binding) => {
            Ok(binding_reference(*binding, input_alias, references))
        }
        ir::Expression::Property {
            entity,
            property,
            fields,
        } => {
            if fields.len() > 2 {
                return Err(LowerError::UnsupportedOperator(
                    "struct/union field access deeper than two levels",
                ));
            }
            let binding = bindings
                .get(entity)
                .ok_or(LowerError::MissingBinding(*entity))?;
            // The scan already materialized this property as a column; a
            // direct reference replaces the correlated subquery. Reference
            // overrides (mutation rows, join predicates) bypass this: their
            // contexts never carry materialized property columns.
            let jsonb = catalog.property_column_is_jsonb(binding.source, *property);
            if fields.is_empty()
                && binding.properties.contains(&property.get())
                && !references.contains_key(entity)
            {
                let column = format!("{input_alias}.{}", property_column_ref(*entity, *property));
                return Ok(if jsonb {
                    format!("json({column})")
                } else {
                    column
                });
            }
            let column = catalog.property_column(binding.source, *property).ok_or(
                LowerError::MissingProperty {
                    source_id: binding.source,
                    property: *property,
                },
            )?;
            let (table, identity) = match binding.kind {
                EntityKind::Node => {
                    let layout = catalog
                        .node_layout(binding.source)
                        .ok_or(LowerError::MissingSource(binding.source))?;
                    (layout.table, layout.identity_column)
                }
                EntityKind::Relationship => {
                    let layout = catalog
                        .relationship_layout(binding.source)
                        .ok_or(LowerError::MissingSource(binding.source))?;
                    (layout.table, layout.identity_column)
                }
            };
            let mut selector = quote_identifier(&column);
            for field in fields {
                validate_bare_name(field)?;
                selector.push('.');
                selector.push_str(&quote_identifier(field));
            }
            // core's SQL grammar caps dot-chain expressions at 3 identifiers
            // (`Expr::DoublyQualified`, core/translate/expr/binding.rs). The
            // `p.col.field1` form (<=1 nested field) fits that cap, but
            // `p.col.field1.field2` (2 nested fields) would be a 4th
            // identifier core's parser cannot parse at all. core's only AST
            // path for genuine 2-level nested field access instead requires
            // an unqualified column name as the chain's root
            // (`col.field1.field2`, `try_resolve_nested_field_access`), so
            // the `p.` alias prefix is dropped for that case. This is safe:
            // `find_custom_type_column` only searches this subquery's own
            // single joined table, never the outer query's scope, so the
            // bare column name stays unambiguous. The WHERE clause's
            // identity correlation keeps using the `p.` alias either way.
            let selector = if fields.len() < 2 {
                format!("p.{selector}")
            } else {
                selector
            };
            let selector = if jsonb && fields.is_empty() {
                format!("json({selector})")
            } else {
                selector
            };
            Ok(format!(
                "(SELECT {} FROM {} AS p WHERE p.{} = {})",
                selector,
                quote_identifier(&table),
                quote_identifier(&identity),
                binding_reference(*entity, input_alias, references)
            ))
        }
        ir::Expression::Parameter(name) => {
            validate_bare_name(name)?;
            Ok(format!("${name}"))
        }
        ir::Expression::Unary { op, expression } => {
            let value = lower_expression_with_references(
                expression,
                bindings,
                catalog,
                input_alias,
                references,
            )?;
            Ok(match op {
                ir::UnaryOp::Not => format!("NOT ({value})"),
                ir::UnaryOp::Negate => format!("-({value})"),
                ir::UnaryOp::IsNull => format!("({value}) IS NULL"),
                ir::UnaryOp::IsNotNull => format!("({value}) IS NOT NULL"),
            })
        }
        ir::Expression::Binary { left, op, right } => {
            let left_type = left.value_type.clone();
            let right_type = right.value_type.clone();
            let left =
                lower_expression_with_references(left, bindings, catalog, input_alias, references)?;
            let right = lower_expression_with_references(
                right,
                bindings,
                catalog,
                input_alias,
                references,
            )?;
            Ok(match op {
                // Cypher lists lower to JSON arrays; membership probes the
                // array's elements with strict typing (1 is not '1') and
                // ternary semantics: an unmatched probe over a list that
                // contains null, or a null probe over a non-empty list, is
                // null rather than false.
                ir::BinaryOp::In => {
                    let strict_match = format!(
                        "((typeof(e.value) IN ('integer', 'real') \
                          AND typeof(({left})) IN ('integer', 'real')) \
                          OR typeof(e.value) = typeof(({left}))) AND e.value = ({left})"
                    );
                    format!(
                        "(CASE WHEN ({right}) IS NULL THEN NULL \
                          WHEN EXISTS (SELECT 1 FROM json_each(({right})) AS e \
                          WHERE {strict_match}) THEN 1 \
                          WHEN ({left}) IS NULL AND json_array_length(({right})) > 0 THEN NULL \
                          WHEN EXISTS (SELECT 1 FROM json_each(({right})) AS e \
                          WHERE e.value IS NULL) THEN NULL \
                          ELSE 0 END)"
                    )
                }
                // List append: `[1] + 2` produces `[1, 2]`.
                ir::BinaryOp::Add
                    if matches!(left_type, ir::ValueType::List(_))
                        && !matches!(right_type, ir::ValueType::List(_)) =>
                {
                    format!("json_insert(({left}), '$[#]', ({right}))")
                }
                // Ordering comparisons are ternary and type-strict: numbers
                // compare with numbers and text with text; anything else
                // (including null operands) is null. SQLite's cross-type
                // ordering would otherwise make 1 < 'a' true.
                ir::BinaryOp::Less
                | ir::BinaryOp::LessOrEqual
                | ir::BinaryOp::Greater
                | ir::BinaryOp::GreaterOrEqual
                    if comparison_needs_type_guard(&left_type, &right_type) =>
                {
                    let operator = binary_operator(*op);
                    format!(
                        "(CASE WHEN ({left}) IS NULL OR ({right}) IS NULL THEN NULL \
                          WHEN typeof(({left})) IN ('integer', 'real') \
                          AND typeof(({right})) IN ('integer', 'real') \
                          THEN ({left}) {operator} ({right}) \
                          WHEN typeof(({left})) = 'text' AND typeof(({right})) = 'text' \
                          THEN ({left}) {operator} ({right}) \
                          ELSE NULL END)"
                    )
                }
                // Boolean XOR over 1/0/NULL: inequality preserves Cypher's
                // three-valued semantics.
                ir::BinaryOp::Xor => format!("(({left}) <> ({right}))"),
                ir::BinaryOp::Power => format!("pow(({left}), ({right}))"),
                // The CASE guards keep the empty-needle results Cypher
                // defines (always true) while still propagating NULL inputs;
                // instr/substr alone return false for an empty needle.
                ir::BinaryOp::StartsWith => format!(
                    "(CASE WHEN ({left}) IS NULL OR ({right}) IS NULL THEN NULL \
                     ELSE substr(({left}), 1, length(({right}))) = ({right}) END)"
                ),
                ir::BinaryOp::EndsWith => format!(
                    "(CASE WHEN ({left}) IS NULL OR ({right}) IS NULL THEN NULL \
                     WHEN length(({right})) = 0 THEN 1 \
                     ELSE substr(({left}), -length(({right}))) = ({right}) END)"
                ),
                ir::BinaryOp::Contains => format!(
                    "(CASE WHEN ({left}) IS NULL OR ({right}) IS NULL THEN NULL \
                     WHEN length(({right})) = 0 THEN 1 \
                     ELSE instr(({left}), ({right})) > 0 END)"
                ),
                _ => format!("({left}) {} ({right})", binary_operator(*op)),
            })
        }
        ir::Expression::ListElement(depth) => Ok(format!("lst{depth}.value")),
        ir::Expression::PathValue {
            nodes,
            relationships,
        } => {
            let render = |bindings_list: &[ir::BindingId]| {
                bindings_list
                    .iter()
                    .map(|binding| binding_reference(*binding, input_alias, references))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            Ok(format!(
                "json_object('nodes', json_array({}), 'relationships', json_array({}))",
                render(nodes),
                render(relationships)
            ))
        }
        ir::Expression::PatternSubquery {
            count,
            plan,
            correlations,
        } => {
            let sub = {
                let mut sub_wanted = WantedProperties::new();
                collect_wanted(plan, &mut sub_wanted);
                lower_plan(plan, catalog, false, &sub_wanted)?
            };
            let conditions = correlations
                .iter()
                .map(|(outer, inner)| {
                    // The outer side may be a binding the enclosing join is
                    // introducing (a predicate in a LEFT JOIN ON clause), so
                    // resolve it through the reference overrides.
                    format!(
                        "sub.{} = {}",
                        binding_column(*inner),
                        binding_reference(*outer, input_alias, references)
                    )
                })
                .collect::<Vec<_>>();
            let filter = if conditions.is_empty() {
                String::new()
            } else {
                format!(" WHERE {}", conditions.join(" AND "))
            };
            Ok(if *count {
                format!("(SELECT count(*) FROM ({}) AS sub{filter})", sub.sql)
            } else {
                format!("EXISTS (SELECT 1 FROM ({}) AS sub{filter})", sub.sql)
            })
        }
        ir::Expression::Index { base, index } => {
            let base =
                lower_expression_with_references(base, bindings, catalog, input_alias, references)?;
            let text_key = index.value_type == ir::ValueType::Text;
            // Dynamically-typed indices raise Cypher's runtime TypeError
            // when the value is not an integer (or a key string on maps).
            let dynamic = index.value_type == ir::ValueType::Any;
            let index = lower_expression_with_references(
                index,
                bindings,
                catalog,
                input_alias,
                references,
            )?;
            Ok(if text_key {
                format!("json_extract(({base}), '$.\"' || ({index}) || '\"')")
            } else if dynamic {
                format!(
                    "(CASE WHEN ({index}) IS NULL THEN NULL \
                     WHEN typeof(({index})) = 'text' AND json_type(({base})) = 'object' \
                     THEN json_extract(({base}), '$.\"' || ({index}) || '\"') \
                     WHEN typeof(({index})) != 'integer' \
                     THEN cypher_raise('TypeError', 'list index must be an integer') \
                     ELSE json_extract(({base}), '$[' || (CASE WHEN ({index}) >= 0 \
                     THEN ({index}) ELSE '#' || ({index}) END) || ']') END)"
                )
            } else {
                // Negative indices address from the end via the '#' form.
                format!(
                    "json_extract(({base}), '$[' || (CASE WHEN ({index}) >= 0 \
                     THEN ({index}) ELSE '#' || ({index}) END) || ']')"
                )
            })
        }
        ir::Expression::Slice { base, from, to } => {
            let base =
                lower_expression_with_references(base, bindings, catalog, input_alias, references)?;
            let normalized = |bound: &str| {
                format!(
                    "(CASE WHEN ({bound}) < 0 THEN json_array_length(({base})) + ({bound}) \
                     ELSE ({bound}) END)"
                )
            };
            let from = match from {
                Some(from) => normalized(&lower_expression_with_references(
                    from,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?),
                None => "0".to_owned(),
            };
            let to = match to {
                Some(to) => normalized(&lower_expression_with_references(
                    to,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?),
                None => format!("json_array_length(({base}))"),
            };
            Ok(format!(
                "(SELECT json_group_array(slc.value) FROM json_each(({base})) AS slc \
                 WHERE CAST(slc.key AS INTEGER) >= {from} AND CAST(slc.key AS INTEGER) < {to})"
            ))
        }
        ir::Expression::Cast { expression, target } => {
            let value = lower_expression_with_references(
                expression,
                bindings,
                catalog,
                input_alias,
                references,
            )?;
            Ok(match target {
                // Boolean casts need text-name handling; CAST('true' AS
                // INTEGER) is 0, not 1.
                ir::ValueType::Boolean => format!(
                    "(CASE WHEN ({value}) IS NULL THEN NULL \
                     WHEN lower(CAST(({value}) AS TEXT)) = 'true' THEN 1 \
                     WHEN lower(CAST(({value}) AS TEXT)) = 'false' THEN 0 \
                     WHEN typeof(({value})) IN ('integer', 'real') THEN ({value}) != 0 \
                     ELSE NULL END)"
                ),
                ir::ValueType::Integer => format!("CAST(({value}) AS INTEGER)"),
                ir::ValueType::Real => format!("CAST(({value}) AS REAL)"),
                _ => format!("CAST(({value}) AS TEXT)"),
            })
        }
        ir::Expression::Quantifier {
            kind,
            depth,
            list,
            predicate,
        } => {
            let list =
                lower_expression_with_references(list, bindings, catalog, input_alias, references)?;
            let predicate = lower_expression_with_references(
                predicate,
                bindings,
                catalog,
                input_alias,
                references,
            )?;
            let alias = format!("lst{depth}");
            Ok(match kind {
                ir::QuantifierKind::Any => format!(
                    "EXISTS (SELECT 1 FROM json_each(({list})) AS {alias} WHERE ({predicate}))"
                ),
                ir::QuantifierKind::All => format!(
                    "NOT EXISTS (SELECT 1 FROM json_each(({list})) AS {alias} \
                     WHERE NOT ({predicate}))"
                ),
                ir::QuantifierKind::None => format!(
                    "NOT EXISTS (SELECT 1 FROM json_each(({list})) AS {alias} \
                     WHERE ({predicate}))"
                ),
                ir::QuantifierKind::Single => format!(
                    "((SELECT count(*) FROM json_each(({list})) AS {alias} \
                     WHERE ({predicate})) = 1)"
                ),
            })
        }
        ir::Expression::ListComprehension {
            depth,
            list,
            predicate,
            map,
        } => {
            let list =
                lower_expression_with_references(list, bindings, catalog, input_alias, references)?;
            let alias = format!("lst{depth}");
            let element = match map {
                Some(map) => lower_expression_with_references(
                    map,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?,
                None => format!("{alias}.value"),
            };
            let filter = match predicate {
                Some(predicate) => lower_expression_with_references(
                    predicate,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?,
                None => "1".to_owned(),
            };
            Ok(format!(
                "(SELECT json_group_array({element}) FROM json_each(({list})) AS {alias} \
                 WHERE ({filter}))"
            ))
        }
        ir::Expression::Case {
            subject,
            branches,
            default,
        } => {
            let mut sql = String::from("CASE");
            if let Some(subject) = subject {
                let subject = lower_expression_with_references(
                    subject,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                sql.push_str(&format!(" ({subject})"));
            }
            for (condition, result) in branches {
                let condition = lower_expression_with_references(
                    condition,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                let result = lower_expression_with_references(
                    result,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                sql.push_str(&format!(" WHEN ({condition}) THEN ({result})"));
            }
            if let Some(default) = default {
                let default = lower_expression_with_references(
                    default,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                sql.push_str(&format!(" ELSE ({default})"));
            }
            sql.push_str(" END");
            Ok(format!("({sql})"))
        }
        ir::Expression::Function {
            function,
            arguments,
        } => {
            validate_bare_name(function.as_str())?;
            // properties(n) needs the argument's source table, so intercept
            // before argument lowering while the binding is still visible.
            if function.as_str() == "__cypher_properties" {
                let [argument] = arguments.as_slice() else {
                    return Err(LowerError::UnsupportedOperator(
                        "properties() with a non-entity argument",
                    ));
                };
                let ir::Expression::Binding(id) = &argument.expression else {
                    return Err(LowerError::UnsupportedOperator(
                        "properties() with a non-entity argument",
                    ));
                };
                let layout = bindings.get(id).ok_or(LowerError::MissingBinding(*id))?;
                let columns = catalog.payload_columns(layout.source).ok_or(
                    LowerError::UnsupportedOperator("properties() without payload columns"),
                )?;
                let (table, identity) = match layout.kind {
                    EntityKind::Node => {
                        let layout = catalog
                            .node_layout(layout.source)
                            .ok_or(LowerError::MissingSource(layout.source))?;
                        (layout.table, layout.identity_column)
                    }
                    EntityKind::Relationship => {
                        let layout = catalog
                            .relationship_layout(layout.source)
                            .ok_or(LowerError::MissingSource(layout.source))?;
                        (layout.table, layout.identity_column)
                    }
                };
                let identity_value = lower_expression_with_references(
                    argument,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                let pairs = columns
                    .iter()
                    .map(|(logical, physical)| {
                        format!(
                            "'{}', prp.{}",
                            logical.replace('\'', "''"),
                            quote_identifier(physical)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let object = if pairs.is_empty() {
                    "json_object()".to_owned()
                } else {
                    format!("json_object({pairs})")
                };
                // json_group_object over json_each strips null-valued keys,
                // matching Cypher's properties() (absent, not null); the
                // COALESCE keeps a propertyless entity at {} instead of null.
                return Ok(format!(
                    "(SELECT coalesce(json_group_object(je.key, je.value), json_object()) \
                     FROM json_each((SELECT {object} FROM {} AS prp \
                     WHERE prp.{} = ({identity_value}))) AS je \
                     WHERE je.value IS NOT NULL)",
                    quote_identifier(&table),
                    quote_identifier(&identity),
                ));
            }
            let arguments = arguments
                .iter()
                .map(|argument| {
                    lower_expression_with_references(
                        argument,
                        bindings,
                        catalog,
                        input_alias,
                        references,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            // Sentinel names the binder emits for Cypher builtins with no
            // single-function SQL equivalent.
            match (function.as_str(), arguments.as_slice()) {
                ("__cypher_size", [value]) => {
                    // json_type errors on invalid JSON and AND does not
                    // short-circuit, so nest the validity check.
                    return Ok(format!(
                        "(CASE WHEN ({value}) IS NULL THEN NULL \
                         WHEN json_valid(({value})) THEN \
                         (CASE WHEN json_type(({value})) = 'array' \
                         THEN json_array_length(({value})) ELSE length(({value})) END) \
                         ELSE length(({value})) END)"
                    ));
                }
                ("__cypher_range", [start, stop]) => {
                    return Ok(format!(
                        "(SELECT json_group_array(value) FROM generate_series(({start}), ({stop})))"
                    ));
                }
                ("__cypher_range", [start, stop, step]) => {
                    return Ok(format!(
                        "(SELECT json_group_array(value) \
                         FROM generate_series(({start}), ({stop}), ({step})))"
                    ));
                }
                ("__cypher_time_parse", [value]) => {
                    // Offset-less datetime strings parse as UTC; time_parse
                    // itself requires an offset on datetime forms.
                    let tail = format!("substr(({value}), instr(({value}), 'T'))");
                    return Ok(format!(
                        "time_parse(CASE WHEN instr(({value}), 'T') > 0 \
                         AND instr({tail}, 'Z') = 0 \
                         AND instr({tail}, '+') = 0 \
                         AND instr({tail}, '-') = 0 \
                         THEN ({value}) || 'Z' ELSE ({value}) END)"
                    ));
                }
                ("__cypher_all_labels", []) => {
                    let table = catalog
                        .labels_table()
                        .ok_or(LowerError::UnsupportedOperator(
                            "db.labels without a label table",
                        ))?;
                    return Ok(format!(
                        "(SELECT json_group_array(label) FROM \
                         (SELECT DISTINCT label FROM {} ORDER BY label))",
                        quote_identifier(&table)
                    ));
                }
                ("__cypher_relationship_type", [value]) => {
                    // No junction table means the graph does not track
                    // relationship types; nothing to report.
                    let Some(table) = catalog.relationship_types_table() else {
                        return Ok("NULL".to_owned());
                    };
                    return Ok(format!(
                        "(SELECT jt.type FROM {} AS jt \
                         WHERE jt.relationship_id = ({value}) LIMIT 1)",
                        quote_identifier(&table)
                    ));
                }
                ("__cypher_all_relationship_types", []) => {
                    let table = catalog.relationship_types_table().ok_or(
                        LowerError::UnsupportedOperator(
                            "db.relationshipTypes without a type table",
                        ),
                    )?;
                    return Ok(format!(
                        "(SELECT json_group_array(type) FROM \
                         (SELECT DISTINCT type FROM {} ORDER BY type))",
                        quote_identifier(&table)
                    ));
                }
                ("__cypher_labels", [value]) => {
                    let table = catalog
                        .labels_table()
                        .ok_or(LowerError::UnsupportedOperator(
                            "labels() without a label table",
                        ))?;
                    return Ok(format!(
                        "(SELECT json_group_array(label) FROM (SELECT lbl.label FROM {} AS lbl \
                         WHERE lbl.node_id = ({value}) ORDER BY lbl.rowid))",
                        quote_identifier(&table)
                    ));
                }
                ("__cypher_label", [value]) => {
                    let table = catalog
                        .labels_table()
                        .ok_or(LowerError::UnsupportedOperator(
                            "label() without a label table",
                        ))?;
                    return Ok(format!(
                        "(SELECT lbl.label FROM {} AS lbl WHERE lbl.node_id = ({value}) \
                         ORDER BY lbl.rowid LIMIT 1)",
                        quote_identifier(&table)
                    ));
                }
                ("__cypher_has_label", [value, name]) => {
                    // No junction table means the graph does not track
                    // labels; stay permissive as scans do.
                    let Some(table) = catalog.labels_table() else {
                        return Ok("TRUE".to_owned());
                    };
                    return Ok(format!(
                        "EXISTS (SELECT 1 FROM {} AS lbl WHERE lbl.node_id = ({value}) \
                         AND lbl.label = ({name}))",
                        quote_identifier(&table)
                    ));
                }
                ("__cypher_keys", [value]) => {
                    return Ok(format!(
                        "(SELECT json_group_array(k.key) FROM json_each(({value})) AS k)"
                    ));
                }
                ("__cypher_rand", []) => {
                    return Ok("(0.5 + CAST(random() AS REAL) / 18446744073709551616.0)".to_owned());
                }
                ("__cypher_isempty", [value]) => {
                    return Ok(format!(
                        "(CASE WHEN ({value}) IS NULL THEN NULL \
                         WHEN json_valid(({value})) THEN \
                         (CASE WHEN json_type(({value})) = 'array' \
                         THEN json_array_length(({value})) = 0 \
                         ELSE length(({value})) = 0 END) \
                         ELSE length(({value})) = 0 END)"
                    ));
                }
                ("__cypher_list_real", [value]) => {
                    return Ok(format!(
                        "(SELECT json_group_array(CAST(value AS REAL)) \
                         FROM json_each(({value})))"
                    ));
                }
                ("__cypher_list_integer", [value]) => {
                    return Ok(format!(
                        "(SELECT json_group_array(CAST(value AS INTEGER)) \
                         FROM json_each(({value})))"
                    ));
                }
                ("__cypher_list_text", [value]) => {
                    return Ok(format!(
                        "(SELECT json_group_array(CAST(value AS TEXT)) \
                         FROM json_each(({value})))"
                    ));
                }
                ("__cypher_list_boolean", [value]) => {
                    return Ok(format!(
                        "(SELECT json_group_array(CASE \
                         WHEN lower(CAST(value AS TEXT)) = 'true' THEN 1 \
                         WHEN lower(CAST(value AS TEXT)) = 'false' THEN 0 \
                         WHEN typeof(value) IN ('integer', 'real') THEN value != 0 \
                         ELSE NULL END) FROM json_each(({value})))"
                    ));
                }
                _ => {}
            }
            Ok(format!("{}({})", function.as_str(), arguments.join(", ")))
        }
        ir::Expression::List(values) => {
            let values = values
                .iter()
                .map(|value| {
                    lower_expression_with_references(
                        value,
                        bindings,
                        catalog,
                        input_alias,
                        references,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            Ok(format!("json_array({values})"))
        }
        ir::Expression::Map(entries) => match &expression.value_type {
            ir::ValueType::Struct(fields) => {
                let mut ordered = Vec::with_capacity(fields.len());
                for (field_name, _) in fields {
                    let (_, value) = entries
                        .iter()
                        .find(|(name, _)| name == field_name)
                        .ok_or_else(|| LowerError::InvalidName(field_name.clone()))?;
                    ordered.push(lower_expression_with_references(
                        value,
                        bindings,
                        catalog,
                        input_alias,
                        references,
                    )?);
                }
                Ok(format!("struct_pack({})", ordered.join(", ")))
            }
            ir::ValueType::Union(_) => {
                // Invariant: bind_map_property (graph/frontend/src/binder.rs)
                // only ever constructs a Union-typed Map with exactly one entry.
                let (tag, value) = &entries[0];
                let value_sql = lower_expression_with_references(
                    value,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                Ok(format!(
                    "union_value('{}', {value_sql})",
                    tag.replace('\'', "''")
                ))
            }
            _ => {
                // General maps lower to JSON objects, matching list lowering.
                let mut parts = Vec::with_capacity(entries.len() * 2);
                for (key, value) in entries {
                    parts.push(format!("'{}'", key.replace('\'', "''")));
                    parts.push(lower_expression_with_references(
                        value,
                        bindings,
                        catalog,
                        input_alias,
                        references,
                    )?);
                }
                Ok(format!("json_object({})", parts.join(", ")))
            }
        },
    }
}

fn binding_reference(
    binding: ir::BindingId,
    input_alias: &str,
    references: &HashMap<ir::BindingId, String>,
) -> String {
    references
        .get(&binding)
        .cloned()
        .unwrap_or_else(|| format!("{input_alias}.{}", binding_column(binding)))
}

fn lower_literal(literal: &ir::Literal) -> String {
    match literal {
        ir::Literal::Null => "NULL".to_owned(),
        ir::Literal::Boolean(true) => "TRUE".to_owned(),
        ir::Literal::Boolean(false) => "FALSE".to_owned(),
        ir::Literal::Integer(value) => value.to_string(),
        // Debug formatting keeps a decimal point or exponent, so the SQL
        // literal stays REAL (Display renders 1.0 as "1").
        ir::Literal::Real(value) => format!("{value:?}"),
        ir::Literal::Text(value) => format!("'{}'", value.replace('\'', "''")),
        ir::Literal::Bytes(value) => {
            let hex = value
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            format!("X'{hex}'")
        }
    }
}

/// True when an ordering comparison needs the runtime typeof guard: the
/// operands are not statically known to be the same comparable class.
fn comparison_needs_type_guard(left: &ir::ValueType, right: &ir::ValueType) -> bool {
    fn class(value_type: &ir::ValueType) -> Option<u8> {
        match value_type {
            ir::ValueType::Integer | ir::ValueType::Real => Some(0),
            ir::ValueType::Text => Some(1),
            ir::ValueType::Custom { base, .. } => class(base),
            _ => None,
        }
    }
    !matches!((class(left), class(right)), (Some(a), Some(b)) if a == b)
}

fn binary_operator(operator: ir::BinaryOp) -> &'static str {
    match operator {
        ir::BinaryOp::Equal => "=",
        ir::BinaryOp::NotEqual => "!=",
        ir::BinaryOp::Less => "<",
        ir::BinaryOp::LessOrEqual => "<=",
        ir::BinaryOp::Greater => ">",
        ir::BinaryOp::GreaterOrEqual => ">=",
        ir::BinaryOp::Add => "+",
        ir::BinaryOp::Subtract => "-",
        ir::BinaryOp::Multiply => "*",
        ir::BinaryOp::Divide => "/",
        ir::BinaryOp::Modulo => "%",
        ir::BinaryOp::And => "AND",
        ir::BinaryOp::Or => "OR",
        ir::BinaryOp::In
        | ir::BinaryOp::Xor
        | ir::BinaryOp::Power
        | ir::BinaryOp::StartsWith
        | ir::BinaryOp::EndsWith
        | ir::BinaryOp::Contains => {
            unreachable!("operator lowers through a dedicated SQL form")
        }
    }
}

fn binding_column(binding: ir::BindingId) -> String {
    format!("b{}", binding.get())
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn validate_bare_name(name: &str) -> Result<(), LowerError> {
    if !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        Ok(())
    } else {
        Err(LowerError::InvalidName(name.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Catalog;

    impl RelationalCatalogSnapshot for Catalog {
        fn node_layout(&self, _source: ir::SourceTableId) -> Option<NodeTableLayout> {
            Some(NodeTableLayout {
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            })
        }

        fn relationship_layout(
            &self,
            _source: ir::SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            None
        }

        fn property_column(
            &self,
            _source: ir::SourceTableId,
            _property: ir::PropertyId,
        ) -> Option<String> {
            Some("address".to_owned())
        }
    }

    #[test]
    fn lowers_nested_property_field_chain_inside_correlated_subquery() {
        let catalog = Catalog;
        let source = ir::SourceTableId::new(1).unwrap();
        let entity = ir::BindingId::new(1).unwrap();
        let mut bindings = HashMap::new();
        bindings.insert(
            entity,
            BindingLayout {
                source,
                kind: EntityKind::Node,
                properties: Default::default(),
            },
        );
        let expression = ir::TypedExpression {
            expression: ir::Expression::Property {
                entity,
                property: ir::PropertyId::new(1).unwrap(),
                fields: vec!["city".to_owned()],
            },
            value_type: ir::ValueType::Text,
            nullability: ir::Nullability::Nullable,
        };
        let sql = lower_expression(&expression, &bindings, &catalog, "n")
            .expect("property lowering should succeed");
        assert_eq!(
            sql,
            "(SELECT p.\"address\".\"city\" FROM \"people\" AS p WHERE p.\"id\" = n.b1)"
        );
    }

    #[test]
    fn pushes_wanted_properties_into_scan_columns() {
        let catalog = Catalog;
        let source = ir::SourceTableId::new(1).unwrap();
        let binding_id = ir::BindingId::new(1).unwrap();
        let binding = ir::Binding::new(
            binding_id,
            "n",
            ir::ValueType::Node,
            ir::Nullability::NonNull,
        )
        .unwrap();
        let scope = ir::Scope::new(vec![binding]).unwrap();
        let scan = ir::Plan::new(
            ir::PlanKind::NodeScan(ir::NodeScan {
                graph: ir::GraphId::new(1).unwrap(),
                source,
                binding: binding_id,
                labels: vec![],
            }),
            scope.clone(),
            ir::ResultShape::default(),
        )
        .unwrap();
        let property = ir::TypedExpression {
            expression: ir::Expression::Property {
                entity: binding_id,
                property: ir::PropertyId::new(1).unwrap(),
                fields: vec![],
            },
            value_type: ir::ValueType::Text,
            nullability: ir::Nullability::Nullable,
        };
        let filtered = ir::Plan::new(
            ir::PlanKind::Filter(ir::Filter {
                input: Box::new(scan),
                predicate: ir::TypedExpression {
                    expression: ir::Expression::Binary {
                        left: Box::new(property),
                        op: ir::BinaryOp::Equal,
                        right: Box::new(ir::TypedExpression {
                            expression: ir::Expression::Literal(ir::Literal::Text("x".to_owned())),
                            value_type: ir::ValueType::Text,
                            nullability: ir::Nullability::NonNull,
                        }),
                    },
                    value_type: ir::ValueType::Boolean,
                    nullability: ir::Nullability::NonNull,
                },
            }),
            scope,
            ir::ResultShape::default(),
        )
        .unwrap();
        let mut wanted = WantedProperties::new();
        collect_wanted(&filtered, &mut wanted);
        let lowered = lower_plan(&filtered, &catalog, false, &wanted).unwrap();
        // The scan materializes the property once and the filter references
        // the column instead of a correlated subquery.
        assert!(lowered.sql.contains("AS b1_p1"), "{}", lowered.sql);
        assert!(
            lowered.sql.contains("WHERE (q.b1_p1) = ('x')"),
            "{}",
            lowered.sql
        );
        assert!(!lowered.sql.contains("(SELECT p."), "{}", lowered.sql);
    }

    /// Cypher lists lower to JSON arrays, so `IN` membership must probe the
    /// array's elements through json_each; a direct SQL `IN (json_array(...))`
    /// would compare against the single JSON text value instead.
    #[test]
    fn lowers_in_membership_as_json_each_subquery() {
        let catalog = Catalog;
        let bindings = HashMap::new();
        let integer = |value| ir::TypedExpression {
            expression: ir::Expression::Literal(ir::Literal::Integer(value)),
            value_type: ir::ValueType::Integer,
            nullability: ir::Nullability::NonNull,
        };
        let expression = ir::TypedExpression {
            expression: ir::Expression::Binary {
                left: Box::new(integer(1)),
                op: ir::BinaryOp::In,
                right: Box::new(ir::TypedExpression {
                    expression: ir::Expression::List(vec![integer(1), integer(2)]),
                    value_type: ir::ValueType::List(Box::new(ir::ValueType::Integer)),
                    nullability: ir::Nullability::NonNull,
                }),
            },
            value_type: ir::ValueType::Boolean,
            nullability: ir::Nullability::NonNull,
        };
        let sql = lower_expression(&expression, &bindings, &catalog, "n")
            .expect("IN lowering should succeed");
        assert!(sql.starts_with("(CASE WHEN (json_array(1, 2)) IS NULL"));
        assert!(sql.contains("typeof(e.value)"));
    }

    #[test]
    fn lowers_two_level_nested_property_field_chain_without_alias_prefix() {
        let catalog = Catalog;
        let source = ir::SourceTableId::new(1).unwrap();
        let entity = ir::BindingId::new(1).unwrap();
        let mut bindings = HashMap::new();
        bindings.insert(
            entity,
            BindingLayout {
                source,
                kind: EntityKind::Node,
                properties: Default::default(),
            },
        );
        let expression = ir::TypedExpression {
            expression: ir::Expression::Property {
                entity,
                property: ir::PropertyId::new(1).unwrap(),
                fields: vec!["address".to_owned(), "city".to_owned()],
            },
            value_type: ir::ValueType::Text,
            nullability: ir::Nullability::Nullable,
        };
        let sql = lower_expression(&expression, &bindings, &catalog, "n")
            .expect("property lowering should succeed");
        // 2-nested-field access must NOT prefix the SELECT-list expression
        // with the correlated subquery's `p.` alias: core's parser has no
        // 4-identifier dot-chain AST node, so `p.col.field1.field2` is a
        // syntax error. The bare 3-identifier form `col.field1.field2` hits
        // core's unqualified-column nested-field-access fallback instead.
        // The WHERE clause's identity correlation keeps using `p.` as before.
        assert_eq!(
            sql,
            "(SELECT \"address\".\"address\".\"city\" FROM \"people\" AS p WHERE p.\"id\" = n.b1)"
        );
    }
}
