use std::collections::{BTreeSet, HashMap};

use thiserror::Error;
use turso_graph_ir as ir;
use turso_parser::ast;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeTableLayout {
    pub table: String,
    pub identity_column: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipRoleLayout {
    pub role: ir::RoleId,
    pub name: String,
    /// Endpoint column on the relation table. Empty for `Many` roles.
    pub column: String,
    pub cardinality: ir::RoleCardinality,
    /// Set for `Many` roles: `<table>__<role>(relation_id, node_id)`.
    pub spill_table: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipTableLayout {
    pub table: String,
    pub identity_column: String,
    /// Declaration order. A two-role relation is `[start, end]`.
    pub roles: Vec<RelationshipRoleLayout>,
}

impl RelationshipTableLayout {
    pub fn role(&self, role: ir::RoleId) -> Option<&RelationshipRoleLayout> {
        self.roles.iter().find(|entry| entry.role == role)
    }

    fn role_by_name(&self, name: &str) -> Option<&RelationshipRoleLayout> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }

    /// The `start` role of a two-role pattern-hop relationship, resolved by
    /// name rather than declaration order: role declaration order is not
    /// guaranteed to put `start` before `end`.
    pub fn start_role(&self) -> Option<&RelationshipRoleLayout> {
        self.role_by_name("start")
    }

    /// The `end` role of a two-role pattern-hop relationship, resolved by
    /// name rather than declaration order.
    pub fn end_role(&self) -> Option<&RelationshipRoleLayout> {
        self.role_by_name("end")
    }

    /// Columns that carry participation rather than payload.
    pub fn structural_columns(&self) -> Vec<String> {
        let mut columns = vec![self.identity_column.clone()];
        columns.extend(
            self.roles
                .iter()
                .filter(|role| role.cardinality == ir::RoleCardinality::One)
                .map(|role| role.column.clone()),
        );
        columns
    }
}

/// Physical relational names resolved from stable graph catalog identities.
pub trait RelationalCatalogSnapshot {
    fn registered_node_sources(&self) -> Vec<ir::SourceTableId> {
        Vec::new()
    }
    fn registered_relationship_sources(&self) -> Vec<ir::SourceTableId> {
        Vec::new()
    }
    fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout>;
    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout>;
    /// Whether label/type junction rows include `source_id`.
    fn source_qualified_membership(&self) -> bool {
        false
    }
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
    /// Semantic catalog labels when strict schema mode is active. Legacy
    /// catalogs return `None` and enumerate the data-backed junction.
    fn procedure_labels(&self) -> Option<Vec<String>> {
        None
    }
    /// Semantic catalog relationship types when strict schema mode is active.
    fn procedure_relationship_types(&self) -> Option<Vec<String>> {
        None
    }
    /// Payload properties of a source as (cypher name, physical column)
    /// pairs — every column except identity/endpoint columns. Enables
    /// whole-entity property reads (`properties(n)`).
    fn payload_columns(&self, _source: ir::SourceTableId) -> Option<Vec<(String, String)>> {
        None
    }
    /// Logical property names declared for catalog introspection. This is a
    /// union across semantic owners, unlike `semantic_properties`, whose
    /// intersection semantics protect a specific polymorphic binding.
    fn procedure_property_keys(&self, source: ir::SourceTableId) -> Option<Vec<String>> {
        self.payload_columns(source)
            .map(|columns| columns.into_iter().map(|(logical, _)| logical).collect())
    }

    fn semantic_property_for_key(
        &self,
        _source: ir::SourceTableId,
        _type_names: &[String],
        _key: &str,
    ) -> Option<Option<(String, ir::ValueType, String)>> {
        None
    }

    fn semantic_property_for_id(
        &self,
        _source: ir::SourceTableId,
        _type_names: &[String],
        _property: ir::PropertyId,
    ) -> Option<Option<(String, ir::ValueType, String)>> {
        None
    }

    fn semantic_properties(
        &self,
        _source: ir::SourceTableId,
        _type_names: &[String],
    ) -> Option<Vec<(ir::PropertyId, String, ir::ValueType, String)>> {
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
    #[error("relation {relation} has no role {role:?}")]
    UnknownRole { relation: String, role: ir::RoleId },
}

#[derive(Clone, Copy)]
enum EntityKind {
    Node,
    Relationship,
}

#[derive(Clone)]
struct BindingLayout {
    source: ir::SourceTableId,
    sources: std::collections::BTreeSet<ir::SourceTableId>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MutationEntityKind {
    Node,
    Relationship,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MutationRowColumn {
    Value(ir::BindingId),
    Source(ir::BindingId, MutationEntityKind),
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
                sources: [*source].into_iter().collect(),
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

pub(crate) fn mutation_rows_with_sources_sql(
    input: &LoweredMutationInput,
    bindings: &[ir::BindingId],
) -> (String, Vec<MutationRowColumn>) {
    if bindings.is_empty() {
        return (format!("SELECT 1 FROM ({}) AS q", input.sql), Vec::new());
    }
    let mut columns = Vec::new();
    let mut row_columns = Vec::new();
    for binding in bindings {
        columns.push(format!("q.{}", binding_column(*binding)));
        row_columns.push(MutationRowColumn::Value(*binding));
        let Some(layout) = input.bindings.get(binding) else {
            continue;
        };
        let source = if layout.sources.len() == 1 {
            layout.source.get().to_string()
        } else {
            format!("q.{}", source_column_ref(*binding))
        };
        columns.push(source);
        row_columns.push(MutationRowColumn::Source(
            *binding,
            match layout.kind {
                EntityKind::Node => MutationEntityKind::Node,
                EntityKind::Relationship => MutationEntityKind::Relationship,
            },
        ));
    }
    (
        format!("SELECT {} FROM ({}) AS q", columns.join(", "), input.sql),
        row_columns,
    )
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
type WantedProperties =
    HashMap<ir::BindingId, std::collections::BTreeMap<u32, Option<Vec<String>>>>;

fn property_column_ref(binding: ir::BindingId, property: ir::PropertyId) -> String {
    format!("{}_p{}", binding_column(binding), property.get())
}

fn source_column_ref(binding: ir::BindingId) -> String {
    format!("{}_source", binding_column(binding))
}

fn resolved_property_column(
    catalog: &dyn RelationalCatalogSnapshot,
    source: ir::SourceTableId,
    semantic_types: &[String],
    property: ir::PropertyId,
) -> Option<String> {
    match catalog.semantic_property_for_id(source, semantic_types, property) {
        Some(Some((_, _, column))) => Some(column),
        Some(None) => None,
        None => catalog.property_column(source, property),
    }
}

#[derive(Clone, Copy, Default)]
struct MaterializationContext<'a> {
    null_probe: Option<&'a str>,
    semantic_types: Option<&'a [String]>,
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
    context: MaterializationContext<'_>,
) -> String {
    let Some(properties) = wanted.get(&binding) else {
        return String::new();
    };
    let mut columns = String::new();
    for (property, wanted_semantic_types) in properties {
        let semantic_types = context.semantic_types.or(wanted_semantic_types.as_deref());
        let Some(semantic_types) = semantic_types else {
            continue;
        };
        let Some(id) = ir::PropertyId::new(*property).ok() else {
            continue;
        };
        let Some(column) = resolved_property_column(catalog, source, semantic_types, id) else {
            if matches!(
                catalog.semantic_property_for_id(source, semantic_types, id),
                Some(None)
            ) {
                // A label predicate can narrow a polymorphic binding after
                // its Union scan is built. Sources excluded by that
                // predicate still need a positional placeholder so every
                // SQL set-operation branch retains the same shape.
                columns.push_str(&format!(", NULL AS {}", property_column_ref(binding, id)));
                layout.properties.insert(*property);
            }
            continue;
        };
        let value = format!("{alias}.{}", quote_identifier(&column));
        let value = if catalog.property_column_is_jsonb(source, id) {
            format!("json({value})")
        } else {
            value
        };
        let value = context.null_probe.map_or(value.clone(), |null_probe| {
            format!("CASE WHEN {null_probe} IS NULL THEN NULL ELSE {value} END")
        });
        columns.push_str(&format!(
            ", {value} AS {}",
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
        ir::PlanKind::RoleExpand(expand) => collect_wanted(&expand.input, wanted),
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
        ir::PlanKind::ProcedureCall(call) => {
            expressions.extend(call.arguments.iter());
            collect_wanted(&call.input, wanted);
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
        ir::PlanKind::RelationScan(_) => {}
        ir::PlanKind::RoleJoin(join) => collect_wanted(&join.input, wanted),
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
                semantic_types,
                fields,
            } if fields.is_empty() => {
                let properties = wanted.entry(*entity).or_default();
                match properties.entry(property.get()) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(Some(semantic_types.clone()));
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry)
                        if entry.get().as_ref() != Some(semantic_types) =>
                    {
                        entry.insert(None);
                    }
                    std::collections::btree_map::Entry::Occupied(_) => {}
                }
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
        ir::PlanKind::RoleExpand(expand) => {
            lower_role_expand(expand, catalog, optional, &[], wanted, None)
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
            // Column-free count(*) over a bare node scan can skip materializing
            // node rows and ride complete B-tree indexes core now treats as
            // covering (table PK / secondary indexes) or the label-first
            // junction index for labeled membership counts.
            if let Some(lowered) = try_lower_column_free_node_count(aggregate, catalog)? {
                return Ok(lowered);
            }
            let input = lower_plan(&aggregate.input, catalog, optional, wanted)?;
            let mut selects = Vec::new();
            let mut groups = Vec::new();
            let mut bindings = HashMap::new();
            let mut columns = Vec::new();
            let mut collect_columns = Vec::new();
            for grouping in &aggregate.groupings {
                let sql = lower_expression(&grouping.expression, &input.bindings, catalog, "q")?;
                let column = binding_column(grouping.output.id());
                selects.push(format!("({sql}) AS {column}"));
                columns.push(column);
                groups.push(format!("({sql})"));
                // Entity groupings keep their relational layout addressable
                // for later property access.
                if let ir::Expression::Binding(source_binding) = &grouping.expression.expression {
                    if let Some(layout) = input.bindings.get(source_binding) {
                        let mut layout = layout.clone();
                        layout.properties.clear();
                        let multiple_sources = layout.sources.len() > 1;
                        bindings.insert(grouping.output.id(), layout);
                        if multiple_sources {
                            let source_sql = format!("q.{}", source_column_ref(*source_binding));
                            let source_column = source_column_ref(grouping.output.id());
                            selects.push(format!("{source_sql} AS {source_column}"));
                            columns.push(source_column);
                            groups.push(source_sql);
                        }
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
                        collect_columns.push(binding_column(aggregation.output.id()));
                        format!("json_group_array({distinct}({argument}))")
                    }
                    _ => {
                        return Err(LowerError::UnsupportedOperator(
                            "aggregate call without an argument",
                        ));
                    }
                };
                let column = binding_column(aggregation.output.id());
                selects.push(format!("{call} AS {column}"));
                columns.push(column);
            }
            let group_by = if groups.is_empty() {
                String::new()
            } else {
                format!(" GROUP BY {}", groups.join(", "))
            };
            let sql = format!(
                "SELECT {} FROM ({}) AS q{group_by}",
                selects.join(", "),
                input.sql
            );
            // Cypher's collect() ignores null inputs, unlike json_group_array
            // (which records them as JSON null); round-trip the affected
            // columns through json_each to drop them after aggregating.
            let sql = if collect_columns.is_empty() {
                sql
            } else {
                let projection = columns
                    .iter()
                    .map(|column| {
                        if collect_columns.contains(column) {
                            format!(
                                "(SELECT json_group_array(collected.value) \
                                 FROM json_each(agg.{column}) AS collected \
                                 WHERE collected.value IS NOT NULL) AS {column}"
                            )
                        } else {
                            format!("agg.{column} AS {column}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("SELECT {projection} FROM ({sql}) AS agg")
            };
            Ok(Lowered { sql, bindings })
        }
        ir::PlanKind::Sort(sort) => {
            let input = lower_plan(&sort.input, catalog, optional, wanted)?;
            let ordering = lower_ordering(&sort.keys, &input.bindings, catalog)?
                .map(|ordering| format!(" ORDER BY {ordering}"))
                .unwrap_or_default();
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q{ordering}", input.sql),
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
        ir::PlanKind::ProcedureCall(call) => lower_procedure_call(call, catalog, optional, wanted),
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
            let mut bindings: Option<HashMap<ir::BindingId, BindingLayout>> = None;
            for input in union.inputs() {
                let lowered = lower_plan(input, catalog, optional, wanted)?;
                // Branch column names differ (per-branch binding ids); SQL
                // set operators combine positionally and the first branch's
                // names win, matching this Union node's scope.
                parts.push(format!("SELECT q.* FROM ({}) AS q", lowered.sql));
                if let Some(combined) = &mut bindings {
                    for (binding, layout) in lowered.bindings {
                        if let Some(existing) = combined.get_mut(&binding) {
                            existing.sources.extend(layout.sources);
                            existing
                                .properties
                                .retain(|property| layout.properties.contains(property));
                        }
                    }
                } else {
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
        ir::PlanKind::RelationScan(scan) => lower_relation_scan(scan, catalog, wanted),
        ir::PlanKind::RoleJoin(join) => lower_role_join(join, catalog, wanted),
    }
}

fn lower_procedure_call(
    call: &ir::ProcedureCall,
    catalog: &dyn RelationalCatalogSnapshot,
    optional: bool,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    if !call.arguments.is_empty() {
        return Err(LowerError::UnsupportedOperator(
            "arguments for built-in catalog procedures",
        ));
    }
    let input = lower_plan(&call.input, catalog, optional, wanted)?;
    let rows = match call.procedure {
        ir::ProcedureIdentity::DbLabels => {
            if let Some(labels) = catalog.procedure_labels() {
                text_procedure_rows(labels)
            } else {
                let table = catalog
                    .labels_table()
                    .ok_or(LowerError::UnsupportedOperator(
                        "db.labels without a label table",
                    ))?;
                format!(
                    "SELECT DISTINCT label AS c0 FROM {} ORDER BY label",
                    quote_identifier(&table)
                )
            }
        }
        ir::ProcedureIdentity::DbRelationshipTypes => {
            if let Some(types) = catalog.procedure_relationship_types() {
                text_procedure_rows(types)
            } else {
                let table =
                    catalog
                        .relationship_types_table()
                        .ok_or(LowerError::UnsupportedOperator(
                            "db.relationshipTypes without a type table",
                        ))?;
                format!(
                    "SELECT DISTINCT type AS c0 FROM {} ORDER BY type",
                    quote_identifier(&table)
                )
            }
        }
        ir::ProcedureIdentity::DbPropertyKeys => {
            let mut keys = BTreeSet::new();
            for source in catalog
                .registered_node_sources()
                .into_iter()
                .chain(catalog.registered_relationship_sources())
            {
                keys.extend(
                    catalog
                        .procedure_property_keys(source)
                        .ok_or(LowerError::MissingSource(source))?,
                );
            }
            text_procedure_rows(keys)
        }
    };
    let projections = call
        .outputs
        .iter()
        .map(|output| {
            if output.column != 0 {
                return Err(LowerError::UnsupportedOperator(
                    "unknown built-in procedure output",
                ));
            }
            Ok(format!(
                "p.c{} AS {}",
                output.column,
                binding_column(output.output.id())
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if projections.is_empty() {
        return Err(LowerError::UnsupportedOperator(
            "procedure call without selected outputs",
        ));
    }
    Ok(Lowered {
        sql: format!(
            "SELECT q.*, {} FROM ({}) AS q JOIN ({rows}) AS p",
            projections.join(", "),
            input.sql
        ),
        bindings: input.bindings,
    })
}

fn text_procedure_rows(values: impl IntoIterator<Item = String>) -> String {
    let rows = values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|value| format!("SELECT {} AS c0", crate::catalog::sql_string(&value)))
        .collect::<Vec<_>>();
    if rows.is_empty() {
        "SELECT NULL AS c0 WHERE 0".to_owned()
    } else {
        rows.join(" UNION ALL ")
    }
}

fn lower_graph_expand(
    expand: &ir::GraphExpand,
    catalog: &dyn RelationalCatalogSnapshot,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    let input = lower_plan(&expand.input, catalog, false, wanted)?;
    if !input.bindings.contains_key(&expand.from) {
        return Err(LowerError::MissingBinding(expand.from));
    }
    let source_join = input
        .bindings
        .get(&expand.from)
        .filter(|layout| layout.sources.len() > 1)
        .map(|_| {
            format!(
                " ON q.{} = {}",
                source_column_ref(expand.from),
                expand.from_node_source.get()
            )
        })
        .unwrap_or_default();
    let relationship = catalog
        .relationship_layout(expand.relationship_source)
        .ok_or(LowerError::MissingSource(expand.relationship_source))?;
    let target = catalog
        .node_layout(expand.target_node_source)
        .ok_or(LowerError::MissingSource(expand.target_node_source))?;
    // Lowering only names the two roles here -- exactly what the fixed hop
    // already does via `relationship.role(id)` -- and makes no
    // outgoing/incoming/both judgment of its own. The variable-length
    // expand vtab and the traversal runtime it drives are role-pair-keyed
    // (Task 17), so the numeric role ordinals resolved below pass straight
    // through as `from_role`/`to_role` arguments with no direction
    // translation anywhere in this path.
    let from_role = relationship
        .role(expand.from_role)
        .ok_or_else(|| LowerError::UnknownRole {
            relation: relationship.table.clone(),
            role: expand.from_role,
        })?;
    let to_role = relationship
        .role(expand.to_role)
        .ok_or_else(|| LowerError::UnknownRole {
            relation: relationship.table.clone(),
            role: expand.to_role,
        })?;
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
            sources: [expand.relationship_source].into_iter().collect(),
            kind: EntityKind::Relationship,
            properties: Default::default(),
        },
    );
    bindings.insert(
        expand.to.id(),
        BindingLayout {
            source: expand.target_node_source,
            sources: [expand.target_node_source].into_iter().collect(),
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
                "SELECT g.*, r.{} AS {}, {} AS {}, n.{} AS {}, {} AS {} \
                 FROM (SELECT {}{aggregates}, \
                 max(CASE WHEN gx.is_terminal = 1 THEN gx.node_identity END) AS __gx_node, \
                 max(CASE WHEN gx.is_terminal = 1 THEN gx.relationship_identity END) AS __gx_rel \
                 FROM ({}) AS q \
                 JOIN __turso_graph_expand({}, {}, q.{}, {}, {}, {}, '{}', {}, {}, {}, '{}', {}, {}, {}, {}, {}) AS gx{source_join} \
                 GROUP BY {}gx.path_id) AS g \
                 JOIN {} AS n ON n.{} = g.__gx_node \
                 LEFT JOIN {} AS r ON r.{} = g.__gx_rel",
                quote_identifier(&relationship.identity_column),
                binding_column(expand.relationship.id()),
                expand.relationship_source.get(),
                source_column_ref(expand.relationship.id()),
                quote_identifier(&target.identity_column),
                binding_column(expand.to.id()),
                expand.target_node_source.get(),
                source_column_ref(expand.to.id()),
                inner_select,
                input.sql,
                expand.graph.get(),
                expand.from_node_source.get(),
                binding_column(expand.from),
                from_role.role.get(),
                to_role.role.get(),
                u8::from(expand.symmetric),
                relationship_types,
                expand.min_hops,
                expand.max_hops,
                u8::from(expand.unbounded),
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
            "SELECT q.*, r.{} AS {}, {} AS {}, n.{} AS {}, {} AS {} \
             FROM ({}) AS q \
             JOIN __turso_graph_expand({}, {}, q.{}, {}, {}, {}, '{}', {}, {}, {}, '{}', {}, {}, {}, {}, {}) AS gx{source_join} \
             JOIN {} AS n ON gx.is_terminal = 1 AND gx.node_source_id = {} AND n.{} = gx.node_identity \
             LEFT JOIN {} AS r ON gx.relationship_source_id = {} AND r.{} = gx.relationship_identity",
            quote_identifier(&relationship.identity_column),
            binding_column(expand.relationship.id()),
            expand.relationship_source.get(),
            source_column_ref(expand.relationship.id()),
            quote_identifier(&target.identity_column),
            binding_column(expand.to.id()),
            expand.target_node_source.get(),
            source_column_ref(expand.to.id()),
            input.sql,
            expand.graph.get(),
            expand.from_node_source.get(),
            binding_column(expand.from),
            from_role.role.get(),
            to_role.role.get(),
            u8::from(expand.symmetric),
            relationship_types,
            expand.min_hops,
            expand.max_hops,
            u8::from(expand.unbounded),
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
        ir::PlanKind::RoleExpand(expand) => {
            lower_role_expand(expand, catalog, true, &predicates, wanted, boundary)
        }
        ir::PlanKind::Union(union) => {
            let mut parts = Vec::new();
            let mut bindings: Option<HashMap<ir::BindingId, BindingLayout>> = None;
            for branch in union.inputs() {
                let lowered = match branch.kind() {
                    ir::PlanKind::RoleExpand(expand) => {
                        lower_role_expand(expand, catalog, true, &predicates, wanted, boundary)?
                    }
                    _ => lower_optional_chain(branch, boundary, catalog, wanted)?,
                };
                parts.push(format!("SELECT q.* FROM ({}) AS q", lowered.sql));
                if let Some(combined) = &mut bindings {
                    for (binding, layout) in lowered.bindings {
                        if let Some(existing) = combined.get_mut(&binding) {
                            existing.sources.extend(layout.sources);
                            existing
                                .properties
                                .retain(|property| layout.properties.contains(property));
                        }
                    }
                } else {
                    bindings = Some(lowered.bindings);
                }
            }
            Ok(Lowered {
                sql: parts.join(if union.is_all() {
                    " UNION ALL "
                } else {
                    " UNION "
                }),
                bindings: bindings.unwrap_or_default(),
            })
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
            let mut source_projection = None;
            if let ir::Expression::Binding(input_binding) = projection.expression.expression {
                if let Some(layout) = input.bindings.get(&input_binding) {
                    let mut layout = layout.clone();
                    layout.properties.clear();
                    let multiple_sources = layout.sources.len() > 1;
                    bindings.insert(projection.output.id(), layout);
                    if multiple_sources {
                        source_projection = Some(format!(
                            ", q.{} AS {}",
                            source_column_ref(input_binding),
                            source_column_ref(projection.output.id())
                        ));
                    }
                }
            }
            let expression =
                lower_expression(&projection.expression, &input.bindings, catalog, "q")?;
            Ok(format!(
                "{expression} AS {}{}",
                binding_column(projection.output.id()),
                source_projection.unwrap_or_default(),
            ))
        })
        .collect::<Result<Vec<_>, LowerError>>()?
        .join(", ");
    let ordering = sort_keys
        .map(|keys| lower_ordering(keys, &input.bindings, catalog))
        .transpose()?
        .flatten()
        .map(|ordering| format!(" ORDER BY {ordering}"))
        .unwrap_or_default();
    Ok(Lowered {
        sql: format!("SELECT {columns} FROM ({}) AS q{ordering}", input.sql),
        bindings,
    })
}

/// A sort key that is a bare numeric literal orders nothing in Cypher, but SQL
/// reads a small integer in `ORDER BY` as a column position: `ORDER BY 1 DESC`
/// would reverse the result and `ORDER BY 2` over a one-column projection is a
/// "term out of range" error. Every row shares a literal's value, so dropping
/// the key is exactly what Cypher means and avoids the reinterpretation.
fn sort_key_orders_nothing(expression: &ir::Expression) -> bool {
    match expression {
        ir::Expression::Literal(_) => true,
        ir::Expression::Unary { op, expression } => {
            *op == ir::UnaryOp::Negate
                && matches!(expression.expression, ir::Expression::Literal(_))
        }
        _ => false,
    }
}

/// Renders the sort keys, or `None` when no key survives and the caller must
/// emit no `ORDER BY` at all.
fn lower_ordering(
    keys: &[ir::SortKey],
    bindings: &HashMap<ir::BindingId, BindingLayout>,
    catalog: &dyn RelationalCatalogSnapshot,
) -> Result<Option<String>, LowerError> {
    keys.iter()
        .filter(|key| !sort_key_orders_nothing(&key.expression.expression))
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
        .map(|items| (!items.is_empty()).then(|| items.join(", ")))
}

/// When `RETURN count(*)` (or an equivalent pure star-count aggregate) sits
/// directly on a node scan, emit a column-free `count(*)` that core can plan
/// with covering indexes instead of wrapping a full node projection.
fn try_lower_column_free_node_count(
    aggregate: &ir::Aggregate,
    catalog: &dyn RelationalCatalogSnapshot,
) -> Result<Option<Lowered>, LowerError> {
    if !aggregate.groupings.is_empty() || aggregate.aggregations.len() != 1 {
        return Ok(None);
    }
    let aggregation = &aggregate.aggregations[0];
    if aggregation.function != ir::AggregateFunction::Count
        || aggregation.expression.is_some()
        || aggregation.distinct
    {
        return Ok(None);
    }
    let ir::PlanKind::NodeScan(scan) = aggregate.input.kind() else {
        return Ok(None);
    };
    let count_column = binding_column(aggregation.output.id());
    let sql = if scan.labels.is_empty() {
        let layout = catalog
            .node_layout(scan.source)
            .ok_or(LowerError::MissingSource(scan.source))?;
        format!(
            "SELECT count(*) AS {count_column} FROM {}",
            quote_identifier(&layout.table)
        )
    } else {
        let Some(labels_table) = catalog.labels_table() else {
            // Without a junction, labels do not filter the scan; count the
            // full source table the same way unlabeled scans do.
            let layout = catalog
                .node_layout(scan.source)
                .ok_or(LowerError::MissingSource(scan.source))?;
            return Ok(Some(Lowered {
                sql: format!(
                    "SELECT count(*) AS {count_column} FROM {}",
                    quote_identifier(&layout.table)
                ),
                bindings: HashMap::new(),
            }));
        };
        let mut label_names = Vec::with_capacity(scan.labels.len());
        for label in &scan.labels {
            let Some(name) = catalog.label_name(*label) else {
                // Unresolved label identities fall back to the generic path.
                return Ok(None);
            };
            label_names.push(name);
        }
        let labels_table = quote_identifier(&labels_table);
        let mut from = format!("{labels_table} AS lbl0");
        let source_predicate = |alias: &str| {
            if catalog.source_qualified_membership() {
                format!("{alias}.source_id = {} AND ", scan.source.get())
            } else {
                String::new()
            }
        };
        let where_clause = format!(
            "{}lbl0.label = '{}'",
            source_predicate("lbl0"),
            label_names[0].replace('\'', "''")
        );
        for (index, name) in label_names.iter().enumerate().skip(1) {
            let alias = format!("lbl{index}");
            from.push_str(&format!(
                " JOIN {labels_table} AS {alias} ON {alias}.node_id = lbl0.node_id AND {}{alias}.label = '{}'",
                source_predicate(&alias),
                name.replace('\'', "''")
            ));
        }
        // Multi-label intersection still needs a derived row set; single-label
        // membership is a direct indexable count on the junction table.
        if label_names.len() == 1 {
            format!("SELECT count(*) AS {count_column} FROM {from} WHERE {where_clause}")
        } else {
            format!(
                "SELECT count(*) AS {count_column} FROM (SELECT lbl0.node_id FROM {from} WHERE {where_clause}) AS membership"
            )
        }
    };
    Ok(Some(Lowered {
        sql,
        bindings: HashMap::new(),
    }))
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
        sources: [scan.source].into_iter().collect(),
        kind: EntityKind::Node,
        properties: Default::default(),
    };
    let scan_semantic_types = scan
        .labels
        .iter()
        .filter_map(|label| catalog.label_name(*label))
        .collect::<Vec<_>>();
    let extra = materialize_properties(
        wanted,
        scan.binding,
        scan.source,
        "n",
        catalog,
        &mut binding_layout,
        MaterializationContext {
            semantic_types: (!scan_semantic_types.is_empty())
                .then_some(scan_semantic_types.as_slice()),
            ..MaterializationContext::default()
        },
    );
    bindings.insert(scan.binding, binding_layout);
    let mut sql = format!(
        "SELECT n.{} AS {}, {} AS {}{extra} FROM {} AS n",
        quote_identifier(&layout.identity_column),
        binding_column(scan.binding),
        scan.source.get(),
        source_column_ref(scan.binding),
        quote_identifier(&layout.table)
    );
    // Filter labeled scans through the node-label junction when available.
    // Joins (each label yields at most one junction row per node) keep the
    // scan shape simple for downstream traversal joins.
    if let Some(labels_table) = catalog.labels_table() {
        for (index, label) in scan.labels.iter().enumerate() {
            if let Some(name) = catalog.label_name(*label) {
                let source_predicate = if catalog.source_qualified_membership() {
                    format!("lbl{index}.source_id = {} AND ", scan.source.get())
                } else {
                    String::new()
                };
                sql.push_str(&format!(
                    " JOIN {} AS lbl{index} ON {source_predicate}\
                     lbl{index}.node_id = n.{} AND lbl{index}.label = '{}'",
                    quote_identifier(&labels_table),
                    quote_identifier(&layout.identity_column),
                    name.replace('\'', "''")
                ));
            }
        }
    }
    Ok(Lowered { sql, bindings })
}

/// Synthetic column carrying a `RelationScan`'s projection of one named
/// role's endpoint column, so a later `RoleJoin` can address it without a
/// second lookup against the relationship table. Keyed by the relation
/// binding (not just the role) so a query with two independently scanned
/// relations of the same type never collides.
fn role_column_ref(relation: ir::BindingId, role: ir::RoleId) -> String {
    format!("{}_role{}", binding_column(relation), role.get())
}

/// Anchors a scan on a relation's own table. Every `One`-cardinality role's
/// endpoint column is projected under a synthetic, role-keyed name so any
/// number of `RoleJoin`s can be composed on top — the scan itself carries no
/// knowledge of which roles a query will actually join through.
fn lower_relation_scan(
    scan: &ir::RelationScan,
    catalog: &dyn RelationalCatalogSnapshot,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    let layout = catalog
        .relationship_layout(scan.source)
        .ok_or(LowerError::MissingSource(scan.source))?;
    let alias = "r";
    let mut binding_layout = BindingLayout {
        source: scan.source,
        sources: [scan.source].into_iter().collect(),
        kind: EntityKind::Relationship,
        properties: Default::default(),
    };
    let extra = materialize_properties(
        wanted,
        scan.binding,
        scan.source,
        alias,
        catalog,
        &mut binding_layout,
        MaterializationContext::default(),
    );
    let mut bindings = HashMap::new();
    bindings.insert(scan.binding, binding_layout);
    let mut role_columns = String::new();
    for role in layout
        .roles
        .iter()
        .filter(|role| role.cardinality == ir::RoleCardinality::One)
    {
        role_columns.push_str(&format!(
            ", {alias}.{} AS {}",
            quote_identifier(&role.column),
            role_column_ref(scan.binding, role.role)
        ));
    }
    let mut sql = format!(
        "SELECT {alias}.{} AS {}, {} AS {}{role_columns}{extra} FROM {} AS {alias}",
        quote_identifier(&layout.identity_column),
        binding_column(scan.binding),
        scan.source.get(),
        source_column_ref(scan.binding),
        quote_identifier(&layout.table)
    );
    if !scan.relationship_types.is_empty() {
        if let Some(types_table) = catalog.relationship_types_table() {
            let names = scan
                .relationship_types
                .iter()
                .filter_map(|relationship_type| catalog.relationship_type_name(*relationship_type))
                .map(|name| format!("'{}'", name.replace('\'', "''")))
                .collect::<Vec<_>>();
            if !names.is_empty() {
                let source_predicate = if catalog.source_qualified_membership() {
                    format!("jt.source_id = {} AND ", scan.source.get())
                } else {
                    String::new()
                };
                sql.push_str(&format!(
                    " JOIN {} AS jt ON {source_predicate}jt.relationship_id = {alias}.{} \
                     AND jt.type IN ({})",
                    quote_identifier(&types_table),
                    quote_identifier(&layout.identity_column),
                    names.join(", ")
                ));
            }
        }
    }
    Ok(Lowered { sql, bindings })
}

/// Joins one named role of an already-scanned relation out to its player.
/// `Bound` folds to an identity equality for a `One` role, or a spill-table
/// membership test for a `Many` role (mirrors the merge-key predicate
/// `mutation.rs` builds for a `Many` role); `Fresh` joins the role's physical
/// node table, through the spill table first when the role is `Many` so `n`
/// players produce `n` rows. Composing `n` of these onto a `RelationScan`
/// reads a relation with `n` named roles with no arity branch — each role
/// resolves independently by `RoleId`, and a role is `Many` exactly when its
/// layout carries a `spill_table`, never by name, position, or arity.
fn lower_role_join(
    join: &ir::RoleJoin,
    catalog: &dyn RelationalCatalogSnapshot,
    wanted: &WantedProperties,
) -> Result<Lowered, LowerError> {
    let input = lower_plan(&join.input, catalog, false, wanted)?;
    let relationship = catalog
        .relationship_layout(join.relationship_source)
        .ok_or(LowerError::MissingSource(join.relationship_source))?;
    let role = relationship
        .role(join.role)
        .ok_or_else(|| LowerError::UnknownRole {
            relation: relationship.table.clone(),
            role: join.role,
        })?;
    let relation_column = binding_column(join.relationship);
    let mut bindings = input.bindings;
    match &join.player {
        ir::RolePlayer::Bound(binding) => {
            let predicate = match &role.spill_table {
                // A `Many` role has no role column to equate against; test
                // membership in its spill table instead.
                Some(spill_table) => format!(
                    "EXISTS (SELECT 1 FROM {} AS s WHERE s.relation_id = q.{relation_column} \
                     AND s.node_id = q.{})",
                    quote_identifier(spill_table),
                    binding_column(*binding),
                ),
                None => format!(
                    "q.{} = q.{}",
                    role_column_ref(join.relationship, join.role),
                    binding_column(*binding),
                ),
            };
            Ok(Lowered {
                sql: format!("SELECT q.* FROM ({}) AS q WHERE {predicate}", input.sql),
                bindings,
            })
        }
        ir::RolePlayer::Fresh {
            binding,
            node_source,
        } => {
            let target = catalog
                .node_layout(*node_source)
                .ok_or(LowerError::MissingSource(*node_source))?;
            let alias = "n";
            let mut binding_layout = BindingLayout {
                source: *node_source,
                sources: [*node_source].into_iter().collect(),
                kind: EntityKind::Node,
                properties: Default::default(),
            };
            let extra = materialize_properties(
                wanted,
                binding.id(),
                *node_source,
                alias,
                catalog,
                &mut binding_layout,
                MaterializationContext::default(),
            );
            bindings.insert(binding.id(), binding_layout);
            // A `One` role's player is the relation's endpoint column,
            // joined directly to the target table. A `Many` role's players
            // live in a spill table, so a fresh player needs an extra join
            // through it, one row per spilled player — the difference from
            // the `One` arm is one extra join, not a different shape.
            let join_clause = match &role.spill_table {
                Some(spill_table) => format!(
                    "JOIN {} AS s ON s.relation_id = q.{relation_column} \
                     JOIN {} AS {alias} ON {alias}.{} = s.node_id",
                    quote_identifier(spill_table),
                    quote_identifier(&target.table),
                    quote_identifier(&target.identity_column),
                ),
                None => format!(
                    "JOIN {} AS {alias} ON {alias}.{} = q.{}",
                    quote_identifier(&target.table),
                    quote_identifier(&target.identity_column),
                    role_column_ref(join.relationship, join.role),
                ),
            };
            Ok(Lowered {
                sql: format!(
                    "SELECT q.*, {alias}.{} AS {}, {} AS {}{extra} FROM ({}) AS q {join_clause}",
                    quote_identifier(&target.identity_column),
                    binding_column(binding.id()),
                    node_source.get(),
                    source_column_ref(binding.id()),
                    input.sql,
                ),
                bindings,
            })
        }
    }
}

fn lower_role_expand(
    expand: &ir::RoleExpand,
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
    let needs_source_filter = input
        .bindings
        .get(&expand.from)
        .is_some_and(|layout| layout.sources.len() > 1);
    let input_sql = if needs_source_filter {
        format!(
            "SELECT source_q.* FROM ({}) AS source_q WHERE source_q.{} = {}",
            input.sql,
            source_column_ref(expand.from),
            expand.from_node_source.get()
        )
    } else {
        input.sql.clone()
    };
    let relationship = catalog
        .relationship_layout(expand.relationship_source)
        .ok_or(LowerError::MissingSource(expand.relationship_source))?;
    // The role pair says which column is `from` and which is `to`; binary is
    // a layout of the role model, not a separate kind, so this is the same
    // resolution an n-ary relation's roles get.
    let from_column = &relationship
        .role(expand.from_role)
        .ok_or_else(|| LowerError::UnknownRole {
            relation: relationship.table.clone(),
            role: expand.from_role,
        })?
        .column;
    let to_column = &relationship
        .role(expand.to_role)
        .ok_or_else(|| LowerError::UnknownRole {
            relation: relationship.table.clone(),
            role: expand.to_role,
        })?
        .column;
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
    // `symmetric` -- not a `Both` direction arm -- is what an undirected
    // pattern means: the reversed role pair also matches.
    let (relationship_on, mut node_on) = match (&bound_reference, expand.symmetric) {
        (Some(bound), false) => (
            format!(
                "{relationship_alias}.{} = {from} AND {relationship_alias}.{} = {bound}",
                quote_identifier(from_column),
                quote_identifier(to_column)
            ),
            String::new(),
        ),
        (Some(bound), true) => (
            format!(
                "(({relationship_alias}.{from_col} = {from} AND {relationship_alias}.{to_col} = {bound})                  OR ({relationship_alias}.{to_col} = {from} AND {relationship_alias}.{from_col} = {bound}))",
                from_col = quote_identifier(from_column),
                to_col = quote_identifier(to_column)
            ),
            String::new(),
        ),
        (None, false) => (
            format!(
                "{relationship_alias}.{} = {from}",
                quote_identifier(from_column)
            ),
            format!(
                "{target_alias}.{} = {relationship_alias}.{}",
                quote_identifier(&target.identity_column),
                quote_identifier(to_column)
            ),
        ),
        (None, true) => (
            format!(
                "({relationship_alias}.{} = {from} OR {relationship_alias}.{} = {from})",
                quote_identifier(from_column),
                quote_identifier(to_column)
            ),
            format!(
                "{target_alias}.{} = CASE WHEN {relationship_alias}.{} = {from} \
                 THEN {relationship_alias}.{} ELSE {relationship_alias}.{} END",
                quote_identifier(&target.identity_column),
                quote_identifier(from_column),
                quote_identifier(to_column),
                quote_identifier(from_column)
            ),
        ),
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
            let source_predicate = if catalog.source_qualified_membership() {
                format!("jt.source_id = {} AND ", expand.relationship_source.get())
            } else {
                String::new()
            };
            format!(
                "({relationship_on}) AND EXISTS (SELECT 1 FROM {} AS jt \
                 WHERE {source_predicate}jt.relationship_id = {relationship_alias}.{} \
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
            sources: [expand.relationship_source].into_iter().collect(),
            kind: EntityKind::Relationship,
            properties: Default::default(),
        },
    );
    bindings.insert(
        expand.to.id(),
        BindingLayout {
            source: expand.target_node_source,
            sources: [expand.target_node_source].into_iter().collect(),
            kind: EntityKind::Node,
            properties: Default::default(),
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
    let optional_probe = optional.then_some(null_probe.as_str());
    let mut relationship_layout = bindings
        .get(&expand.relationship.id())
        .cloned()
        .expect("inserted above");
    let mut extra = materialize_properties(
        wanted,
        expand.relationship.id(),
        expand.relationship_source,
        relationship_alias,
        catalog,
        &mut relationship_layout,
        MaterializationContext {
            null_probe: optional_probe,
            ..MaterializationContext::default()
        },
    );
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
            MaterializationContext {
                null_probe: optional_probe,
                ..MaterializationContext::default()
            },
        ));
        bindings.insert(expand.to.id(), to_layout);
    }
    bindings.insert(expand.relationship.id(), relationship_layout);
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
            "SELECT q.*, {relationship_identity} AS {}, {} AS {}, \
             {target_value} AS {}, {} AS {}{extra} \
             FROM ({}) AS q {join} {} AS {relationship_alias} ON {relationship_on}{node_join}",
            binding_column(expand.relationship.id()),
            expand.relationship_source.get(),
            source_column_ref(expand.relationship.id()),
            binding_column(expand.to.id()),
            expand.target_node_source.get(),
            source_column_ref(expand.to.id()),
            input_sql,
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
            semantic_types,
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
            if fields.is_empty()
                && binding.properties.contains(&property.get())
                && !references.contains_key(entity)
            {
                return Ok(format!(
                    "{input_alias}.{}",
                    property_column_ref(*entity, *property)
                ));
            }
            let jsonb = catalog.property_column_is_jsonb(binding.source, *property);
            let column =
                resolved_property_column(catalog, binding.source, semantic_types, *property)
                    .ok_or(LowerError::MissingProperty {
                        source_id: binding.source,
                        property: *property,
                    })?;
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
                    // cypher_equals gives each membership probe Cypher's
                    // three-valued deep equality (nested lists/maps, null
                    // uncertainty); a definite hit wins, any uncertain
                    // probe makes the whole membership null.
                    format!(
                        "(CASE WHEN ({right}) IS NULL THEN NULL \
                          WHEN EXISTS (SELECT 1 FROM json_each(({right})) AS e \
                          WHERE cypher_equals(e.value, ({left})) = 1) THEN 1 \
                          WHEN EXISTS (SELECT 1 FROM json_each(({right})) AS e \
                          WHERE cypher_equals(e.value, ({left})) IS NULL) THEN NULL \
                          ELSE 0 END)"
                    )
                }
                // Structural equality: lists/maps lower to JSON text, where
                // SQL's = does definite text comparison; Cypher requires
                // recursive three-valued equality (a nested null makes the
                // result uncertain, not false).
                ir::BinaryOp::Equal
                    if structural_comparison(&left_type) || structural_comparison(&right_type) =>
                {
                    format!("cypher_equals(({left}), ({right}))")
                }
                ir::BinaryOp::NotEqual
                    if structural_comparison(&left_type) || structural_comparison(&right_type) =>
                {
                    format!("(NOT cypher_equals(({left}), ({right})))")
                }
                // List append: `[1] + 2` produces `[1, 2]`.
                ir::BinaryOp::Add
                    if matches!(left_type, ir::ValueType::List(_))
                        && !matches!(right_type, ir::ValueType::List(_)) =>
                {
                    format!("json_insert(({left}), '$[#]', ({right}))")
                }
                // Cypher + concatenates strings; SQL + coerces them to 0.
                ir::BinaryOp::Add
                    if left_type == ir::ValueType::Text && right_type == ir::ValueType::Text =>
                {
                    format!("(({left}) || ({right}))")
                }
                // Dynamically typed operands dispatch in the cypher_add
                // extension scalar (numbers add, strings concatenate, lists
                // concatenate/append, null propagates); a compact call keeps
                // repeated contexts like a reduce() body small.
                ir::BinaryOp::Add
                    if matches!(left_type, ir::ValueType::Any | ir::ValueType::Text)
                        || matches!(right_type, ir::ValueType::Any | ir::ValueType::Text) =>
                {
                    format!("cypher_add(({left}), ({right}))")
                }
                // Persisted duration properties lose their marker types, so
                // dynamic subtraction must recover duration/temporal behavior
                // before applying ordinary numeric subtraction.
                ir::BinaryOp::Subtract
                    if matches!(left_type, ir::ValueType::Any)
                        || matches!(right_type, ir::ValueType::Any) =>
                {
                    format!("cypher_sub(({left}), ({right}))")
                }
                // Division on dynamic operands raises Cypher's zero-divisor
                // error instead of SQL's silent NULL.
                ir::BinaryOp::Divide
                    if matches!(left_type, ir::ValueType::Any)
                        || matches!(right_type, ir::ValueType::Any) =>
                {
                    format!("cypher_div(({left}), ({right}))")
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
                // instr/substr alone return false for an empty needle. A
                // non-string operand (int/float/bool/list/map) is a type
                // mismatch in Cypher, which these operators define as NULL
                // rather than coercing through SQLite's lenient substr/instr.
                // Lists/maps lower to JSON text, so typeof() alone can't
                // exclude them; is_container_text() layers a json_valid-
                // gated check on top (json_type errors on non-JSON text).
                ir::BinaryOp::StartsWith => format!(
                    "(CASE WHEN ({left}) IS NULL OR ({right}) IS NULL THEN NULL \
                     WHEN typeof(({left})) != 'text' OR typeof(({right})) != 'text' THEN NULL \
                     WHEN {left_container} OR {right_container} THEN NULL \
                     ELSE substr(({left}), 1, length(({right}))) = ({right}) END)",
                    left_container = is_container_text(&left),
                    right_container = is_container_text(&right),
                ),
                ir::BinaryOp::EndsWith => format!(
                    "(CASE WHEN ({left}) IS NULL OR ({right}) IS NULL THEN NULL \
                     WHEN typeof(({left})) != 'text' OR typeof(({right})) != 'text' THEN NULL \
                     WHEN {left_container} OR {right_container} THEN NULL \
                     WHEN length(({right})) = 0 THEN 1 \
                     ELSE substr(({left}), -length(({right}))) = ({right}) END)",
                    left_container = is_container_text(&left),
                    right_container = is_container_text(&right),
                ),
                ir::BinaryOp::Contains => format!(
                    "(CASE WHEN ({left}) IS NULL OR ({right}) IS NULL THEN NULL \
                     WHEN typeof(({left})) != 'text' OR typeof(({right})) != 'text' THEN NULL \
                     WHEN {left_container} OR {right_container} THEN NULL \
                     WHEN length(({right})) = 0 THEN 1 \
                     ELSE instr(({left}), ({right})) > 0 END)",
                    left_container = is_container_text(&left),
                    right_container = is_container_text(&right),
                ),
                _ => format!("({left}) {} ({right})", binary_operator(*op)),
            })
        }
        ir::Expression::ListElement(depth) => Ok(format!("lst{depth}.value")),
        ir::Expression::ReduceAccumulator(depth) => Ok(format!("rq{depth}.acc{depth}")),
        ir::Expression::ReduceElement(depth) => Ok(format!(
            "json_extract(rq{depth}.lst{depth}, '$[' || rq{depth}.idx{depth} || ']')"
        )),
        ir::Expression::Reduce {
            depth,
            initial,
            list,
            body,
        } => {
            // The fold is a recursive CTE carrying (accumulator, list, index).
            // The recursive arm applies the body once per element and stops
            // when the index leaves the list, so the SQL is a fixed size and
            // the fold runs for lists of any length.
            let initial = lower_expression_with_references(
                initial,
                bindings,
                catalog,
                input_alias,
                references,
            )?;
            let list =
                lower_expression_with_references(list, bindings, catalog, input_alias, references)?;
            let body =
                lower_expression_with_references(body, bindings, catalog, input_alias, references)?;
            // A NULL list makes `json_array_length` NULL, so the recursive arm
            // never fires and the final predicate matches no row: the scalar
            // subquery yields NULL, which is `reduce()` over NULL. An empty
            // list matches the seed row and yields the initial accumulator.
            Ok(format!(
                "(WITH RECURSIVE r{depth}(acc{depth}, lst{depth}, idx{depth}) AS (\
                 SELECT ({initial}), ({list}), 0 \
                 UNION ALL \
                 SELECT ({body}), rq{depth}.lst{depth}, rq{depth}.idx{depth} + 1 \
                 FROM r{depth} AS rq{depth} \
                 WHERE rq{depth}.idx{depth} < json_array_length(rq{depth}.lst{depth})) \
                 SELECT rq{depth}.acc{depth} FROM r{depth} AS rq{depth} \
                 WHERE rq{depth}.idx{depth} = json_array_length(rq{depth}.lst{depth}))"
            ))
        }
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
            // A bare `WHERE (predicate)` filters out rows where predicate is
            // NULL the same way it filters out false rows, collapsing
            // Cypher's three-valued quantifier semantics to a boolean. Each
            // arm below probes true/false/null membership separately so a
            // NULL-valued predicate can surface as NULL instead of vanishing.
            let has_true = format!(
                "EXISTS (SELECT 1 FROM json_each(({list})) AS {alias} WHERE ({predicate}) = 1)"
            );
            let has_false = format!(
                "EXISTS (SELECT 1 FROM json_each(({list})) AS {alias} WHERE ({predicate}) = 0)"
            );
            let has_null = format!(
                "EXISTS (SELECT 1 FROM json_each(({list})) AS {alias} WHERE ({predicate}) IS NULL)"
            );
            Ok(match kind {
                ir::QuantifierKind::Any => {
                    format!("(CASE WHEN {has_true} THEN 1 WHEN {has_null} THEN NULL ELSE 0 END)")
                }
                ir::QuantifierKind::All => {
                    format!("(CASE WHEN {has_false} THEN 0 WHEN {has_null} THEN NULL ELSE 1 END)")
                }
                ir::QuantifierKind::None => {
                    format!("(CASE WHEN {has_true} THEN 0 WHEN {has_null} THEN NULL ELSE 1 END)")
                }
                // A count of two or more true matches already breaks
                // "exactly one" no matter how any null element resolves, so
                // that branch must be checked before the null branch.
                ir::QuantifierKind::Single => format!(
                    "(CASE WHEN (SELECT count(*) FROM json_each(({list})) AS {alias} \
                     WHERE ({predicate}) = 1) >= 2 THEN 0 \
                     WHEN (SELECT count(*) FROM json_each(({list})) AS {alias} \
                     WHERE ({predicate}) = 1) = 1 AND NOT {has_null} THEN 1 \
                     WHEN {has_null} THEN NULL ELSE 0 END)"
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
            #[cfg(feature = "fts")]
            if matches!(function.as_str(), "fts_match" | "fts_score") {
                let Some((query, properties)) = arguments.split_last() else {
                    return Err(LowerError::UnsupportedOperator(
                        "FTS calls require properties and a query",
                    ));
                };
                let mut entity = None;
                let mut property_arguments = Vec::with_capacity(properties.len());
                for property in properties {
                    let ir::Expression::Property {
                        entity: property_entity,
                        property,
                        semantic_types,
                        fields,
                    } = &property.expression
                    else {
                        property_arguments.clear();
                        break;
                    };
                    if !fields.is_empty() || entity.is_some_and(|entity| entity != *property_entity)
                    {
                        property_arguments.clear();
                        break;
                    }
                    entity = Some(*property_entity);
                    property_arguments.push((*property, semantic_types.as_slice()));
                }
                if let Some(entity) = entity.filter(|_| !property_arguments.is_empty()) {
                    let binding = bindings
                        .get(&entity)
                        .ok_or(LowerError::MissingBinding(entity))?;
                    if matches!(binding.kind, EntityKind::Node) {
                        let query = lower_expression_with_references(
                            query,
                            bindings,
                            catalog,
                            input_alias,
                            references,
                        )?;
                        let identity = binding_reference(entity, input_alias, references);
                        let mut branches = Vec::new();
                        for source in &binding.sources {
                            let layout = catalog
                                .node_layout(*source)
                                .ok_or(LowerError::MissingSource(*source))?;
                            let columns = property_arguments
                                .iter()
                                .map(|(property, semantic_types)| {
                                    resolved_property_column(
                                        catalog,
                                        *source,
                                        semantic_types,
                                        *property,
                                    )
                                    .map(|column| format!("fts.{}", quote_identifier(&column)))
                                    .ok_or(
                                        LowerError::MissingProperty {
                                            source_id: *source,
                                            property: *property,
                                        },
                                    )
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            let arguments = columns
                                .iter()
                                .cloned()
                                .chain(std::iter::once(query.clone()))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let match_call = format!("fts_match({arguments})");
                            let sql = if function.as_str() == "fts_match" {
                                format!(
                                    "({identity}) IN (SELECT fts.{} FROM {} AS fts \
                                     WHERE {match_call})",
                                    quote_identifier(&layout.identity_column),
                                    quote_identifier(&layout.table),
                                )
                            } else {
                                format!(
                                    "(SELECT fts_score({arguments}) FROM {} AS fts \
                                     WHERE {match_call} AND fts.{} = ({identity}) LIMIT 1)",
                                    quote_identifier(&layout.table),
                                    quote_identifier(&layout.identity_column),
                                )
                            };
                            branches.push((*source, sql));
                        }
                        if let [(_, sql)] = branches.as_slice() {
                            return Ok(sql.clone());
                        }
                        let source_value = format!("{input_alias}.{}", source_column_ref(entity));
                        let cases = branches
                            .into_iter()
                            .map(|(source, sql)| format!("WHEN {} THEN {sql}", source.get()))
                            .collect::<Vec<_>>()
                            .join(" ");
                        let sql = format!("(CASE {source_value} {cases} ELSE NULL END)");
                        return Ok(sql);
                    }
                }
            }
            if matches!(
                function.as_str(),
                "__cypher_start_node" | "__cypher_end_node"
            ) {
                let [argument] = arguments.as_slice() else {
                    return Err(LowerError::UnsupportedOperator(
                        "startNode()/endNode() require one relationship argument",
                    ));
                };
                if matches!(
                    argument.expression,
                    ir::Expression::Literal(ir::Literal::Null)
                ) {
                    return Ok("NULL".to_owned());
                }
                let ir::Expression::Binding(id) = &argument.expression else {
                    return Err(LowerError::UnsupportedOperator(
                        "startNode()/endNode() require a relationship binding",
                    ));
                };
                let layout = bindings.get(id).ok_or(LowerError::MissingBinding(*id))?;
                if !matches!(layout.kind, EntityKind::Relationship) {
                    return Err(LowerError::UnsupportedOperator(
                        "startNode()/endNode() require a relationship binding",
                    ));
                }
                let identity_value = lower_expression_with_references(
                    argument,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                let mut branches = Vec::new();
                for source in &layout.sources {
                    let relationship = catalog
                        .relationship_layout(*source)
                        .ok_or(LowerError::MissingSource(*source))?;
                    // startNode()/endNode() only ever address the two-role
                    // pattern-hop relationship. Resolve by name, not
                    // declaration order -- role order is not guaranteed to
                    // put `start` before `end`.
                    let role = if function.as_str() == "__cypher_start_node" {
                        relationship.start_role()
                    } else {
                        relationship.end_role()
                    };
                    let endpoint = role
                        .ok_or(LowerError::MissingSource(*source))?
                        .column
                        .clone();
                    branches.push((
                        *source,
                        format!(
                            "(SELECT ep.{} FROM {} AS ep WHERE ep.{} = ({}))",
                            quote_identifier(&endpoint),
                            quote_identifier(&relationship.table),
                            quote_identifier(&relationship.identity_column),
                            identity_value,
                        ),
                    ));
                }
                if let [(_, sql)] = branches.as_slice() {
                    return Ok(sql.clone());
                }
                let source_value = format!("{input_alias}.{}", source_column_ref(*id));
                let cases = branches
                    .into_iter()
                    .map(|(source, sql)| format!("WHEN {} THEN {sql}", source.get()))
                    .collect::<Vec<_>>()
                    .join(" ");
                return Ok(format!("(CASE {source_value} {cases} ELSE NULL END)"));
            }
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
                let identity_value = lower_expression_with_references(
                    argument,
                    bindings,
                    catalog,
                    input_alias,
                    references,
                )?;
                let mut branches = Vec::new();
                for source in &layout.sources {
                    let columns =
                        if let Some(properties) = catalog.semantic_properties(*source, &[]) {
                            properties
                                .into_iter()
                                .map(|(_, name, _, column)| (name, column))
                                .collect()
                        } else {
                            catalog.payload_columns(*source).ok_or(
                                LowerError::UnsupportedOperator(
                                    "properties() without payload columns",
                                ),
                            )?
                        };
                    let (table, identity) = match layout.kind {
                        EntityKind::Node => {
                            let source_layout = catalog
                                .node_layout(*source)
                                .ok_or(LowerError::MissingSource(*source))?;
                            (source_layout.table, source_layout.identity_column)
                        }
                        EntityKind::Relationship => {
                            let source_layout = catalog
                                .relationship_layout(*source)
                                .ok_or(LowerError::MissingSource(*source))?;
                            (source_layout.table, source_layout.identity_column)
                        }
                    };
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
                    // json_group_object over json_each strips null-valued
                    // keys, matching Cypher's properties() (absent, not null).
                    let sql = format!(
                        "(SELECT coalesce(json_group_object(je.key, je.value), json_object()) \
                         FROM json_each((SELECT {object} FROM {} AS prp \
                         WHERE prp.{} = ({identity_value}))) AS je \
                         WHERE je.value IS NOT NULL)",
                        quote_identifier(&table),
                        quote_identifier(&identity),
                    );
                    branches.push((*source, sql));
                }
                if let [(_, sql)] = branches.as_slice() {
                    return Ok(sql.clone());
                }
                let source_value = format!("{input_alias}.{}", source_column_ref(*id));
                let cases = branches
                    .into_iter()
                    .map(|(source, sql)| format!("WHEN {} THEN {sql}", source.get()))
                    .collect::<Vec<_>>()
                    .join(" ");
                return Ok(format!(
                    "(CASE {source_value} {cases} ELSE json_object() END)"
                ));
            }
            let source_reference = arguments.first().and_then(|argument| {
                let ir::Expression::Binding(binding) = &argument.expression else {
                    return None;
                };
                let layout = bindings.get(binding)?;
                if layout.sources.len() == 1 {
                    Some(
                        layout
                            .sources
                            .first()
                            .expect("one source")
                            .get()
                            .to_string(),
                    )
                } else {
                    Some(format!("{input_alias}.{}", source_column_ref(*binding)))
                }
            });
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
                ("__cypher_relationship_type", [value]) => {
                    // No junction table means the graph does not track
                    // relationship types; nothing to report.
                    let Some(table) = catalog.relationship_types_table() else {
                        return Ok("NULL".to_owned());
                    };
                    let source_predicate = if catalog.source_qualified_membership() {
                        format!(
                            "jt.source_id = {} AND ",
                            source_reference.as_deref().unwrap_or("-1")
                        )
                    } else {
                        String::new()
                    };
                    return Ok(format!(
                        "(SELECT jt.type FROM {} AS jt \
                         WHERE {source_predicate}jt.relationship_id = ({value}) LIMIT 1)",
                        quote_identifier(&table),
                    ));
                }
                ("__cypher_labels", [value]) => {
                    let table = catalog
                        .labels_table()
                        .ok_or(LowerError::UnsupportedOperator(
                            "labels() without a label table",
                        ))?;
                    let source_predicate = if catalog.source_qualified_membership() {
                        format!(
                            "lbl.source_id = {} AND ",
                            source_reference.as_deref().unwrap_or("-1")
                        )
                    } else {
                        String::new()
                    };
                    return Ok(format!(
                        "(SELECT json_group_array(label) FROM (SELECT lbl.label FROM {} AS lbl \
                         WHERE {source_predicate}lbl.node_id = ({value}) ORDER BY lbl.rowid))",
                        quote_identifier(&table),
                    ));
                }
                ("__cypher_label", [value]) => {
                    let table = catalog
                        .labels_table()
                        .ok_or(LowerError::UnsupportedOperator(
                            "label() without a label table",
                        ))?;
                    let source_predicate = if catalog.source_qualified_membership() {
                        format!(
                            "lbl.source_id = {} AND ",
                            source_reference.as_deref().unwrap_or("-1")
                        )
                    } else {
                        String::new()
                    };
                    return Ok(format!(
                        "(SELECT lbl.label FROM {} AS lbl WHERE \
                         {source_predicate}lbl.node_id = ({value}) \
                         ORDER BY lbl.rowid LIMIT 1)",
                        quote_identifier(&table),
                    ));
                }
                ("__cypher_has_label", [value, name]) => {
                    // No junction table means the graph does not track
                    // labels; stay permissive as scans do.
                    let Some(table) = catalog.labels_table() else {
                        return Ok("TRUE".to_owned());
                    };
                    let source_predicate = if catalog.source_qualified_membership() {
                        format!(
                            "lbl.source_id = {} AND ",
                            source_reference.as_deref().unwrap_or("-1")
                        )
                    } else {
                        String::new()
                    };
                    return Ok(format!(
                        "EXISTS (SELECT 1 FROM {} AS lbl WHERE \
                         {source_predicate}lbl.node_id = ({value}) \
                         AND lbl.label = ({name}))",
                        quote_identifier(&table),
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
                ("__cypher_list_reverse", [value]) => {
                    return Ok(format!(
                        "(CASE WHEN ({value}) IS NULL THEN NULL ELSE \
                         (SELECT json_group_array(value) FROM \
                         (SELECT value FROM json_each(({value})) \
                         ORDER BY key DESC)) END)"
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

/// SQL fragment that evaluates to true when `operand` is JSON text
/// representing a list or map, as opposed to a genuine Cypher string (both
/// lower to SQLite TEXT, so typeof() alone can't tell them apart). Nests the
/// json_type() check under json_valid() because json_type() errors on text
/// that isn't valid JSON, e.g. an ordinary string.
fn is_container_text(operand: &str) -> String {
    format!(
        "(CASE WHEN json_valid(({operand})) \
         THEN json_type(({operand})) IN ('array', 'object') ELSE 0 END)"
    )
}

/// True when an ordering comparison needs the runtime typeof guard: the
/// operands are not statically known to be the same comparable class.
/// Types whose SQL encoding is JSON text and whose equality therefore
/// needs recursive three-valued comparison rather than SQL's text =.
/// `Any` is included because dynamically-typed values (parameters, UNWIND
/// elements, untyped properties) may hold lists or maps at runtime; SQL `=`
/// on their text encoding would silently do definite text comparison. This
/// also keeps `=` consistent with `IN`, which always routes through
/// `cypher_equals`.
fn structural_comparison(value_type: &ir::ValueType) -> bool {
    matches!(
        value_type,
        ir::ValueType::List(_) | ir::ValueType::Map | ir::ValueType::Any
    )
}

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

    struct EndpointCatalog;

    impl RelationalCatalogSnapshot for EndpointCatalog {
        fn node_layout(&self, _source: ir::SourceTableId) -> Option<NodeTableLayout> {
            None
        }

        fn relationship_layout(
            &self,
            _source: ir::SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            Some(RelationshipTableLayout {
                table: "relationship table".to_owned(),
                identity_column: "relationship id".to_owned(),
                roles: vec![
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(1).unwrap(),
                        name: "start".to_owned(),
                        column: "start node".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(2).unwrap(),
                        name: "end".to_owned(),
                        column: "end node".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                ],
            })
        }

        fn property_column(
            &self,
            _source: ir::SourceTableId,
            _property: ir::PropertyId,
        ) -> Option<String> {
            None
        }
    }

    /// Same layout as `EndpointCatalog`, but with `end` declared BEFORE
    /// `start`. Declaration order is not guaranteed to put `start` first, so
    /// a consumer that indexes `roles[0]`/`roles[1]` positionally instead of
    /// resolving by name would silently swap start/end here.
    struct ReversedEndpointCatalog;

    impl RelationalCatalogSnapshot for ReversedEndpointCatalog {
        fn node_layout(&self, _source: ir::SourceTableId) -> Option<NodeTableLayout> {
            None
        }

        fn relationship_layout(
            &self,
            _source: ir::SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            Some(RelationshipTableLayout {
                table: "relationship table".to_owned(),
                identity_column: "relationship id".to_owned(),
                roles: vec![
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(2).unwrap(),
                        name: "end".to_owned(),
                        column: "end node".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(1).unwrap(),
                        name: "start".to_owned(),
                        column: "start node".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                ],
            })
        }

        fn property_column(
            &self,
            _source: ir::SourceTableId,
            _property: ir::PropertyId,
        ) -> Option<String> {
            None
        }
    }

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

    /// Catalog with a label junction so pure `count(*)` can use the
    /// column-free membership path.
    struct LabeledCatalog;

    impl RelationalCatalogSnapshot for LabeledCatalog {
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
            Some("name".to_owned())
        }

        fn source_qualified_membership(&self) -> bool {
            true
        }

        fn labels_table(&self) -> Option<String> {
            Some("__turso_graph_node_labels_1".to_owned())
        }

        fn label_name(&self, label: ir::LabelId) -> Option<String> {
            match label.get() {
                1 => Some("Person".to_owned()),
                2 => Some("Engineer".to_owned()),
                _ => None,
            }
        }
    }

    fn pure_count_aggregate(scan: ir::NodeScan, count_binding: ir::BindingId) -> ir::Plan {
        let node_binding = ir::Binding::new(
            scan.binding,
            "n",
            ir::ValueType::Node,
            ir::Nullability::NonNull,
        )
        .unwrap();
        let count_out = ir::Binding::new(
            count_binding,
            "c",
            ir::ValueType::Integer,
            ir::Nullability::NonNull,
        )
        .unwrap();
        let input_scope = ir::Scope::new(vec![node_binding]).unwrap();
        let input = ir::Plan::new(
            ir::PlanKind::NodeScan(scan),
            input_scope,
            ir::ResultShape::default(),
        )
        .unwrap();
        let output_scope = ir::Scope::new(vec![count_out.clone()]).unwrap();
        ir::Plan::new(
            ir::PlanKind::Aggregate(ir::Aggregate {
                input: Box::new(input),
                groupings: vec![],
                aggregations: vec![ir::Aggregation {
                    output: count_out,
                    function: ir::AggregateFunction::Count,
                    expression: None,
                    distinct: false,
                }],
            }),
            output_scope,
            ir::ResultShape::default(),
        )
        .unwrap()
    }

    #[test]
    fn pure_count_star_over_unlabeled_node_scan_is_column_free() {
        let source = ir::SourceTableId::new(1).unwrap();
        let binding = ir::BindingId::new(1).unwrap();
        let count = ir::BindingId::new(2).unwrap();
        let plan = pure_count_aggregate(
            ir::NodeScan {
                graph: ir::GraphId::new(1).unwrap(),
                source,
                binding,
                labels: vec![],
            },
            count,
        );
        let lowered = lower_plan(&plan, &LabeledCatalog, false, &WantedProperties::new()).unwrap();
        assert_eq!(
            lowered.sql, "SELECT count(*) AS b2 FROM \"people\"",
            "unlabeled star-count must not wrap a node projection: {}",
            lowered.sql
        );
    }

    #[test]
    fn pure_count_star_over_labeled_node_scan_counts_junction_membership() {
        let source = ir::SourceTableId::new(1).unwrap();
        let binding = ir::BindingId::new(1).unwrap();
        let count = ir::BindingId::new(2).unwrap();
        let plan = pure_count_aggregate(
            ir::NodeScan {
                graph: ir::GraphId::new(1).unwrap(),
                source,
                binding,
                labels: vec![ir::LabelId::new(1).unwrap()],
            },
            count,
        );
        let lowered = lower_plan(&plan, &LabeledCatalog, false, &WantedProperties::new()).unwrap();
        assert_eq!(
            lowered.sql,
            "SELECT count(*) AS b2 FROM \"__turso_graph_node_labels_1\" AS lbl0 \
             WHERE lbl0.source_id = 1 AND lbl0.label = 'Person'",
            "labeled star-count must count junction rows directly: {}",
            lowered.sql
        );
        assert!(
            !lowered.sql.contains("people"),
            "must not join the node table for pure label membership count: {}",
            lowered.sql
        );
    }

    #[test]
    fn endpoint_functions_use_quoted_relationship_layout_columns() {
        let source = ir::SourceTableId::new(7).unwrap();
        let relationship = ir::BindingId::new(3).unwrap();
        let bindings = HashMap::from([(
            relationship,
            BindingLayout {
                source,
                sources: [source].into_iter().collect(),
                kind: EntityKind::Relationship,
                properties: Default::default(),
            },
        )]);
        let endpoint = |function: &str| ir::TypedExpression {
            expression: ir::Expression::Function {
                function: ir::FunctionName::new(function).unwrap(),
                arguments: vec![ir::TypedExpression {
                    expression: ir::Expression::Binding(relationship),
                    value_type: ir::ValueType::Relationship,
                    nullability: ir::Nullability::NonNull,
                }],
            },
            value_type: ir::ValueType::Node,
            nullability: ir::Nullability::NonNull,
        };

        assert_eq!(
            lower_expression(
                &endpoint("__cypher_start_node"),
                &bindings,
                &EndpointCatalog,
                "q"
            )
            .unwrap(),
            "(SELECT ep.\"start node\" FROM \"relationship table\" AS ep WHERE ep.\"relationship id\" = (q.b3))"
        );
        assert!(matches!(
            lower_expression(&endpoint("__cypher_end_node"), &bindings, &Catalog, "q"),
            Err(LowerError::MissingSource(missing)) if missing == source
        ));
    }

    /// Regression test: `startNode()`/`endNode()` must resolve the relevant
    /// role column by NAME, not by position in `RelationshipTableLayout::roles`.
    /// A positional `roles[0]`/`roles[1]` reader would swap these two
    /// assertions relative to `endpoint_functions_use_quoted_relationship_layout_columns`
    /// above, because `ReversedEndpointCatalog` declares `end` first.
    #[test]
    fn start_end_role_lookup_is_name_based_not_positional() {
        let source = ir::SourceTableId::new(7).unwrap();
        let relationship = ir::BindingId::new(3).unwrap();
        let bindings = HashMap::from([(
            relationship,
            BindingLayout {
                source,
                sources: [source].into_iter().collect(),
                kind: EntityKind::Relationship,
                properties: Default::default(),
            },
        )]);
        let endpoint = |function: &str| ir::TypedExpression {
            expression: ir::Expression::Function {
                function: ir::FunctionName::new(function).unwrap(),
                arguments: vec![ir::TypedExpression {
                    expression: ir::Expression::Binding(relationship),
                    value_type: ir::ValueType::Relationship,
                    nullability: ir::Nullability::NonNull,
                }],
            },
            value_type: ir::ValueType::Node,
            nullability: ir::Nullability::NonNull,
        };

        assert_eq!(
            lower_expression(
                &endpoint("__cypher_start_node"),
                &bindings,
                &ReversedEndpointCatalog,
                "q"
            )
            .unwrap(),
            "(SELECT ep.\"start node\" FROM \"relationship table\" AS ep WHERE ep.\"relationship id\" = (q.b3))",
            "startNode() must resolve the `start` role's column even though it is declared second"
        );
        assert_eq!(
            lower_expression(
                &endpoint("__cypher_end_node"),
                &bindings,
                &ReversedEndpointCatalog,
                "q"
            )
            .unwrap(),
            "(SELECT ep.\"end node\" FROM \"relationship table\" AS ep WHERE ep.\"relationship id\" = (q.b3))",
            "endNode() must resolve the `end` role's column even though it is declared first"
        );
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
                sources: [source].into_iter().collect(),
                kind: EntityKind::Node,
                properties: Default::default(),
            },
        );
        let expression = ir::TypedExpression {
            expression: ir::Expression::Property {
                entity,
                property: ir::PropertyId::new(1).unwrap(),
                semantic_types: Vec::new(),
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
    fn mutation_rows_preserve_one_unit_row_without_bindings() {
        let (sql, columns) = mutation_rows_with_sources_sql(&unit_mutation_input(), &[]);
        assert_eq!(sql, "SELECT 1 FROM (SELECT 1 AS __unit) AS q");
        assert!(columns.is_empty());
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
                semantic_types: Vec::new(),
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
        assert!(sql.contains("cypher_equals(e.value"));
    }

    /// Dynamically-typed operands (parameters, UNWIND elements, untyped
    /// properties) may hold lists or maps at runtime, so `=` must use
    /// Cypher's null-aware deep equality, not SQL's definite text `=`.
    #[test]
    fn lowers_any_typed_equality_through_cypher_equals() {
        let catalog = Catalog;
        let bindings = HashMap::new();
        let any = |name: &str| ir::TypedExpression {
            expression: ir::Expression::Parameter(name.to_owned()),
            value_type: ir::ValueType::Any,
            nullability: ir::Nullability::Nullable,
        };
        let expression = ir::TypedExpression {
            expression: ir::Expression::Binary {
                left: Box::new(any("left")),
                op: ir::BinaryOp::Equal,
                right: Box::new(any("right")),
            },
            value_type: ir::ValueType::Boolean,
            nullability: ir::Nullability::Nullable,
        };
        let sql = lower_expression(&expression, &bindings, &catalog, "n")
            .expect("Any equality lowering should succeed");
        assert!(sql.contains("cypher_equals"), "{sql}");
    }

    #[test]
    fn lowers_any_typed_subtraction_through_cypher_sub() {
        let catalog = Catalog;
        let bindings = HashMap::new();
        let any = |name: &str| ir::TypedExpression {
            expression: ir::Expression::Parameter(name.to_owned()),
            value_type: ir::ValueType::Any,
            nullability: ir::Nullability::Nullable,
        };
        let expression = ir::TypedExpression {
            expression: ir::Expression::Binary {
                left: Box::new(any("left")),
                op: ir::BinaryOp::Subtract,
                right: Box::new(any("right")),
            },
            value_type: ir::ValueType::Any,
            nullability: ir::Nullability::Nullable,
        };
        let sql = lower_expression(&expression, &bindings, &catalog, "n")
            .expect("Any subtraction lowering should succeed");
        assert!(sql.contains("cypher_sub"), "{sql}");
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
                sources: [source].into_iter().collect(),
                kind: EntityKind::Node,
                properties: Default::default(),
            },
        );
        let expression = ir::TypedExpression {
            expression: ir::Expression::Property {
                entity,
                property: ir::PropertyId::new(1).unwrap(),
                semantic_types: Vec::new(),
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
