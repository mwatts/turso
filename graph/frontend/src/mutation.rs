use std::{collections::HashMap, sync::Arc};

use thiserror::Error;
use turso_core::{Connection, Numeric, Value};
use turso_graph_ir as ir;

use crate::{
    bind_mutation,
    binder::{BoundMutation, StageItem, StageProjection},
    lowering::{
        lower_mutation_expression, lower_mutation_input, mutation_rows_sql, quoted_identifier,
        unit_mutation_input, LoweredMutationInput, MutationEntityKind,
    },
    BindError, GraphCompilationCatalog, LowerError, ParameterTypes,
};

const SAVEPOINT: &str = "__turso_graph_mutation";
pub(crate) const INTERNAL_PARAMETER_PREFIX: &str = "__turso_internal_graph_ref_";

pub type Parameters = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct MutationSummary {
    pub matched_rows: u64,
    pub operations_executed: u64,
    /// Rows produced by a trailing RETURN clause, one per input row.
    pub rows: Vec<Vec<Value>>,
    /// Static types of the user-visible RETURN columns, in projection order.
    pub result_types: Vec<ir::ValueType>,
}

#[derive(Debug, Error)]
pub enum MutationError {
    #[error("Cypher parse failed: {0}")]
    Parse(#[from] turso_graph_cypher::ParseError),
    #[error("Cypher mutation binding failed: {0}")]
    Bind(#[from] BindError),
    #[error("Cypher mutation lowering failed: {0}")]
    Lower(#[from] LowerError),
    #[error("graph mutation database operation failed: {0}")]
    Database(#[from] turso_core::LimboError),
    #[error("mutation parameter `${0}` uses a reserved internal name")]
    ReservedParameter(String),
    #[error("created {entity} did not return exactly one identity")]
    MissingCreatedIdentity { entity: &'static str },
    #[error("mutation references binding {0} before it has a value")]
    MissingBinding(ir::BindingId),
    #[error("cannot delete node while relationships still reference it; use DETACH DELETE")]
    NodeHasRelationships,
    #[error("runtime value for property `{property}` is not assignable to {expected:?}")]
    IncompatibleRuntimeValue {
        property: String,
        expected: ir::ValueType,
    },
    #[error("dynamic map key `{key}` is not an owned semantic property")]
    UnknownDynamicKey { key: String },
    #[error("mutation expression did not produce exactly one scalar value")]
    MissingEvaluatedValue,
    #[error("mutation failed and savepoint rollback also failed: {cause}; rollback: {rollback}")]
    RollbackFailed {
        cause: Box<MutationError>,
        rollback: turso_core::LimboError,
    },
}

fn check_runtime_value(
    catalog: &dyn GraphCompilationCatalog,
    source: ir::SourceTableId,
    semantic_types: &[String],
    property: ir::PropertyId,
    value: &Value,
) -> Result<(), MutationError> {
    let Some(resolution) = GraphCompilationCatalog::semantic_property_for_id(
        catalog,
        source,
        semantic_types,
        property,
    ) else {
        return Ok(());
    };
    let Some((name, expected, _)) = resolution else {
        return Err(LowerError::MissingProperty {
            source_id: source,
            property,
        }
        .into());
    };
    check_runtime_value_against(&name, &expected, value)
}

fn check_runtime_value_against(
    name: &str,
    expected: &ir::ValueType,
    value: &Value,
) -> Result<(), MutationError> {
    if runtime_value_compatible(expected, value) {
        Ok(())
    } else {
        Err(MutationError::IncompatibleRuntimeValue {
            property: name.to_owned(),
            expected: expected.clone(),
        })
    }
}

fn runtime_value_compatible(expected: &ir::ValueType, value: &Value) -> bool {
    match (expected, value) {
        (_, Value::Null) | (ir::ValueType::Any, _) => true,
        (ir::ValueType::Boolean | ir::ValueType::Integer, Value::Numeric(Numeric::Integer(_))) => {
            true
        }
        (ir::ValueType::Real, Value::Numeric(_)) => true,
        (ir::ValueType::Text, Value::Text(_)) => true,
        (ir::ValueType::Bytes, Value::Blob(_)) => true,
        (ir::ValueType::Custom { base, .. }, value) => runtime_value_compatible(base, value),
        (
            ir::ValueType::Struct(_)
            | ir::ValueType::Union(_)
            | ir::ValueType::List(_)
            | ir::ValueType::Vector(_, _),
            Value::Blob(_),
        ) => true,
        (ir::ValueType::Map, Value::Text(_)) => true,
        (ir::ValueType::Node | ir::ValueType::Relationship | ir::ValueType::Path, _) => false,
        _ => false,
    }
}

pub fn execute_cypher_mutation(
    connection: &Arc<Connection>,
    graph: ir::GraphId,
    catalog: Arc<dyn GraphCompilationCatalog>,
    source: &str,
    parameters: &Parameters,
) -> Result<MutationSummary, MutationError> {
    for name in parameters.keys() {
        if name.starts_with(INTERNAL_PARAMETER_PREFIX) {
            return Err(MutationError::ReservedParameter(name.clone()));
        }
    }
    let syntax = turso_graph_cypher::parse(source)?;
    let parameter_types = parameter_types(parameters);
    let bound = bind_mutation(&syntax, graph, catalog.as_ref(), &parameter_types)?;
    let input = match &bound.request.input {
        Some(plan) => lower_mutation_input(plan, catalog.as_ref())?,
        None => unit_mutation_input(),
    };

    connection.execute(format!("SAVEPOINT {SAVEPOINT}"))?;
    let result = execute_bound(connection, catalog.as_ref(), &bound, &input, parameters).map(
        |mut summary| {
            if !bound.returns_order.is_empty() {
                summary.rows.sort_by(|left, right| {
                    for (index, descending) in &bound.returns_order {
                        let ordering = compare_returned_values(&left[*index], &right[*index]);
                        let ordering = if *descending {
                            ordering.reverse()
                        } else {
                            ordering
                        };
                        if ordering != std::cmp::Ordering::Equal {
                            return ordering;
                        }
                    }
                    std::cmp::Ordering::Equal
                });
            }
            if !bound.returns_order.is_empty() {
                for row in &mut summary.rows {
                    row.truncate(bound.returns_visible);
                }
            }
            if bound.returns_distinct {
                let mut seen = std::collections::HashSet::new();
                summary.rows.retain(|row| seen.insert(format!("{row:?}")));
            }
            if let Some(skip) = bound.returns_skip {
                summary.rows.drain(..skip.min(summary.rows.len()));
            }
            if let Some(limit) = bound.returns_limit {
                summary.rows.truncate(limit);
            }
            summary
        },
    );
    match result {
        Ok(summary) => {
            connection.execute(format!("RELEASE {SAVEPOINT}"))?;
            Ok(summary)
        }
        Err(cause) => {
            let rollback = connection
                .execute(format!("ROLLBACK TO {SAVEPOINT}"))
                .and_then(|()| connection.execute(format!("RELEASE {SAVEPOINT}")));
            match rollback {
                Ok(()) => Err(cause),
                Err(rollback) => Err(MutationError::RollbackFailed {
                    cause: Box::new(cause),
                    rollback,
                }),
            }
        }
    }
}

/// Cypher ORDER BY comparison over returned SQL values: numbers before
/// text, text before blobs, null last ascending.
fn compare_returned_values(left: &Value, right: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    fn rank(value: &Value) -> u8 {
        match value {
            Value::Numeric(_) => 0,
            Value::Text(_) => 1,
            Value::Blob(_) => 2,
            Value::Null => 3,
        }
    }
    match (left, right) {
        (Value::Numeric(left), Value::Numeric(right)) => {
            let left = match left {
                Numeric::Integer(value) => *value as f64,
                Numeric::Float(value) => f64::from(*value),
            };
            let right = match right {
                Numeric::Integer(value) => *value as f64,
                Numeric::Float(value) => f64::from(*value),
            };
            left.partial_cmp(&right).unwrap_or(Ordering::Equal)
        }
        (Value::Text(left), Value::Text(right)) => left.value.cmp(&right.value),
        _ => rank(left).cmp(&rank(right)),
    }
}

fn execute_bound(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    bound: &BoundMutation,
    input: &LoweredMutationInput,
    parameters: &Parameters,
) -> Result<MutationSummary, MutationError> {
    let request = &bound.request;
    let input_bindings = request
        .input
        .as_ref()
        .map(|plan| {
            plan.scope()
                .iter()
                // Named paths have no backing column in the input plan;
                // projections rebuild them from their component bindings.
                .filter(|binding| !matches!(binding.value_type(), ir::ValueType::Path))
                .map(ir::Binding::id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let initial = if request.input.is_some() {
        let sql = mutation_rows_sql(input, &input_bindings);
        run_rows(connection, &sql, parameters, &HashMap::new())?
    } else {
        vec![Vec::new()]
    };
    let matched_rows = initial.len() as u64;
    let mut operations_executed = 0_u64;
    // Every binding kind is known at bind time, so relational layouts for
    // projected entities can be resolved up front.
    let mut entity_layouts: HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)> =
        HashMap::new();
    for (id, kind) in &bound.entity_kinds {
        let (source, kind) = match kind {
            crate::CatalogEntity::Node => {
                (catalog.node_source(request.graph), MutationEntityKind::Node)
            }
            crate::CatalogEntity::Relationship => (
                catalog.relationship_source(request.graph),
                MutationEntityKind::Relationship,
            ),
        };
        if let Some(source) = source {
            entity_layouts.insert(*id, (source, kind));
        }
    }
    let mut rows: Vec<HashMap<ir::BindingId, Value>> = initial
        .into_iter()
        .map(|row| input_bindings.iter().copied().zip(row).collect())
        .collect();
    for values in &mut rows {
        for operation in &request.operations {
            execute_operation(
                connection,
                catalog,
                request.graph,
                input,
                operation,
                parameters,
                values,
                &mut entity_layouts,
            )?;
            operations_executed += 1;
        }
    }
    for stage in &bound.stages {
        rows = project_stage(
            connection,
            catalog,
            input,
            parameters,
            &stage.projections,
            stage.predicate.as_ref(),
            stage.distinct,
            rows,
            &entity_layouts,
        )?;
        rows = sort_stage_rows(
            connection,
            catalog,
            input,
            parameters,
            &stage.order,
            rows,
            &entity_layouts,
        )?;
        if let Some(skip) = stage.skip {
            rows.drain(..skip.min(rows.len()));
        }
        if let Some(limit) = stage.limit {
            rows.truncate(limit);
        }
        for item in &stage.items {
            match item {
                StageItem::Operation(operation) => {
                    for values in &mut rows {
                        execute_operation(
                            connection,
                            catalog,
                            request.graph,
                            input,
                            operation,
                            parameters,
                            values,
                            &mut entity_layouts,
                        )?;
                        operations_executed += 1;
                    }
                }
                StageItem::Foreach { .. } => {
                    for values in &mut rows {
                        run_stage_items_once(
                            connection,
                            catalog,
                            request.graph,
                            input,
                            std::slice::from_ref(item),
                            parameters,
                            values,
                            &mut entity_layouts,
                            &mut operations_executed,
                        )?;
                    }
                }
                StageItem::Unwind { output, list } => {
                    let mut expanded = Vec::new();
                    for values in &rows {
                        let references = reference_parameters(values);
                        let sql = lower_mutation_expression(
                            list,
                            input,
                            catalog,
                            &references.sql,
                            &entity_layouts,
                        )?;
                        let elements = run_rows(
                            connection,
                            &format!("SELECT value FROM json_each(({sql}))"),
                            parameters,
                            &references.values,
                        )?;
                        for mut element in elements {
                            let Some(element) = element.pop() else {
                                continue;
                            };
                            let mut next = values.clone();
                            next.insert(*output, element);
                            expanded.push(next);
                        }
                    }
                    rows = expanded;
                }
                StageItem::Match {
                    plan,
                    outputs,
                    optional,
                } => {
                    let lowered = lower_mutation_input(plan, catalog)?;
                    let sql = mutation_rows_sql(&lowered, outputs);
                    let mut expanded = Vec::new();
                    for values in &rows {
                        // Correlated plans reference the row's bindings
                        // through internal reference parameters.
                        let references = reference_parameters(values);
                        let matched = run_rows(connection, &sql, parameters, &references.values)?;
                        if matched.is_empty() && *optional {
                            let mut next = values.clone();
                            for binding in outputs {
                                next.insert(*binding, Value::Null);
                            }
                            expanded.push(next);
                            continue;
                        }
                        for matched_row in &matched {
                            let mut next = values.clone();
                            for (binding, value) in outputs.iter().zip(matched_row) {
                                next.insert(*binding, value.clone());
                            }
                            expanded.push(next);
                        }
                    }
                    rows = expanded;
                }
            }
        }
    }
    let returned_rows = if bound.returns.is_empty() {
        Vec::new()
    } else {
        let projected = project_stage(
            connection,
            catalog,
            input,
            parameters,
            &bound.returns,
            None,
            false,
            rows,
            &entity_layouts,
        )?;
        let order: Vec<ir::BindingId> = bound
            .returns
            .iter()
            .map(|projection| match projection {
                StageProjection::Expression { output, .. }
                | StageProjection::Aggregate { output, .. } => *output,
            })
            .collect();
        projected
            .into_iter()
            .map(|values| {
                order
                    .iter()
                    .map(|output| values.get(output).cloned().unwrap_or(Value::Null))
                    .collect()
            })
            .collect()
    };
    Ok(MutationSummary {
        matched_rows,
        operations_executed,
        rows: returned_rows,
        result_types: bound.return_types.clone(),
    })
}

/// Runs stage items against one row's values. FOREACH evaluates its list,
/// then runs its nested items once per element on a scratch copy of the row,
/// leaving the surrounding row set unchanged.
#[allow(clippy::too_many_arguments)]
fn run_stage_items_once(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    graph: ir::GraphId,
    input: &LoweredMutationInput,
    items: &[StageItem],
    parameters: &Parameters,
    values: &mut HashMap<ir::BindingId, Value>,
    entity_layouts: &mut HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
    operations_executed: &mut u64,
) -> Result<(), MutationError> {
    for item in items {
        match item {
            StageItem::Operation(operation) => {
                execute_operation(
                    connection,
                    catalog,
                    graph,
                    input,
                    operation,
                    parameters,
                    values,
                    entity_layouts,
                )?;
                *operations_executed += 1;
            }
            StageItem::Foreach {
                output,
                list,
                items,
            } => {
                let references = reference_parameters(values);
                let sql = lower_mutation_expression(
                    list,
                    input,
                    catalog,
                    &references.sql,
                    entity_layouts,
                )?;
                let elements = run_rows(
                    connection,
                    &format!("SELECT value FROM json_each(({sql}))"),
                    parameters,
                    &references.values,
                )?;
                for mut element in elements {
                    let Some(element) = element.pop() else {
                        continue;
                    };
                    let mut scratch = values.clone();
                    scratch.insert(*output, element);
                    run_stage_items_once(
                        connection,
                        catalog,
                        graph,
                        input,
                        items,
                        parameters,
                        &mut scratch,
                        entity_layouts,
                        operations_executed,
                    )?;
                }
            }
            StageItem::Unwind { .. } | StageItem::Match { .. } => {
                return Err(MutationError::Database(
                    turso_core::LimboError::InternalError(
                        "UNWIND and MATCH are stage-level, not FOREACH-level".to_owned(),
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Evaluates a stage's projections over the row set: plain expressions map
/// row-by-row, aggregates fold with implicit Cypher grouping over the plain
/// items, then the optional predicate and DISTINCT apply to the output rows.
#[allow(clippy::too_many_arguments)]
fn project_stage(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    input: &LoweredMutationInput,
    parameters: &Parameters,
    projections: &[StageProjection],
    predicate: Option<&ir::TypedExpression>,
    distinct: bool,
    rows: Vec<HashMap<ir::BindingId, Value>>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
) -> Result<Vec<HashMap<ir::BindingId, Value>>, MutationError> {
    let has_aggregates = projections
        .iter()
        .any(|projection| matches!(projection, StageProjection::Aggregate { .. }));
    // Evaluate every projection input per row in a single SELECT.
    let mut evaluated: Vec<Vec<Value>> = Vec::with_capacity(rows.len());
    for values in &rows {
        let references = reference_parameters(values);
        let columns = projections
            .iter()
            .map(|projection| {
                let expression = match projection {
                    StageProjection::Expression { expression, .. } => Some(expression),
                    StageProjection::Aggregate { argument, .. } => argument.as_ref(),
                };
                match expression {
                    Some(expression) => lower_mutation_expression(
                        expression,
                        input,
                        catalog,
                        &references.sql,
                        entity_layouts,
                    )
                    .map(|sql| format!("({sql})")),
                    // count(*) has no argument; any placeholder counts.
                    None => Ok("1".to_owned()),
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let mut produced = run_rows(
            connection,
            &format!("SELECT {columns}"),
            parameters,
            &references.values,
        )?;
        evaluated.push(produced.pop().unwrap_or_default());
    }
    let mut output_rows: Vec<HashMap<ir::BindingId, Value>> = if has_aggregates {
        let key_positions: Vec<usize> = projections
            .iter()
            .enumerate()
            .filter(|(_, projection)| matches!(projection, StageProjection::Expression { .. }))
            .map(|(position, _)| position)
            .collect();
        let mut groups: Vec<(String, Vec<Vec<Value>>)> = Vec::new();
        for row in evaluated {
            let key = key_positions
                .iter()
                .map(|position| format!("{:?}", row[*position]))
                .collect::<Vec<_>>()
                .join("\u{1}");
            match groups.iter_mut().find(|(existing, _)| *existing == key) {
                Some((_, members)) => members.push(row),
                None => groups.push((key, vec![row])),
            }
        }
        let mut output = Vec::new();
        for (_, members) in groups {
            let mut values = HashMap::new();
            for (position, projection) in projections.iter().enumerate() {
                match projection {
                    StageProjection::Expression { output: id, .. } => {
                        values.insert(*id, members[0][position].clone());
                    }
                    StageProjection::Aggregate {
                        output: id,
                        function,
                        argument,
                        distinct,
                    } => {
                        let collected: Vec<Value> = members
                            .iter()
                            .map(|member| member[position].clone())
                            .collect();
                        values.insert(
                            *id,
                            fold_aggregate(
                                connection,
                                parameters,
                                *function,
                                argument.is_none(),
                                collected,
                                *distinct,
                            )?,
                        );
                    }
                }
            }
            output.push(values);
        }
        output
    } else {
        evaluated
            .into_iter()
            .map(|row| {
                projections
                    .iter()
                    .zip(row)
                    .map(|(projection, value)| match projection {
                        StageProjection::Expression { output, .. }
                        | StageProjection::Aggregate { output, .. } => (*output, value),
                    })
                    .collect()
            })
            .collect()
    };
    if let Some(predicate) = predicate {
        let mut kept = Vec::new();
        for values in output_rows {
            let references = reference_parameters(&values);
            let sql = lower_mutation_expression(
                predicate,
                input,
                catalog,
                &references.sql,
                entity_layouts,
            )?;
            let result = run_rows(
                connection,
                &format!("SELECT ({sql})"),
                parameters,
                &references.values,
            )?;
            let truthy = matches!(
                result.first().and_then(|row| row.first()),
                Some(Value::Numeric(Numeric::Integer(value))) if *value != 0
            );
            if truthy {
                kept.push(values);
            }
        }
        output_rows = kept;
    }
    if distinct {
        let mut seen = Vec::new();
        output_rows.retain(|values| {
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_by_key(|(id, _)| id.get());
            let key = format!("{entries:?}");
            if seen.contains(&key) {
                false
            } else {
                seen.push(key);
                true
            }
        });
    }
    Ok(output_rows)
}

/// Applies a stage's `ORDER BY` to its already-projected (and
/// DISTINCT-filtered) rows: each sort key evaluates once per row against
/// that row's own bindings, exactly like a stage predicate, so it sees
/// aggregate outputs the same way `WITH ... WHERE` does.
fn sort_stage_rows(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    input: &LoweredMutationInput,
    parameters: &Parameters,
    order: &[(ir::TypedExpression, bool)],
    rows: Vec<HashMap<ir::BindingId, Value>>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
) -> Result<Vec<HashMap<ir::BindingId, Value>>, MutationError> {
    if order.is_empty() {
        return Ok(rows);
    }
    let mut keyed: Vec<(Vec<Value>, HashMap<ir::BindingId, Value>)> =
        Vec::with_capacity(rows.len());
    for values in rows {
        let references = reference_parameters(&values);
        let mut keys = Vec::with_capacity(order.len());
        for (expression, _) in order {
            let sql = lower_mutation_expression(
                expression,
                input,
                catalog,
                &references.sql,
                entity_layouts,
            )?;
            let mut produced = run_rows(
                connection,
                &format!("SELECT ({sql})"),
                parameters,
                &references.values,
            )?;
            keys.push(
                produced
                    .pop()
                    .and_then(|mut row| row.pop())
                    .unwrap_or(Value::Null),
            );
        }
        keyed.push((keys, values));
    }
    keyed.sort_by(|(left, _), (right, _)| {
        for (index, (_, descending)) in order.iter().enumerate() {
            let ordering = compare_returned_values(&left[index], &right[index]);
            let ordering = if *descending {
                ordering.reverse()
            } else {
                ordering
            };
            if ordering != std::cmp::Ordering::Equal {
                return ordering;
            }
        }
        std::cmp::Ordering::Equal
    });
    Ok(keyed.into_iter().map(|(_, values)| values).collect())
}

fn fold_aggregate(
    connection: &Arc<Connection>,
    parameters: &Parameters,
    function: ir::AggregateFunction,
    count_star: bool,
    mut values: Vec<Value>,
    distinct: bool,
) -> Result<Value, MutationError> {
    if distinct {
        let mut seen = Vec::new();
        values.retain(|value| {
            let key = format!("{value:?}");
            if seen.contains(&key) {
                false
            } else {
                seen.push(key);
                true
            }
        });
    }
    let non_null: Vec<&Value> = values
        .iter()
        .filter(|value| !matches!(value, Value::Null))
        .collect();
    let as_float = |value: &Value| match value {
        Value::Numeric(Numeric::Integer(value)) => Some(*value as f64),
        Value::Numeric(Numeric::Float(value)) => Some(f64::from(*value)),
        _ => None,
    };
    let float_value = |value: f64| match turso_core::NonNan::new(value) {
        Some(value) => Value::Numeric(Numeric::Float(value)),
        None => Value::Null,
    };
    Ok(match function {
        ir::AggregateFunction::Count => {
            let count = if count_star {
                values.len()
            } else {
                non_null.len()
            };
            Value::Numeric(Numeric::Integer(count as i64))
        }
        ir::AggregateFunction::Sum => {
            let all_integers = non_null
                .iter()
                .all(|value| matches!(value, Value::Numeric(Numeric::Integer(_))));
            if all_integers {
                Value::Numeric(Numeric::Integer(
                    non_null
                        .iter()
                        .filter_map(|value| match value {
                            Value::Numeric(Numeric::Integer(value)) => Some(*value),
                            _ => None,
                        })
                        .sum(),
                ))
            } else {
                float_value(non_null.iter().filter_map(|value| as_float(value)).sum())
            }
        }
        ir::AggregateFunction::Average => {
            if non_null.is_empty() {
                Value::Null
            } else {
                let total: f64 = non_null.iter().filter_map(|value| as_float(value)).sum();
                float_value(total / non_null.len() as f64)
            }
        }
        ir::AggregateFunction::Minimum | ir::AggregateFunction::Maximum => {
            let want_minimum = function == ir::AggregateFunction::Minimum;
            let mut best: Option<&Value> = None;
            for value in &non_null {
                let better = match best {
                    None => true,
                    Some(current) => {
                        let ordering = match (as_float(current), as_float(value)) {
                            (Some(left), Some(right)) => right.partial_cmp(&left),
                            _ => match (current, value) {
                                (Value::Text(left), Value::Text(right)) => {
                                    Some(right.as_str().cmp(left.as_str()))
                                }
                                _ => None,
                            },
                        };
                        matches!(
                            ordering,
                            Some(std::cmp::Ordering::Less) if want_minimum
                        ) || matches!(
                            ordering,
                            Some(std::cmp::Ordering::Greater) if !want_minimum
                        )
                    }
                };
                if better {
                    best = Some(value);
                }
            }
            best.cloned().unwrap_or(Value::Null)
        }
        ir::AggregateFunction::Collect => {
            // Build the JSON array in SQL so element encoding matches the
            // read path's json_group_array output.
            if non_null.is_empty() {
                Value::Text("[]".to_owned().into())
            } else {
                let mut internal = HashMap::new();
                let selects = non_null
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        let name = format!("{INTERNAL_PARAMETER_PREFIX}collect_{index}");
                        let select = format!("SELECT ${name} AS value");
                        internal.insert(name, (*value).clone());
                        select
                    })
                    .collect::<Vec<_>>()
                    .join(" UNION ALL ");
                run_rows(
                    connection,
                    &format!("SELECT json_group_array(value) FROM ({selects})"),
                    parameters,
                    &internal,
                )?
                .pop()
                .and_then(|mut row| row.pop())
                .unwrap_or(Value::Null)
            }
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn execute_operation(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    graph: ir::GraphId,
    input: &LoweredMutationInput,
    operation: &ir::Mutation,
    parameters: &Parameters,
    values: &mut HashMap<ir::BindingId, Value>,
    entity_layouts: &mut HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
) -> Result<(), MutationError> {
    match operation {
        ir::Mutation::CreateNode(create) => {
            let (identity, _) = insert_node(
                connection,
                catalog,
                input,
                create,
                parameters,
                values,
                entity_layouts,
                false,
            )?;
            record_node_labels(connection, catalog, &create.labels, &identity, parameters)?;
            values.insert(create.binding.id(), identity);
            entity_layouts.insert(
                create.binding.id(),
                (create.source, MutationEntityKind::Node),
            );
        }
        ir::Mutation::MergeNode(merge) => {
            let (identity, created) = insert_node(
                connection,
                catalog,
                input,
                &merge.create,
                parameters,
                values,
                entity_layouts,
                true,
            )?;
            record_node_labels(
                connection,
                catalog,
                &merge.create.labels,
                &identity,
                parameters,
            )?;
            values.insert(merge.create.binding.id(), identity);
            entity_layouts.insert(
                merge.create.binding.id(),
                (merge.create.source, MutationEntityKind::Node),
            );
            let actions = if created {
                &merge.on_create
            } else {
                &merge.on_match
            };
            for action in actions {
                execute_operation(
                    connection,
                    catalog,
                    graph,
                    input,
                    action,
                    parameters,
                    values,
                    entity_layouts,
                )?;
            }
        }
        ir::Mutation::CreateRelationship(create) => {
            let (identity, _) = insert_relationship(
                connection,
                catalog,
                input,
                create,
                parameters,
                values,
                entity_layouts,
                false,
            )?;
            record_relationship_type(
                connection,
                catalog,
                &create.relationship_types,
                &identity,
                parameters,
            )?;
            values.insert(create.binding.id(), identity);
            entity_layouts.insert(
                create.binding.id(),
                (create.source, MutationEntityKind::Relationship),
            );
        }
        ir::Mutation::MergeRelationship(merge) => {
            let (identity, created) = insert_relationship(
                connection,
                catalog,
                input,
                &merge.create,
                parameters,
                values,
                entity_layouts,
                true,
            )?;
            if created {
                // A pure match already carries the requested types (the merge
                // predicates require them); recording here would attach types
                // to a relationship the merge did not create.
                record_relationship_type(
                    connection,
                    catalog,
                    &merge.create.relationship_types,
                    &identity,
                    parameters,
                )?;
            }
            values.insert(merge.create.binding.id(), identity);
            entity_layouts.insert(
                merge.create.binding.id(),
                (merge.create.source, MutationEntityKind::Relationship),
            );
            let actions = if created {
                &merge.on_create
            } else {
                &merge.on_match
            };
            for action in actions {
                execute_operation(
                    connection,
                    catalog,
                    graph,
                    input,
                    action,
                    parameters,
                    values,
                    entity_layouts,
                )?;
            }
        }
        ir::Mutation::SetProperty(set) => {
            let layout = entity_table(catalog, set.source)?;
            let column = property_column(catalog, set.source, &set.semantic_types, set.property)?;
            let references = reference_parameters(values);
            let expression = lower_mutation_expression(
                &set.value,
                input,
                catalog,
                &references.sql,
                entity_layouts,
            )?;
            let identity = values
                .get(&set.entity)
                .ok_or(MutationError::MissingBinding(set.entity))?;
            let mut internal = references.values;
            internal.insert(identity_parameter(set.entity), identity.clone());
            let assignment = if set.value.value_type == ir::ValueType::Any {
                let evaluated = evaluate_scalar(connection, &expression, parameters, &internal)?;
                check_runtime_value(
                    catalog,
                    set.source,
                    &set.semantic_types,
                    set.property,
                    &evaluated,
                )?;
                let value_parameter = format!(
                    "{INTERNAL_PARAMETER_PREFIX}set_property_{}",
                    set.entity.get()
                );
                internal.insert(value_parameter.clone(), evaluated);
                format!("${value_parameter}")
            } else {
                expression
            };
            run_ignore(
                connection,
                &format!(
                    "UPDATE {} SET {} = {assignment} WHERE {} = ${}",
                    quoted_identifier(&layout.table),
                    quoted_identifier(&column),
                    quoted_identifier(&layout.identity),
                    identity_parameter(set.entity),
                ),
                parameters,
                &internal,
            )?;
        }
        ir::Mutation::SetLabels(set) => {
            let identity = values
                .get(&set.entity)
                .ok_or(MutationError::MissingBinding(set.entity))?
                .clone();
            record_node_labels(connection, catalog, &set.labels, &identity, parameters)?;
        }
        ir::Mutation::ReplaceProperties(replace) => {
            let layout = entity_table(catalog, replace.source)?;
            let references = reference_parameters(values);
            let identity = values
                .get(&replace.entity)
                .ok_or(MutationError::MissingBinding(replace.entity))?;
            let mut internal = references.values;
            internal.insert(identity_parameter(replace.entity), identity.clone());
            let mut assignments = Vec::new();
            let mut assigned_columns = Vec::new();
            for entry in &replace.entries {
                let column = property_column(
                    catalog,
                    replace.source,
                    &entry.semantic_types,
                    entry.property,
                )?;
                let expression = lower_mutation_expression(
                    &entry.value,
                    input,
                    catalog,
                    &references.sql,
                    entity_layouts,
                )?;
                let assignment = if entry.value.value_type == ir::ValueType::Any {
                    let evaluated =
                        evaluate_scalar(connection, &expression, parameters, &internal)?;
                    check_runtime_value(
                        catalog,
                        replace.source,
                        &entry.semantic_types,
                        entry.property,
                        &evaluated,
                    )?;
                    let value_parameter = format!(
                        "{INTERNAL_PARAMETER_PREFIX}replace_{}_{}",
                        replace.entity.get(),
                        entry.property.get()
                    );
                    internal.insert(value_parameter.clone(), evaluated);
                    format!("${value_parameter}")
                } else {
                    expression
                };
                assignments.push(format!("{} = {assignment}", quoted_identifier(&column)));
                assigned_columns.push(column);
            }
            if replace.clear {
                // `SET n = map` wipes every payload column the map omits.
                if let Some(properties) = GraphCompilationCatalog::semantic_properties(
                    catalog,
                    replace.source,
                    &replace.semantic_types,
                ) {
                    for (_, _, _, column) in properties {
                        if !assigned_columns.contains(&column) {
                            assignments.push(format!("{} = NULL", quoted_identifier(&column)));
                        }
                    }
                } else {
                    let mut structural = vec![layout.identity.clone()];
                    if let Some(relationship) = catalog.relationship_layout(replace.source) {
                        structural.push(relationship.start_column);
                        structural.push(relationship.end_column);
                    }
                    let escaped = layout.table.replace('\'', "''");
                    let columns = run_rows(
                        connection,
                        &format!("SELECT name FROM pragma_table_info('{escaped}')"),
                        parameters,
                        &HashMap::new(),
                    )?;
                    for row in columns {
                        let Some(Value::Text(name)) = row.first() else {
                            continue;
                        };
                        let name = name.to_string();
                        if structural.contains(&name) || assigned_columns.contains(&name) {
                            continue;
                        }
                        assignments.push(format!("{} = NULL", quoted_identifier(&name)));
                    }
                }
            }
            if !assignments.is_empty() {
                run_ignore(
                    connection,
                    &format!(
                        "UPDATE {} SET {} WHERE {} = ${}",
                        quoted_identifier(&layout.table),
                        assignments.join(", "),
                        quoted_identifier(&layout.identity),
                        identity_parameter(replace.entity),
                    ),
                    parameters,
                    &internal,
                )?;
            }
        }
        ir::Mutation::ReplacePropertiesDynamic(replace) => {
            let layout = entity_table(catalog, replace.source)?;
            let references = reference_parameters(values);
            let identity = values
                .get(&replace.entity)
                .ok_or(MutationError::MissingBinding(replace.entity))?;
            let mut internal = references.values;
            internal.insert(identity_parameter(replace.entity), identity.clone());
            let value = lower_mutation_expression(
                &replace.value,
                input,
                catalog,
                &references.sql,
                entity_layouts,
            )?;
            let evaluated = evaluate_scalar(connection, &value, parameters, &internal)?;
            let map_parameter = format!(
                "{INTERNAL_PARAMETER_PREFIX}replace_map_{}",
                replace.entity.get()
            );
            internal.insert(map_parameter.clone(), evaluated);
            if let Some(owned_properties) = GraphCompilationCatalog::semantic_properties(
                catalog,
                replace.source,
                &replace.semantic_types,
            ) {
                let map_rows = run_rows(
                    connection,
                    &format!("SELECT key, value FROM json_each(${map_parameter})"),
                    parameters,
                    &internal,
                )?;
                let mut updates = HashMap::<String, Value>::new();
                for row in map_rows {
                    let (Some(Value::Text(key)), Some(value)) = (row.first(), row.get(1)) else {
                        return Err(MutationError::UnknownDynamicKey {
                            key: "<non-text>".to_owned(),
                        });
                    };
                    let key = key.to_string();
                    let Some((property_name, expected, column)) =
                        GraphCompilationCatalog::semantic_property_for_key(
                            catalog,
                            replace.source,
                            &replace.semantic_types,
                            &key,
                        )
                        .flatten()
                    else {
                        return Err(MutationError::UnknownDynamicKey { key });
                    };
                    check_runtime_value_against(&property_name, &expected, value)?;
                    updates.insert(column, value.clone());
                }
                let mut assignments = Vec::new();
                for (property, _, _, column) in owned_properties {
                    let value = match updates.remove(&column) {
                        Some(value) => value,
                        None if replace.clear => Value::Null,
                        None => continue,
                    };
                    let value_parameter = format!(
                        "{INTERNAL_PARAMETER_PREFIX}dynamic_{}_{}",
                        replace.entity.get(),
                        property.get()
                    );
                    internal.insert(value_parameter.clone(), value);
                    assignments.push(format!(
                        "{} = ${value_parameter}",
                        quoted_identifier(&column)
                    ));
                }
                if !assignments.is_empty() {
                    run_ignore(
                        connection,
                        &format!(
                            "UPDATE {} SET {} WHERE {} = ${}",
                            quoted_identifier(&layout.table),
                            assignments.join(", "),
                            quoted_identifier(&layout.identity),
                            identity_parameter(replace.entity),
                        ),
                        parameters,
                        &internal,
                    )?;
                }
                return Ok(());
            }
            let columns =
                catalog
                    .payload_columns(replace.source)
                    .ok_or(LowerError::UnsupportedOperator(
                        "whole-entity SET without payload columns",
                    ))?;
            let assignments = columns
                .iter()
                .map(|(logical, physical)| {
                    let path =
                        format!("$.\"{}\"", logical.replace('"', "\\\"").replace('\'', "''"));
                    let extract = format!("json_extract(${map_parameter}, '{path}')");
                    if replace.clear {
                        format!("{} = {extract}", quoted_identifier(physical))
                    } else {
                        format!(
                            "{} = coalesce({extract}, {})",
                            quoted_identifier(physical),
                            quoted_identifier(physical)
                        )
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            if !assignments.is_empty() {
                run_ignore(
                    connection,
                    &format!(
                        "UPDATE {} SET {assignments} WHERE {} = ${}",
                        quoted_identifier(&layout.table),
                        quoted_identifier(&layout.identity),
                        identity_parameter(replace.entity),
                    ),
                    parameters,
                    &internal,
                )?;
            }
        }
        ir::Mutation::RemoveProperty(remove) => {
            let layout = entity_table(catalog, remove.source)?;
            let column = property_column(
                catalog,
                remove.source,
                &remove.semantic_types,
                remove.property,
            )?;
            let identity = values
                .get(&remove.entity)
                .ok_or(MutationError::MissingBinding(remove.entity))?;
            let internal = HashMap::from([(identity_parameter(remove.entity), identity.clone())]);
            run_ignore(
                connection,
                &format!(
                    "UPDATE {} SET {} = NULL WHERE {} = ${}",
                    quoted_identifier(&layout.table),
                    quoted_identifier(&column),
                    quoted_identifier(&layout.identity),
                    identity_parameter(remove.entity),
                ),
                parameters,
                &internal,
            )?;
        }
        ir::Mutation::Delete(delete) => {
            delete_entity(connection, catalog, graph, delete, parameters, values)?;
        }
    }
    Ok(())
}

/// Records a created or merged node's labels in the graph's junction
/// table; idempotent so MERGE matches do not duplicate rows.
fn record_node_labels(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    labels: &[ir::LabelId],
    identity: &Value,
    parameters: &Parameters,
) -> Result<(), MutationError> {
    let Some(table) = catalog.labels_table() else {
        return Ok(());
    };
    let table = table.replace('"', "\"\"");
    for label in labels {
        let Some(name) = catalog.label_name(*label) else {
            continue;
        };
        let name = name.replace('\'', "''");
        let parameter = format!("{INTERNAL_PARAMETER_PREFIX}label_node");
        let internal = HashMap::from([(parameter.clone(), identity.clone())]);
        run_ignore(
            connection,
            &format!(
                "INSERT INTO \"{table}\"(node_id, label) SELECT ${parameter}, '{name}' \
                 WHERE NOT EXISTS (SELECT 1 FROM \"{table}\" \
                 WHERE node_id = ${parameter} AND label = '{name}')"
            ),
            parameters,
            &internal,
        )?;
    }
    Ok(())
}

/// Records a created or merged relationship's type in the graph's junction
/// table; idempotent so MERGE matches do not duplicate rows.
fn record_relationship_type(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    relationship_types: &[ir::RelationshipTypeId],
    identity: &Value,
    parameters: &Parameters,
) -> Result<(), MutationError> {
    let Some(table) = catalog.relationship_types_table() else {
        return Ok(());
    };
    let table = table.replace('"', "\"\"");
    for relationship_type in relationship_types {
        let Some(name) = catalog.relationship_type_name(*relationship_type) else {
            continue;
        };
        let name = name.replace('\'', "''");
        let parameter = format!("{INTERNAL_PARAMETER_PREFIX}type_relationship");
        let internal = HashMap::from([(parameter.clone(), identity.clone())]);
        run_ignore(
            connection,
            &format!(
                "INSERT INTO \"{table}\"(relationship_id, type) SELECT ${parameter}, '{name}' \
                 WHERE NOT EXISTS (SELECT 1 FROM \"{table}\" \
                 WHERE relationship_id = ${parameter} AND type = '{name}')"
            ),
            parameters,
            &internal,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_node(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    input: &LoweredMutationInput,
    create: &ir::CreateNode,
    parameters: &Parameters,
    values: &HashMap<ir::BindingId, Value>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
    merge: bool,
) -> Result<(Value, bool), MutationError> {
    let layout = catalog
        .node_layout(create.source)
        .ok_or(LowerError::MissingSource(create.source))?;
    let merge_predicates = if merge {
        node_label_predicates(
            catalog,
            &layout.table,
            &layout.identity_column,
            &create.labels,
        )
    } else {
        Vec::new()
    };
    insert_entity(
        connection,
        catalog,
        input,
        &layout.table,
        &layout.identity_column,
        create.source,
        &create.properties,
        parameters,
        values,
        entity_layouts,
        merge,
        "node",
        &[],
        &merge_predicates,
    )
}

/// Type-membership predicates for a relationship merge match against the
/// type junction table; empty when the graph records no relationship types.
/// Without these, MERGE would match a relationship of a different type
/// between the same endpoints and attach the requested type to it.
fn relationship_type_predicates(
    catalog: &dyn GraphCompilationCatalog,
    table: &str,
    identity: &str,
    relationship_types: &[ir::RelationshipTypeId],
) -> Vec<String> {
    let Some(junction) = catalog.relationship_types_table() else {
        return Vec::new();
    };
    relationship_types
        .iter()
        .filter_map(|relationship_type| catalog.relationship_type_name(*relationship_type))
        .map(|name| {
            format!(
                "EXISTS (SELECT 1 FROM {} WHERE relationship_id = {}.{} AND type = '{}')",
                quoted_identifier(&junction),
                quoted_identifier(table),
                quoted_identifier(identity),
                name.replace('\'', "''"),
            )
        })
        .collect()
}

/// Label-membership predicates for a merge match against the label
/// junction table; empty when the graph records no labels.
fn node_label_predicates(
    catalog: &dyn GraphCompilationCatalog,
    table: &str,
    identity: &str,
    labels: &[ir::LabelId],
) -> Vec<String> {
    let Some(junction) = catalog.labels_table() else {
        return Vec::new();
    };
    labels
        .iter()
        .filter_map(|label| catalog.label_name(*label))
        .map(|name| {
            format!(
                "EXISTS (SELECT 1 FROM {} WHERE node_id = {}.{} AND label = '{}')",
                quoted_identifier(&junction),
                quoted_identifier(table),
                quoted_identifier(identity),
                name.replace('\'', "''"),
            )
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn insert_relationship(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    input: &LoweredMutationInput,
    create: &ir::CreateRelationship,
    parameters: &Parameters,
    values: &HashMap<ir::BindingId, Value>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
    merge: bool,
) -> Result<(Value, bool), MutationError> {
    let layout = catalog
        .relationship_layout(create.source)
        .ok_or(LowerError::MissingSource(create.source))?;
    let from = values
        .get(&create.from)
        .ok_or(MutationError::MissingBinding(create.from))?;
    let to = values
        .get(&create.to)
        .ok_or(MutationError::MissingBinding(create.to))?;
    let merge_predicates = if merge {
        relationship_type_predicates(
            catalog,
            &layout.table,
            &layout.identity_column,
            &create.relationship_types,
        )
    } else {
        Vec::new()
    };
    insert_entity(
        connection,
        catalog,
        input,
        &layout.table,
        &layout.identity_column,
        create.source,
        &create.properties,
        parameters,
        values,
        entity_layouts,
        merge,
        "relationship",
        &[
            (layout.start_column, from.clone()),
            (layout.end_column, to.clone()),
        ],
        &merge_predicates,
    )
}

#[allow(clippy::too_many_arguments)]
fn insert_entity(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    input: &LoweredMutationInput,
    table: &str,
    identity: &str,
    source: ir::SourceTableId,
    properties: &[ir::PropertyValue],
    parameters: &Parameters,
    values: &HashMap<ir::BindingId, Value>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
    merge: bool,
    entity: &'static str,
    fixed: &[(String, Value)],
    merge_predicates: &[String],
) -> Result<(Value, bool), MutationError> {
    let references = reference_parameters(values);
    let mut columns = Vec::new();
    let mut expressions = Vec::new();
    let mut internal = references.values;
    for (index, (column, value)) in fixed.iter().enumerate() {
        let name = format!("{INTERNAL_PARAMETER_PREFIX}fixed_{index}");
        columns.push(column.clone());
        expressions.push(format!("${name}"));
        internal.insert(name, value.clone());
    }
    for property in properties {
        columns.push(property_column(
            catalog,
            source,
            &property.semantic_types,
            property.property,
        )?);
        let expression = lower_mutation_expression(
            &property.value,
            input,
            catalog,
            &references.sql,
            entity_layouts,
        )?;
        if property.value.value_type == ir::ValueType::Any {
            let evaluated = evaluate_scalar(connection, &expression, parameters, &internal)?;
            check_runtime_value(
                catalog,
                source,
                &property.semantic_types,
                property.property,
                &evaluated,
            )?;
            let name = format!(
                "{INTERNAL_PARAMETER_PREFIX}property_{}_{}",
                property.property.get(),
                expressions.len()
            );
            internal.insert(name.clone(), evaluated);
            expressions.push(format!("${name}"));
        } else {
            expressions.push(expression);
        }
    }
    if merge {
        // A propertyless MERGE matches any candidate row (Cypher semantics);
        // label membership arrives through merge_predicates.
        let predicate = columns
            .iter()
            .zip(&expressions)
            .map(|(column, expression)| format!("{} IS ({expression})", quoted_identifier(column)))
            .chain(merge_predicates.iter().cloned())
            .collect::<Vec<_>>()
            .join(" AND ");
        let predicate = if predicate.is_empty() {
            "1 = 1".to_owned()
        } else {
            predicate
        };
        let existing = run_rows(
            connection,
            &format!(
                "SELECT {} FROM {} WHERE {predicate} LIMIT 1",
                quoted_identifier(identity),
                quoted_identifier(table),
            ),
            parameters,
            &internal,
        )?;
        if let Some(value) = existing.first().and_then(|row| row.first()).cloned() {
            return Ok((value, false));
        }
    }
    let sql = if columns.is_empty() {
        format!(
            "INSERT INTO {} DEFAULT VALUES RETURNING {}",
            quoted_identifier(table),
            quoted_identifier(identity),
        )
    } else {
        format!(
            "INSERT INTO {}({}) VALUES ({}) RETURNING {}",
            quoted_identifier(table),
            columns
                .iter()
                .map(|column| quoted_identifier(column))
                .collect::<Vec<_>>()
                .join(", "),
            expressions.join(", "),
            quoted_identifier(identity),
        )
    };
    let rows = run_rows(connection, &sql, parameters, &internal)?;
    rows.into_iter()
        .next()
        .and_then(|mut row| row.pop())
        .map(|identity| (identity, true))
        .ok_or(MutationError::MissingCreatedIdentity { entity })
}

fn delete_entity(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    graph: ir::GraphId,
    delete: &ir::DeleteEntity,
    parameters: &Parameters,
    values: &HashMap<ir::BindingId, Value>,
) -> Result<(), MutationError> {
    let identity = values
        .get(&delete.entity)
        .ok_or(MutationError::MissingBinding(delete.entity))?;
    let internal = HashMap::from([(identity_parameter(delete.entity), identity.clone())]);
    if let Some(layout) = catalog.node_layout(delete.source) {
        for relationship_source in catalog.relationship_sources(graph) {
            let relationship = catalog
                .relationship_layout(relationship_source)
                .ok_or(LowerError::MissingSource(relationship_source))?;
            let parameter = identity_parameter(delete.entity);
            let predicate = format!(
                "{} = ${parameter} OR {} = ${parameter}",
                quoted_identifier(&relationship.start_column),
                quoted_identifier(&relationship.end_column),
            );
            if delete.detach {
                run_ignore(
                    connection,
                    &format!(
                        "DELETE FROM {} WHERE {predicate}",
                        quoted_identifier(&relationship.table)
                    ),
                    parameters,
                    &internal,
                )?;
            } else if !run_rows(
                connection,
                &format!(
                    "SELECT 1 FROM {} WHERE {predicate} LIMIT 1",
                    quoted_identifier(&relationship.table)
                ),
                parameters,
                &internal,
            )?
            .is_empty()
            {
                return Err(MutationError::NodeHasRelationships);
            }
        }
        if let Some(labels_table) = catalog.labels_table() {
            run_ignore(
                connection,
                &format!(
                    "DELETE FROM \"{}\" WHERE node_id = ${}",
                    labels_table.replace('"', "\"\""),
                    identity_parameter(delete.entity),
                ),
                parameters,
                &internal,
            )?;
        }
        Ok(run_ignore(
            connection,
            &format!(
                "DELETE FROM {} WHERE {} = ${}",
                quoted_identifier(&layout.table),
                quoted_identifier(&layout.identity_column),
                identity_parameter(delete.entity),
            ),
            parameters,
            &internal,
        )?)
    } else {
        let layout = catalog
            .relationship_layout(delete.source)
            .ok_or(LowerError::MissingSource(delete.source))?;
        if let Some(types_table) = catalog.relationship_types_table() {
            run_ignore(
                connection,
                &format!(
                    "DELETE FROM \"{}\" WHERE relationship_id = ${}",
                    types_table.replace('"', "\"\""),
                    identity_parameter(delete.entity),
                ),
                parameters,
                &internal,
            )?;
        }
        Ok(run_ignore(
            connection,
            &format!(
                "DELETE FROM {} WHERE {} = ${}",
                quoted_identifier(&layout.table),
                quoted_identifier(&layout.identity_column),
                identity_parameter(delete.entity),
            ),
            parameters,
            &internal,
        )?)
    }
}

struct EntityTable {
    table: String,
    identity: String,
}

fn entity_table(
    catalog: &dyn GraphCompilationCatalog,
    source: ir::SourceTableId,
) -> Result<EntityTable, LowerError> {
    if let Some(layout) = catalog.node_layout(source) {
        Ok(EntityTable {
            table: layout.table,
            identity: layout.identity_column,
        })
    } else if let Some(layout) = catalog.relationship_layout(source) {
        Ok(EntityTable {
            table: layout.table,
            identity: layout.identity_column,
        })
    } else {
        Err(LowerError::MissingSource(source))
    }
}

fn property_column(
    catalog: &dyn GraphCompilationCatalog,
    source: ir::SourceTableId,
    semantic_types: &[String],
    property: ir::PropertyId,
) -> Result<String, LowerError> {
    match GraphCompilationCatalog::semantic_property_for_id(
        catalog,
        source,
        semantic_types,
        property,
    ) {
        Some(Some((_, _, column))) => Ok(column),
        Some(None) => Err(LowerError::MissingProperty {
            source_id: source,
            property,
        }),
        None => catalog
            .property_column(source, property)
            .ok_or(LowerError::MissingProperty {
                source_id: source,
                property,
            }),
    }
}

struct References {
    sql: HashMap<ir::BindingId, String>,
    values: HashMap<String, Value>,
}

fn reference_parameters(values: &HashMap<ir::BindingId, Value>) -> References {
    let mut sql = HashMap::new();
    let mut parameters = HashMap::new();
    for (binding, value) in values {
        let name = identity_parameter(*binding);
        sql.insert(*binding, format!("${name}"));
        parameters.insert(name, value.clone());
    }
    References {
        sql,
        values: parameters,
    }
}

fn identity_parameter(binding: ir::BindingId) -> String {
    format!("{INTERNAL_PARAMETER_PREFIX}{}", binding.get())
}

fn run_ignore(
    connection: &Arc<Connection>,
    sql: &str,
    parameters: &Parameters,
    internal: &HashMap<String, Value>,
) -> Result<(), turso_core::LimboError> {
    let mut statement = connection.prepare(sql)?;
    bind_parameters(&mut statement, parameters, internal)?;
    statement.run_ignore_rows()
}

fn run_rows(
    connection: &Arc<Connection>,
    sql: &str,
    parameters: &Parameters,
    internal: &HashMap<String, Value>,
) -> Result<Vec<Vec<Value>>, turso_core::LimboError> {
    let mut statement = connection.prepare(sql)?;
    bind_parameters(&mut statement, parameters, internal)?;
    statement.run_collect_rows()
}

fn evaluate_scalar(
    connection: &Arc<Connection>,
    expression: &str,
    parameters: &Parameters,
    internal: &HashMap<String, Value>,
) -> Result<Value, MutationError> {
    let mut rows = run_rows(
        connection,
        &format!("SELECT {expression}"),
        parameters,
        internal,
    )?;
    if rows.len() != 1 || rows[0].len() != 1 {
        return Err(MutationError::MissingEvaluatedValue);
    }
    rows.pop()
        .and_then(|mut row| row.pop())
        .ok_or(MutationError::MissingEvaluatedValue)
}

fn bind_parameters(
    statement: &mut turso_core::Statement,
    parameters: &Parameters,
    internal: &HashMap<String, Value>,
) -> Result<(), turso_core::LimboError> {
    for (name, value) in parameters.iter().chain(internal) {
        if let Some(index) = statement.parameter_index(&format!("${name}")) {
            statement.bind_at(index, value.clone())?;
        }
    }
    Ok(())
}

fn parameter_types(parameters: &Parameters) -> ParameterTypes {
    parameters
        .iter()
        .map(|(name, value)| {
            let (value_type, nullability) = match value {
                Value::Null => (ir::ValueType::Any, ir::Nullability::Nullable),
                Value::Numeric(Numeric::Integer(_)) => {
                    (ir::ValueType::Integer, ir::Nullability::NonNull)
                }
                Value::Numeric(Numeric::Float(_)) => {
                    (ir::ValueType::Real, ir::Nullability::NonNull)
                }
                Value::Text(value) if value.value.trim_start().starts_with('{') => {
                    (ir::ValueType::Map, ir::Nullability::NonNull)
                }
                Value::Text(_) => (ir::ValueType::Text, ir::Nullability::NonNull),
                Value::Blob(_) => (ir::ValueType::Bytes, ir::Nullability::NonNull),
            };
            (name.clone(), (value_type, nullability))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CatalogEntity, GraphCatalogSnapshot, NodeTableLayout, RelationalCatalogSnapshot,
        RelationshipTableLayout, ResolvedProperty,
    };
    use turso_core::{Database, MemoryIO, SqliteDialect};

    struct Catalog {
        node_source: ir::SourceTableId,
        relationship_source: ir::SourceTableId,
        relationship_types: &'static [(u32, &'static str)],
        types_table: Option<&'static str>,
    }

    impl GraphCatalogSnapshot for Catalog {
        fn node_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
            Some(self.node_source)
        }

        fn relationship_source(&self, _graph: ir::GraphId) -> Option<ir::SourceTableId> {
            Some(self.relationship_source)
        }

        fn label(&self, _graph: ir::GraphId, name: &str) -> Option<ir::LabelId> {
            (name == "Person").then(|| ir::LabelId::new(1).unwrap())
        }

        fn relationship_type(
            &self,
            _graph: ir::GraphId,
            name: &str,
        ) -> Option<ir::RelationshipTypeId> {
            self.relationship_types
                .iter()
                .find(|(_, candidate)| *candidate == name)
                .map(|(id, _)| ir::RelationshipTypeId::new(*id).unwrap())
        }

        fn property(
            &self,
            _graph: ir::GraphId,
            entity: CatalogEntity,
            name: &str,
        ) -> Option<ResolvedProperty> {
            let (id, value_type, nullability) = match (entity, name) {
                (CatalogEntity::Node, "id") => {
                    (1, ir::ValueType::Integer, ir::Nullability::NonNull)
                }
                (CatalogEntity::Node, "name") => {
                    (2, ir::ValueType::Text, ir::Nullability::Nullable)
                }
                (CatalogEntity::Relationship, "since") => {
                    (3, ir::ValueType::Integer, ir::Nullability::Nullable)
                }
                _ => return None,
            };
            Some(ResolvedProperty {
                id: ir::PropertyId::new(id).unwrap(),
                value_type,
                nullability,
            })
        }
    }

    impl RelationalCatalogSnapshot for Catalog {
        fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout> {
            (source == self.node_source).then(|| NodeTableLayout {
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            })
        }

        fn relationship_layout(
            &self,
            source: ir::SourceTableId,
        ) -> Option<RelationshipTableLayout> {
            (source == self.relationship_source).then(|| RelationshipTableLayout {
                table: "relationships".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
            })
        }

        fn property_column(
            &self,
            source: ir::SourceTableId,
            property: ir::PropertyId,
        ) -> Option<String> {
            match (source, property.get()) {
                (source, 1) if source == self.node_source => Some("id".to_owned()),
                (source, 2) if source == self.node_source => Some("name".to_owned()),
                (source, 3) if source == self.relationship_source => Some("since".to_owned()),
                _ => None,
            }
        }

        fn relationship_types_table(&self) -> Option<String> {
            self.types_table.map(str::to_owned)
        }

        fn relationship_type_name(
            &self,
            relationship_type: ir::RelationshipTypeId,
        ) -> Option<String> {
            self.relationship_types
                .iter()
                .find(|(id, _)| *id == relationship_type.get())
                .map(|(_, name)| (*name).to_owned())
        }
    }

    fn setup() -> (Arc<Connection>, Arc<Catalog>, ir::GraphId) {
        let io = Arc::new(MemoryIO::new());
        let connection = Database::open_file(io, ":memory:graph-mutation", Arc::new(SqliteDialect))
            .unwrap()
            .connect()
            .unwrap();
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT UNIQUE); \
                 CREATE TABLE relationships( \
                   id INTEGER PRIMARY KEY, src INTEGER NOT NULL, dst INTEGER NOT NULL, since INTEGER, \
                   UNIQUE(src, dst), FOREIGN KEY(src) REFERENCES people(id), \
                   FOREIGN KEY(dst) REFERENCES people(id));",
            )
            .unwrap();
        (
            connection,
            Arc::new(Catalog {
                node_source: ir::SourceTableId::new(1).unwrap(),
                relationship_source: ir::SourceTableId::new(2).unwrap(),
                relationship_types: &[(1, "KNOWS")],
                types_table: None,
            }),
            ir::GraphId::new(1).unwrap(),
        )
    }

    /// Fixture with a relationship-type junction table and two registered
    /// types over one relationship table, so merge type matching is
    /// observable. The relationship table deliberately has no UNIQUE(src,
    /// dst): one endpoint pair may carry differently-typed relationships.
    fn setup_typed() -> (Arc<Connection>, Arc<Catalog>, ir::GraphId) {
        let io = Arc::new(MemoryIO::new());
        let connection =
            Database::open_file(io, ":memory:graph-mutation-typed", Arc::new(SqliteDialect))
                .unwrap()
                .connect()
                .unwrap();
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT UNIQUE); \
                 CREATE TABLE relationships( \
                   id INTEGER PRIMARY KEY, src INTEGER NOT NULL, dst INTEGER NOT NULL, since INTEGER, \
                   FOREIGN KEY(src) REFERENCES people(id), \
                   FOREIGN KEY(dst) REFERENCES people(id)); \
                 CREATE TABLE rel_types(relationship_id INTEGER NOT NULL, type TEXT NOT NULL);",
            )
            .unwrap();
        (
            connection,
            Arc::new(Catalog {
                node_source: ir::SourceTableId::new(1).unwrap(),
                relationship_source: ir::SourceTableId::new(2).unwrap(),
                relationship_types: &[(1, "KNOWS"), (2, "LIKES")],
                types_table: Some("rel_types"),
            }),
            ir::GraphId::new(1).unwrap(),
        )
    }

    fn execute(
        connection: &Arc<Connection>,
        catalog: &Arc<Catalog>,
        graph: ir::GraphId,
        source: &str,
    ) -> Result<MutationSummary, MutationError> {
        execute_cypher_mutation(
            connection,
            graph,
            catalog.clone(),
            source,
            &Parameters::new(),
        )
    }

    fn rows(connection: &Arc<Connection>, sql: &str) -> Vec<Vec<Value>> {
        connection.prepare(sql).unwrap().run_collect_rows().unwrap()
    }

    #[test]
    fn deferred_runtime_validation_uses_physical_value_shapes() {
        let blob = Value::from_slice(&[1, 2, 3]).expect("small blob");
        let text = Value::build_text("not encoded");
        let custom_integer = ir::ValueType::Custom {
            name: "cents".to_owned(),
            base: Box::new(ir::ValueType::Integer),
        };

        assert!(runtime_value_compatible(
            &custom_integer,
            &Value::from_i64(42)
        ));
        assert!(!runtime_value_compatible(&custom_integer, &text));
        assert!(runtime_value_compatible(
            &ir::ValueType::List(Box::new(ir::ValueType::Integer)),
            &blob
        ));
        assert!(!runtime_value_compatible(
            &ir::ValueType::Struct(vec![("x".to_owned(), ir::ValueType::Integer)]),
            &text
        ));
    }

    #[test]
    fn creates_nodes_and_relationships_with_returned_identities() {
        let (connection, catalog, graph) = setup();
        let summary = execute(
            &connection,
            &catalog,
            graph,
            "CREATE (a:Person {id: 1, name: 'Ada'})-[:KNOWS {since: 2020}]->(b:Person {id: 2, name: 'Grace'})",
        )
        .unwrap();
        assert_eq!(summary.matched_rows, 1);
        assert_eq!(summary.operations_executed, 3);
        assert_eq!(
            rows(&connection, "SELECT id, name FROM people ORDER BY id"),
            vec![
                vec![Value::from_i64(1), Value::build_text("Ada")],
                vec![Value::from_i64(2), Value::build_text("Grace")],
            ]
        );
        assert_eq!(
            rows(&connection, "SELECT src, dst, since FROM relationships"),
            vec![vec![
                Value::from_i64(1),
                Value::from_i64(2),
                Value::from_i64(2020),
            ]]
        );
    }

    #[test]
    fn mutation_return_preserves_entity_result_types() {
        let (connection, catalog, graph) = setup();
        let summary = execute(
            &connection,
            &catalog,
            graph,
            "CREATE (a:Person {id: 1, name: 'Ada'})-[r:KNOWS]->(b:Person {id: 2}) RETURN a, r",
        )
        .unwrap();

        assert_eq!(
            summary.result_types,
            vec![ir::ValueType::Node, ir::ValueType::Relationship]
        );
    }

    #[test]
    fn match_set_remove_and_parameters_use_bound_values() {
        let (connection, catalog, graph) = setup();
        connection
            .execute("INSERT INTO people VALUES (1, 'Ada')")
            .unwrap();
        let parameters = HashMap::from([("name".to_owned(), Value::build_text("Grace"))]);
        execute_cypher_mutation(
            &connection,
            graph,
            catalog.clone(),
            "MATCH (n:Person {id: 1}) SET n.name = $name",
            &parameters,
        )
        .unwrap();
        assert_eq!(
            rows(&connection, "SELECT name FROM people WHERE id = 1"),
            vec![vec![Value::build_text("Grace")]]
        );
        execute(
            &connection,
            &catalog,
            graph,
            "MATCH (n:Person {id: 1}) REMOVE n.name",
        )
        .unwrap();
        assert_eq!(
            rows(&connection, "SELECT name FROM people WHERE id = 1"),
            vec![vec![Value::Null]]
        );
    }

    #[test]
    fn missing_matches_are_noops_and_create_runs_once_per_match_row() {
        let (connection, catalog, graph) = setup();
        connection
            .execute("INSERT INTO people VALUES (1, 'Ada'), (2, 'Grace')")
            .unwrap();
        let missing = execute(
            &connection,
            &catalog,
            graph,
            "MATCH (n:Person {id: 99}) SET n.name = 'Missing'",
        )
        .unwrap();
        assert_eq!(missing.matched_rows, 0);
        assert_eq!(missing.operations_executed, 0);

        let created = execute(
            &connection,
            &catalog,
            graph,
            "MATCH (n:Person) CREATE (n)-[:KNOWS]->(:Person {id: n.id + 10})",
        )
        .unwrap();
        assert_eq!(created.matched_rows, 2);
        assert_eq!(created.operations_executed, 4);
        assert_eq!(
            rows(
                &connection,
                "SELECT id FROM people WHERE id > 2 ORDER BY id"
            ),
            vec![vec![Value::from_i64(11)], vec![Value::from_i64(12)],]
        );
    }

    #[test]
    fn statement_error_rolls_back_all_prior_mutations() {
        let (connection, catalog, graph) = setup();
        connection
            .execute("INSERT INTO people VALUES (1, 'Ada')")
            .unwrap();
        let error = execute(
            &connection,
            &catalog,
            graph,
            "CREATE (:Person {id: 2, name: 'Grace'}), (:Person {id: 1, name: 'Duplicate'})",
        )
        .expect_err("duplicate identity must fail");
        assert!(matches!(error, MutationError::Database(_)));
        assert_eq!(
            rows(&connection, "SELECT id FROM people ORDER BY id"),
            vec![vec![Value::from_i64(1)]]
        );
    }

    #[test]
    fn delete_requires_detach_and_failure_rolls_back_prior_set() {
        let (connection, catalog, graph) = setup();
        connection
            .execute(
                "INSERT INTO people VALUES (1, 'Ada'), (2, 'Grace'); \
                 INSERT INTO relationships(src, dst) VALUES (1, 2)",
            )
            .unwrap();
        let error = execute(
            &connection,
            &catalog,
            graph,
            "MATCH (n:Person {id: 1}) SET n.name = 'Changed' DELETE n",
        )
        .expect_err("attached node delete must fail");
        assert!(matches!(error, MutationError::NodeHasRelationships));
        assert_eq!(
            rows(&connection, "SELECT name FROM people WHERE id = 1"),
            vec![vec![Value::build_text("Ada")]]
        );

        execute(
            &connection,
            &catalog,
            graph,
            "MATCH (n:Person {id: 1}) DETACH DELETE n",
        )
        .unwrap();
        assert!(rows(&connection, "SELECT id FROM people WHERE id = 1").is_empty());
        assert!(rows(&connection, "SELECT id FROM relationships").is_empty());
    }

    #[test]
    fn merge_is_idempotent_for_nodes_and_relationships() {
        let (connection, catalog, graph) = setup();
        let source =
            "MERGE (a:Person {id: 1, name: 'Ada'})-[:KNOWS]->(b:Person {id: 2, name: 'Grace'})";
        execute(&connection, &catalog, graph, source).unwrap();
        execute(&connection, &catalog, graph, source).unwrap();
        assert_eq!(
            rows(&connection, "SELECT count(*) FROM people"),
            vec![vec![Value::from_i64(2)]]
        );
        assert_eq!(
            rows(&connection, "SELECT count(*) FROM relationships"),
            vec![vec![Value::from_i64(1)]]
        );
    }

    #[test]
    fn merge_relationship_does_not_match_a_different_type() {
        // openCypher: MERGE (a)-[:KNOWS]->(b) must not be satisfied by an
        // existing (a)-[:LIKES]->(b); it creates a distinct relationship and
        // must not attach KNOWS to the LIKES relationship's identity.
        let (connection, catalog, graph) = setup_typed();
        execute(
            &connection,
            &catalog,
            graph,
            "CREATE (a:Person {id: 1, name: 'Ada'})-[:LIKES]->(b:Person {id: 2, name: 'Grace'})",
        )
        .unwrap();
        let source =
            "MERGE (a:Person {id: 1, name: 'Ada'})-[:KNOWS]->(b:Person {id: 2, name: 'Grace'})";
        execute(&connection, &catalog, graph, source).unwrap();
        assert_eq!(
            rows(&connection, "SELECT count(*) FROM relationships"),
            vec![vec![Value::from_i64(2)]]
        );
        assert_eq!(
            rows(
                &connection,
                "SELECT relationship_id, type FROM rel_types ORDER BY relationship_id, type"
            ),
            vec![
                vec![Value::from_i64(1), Value::build_text("LIKES")],
                vec![Value::from_i64(2), Value::build_text("KNOWS")],
            ]
        );
        // Re-merging the same type matches its own relationship: idempotent.
        execute(&connection, &catalog, graph, source).unwrap();
        assert_eq!(
            rows(&connection, "SELECT count(*) FROM relationships"),
            vec![vec![Value::from_i64(2)]]
        );
    }

    #[test]
    fn mutation_savepoint_remains_inside_an_explicit_transaction() {
        let (connection, catalog, graph) = setup();
        connection.execute("BEGIN").unwrap();
        execute(
            &connection,
            &catalog,
            graph,
            "CREATE (:Person {id: 1, name: 'Ada'})",
        )
        .unwrap();
        assert_eq!(
            rows(&connection, "SELECT count(*) FROM people"),
            vec![vec![Value::from_i64(1)]]
        );
        connection.execute("ROLLBACK").unwrap();
        assert_eq!(
            rows(&connection, "SELECT count(*) FROM people"),
            vec![vec![Value::from_i64(0)]]
        );
    }

    #[test]
    fn with_order_by_skip_and_limit_narrow_a_mutation_stage() {
        let (connection, catalog, graph) = setup();
        // ORDER BY id DESC -> [3, 2, 1]; SKIP 1 -> [2, 1]; LIMIT 1 -> [2]:
        // only the middle row should be renamed and returned. The MATCH
        // (staged, since it follows CREATE) re-reads the just-created rows.
        let summary = execute(
            &connection,
            &catalog,
            graph,
            "CREATE (:Person {id: 1, name: 'Ada'}), (:Person {id: 2, name: 'Bob'}), \
             (:Person {id: 3, name: 'Cy'}) \
             MATCH (n:Person) WITH n ORDER BY n.id DESC SKIP 1 LIMIT 1 \
             SET n.name = 'Z' RETURN n.id",
        )
        .unwrap();
        assert_eq!(summary.rows, vec![vec![Value::from_i64(2)]]);
        assert_eq!(
            rows(&connection, "SELECT id, name FROM people ORDER BY id"),
            vec![
                vec![Value::from_i64(1), Value::build_text("Ada")],
                vec![Value::from_i64(2), Value::build_text("Z")],
                vec![Value::from_i64(3), Value::build_text("Cy")],
            ]
        );
    }

    #[test]
    fn merge_returns_the_named_path_it_creates() {
        let (connection, catalog, graph) = setup();
        // The path variable's value is never materialized as its own row of
        // data; it must be rebuilt from the node/relationship bindings that
        // made up the path (mirroring the read-side MATCH p = ... binder).
        let summary = execute(
            &connection,
            &catalog,
            graph,
            "MERGE p = (a:Person {id: 1})-[:KNOWS]->(b:Person {id: 2}) RETURN p",
        )
        .unwrap();
        assert_eq!(summary.rows.len(), 1);
        let Value::Text(path) = &summary.rows[0][0] else {
            panic!("expected a path value, got {:?}", summary.rows[0][0]);
        };
        // The path is rendered as `{"nodes": [...], "relationships": [...]}`;
        // two nodes and one relationship went into building it.
        let nodes = path.as_str().split("\"nodes\":[").nth(1).unwrap();
        let nodes_list = nodes.split(']').next().unwrap();
        assert_eq!(nodes_list.split(',').count(), 2);
        let relationships = path.as_str().split("\"relationships\":[").nth(1).unwrap();
        let relationships_list = relationships.split(']').next().unwrap();
        assert_eq!(relationships_list.split(',').count(), 1);
    }
}
