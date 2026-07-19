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
    /// Junction table recording each node's labels, when the graph has one.
    fn labels_table(&self) -> Option<String> {
        None
    }
    /// Human-readable name of a label identity, when known.
    fn label_name(&self, _label: ir::LabelId) -> Option<String> {
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

#[derive(Clone, Copy)]
struct BindingLayout {
    source: ir::SourceTableId,
    kind: EntityKind,
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
    let lowered = lower_plan(plan, catalog, false)?;
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
    let lowered = lower_plan(plan, catalog, false)?;
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
    let (command, _) = turso_core::dialect::sqlite::parse(&sql)?;
    match command {
        Some(ast::Cmd::Stmt(statement)) => Ok(statement),
        _ => Err(LowerError::EmptyGeneratedSql),
    }
}

fn lower_plan(
    plan: &ir::Plan,
    catalog: &dyn RelationalCatalogSnapshot,
    optional: bool,
) -> Result<Lowered, LowerError> {
    match plan.kind() {
        ir::PlanKind::Unit(_) => Ok(Lowered {
            sql: "SELECT 1 AS __unit".to_owned(),
            bindings: HashMap::new(),
        }),
        ir::PlanKind::NodeScan(scan) => lower_node_scan(scan, catalog),
        ir::PlanKind::FixedExpand(expand) => lower_fixed_expand(expand, catalog, optional, &[]),
        ir::PlanKind::GraphExpand(expand) => lower_graph_expand(expand, catalog),
        ir::PlanKind::Filter(filter) => {
            let input = lower_plan(&filter.input, catalog, optional)?;
            let predicate = lower_expression(&filter.predicate, &input.bindings, catalog, "q")?;
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q WHERE {predicate}", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Project(project) => lower_project(project, catalog, optional),
        ir::PlanKind::Distinct(distinct) => {
            let input = lower_plan(&distinct.input, catalog, optional)?;
            Ok(Lowered {
                sql: format!("SELECT DISTINCT q.* FROM ({}) AS q", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::LeftApply(apply) => lower_optional_chain(&apply.right, catalog),
        ir::PlanKind::Aggregate(aggregate) => {
            let input = lower_plan(&aggregate.input, catalog, optional)?;
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
                        bindings.insert(grouping.output.id(), *layout);
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
            let input = lower_plan(&sort.input, catalog, optional)?;
            let ordering = lower_ordering(&sort.keys, &input.bindings, catalog)?;
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q ORDER BY {ordering}", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Skip(skip) => {
            let input = lower_plan(&skip.input, catalog, optional)?;
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
            let input = lower_plan(&limit.input, catalog, optional)?;
            let count = lower_expression(&limit.count, &input.bindings, catalog, "q")?;
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q LIMIT {count}", input.sql),
                bindings: input.bindings,
            })
        }
        ir::PlanKind::Unwind(unwind) => {
            let input = lower_plan(&unwind.input, catalog, optional)?;
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
            let left = lower_plan(&join.left, catalog, optional)?;
            let right = lower_plan(&join.right, catalog, optional)?;
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
            for input in union.inputs() {
                let lowered = lower_plan(input, catalog, optional)?;
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
) -> Result<Lowered, LowerError> {
    let input = lower_plan(&expand.input, catalog, false)?;
    let from = input
        .bindings
        .get(&expand.from)
        .copied()
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
    let mut bindings = input.bindings;
    bindings.insert(
        expand.relationship.id(),
        BindingLayout {
            source: expand.relationship_source,
            kind: EntityKind::Relationship,
        },
    );
    bindings.insert(
        expand.to.id(),
        BindingLayout {
            source: expand.target_node_source,
            kind: EntityKind::Node,
        },
    );
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

fn lower_optional_chain(
    plan: &ir::Plan,
    catalog: &dyn RelationalCatalogSnapshot,
) -> Result<Lowered, LowerError> {
    let original = plan;
    let mut current = plan;
    let mut predicates = Vec::new();
    while let ir::PlanKind::Filter(filter) = current.kind() {
        predicates.push(&filter.predicate);
        current = &filter.input;
    }
    match current.kind() {
        ir::PlanKind::FixedExpand(expand) => lower_fixed_expand(expand, catalog, true, &predicates),
        _ => lower_plan(original, catalog, false),
    }
}

fn lower_project(
    project: &ir::Project,
    catalog: &dyn RelationalCatalogSnapshot,
    optional: bool,
) -> Result<Lowered, LowerError> {
    let (input, sort_keys) = match project.input.kind() {
        ir::PlanKind::Sort(sort) => (
            lower_plan(&sort.input, catalog, optional)?,
            Some(sort.keys.as_slice()),
        ),
        _ => (lower_plan(&project.input, catalog, optional)?, None),
    };
    let mut bindings = HashMap::new();
    let columns = project
        .projections
        .iter()
        .map(|projection| {
            if let ir::Expression::Binding(input_binding) = projection.expression.expression {
                if let Some(layout) = input.bindings.get(&input_binding) {
                    bindings.insert(projection.output.id(), *layout);
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
) -> Result<Lowered, LowerError> {
    let layout = catalog
        .node_layout(scan.source)
        .ok_or(LowerError::MissingSource(scan.source))?;
    let mut bindings = HashMap::new();
    bindings.insert(
        scan.binding,
        BindingLayout {
            source: scan.source,
            kind: EntityKind::Node,
        },
    );
    let mut sql = format!(
        "SELECT n.{} AS {} FROM {} AS n",
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
) -> Result<Lowered, LowerError> {
    let input = if optional {
        lower_optional_chain(&expand.input, catalog)?
    } else {
        lower_plan(&expand.input, catalog, false)?
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
    let (relationship_on, mut node_on) = match expand.direction {
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
    };
    let mut bindings = input.bindings;
    bindings.insert(
        expand.relationship.id(),
        BindingLayout {
            source: expand.relationship_source,
            kind: EntityKind::Relationship,
        },
    );
    bindings.insert(
        expand.to.id(),
        BindingLayout {
            source: expand.target_node_source,
            kind: EntityKind::Node,
        },
    );
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
                format!(
                    "{target_alias}.{}",
                    quote_identifier(&target.identity_column)
                ),
            ),
        ]);
        let predicates = join_predicates
            .iter()
            .map(|predicate| {
                lower_expression_with_references(predicate, &bindings, catalog, "q", &references)
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(" AND ");
        node_on = format!("({node_on}) AND ({predicates})");
    }
    let relationship_identity = if optional {
        format!(
            "CASE WHEN {target_alias}.{} IS NULL THEN NULL ELSE {relationship_alias}.{} END",
            quote_identifier(&target.identity_column),
            quote_identifier(&relationship.identity_column)
        )
    } else {
        format!(
            "{relationship_alias}.{}",
            quote_identifier(&relationship.identity_column)
        )
    };
    Ok(Lowered {
        sql: format!(
            "SELECT q.*, {relationship_identity} AS {}, {target_alias}.{} AS {} \
             FROM ({}) AS q {join} {} AS {relationship_alias} ON {relationship_on} \
             {join} {} AS {target_alias} ON {node_on}",
            binding_column(expand.relationship.id()),
            quote_identifier(&target.identity_column),
            binding_column(expand.to.id()),
            input.sql,
            quote_identifier(&relationship.table),
            quote_identifier(&target.table),
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
                // Cypher lists lower to JSON arrays; membership must probe
                // the array's elements, not compare against the JSON text.
                ir::BinaryOp::In => {
                    format!("({left}) IN (SELECT value FROM json_each({right}))")
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
        ir::Expression::PatternSubquery {
            count,
            plan,
            correlations,
        } => {
            let sub = lower_plan(plan, catalog, false)?;
            let conditions = correlations
                .iter()
                .map(|(outer, inner)| {
                    format!(
                        "sub.{} = {input_alias}.{}",
                        binding_column(*inner),
                        binding_column(*outer)
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
            let index = lower_expression_with_references(
                index,
                bindings,
                catalog,
                input_alias,
                references,
            )?;
            Ok(if text_key {
                format!("json_extract(({base}), '$.\"' || ({index}) || '\"')")
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
                    return Ok(format!(
                        "(CASE WHEN json_valid(({value})) AND json_type(({value})) = 'array' \
                         THEN json_array_length(({value})) ELSE length(({value})) END)"
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
                         WHEN json_valid(({value})) AND json_type(({value})) = 'array' \
                         THEN json_array_length(({value})) = 0 \
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
        ir::Literal::Real(value) => value.to_string(),
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
        assert_eq!(
            sql,
            "(1) IN (SELECT value FROM json_each(json_array(1, 2)))"
        );
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
