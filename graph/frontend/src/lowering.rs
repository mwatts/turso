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
        ir::PlanKind::Aggregate(_) => Err(LowerError::UnsupportedOperator("aggregate")),
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
        ir::PlanKind::Union(_) => Err(LowerError::UnsupportedOperator("union")),
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
    Ok(Lowered {
        sql: format!(
            "SELECT n.{} AS {} FROM {} AS n",
            quote_identifier(&layout.identity_column),
            binding_column(scan.binding),
            quote_identifier(&layout.table)
        ),
        bindings,
    })
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
                _ => format!("({left}) {} ({right})", binary_operator(*op)),
            })
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
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            Ok(format!("{}({arguments})", function.as_str()))
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
            _ => Err(LowerError::UnsupportedOperator(
                "map literal outside a struct or union property",
            )),
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
        ir::BinaryOp::And => "AND",
        ir::BinaryOp::Or => "OR",
        ir::BinaryOp::In => unreachable!("IN lowers to a json_each membership subquery"),
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
