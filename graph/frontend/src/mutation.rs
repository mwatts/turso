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
const INTERNAL_PARAMETER_PREFIX: &str = "__turso_internal_graph_ref_";

pub type MutationParameters = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct MutationSummary {
    pub matched_rows: u64,
    pub operations_executed: u64,
    /// Rows produced by a trailing RETURN clause, one per input row.
    pub rows: Vec<Vec<Value>>,
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
    #[error("MERGE requires at least one property to identify the entity")]
    MergeWithoutProperties,
    #[error("mutation references binding {0} before it has a value")]
    MissingBinding(ir::BindingId),
    #[error("cannot delete node while relationships still reference it; use DETACH DELETE")]
    NodeHasRelationships,
    #[error("mutation failed and savepoint rollback also failed: {cause}; rollback: {rollback}")]
    RollbackFailed {
        cause: Box<MutationError>,
        rollback: turso_core::LimboError,
    },
}

pub fn execute_cypher_mutation(
    connection: &Arc<Connection>,
    graph: ir::GraphId,
    catalog: Arc<dyn GraphCompilationCatalog>,
    source: &str,
    parameters: &MutationParameters,
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

fn execute_bound(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    bound: &BoundMutation,
    input: &LoweredMutationInput,
    parameters: &MutationParameters,
) -> Result<MutationSummary, MutationError> {
    let request = &bound.request;
    let input_bindings = request
        .input
        .as_ref()
        .map(|plan| plan.scope().iter().map(ir::Binding::id).collect::<Vec<_>>())
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
    parameters: &MutationParameters,
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
            StageItem::Unwind { .. } => {
                return Err(MutationError::Database(
                    turso_core::LimboError::InternalError(
                        "UNWIND is stage-level, not FOREACH-level".to_owned(),
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
    parameters: &MutationParameters,
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

fn fold_aggregate(
    connection: &Arc<Connection>,
    parameters: &MutationParameters,
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
    parameters: &MutationParameters,
    values: &mut HashMap<ir::BindingId, Value>,
    entity_layouts: &mut HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
) -> Result<(), MutationError> {
    match operation {
        ir::Mutation::CreateNode(create) => {
            let identity = insert_node(
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
            let identity = insert_node(
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
        }
        ir::Mutation::CreateRelationship(create) => {
            let identity = insert_relationship(
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
            let identity = insert_relationship(
                connection,
                catalog,
                input,
                &merge.create,
                parameters,
                values,
                entity_layouts,
                true,
            )?;
            record_relationship_type(
                connection,
                catalog,
                &merge.create.relationship_types,
                &identity,
                parameters,
            )?;
            values.insert(merge.create.binding.id(), identity);
            entity_layouts.insert(
                merge.create.binding.id(),
                (merge.create.source, MutationEntityKind::Relationship),
            );
        }
        ir::Mutation::SetProperty(set) => {
            let layout = entity_table(catalog, set.source)?;
            let column = property_column(catalog, set.source, set.property)?;
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
            run_ignore(
                connection,
                &format!(
                    "UPDATE {} SET {} = {expression} WHERE {} = ${}",
                    quoted_identifier(&layout.table),
                    quoted_identifier(&column),
                    quoted_identifier(&layout.identity),
                    identity_parameter(set.entity),
                ),
                parameters,
                &internal,
            )?;
        }
        ir::Mutation::RemoveProperty(remove) => {
            let layout = entity_table(catalog, remove.source)?;
            let column = property_column(catalog, remove.source, remove.property)?;
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
    parameters: &MutationParameters,
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
    parameters: &MutationParameters,
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
    parameters: &MutationParameters,
    values: &HashMap<ir::BindingId, Value>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
    merge: bool,
) -> Result<Value, MutationError> {
    let layout = catalog
        .node_layout(create.source)
        .ok_or(LowerError::MissingSource(create.source))?;
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
    )
}

#[allow(clippy::too_many_arguments)]
fn insert_relationship(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    input: &LoweredMutationInput,
    create: &ir::CreateRelationship,
    parameters: &MutationParameters,
    values: &HashMap<ir::BindingId, Value>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
    merge: bool,
) -> Result<Value, MutationError> {
    let layout = catalog
        .relationship_layout(create.source)
        .ok_or(LowerError::MissingSource(create.source))?;
    let from = values
        .get(&create.from)
        .ok_or(MutationError::MissingBinding(create.from))?;
    let to = values
        .get(&create.to)
        .ok_or(MutationError::MissingBinding(create.to))?;
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
    parameters: &MutationParameters,
    values: &HashMap<ir::BindingId, Value>,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
    merge: bool,
    entity: &'static str,
    fixed: &[(String, Value)],
) -> Result<Value, MutationError> {
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
        columns.push(property_column(catalog, source, property.property)?);
        expressions.push(lower_mutation_expression(
            &property.value,
            input,
            catalog,
            &references.sql,
            entity_layouts,
        )?);
    }
    if merge && columns.is_empty() {
        return Err(MutationError::MergeWithoutProperties);
    }
    if merge {
        let predicate = columns
            .iter()
            .zip(&expressions)
            .map(|(column, expression)| format!("{} IS ({expression})", quoted_identifier(column)))
            .collect::<Vec<_>>()
            .join(" AND ");
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
            return Ok(value);
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
        .ok_or(MutationError::MissingCreatedIdentity { entity })
}

fn delete_entity(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    graph: ir::GraphId,
    delete: &ir::DeleteEntity,
    parameters: &MutationParameters,
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
    property: ir::PropertyId,
) -> Result<String, LowerError> {
    catalog
        .property_column(source, property)
        .ok_or(LowerError::MissingProperty {
            source_id: source,
            property,
        })
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
    parameters: &MutationParameters,
    internal: &HashMap<String, Value>,
) -> Result<(), turso_core::LimboError> {
    let mut statement = connection.prepare(sql)?;
    bind_parameters(&mut statement, parameters, internal)?;
    statement.run_ignore_rows()
}

fn run_rows(
    connection: &Arc<Connection>,
    sql: &str,
    parameters: &MutationParameters,
    internal: &HashMap<String, Value>,
) -> Result<Vec<Vec<Value>>, turso_core::LimboError> {
    let mut statement = connection.prepare(sql)?;
    bind_parameters(&mut statement, parameters, internal)?;
    statement.run_collect_rows()
}

fn bind_parameters(
    statement: &mut turso_core::Statement,
    parameters: &MutationParameters,
    internal: &HashMap<String, Value>,
) -> Result<(), turso_core::LimboError> {
    for (name, value) in parameters.iter().chain(internal) {
        if let Some(index) = statement.parameter_index(&format!("${name}")) {
            statement.bind_at(index, value.clone())?;
        }
    }
    Ok(())
}

fn parameter_types(parameters: &MutationParameters) -> ParameterTypes {
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
            (name == "KNOWS").then(|| ir::RelationshipTypeId::new(1).unwrap())
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
            &MutationParameters::new(),
        )
    }

    fn rows(connection: &Arc<Connection>, sql: &str) -> Vec<Vec<Value>> {
        connection.prepare(sql).unwrap().run_collect_rows().unwrap()
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
}
