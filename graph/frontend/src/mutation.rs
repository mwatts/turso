use std::{
    cell::Cell,
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use thiserror::Error;
use turso_core::{Connection, Numeric, Value};
use turso_graph_ir as ir;

use crate::{
    bind_mutation,
    binder::{BoundMutation, StageItem, StageProjection},
    catalog::CatalogError,
    lowering::{
        lower_mutation_expression, lower_mutation_input, mutation_rows_with_sources_sql,
        quoted_identifier, unit_mutation_input, LoweredMutationInput, MutationEntityKind,
        MutationRowColumn,
    },
    BindError, GraphCompilationCatalog, LowerError, ParameterTypes,
};

const SAVEPOINT: &str = "__turso_graph_mutation";
pub(crate) const INTERNAL_PARAMETER_PREFIX: &str = "__turso_internal_graph_ref_";

/// Count of mutations that took the closed CREATE fast path.
///
/// This is **not** "one VDBE program for the whole mutation": the node INSERT
/// is a single `prepare_internal`, but label-junction membership (when labels
/// exist and the catalog has a labels table) still uses additional helper
/// prepares. True one-program labeled CREATE would need Core multi-cmd or a
/// different encoding.
///
/// Always-on (not `cfg(test)`) so integration tests can observe it, matching
/// `turso_graph_temporal::INSTALL_COUNT`. Prefer
/// [`take_closed_create_fast_path_hit`] under parallel tests — the global
/// counter races across threads.
pub static CLOSED_CREATE_FAST_PATH_HITS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    /// Per-thread observation of the last `execute_cypher_mutation` attempt.
    /// Cleared at the start of each call; set when the closed CREATE fast path runs.
    static CLOSED_CREATE_FAST_PATH_HIT: Cell<bool> = const { Cell::new(false) };
}

/// Returns whether the current thread's last mutation used the closed CREATE
/// fast path, and clears the flag. Safe under parallel tests.
///
/// A hit means the closed single-node CREATE branch ran (one prepare for the
/// node INSERT). Labeled creates may still issue extra prepares for label
/// junction rows — do not interpret a hit as "single VDBE program overall".
pub fn take_closed_create_fast_path_hit() -> bool {
    CLOSED_CREATE_FAST_PATH_HIT.with(|hit| hit.replace(false))
}

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
    #[error("mutation binding {binding} has invalid source provenance {value:?}")]
    InvalidSourceProvenance {
        binding: ir::BindingId,
        value: Value,
    },
    #[error("cannot delete node while relationships still reference it; use DETACH DELETE")]
    NodeHasRelationships,
    #[error("runtime value for property `{property}` is not assignable to {expected:?}")]
    IncompatibleRuntimeValue {
        property: String,
        expected: ir::ValueType,
    },
    #[error("dynamic map key `{key}` is not an owned semantic property")]
    UnknownDynamicKey { key: String },
    #[error("semantic constraint violation: {0}")]
    SemanticConstraintViolation(String),
    #[error("mutation expression did not produce exactly one scalar value")]
    MissingEvaluatedValue,
    #[error("mutation failed and savepoint rollback also failed: {cause}; rollback: {rollback}")]
    RollbackFailed {
        cause: Box<MutationError>,
        rollback: turso_core::LimboError,
    },
    /// Internal helper statements cannot upgrade a deferred read transaction
    /// to write. Callers must use `BEGIN IMMEDIATE` or a prior write first.
    #[error(
        "graph mutation inside an open transaction requires a write transaction (BEGIN IMMEDIATE or a prior write)"
    )]
    RequiresWriteTransaction,
}

#[derive(Clone)]
struct MutationRow {
    values: HashMap<ir::BindingId, Value>,
    entity_layouts: HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
}

fn decode_mutation_rows(
    rows: Vec<Vec<Value>>,
    columns: &[MutationRowColumn],
) -> Result<Vec<MutationRow>, MutationError> {
    rows.into_iter()
        .map(|row| {
            assert_eq!(
                row.len(),
                columns.len(),
                "mutation row shape must match its lowering metadata"
            );
            let mut values = HashMap::new();
            let mut entity_layouts = HashMap::new();
            for (column, value) in columns.iter().zip(row) {
                match column {
                    MutationRowColumn::Value(binding) => {
                        values.insert(*binding, value);
                    }
                    MutationRowColumn::Source(binding, kind) => {
                        let Value::Numeric(Numeric::Integer(source)) = value else {
                            return Err(MutationError::InvalidSourceProvenance {
                                binding: *binding,
                                value,
                            });
                        };
                        let source = u64::try_from(source)
                            .ok()
                            .and_then(|source| ir::SourceTableId::new(source).ok())
                            .ok_or(MutationError::InvalidSourceProvenance {
                                binding: *binding,
                                value: Value::Numeric(Numeric::Integer(source)),
                            })?;
                        entity_layouts.insert(*binding, (source, *kind));
                    }
                }
            }
            Ok(MutationRow {
                values,
                entity_layouts,
            })
        })
        .collect()
}

fn mutation_source(
    source: ir::MutationSource,
    entity_layouts: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
) -> Result<ir::SourceTableId, MutationError> {
    match source {
        ir::MutationSource::Static(source) => Ok(source),
        ir::MutationSource::Binding(binding) => entity_layouts
            .get(&binding)
            .map(|(source, _)| *source)
            .ok_or(MutationError::MissingBinding(binding)),
    }
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
    check_runtime_value_against(&name, &expected, value)?;
    if let Some(constraints) = catalog.semantic_constraints() {
        constraints
            .validate_runtime(source, semantic_types, property, value)
            .map_err(MutationError::SemanticConstraintViolation)?;
    }
    Ok(())
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
    CLOSED_CREATE_FAST_PATH_HIT.with(|hit| hit.set(false));
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

    // Mutation SQL runs via prepare_internal (InternalHelper). Nested helpers
    // cannot upgrade a deferred read transaction to write — same discipline as
    // register_graph / FTS admin: BEGIN IMMEDIATE in autocommit, SAVEPOINT only
    // inside an existing write transaction.
    if !connection.get_auto_commit() && !connection.in_write_transaction() {
        return Err(MutationError::RequiresWriteTransaction);
    }

    let run = || {
        let summary = if let Some(summary) =
            try_single_program_mutation(connection, catalog.as_ref(), &bound, &input, parameters)?
        {
            summary
        } else {
            execute_bound(connection, catalog.as_ref(), &bound, &input, parameters)?
        };
        if let Some(constraints) = catalog.semantic_constraints() {
            constraints
                .validate_state(connection)
                .map_err(|error| match error {
                    crate::SemanticCatalogError::Database(error)
                    | crate::SemanticCatalogError::Catalog(CatalogError::Database(error)) => {
                        MutationError::Database(error)
                    }
                    error => MutationError::SemanticConstraintViolation(error.to_string()),
                })?;
        }
        let mut summary = summary;
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
        Ok(summary)
    };

    if connection.get_auto_commit() {
        connection.execute("BEGIN IMMEDIATE")?;
        match run() {
            Ok(summary) => {
                connection.execute("COMMIT")?;
                Ok(summary)
            }
            Err(cause) => match connection.execute("ROLLBACK") {
                Ok(()) => Err(cause),
                Err(rollback) => Err(MutationError::RollbackFailed {
                    cause: Box::new(cause),
                    rollback,
                }),
            },
        }
    } else {
        connection.execute(format!("SAVEPOINT {SAVEPOINT}"))?;
        match run() {
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
}

/// Closed CREATE fast path for one `CREATE` node with no MATCH input, no WITH
/// stages, and no RETURN. Unsupported shapes return `Ok(None)` so the
/// multi-prepare savepoint executor remains the fallback.
///
/// **Prepare model:** the node INSERT is one `prepare_internal` program.
/// Label-junction membership rows (when the catalog has a labels table and
/// the create lists labels) still use additional helper prepares via
/// [`record_node_labels`] — SQLite/Turso reject writable multi-table CTE
/// inserts. A hit on [`CLOSED_CREATE_FAST_PATH_HITS`] therefore means "closed
/// CREATE branch ran", not "one VDBE program for the whole mutation".
fn try_single_program_mutation(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    bound: &BoundMutation,
    input: &LoweredMutationInput,
    parameters: &Parameters,
) -> Result<Option<MutationSummary>, MutationError> {
    if bound.request.input.is_some()
        || !bound.stages.is_empty()
        || !bound.returns.is_empty()
        || bound.request.operations.len() != 1
    {
        return Ok(None);
    }
    let ir::Mutation::CreateNode(create) = &bound.request.operations[0] else {
        return Ok(None);
    };
    // Deferred Any-typed property values need a separate SELECT evaluate step
    // before INSERT; keep that on the multi-prepare path.
    if create
        .properties
        .iter()
        .any(|property| property.value.value_type == ir::ValueType::Any)
    {
        return Ok(None);
    }

    let empty_values = HashMap::new();
    let empty_layouts = HashMap::new();
    let (identity, _) = insert_node(
        connection,
        catalog,
        input,
        create,
        parameters,
        &empty_values,
        &empty_layouts,
        false,
    )?;
    record_node_labels(
        connection,
        catalog,
        create.source,
        &create.labels,
        &identity,
        parameters,
    )?;
    CLOSED_CREATE_FAST_PATH_HIT.with(|hit| hit.set(true));
    CLOSED_CREATE_FAST_PATH_HITS.fetch_add(1, Ordering::SeqCst);
    Ok(Some(MutationSummary {
        matched_rows: 1,
        operations_executed: 1,
        rows: Vec::new(),
        result_types: bound.return_types.clone(),
    }))
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
    let mut rows = if request.input.is_some() {
        let (sql, columns) = mutation_rows_with_sources_sql(input, &input_bindings);
        decode_mutation_rows(
            run_rows(connection, &sql, parameters, &HashMap::new())?,
            &columns,
        )?
    } else {
        vec![MutationRow {
            values: HashMap::new(),
            entity_layouts: HashMap::new(),
        }]
    };
    let matched_rows = rows.len() as u64;
    let mut operations_executed = 0_u64;
    for row in &mut rows {
        for operation in &request.operations {
            execute_operation(
                connection,
                catalog,
                request.graph,
                input,
                operation,
                parameters,
                &mut row.values,
                &mut row.entity_layouts,
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
        )?;
        rows = sort_stage_rows(connection, catalog, input, parameters, &stage.order, rows)?;
        if let Some(skip) = stage.skip {
            rows.drain(..skip.min(rows.len()));
        }
        if let Some(limit) = stage.limit {
            rows.truncate(limit);
        }
        for item in &stage.items {
            match item {
                StageItem::Operation(operation) => {
                    for row in &mut rows {
                        execute_operation(
                            connection,
                            catalog,
                            request.graph,
                            input,
                            operation,
                            parameters,
                            &mut row.values,
                            &mut row.entity_layouts,
                        )?;
                        operations_executed += 1;
                    }
                }
                StageItem::Foreach { .. } => {
                    for row in &mut rows {
                        run_stage_items_once(
                            connection,
                            catalog,
                            request.graph,
                            input,
                            std::slice::from_ref(item),
                            parameters,
                            &mut row.values,
                            &mut row.entity_layouts,
                            &mut operations_executed,
                        )?;
                    }
                }
                StageItem::Unwind { output, list } => {
                    let mut expanded = Vec::new();
                    for row in &rows {
                        let references = reference_parameters(&row.values);
                        let sql = lower_mutation_expression(
                            list,
                            input,
                            catalog,
                            &references.sql,
                            &row.entity_layouts,
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
                            let mut next = row.clone();
                            next.values.insert(*output, element);
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
                    let (sql, columns) = mutation_rows_with_sources_sql(&lowered, outputs);
                    let mut expanded = Vec::new();
                    for row in &rows {
                        // Correlated plans reference the row's bindings
                        // through internal reference parameters.
                        let references = reference_parameters(&row.values);
                        let matched = decode_mutation_rows(
                            run_rows(connection, &sql, parameters, &references.values)?,
                            &columns,
                        )?;
                        if matched.is_empty() && *optional {
                            let mut next = row.clone();
                            for binding in outputs {
                                next.values.insert(*binding, Value::Null);
                                next.entity_layouts.remove(binding);
                            }
                            expanded.push(next);
                            continue;
                        }
                        for matched_row in matched {
                            let mut next = row.clone();
                            next.values.extend(matched_row.values);
                            next.entity_layouts.extend(matched_row.entity_layouts);
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
            .map(|row| {
                order
                    .iter()
                    .map(|output| row.values.get(output).cloned().unwrap_or(Value::Null))
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

fn projected_entity_layouts(
    projections: &[StageProjection],
    input: &HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)>,
) -> HashMap<ir::BindingId, (ir::SourceTableId, MutationEntityKind)> {
    projections
        .iter()
        .filter_map(|projection| {
            let StageProjection::Expression { output, expression } = projection else {
                return None;
            };
            let ir::Expression::Binding(input_binding) = expression.expression else {
                return None;
            };
            input
                .get(&input_binding)
                .copied()
                .map(|layout| (*output, layout))
        })
        .collect()
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
    rows: Vec<MutationRow>,
) -> Result<Vec<MutationRow>, MutationError> {
    if projections.is_empty() {
        assert!(
            predicate.is_none() && !distinct,
            "an empty mutation projection cannot carry filtering or DISTINCT"
        );
        return Ok(rows);
    }
    let has_aggregates = projections
        .iter()
        .any(|projection| matches!(projection, StageProjection::Aggregate { .. }));
    // Evaluate every projection input per row in a single SELECT.
    let mut evaluated = Vec::with_capacity(rows.len());
    for row in &rows {
        let references = reference_parameters(&row.values);
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
                        &row.entity_layouts,
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
        evaluated.push((
            produced.pop().unwrap_or_default(),
            projected_entity_layouts(projections, &row.entity_layouts),
        ));
    }
    let mut output_rows = if has_aggregates {
        let key_positions: Vec<usize> = projections
            .iter()
            .enumerate()
            .filter(|(_, projection)| matches!(projection, StageProjection::Expression { .. }))
            .map(|(position, _)| position)
            .collect();
        let mut groups = Vec::<(String, Vec<(Vec<Value>, HashMap<_, _>)>)>::new();
        for (row, layouts) in evaluated {
            let key = key_positions
                .iter()
                .map(|position| {
                    let source = match &projections[*position] {
                        StageProjection::Expression { output, .. } => {
                            layouts.get(output).map(|(source, _)| source.get())
                        }
                        StageProjection::Aggregate { .. } => None,
                    };
                    format!("{:?}@{source:?}", row[*position])
                })
                .collect::<Vec<_>>()
                .join("\u{1}");
            match groups.iter_mut().find(|(existing, _)| *existing == key) {
                Some((_, members)) => members.push((row, layouts)),
                None => groups.push((key, vec![(row, layouts)])),
            }
        }
        let mut output = Vec::new();
        for (_, members) in groups {
            let mut values = HashMap::new();
            let entity_layouts = members[0].1.clone();
            for (position, projection) in projections.iter().enumerate() {
                match projection {
                    StageProjection::Expression { output: id, .. } => {
                        values.insert(*id, members[0].0[position].clone());
                    }
                    StageProjection::Aggregate {
                        output: id,
                        function,
                        argument,
                        distinct,
                    } => {
                        let collected: Vec<Value> = members
                            .iter()
                            .map(|member| member.0[position].clone())
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
            output.push(MutationRow {
                values,
                entity_layouts,
            });
        }
        output
    } else {
        evaluated
            .into_iter()
            .map(|(values, entity_layouts)| MutationRow {
                values: projections
                    .iter()
                    .zip(values)
                    .map(|(projection, value)| match projection {
                        StageProjection::Expression { output, .. }
                        | StageProjection::Aggregate { output, .. } => (*output, value),
                    })
                    .collect(),
                entity_layouts,
            })
            .collect()
    };
    if let Some(predicate) = predicate {
        let mut kept = Vec::new();
        for row in output_rows {
            let references = reference_parameters(&row.values);
            let sql = lower_mutation_expression(
                predicate,
                input,
                catalog,
                &references.sql,
                &row.entity_layouts,
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
                kept.push(row);
            }
        }
        output_rows = kept;
    }
    if distinct {
        let mut seen = Vec::new();
        output_rows.retain(|row| {
            let mut entries: Vec<_> = row.values.iter().collect();
            entries.sort_by_key(|(id, _)| id.get());
            let mut layouts: Vec<_> = row.entity_layouts.iter().collect();
            layouts.sort_by_key(|(id, _)| id.get());
            let key = format!("{entries:?}@{layouts:?}");
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
    rows: Vec<MutationRow>,
) -> Result<Vec<MutationRow>, MutationError> {
    if order.is_empty() {
        return Ok(rows);
    }
    let mut keyed = Vec::with_capacity(rows.len());
    for row in rows {
        let references = reference_parameters(&row.values);
        let mut keys = Vec::with_capacity(order.len());
        for (expression, _) in order {
            let sql = lower_mutation_expression(
                expression,
                input,
                catalog,
                &references.sql,
                &row.entity_layouts,
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
        keyed.push((keys, row));
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
    Ok(keyed.into_iter().map(|(_, row)| row).collect())
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
            record_node_labels(
                connection,
                catalog,
                create.source,
                &create.labels,
                &identity,
                parameters,
            )?;
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
                merge.create.source,
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
                create.source,
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
                    merge.create.source,
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
            let source = mutation_source(set.source, entity_layouts)?;
            let layout = entity_table(catalog, source)?;
            let column = property_column(catalog, source, &set.semantic_types, set.property)?;
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
                    source,
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
            let source = mutation_source(set.source, entity_layouts)?;
            let identity = values
                .get(&set.entity)
                .ok_or(MutationError::MissingBinding(set.entity))?
                .clone();
            record_node_labels(
                connection,
                catalog,
                source,
                &set.labels,
                &identity,
                parameters,
            )?;
        }
        ir::Mutation::ReplaceProperties(replace) => {
            let source = mutation_source(replace.source, entity_layouts)?;
            let layout = entity_table(catalog, source)?;
            let references = reference_parameters(values);
            let identity = values
                .get(&replace.entity)
                .ok_or(MutationError::MissingBinding(replace.entity))?;
            let mut internal = references.values;
            internal.insert(identity_parameter(replace.entity), identity.clone());
            let mut assignments = Vec::new();
            let mut assigned_columns = Vec::new();
            for entry in &replace.entries {
                let column =
                    property_column(catalog, source, &entry.semantic_types, entry.property)?;
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
                        source,
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
                    source,
                    &replace.semantic_types,
                ) {
                    for (_, _, _, column) in properties {
                        if !assigned_columns.contains(&column) {
                            assignments.push(format!("{} = NULL", quoted_identifier(&column)));
                        }
                    }
                } else {
                    let structural = if let Some(relationship) = catalog.relationship_layout(source)
                    {
                        relationship.structural_columns()
                    } else {
                        vec![layout.identity.clone()]
                    };
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
            let source = mutation_source(replace.source, entity_layouts)?;
            let layout = entity_table(catalog, source)?;
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
                source,
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
                            source,
                            &replace.semantic_types,
                            &key,
                        )
                        .flatten()
                    else {
                        return Err(MutationError::UnknownDynamicKey { key });
                    };
                    check_runtime_value_against(&property_name, &expected, value)?;
                    if let Some(constraints) = catalog.semantic_constraints() {
                        let property = owned_properties
                            .iter()
                            .find(|(_, name, _, owned_column)| {
                                name.eq_ignore_ascii_case(&property_name)
                                    && owned_column.eq_ignore_ascii_case(&column)
                            })
                            .map(|(property, _, _, _)| *property)
                            .ok_or_else(|| {
                                MutationError::SemanticConstraintViolation(format!(
                                    "property `{property_name}` is missing constraint metadata"
                                ))
                            })?;
                        constraints
                            .validate_runtime(source, &replace.semantic_types, property, value)
                            .map_err(MutationError::SemanticConstraintViolation)?;
                    }
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
                    .payload_columns(source)
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
            let source = mutation_source(remove.source, entity_layouts)?;
            let layout = entity_table(catalog, source)?;
            let column = property_column(catalog, source, &remove.semantic_types, remove.property)?;
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
            let source = mutation_source(delete.source, entity_layouts)?;
            delete_entity(
                connection, catalog, graph, delete, source, parameters, values,
            )?;
        }
    }
    Ok(())
}

/// Records a created or merged node's labels in the graph's junction
/// table; idempotent so MERGE matches do not duplicate rows.
fn record_node_labels(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    source: ir::SourceTableId,
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
        let sql = if catalog.source_qualified_membership() {
            format!(
                "INSERT INTO \"{table}\"(source_id, node_id, label) \
                 SELECT {}, ${parameter}, '{name}' \
                 WHERE NOT EXISTS (SELECT 1 FROM \"{table}\" \
                 WHERE source_id = {} AND node_id = ${parameter} AND label = '{name}')",
                source.get(),
                source.get(),
            )
        } else {
            format!(
                "INSERT INTO \"{table}\"(node_id, label) SELECT ${parameter}, '{name}' \
                 WHERE NOT EXISTS (SELECT 1 FROM \"{table}\" \
                 WHERE node_id = ${parameter} AND label = '{name}')"
            )
        };
        run_ignore(connection, &sql, parameters, &internal)?;
    }
    Ok(())
}

/// Records a created or merged relationship's type in the graph's junction
/// table; idempotent so MERGE matches do not duplicate rows.
fn record_relationship_type(
    connection: &Arc<Connection>,
    catalog: &dyn GraphCompilationCatalog,
    source: ir::SourceTableId,
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
        let sql = if catalog.source_qualified_membership() {
            format!(
                "INSERT INTO \"{table}\"(source_id, relationship_id, type) \
                 SELECT {}, ${parameter}, '{name}' \
                 WHERE NOT EXISTS (SELECT 1 FROM \"{table}\" \
                 WHERE source_id = {} AND relationship_id = ${parameter} AND type = '{name}')",
                source.get(),
                source.get(),
            )
        } else {
            format!(
                "INSERT INTO \"{table}\"(relationship_id, type) \
                 SELECT ${parameter}, '{name}' \
                 WHERE NOT EXISTS (SELECT 1 FROM \"{table}\" \
                 WHERE relationship_id = ${parameter} AND type = '{name}')"
            )
        };
        run_ignore(connection, &sql, parameters, &internal)?;
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
            create.source,
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
    source: ir::SourceTableId,
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
            let source_predicate = if catalog.source_qualified_membership() {
                format!("source_id = {} AND ", source.get())
            } else {
                String::new()
            };
            format!(
                "EXISTS (SELECT 1 FROM {} WHERE {source_predicate}\
                 relationship_id = {}.{} AND type = '{}')",
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
    source: ir::SourceTableId,
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
            let source_predicate = if catalog.source_qualified_membership() {
                format!("source_id = {} AND ", source.get())
            } else {
                String::new()
            };
            format!(
                "EXISTS (SELECT 1 FROM {} WHERE {source_predicate}\
                 node_id = {}.{} AND label = '{}')",
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
            create.source,
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
        // `CREATE (a)-[r]->(b)` is always a two-role pattern hop, in
        // declaration order, regardless of how many roles the relation
        // carries in total.
        &[
            (layout.roles[0].column.clone(), from.clone()),
            (layout.roles[1].column.clone(), to.clone()),
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
    source: ir::SourceTableId,
    parameters: &Parameters,
    values: &HashMap<ir::BindingId, Value>,
) -> Result<(), MutationError> {
    let identity = values
        .get(&delete.entity)
        .ok_or(MutationError::MissingBinding(delete.entity))?;
    let internal = HashMap::from([(identity_parameter(delete.entity), identity.clone())]);
    if let Some(layout) = catalog.node_layout(source) {
        for relationship_source in catalog.relationship_sources(graph) {
            let relationship = catalog
                .relationship_layout(relationship_source)
                .ok_or(LowerError::MissingSource(relationship_source))?;
            let Some((start_source, end_source)) =
                catalog.relationship_endpoint_sources(graph, relationship_source)
            else {
                continue;
            };
            let parameter = identity_parameter(delete.entity);
            let mut predicates = Vec::new();
            // `relationship_endpoint_sources` only resolves for the two-role
            // pattern-hop shape, so the role pair is always [start, end].
            if start_source == source {
                predicates.push(format!(
                    "{} = ${parameter}",
                    quoted_identifier(&relationship.roles[0].column)
                ));
            }
            if end_source == source {
                predicates.push(format!(
                    "{} = ${parameter}",
                    quoted_identifier(&relationship.roles[1].column)
                ));
            }
            if predicates.is_empty() {
                continue;
            }
            let predicate = predicates.join(" OR ");
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
            let source_predicate = if catalog.source_qualified_membership() {
                format!("source_id = {} AND ", source.get())
            } else {
                String::new()
            };
            run_ignore(
                connection,
                &format!(
                    "DELETE FROM \"{}\" WHERE {source_predicate}node_id = ${}",
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
            .relationship_layout(source)
            .ok_or(LowerError::MissingSource(source))?;
        if let Some(types_table) = catalog.relationship_types_table() {
            let source_predicate = if catalog.source_qualified_membership() {
                format!("source_id = {} AND ", source.get())
            } else {
                String::new()
            };
            run_ignore(
                connection,
                &format!(
                    "DELETE FROM \"{}\" WHERE {source_predicate}relationship_id = ${}",
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
    // Engine-generated mutation SQL: InternalHelper origin so SQLite function
    // resolution applies (no user dialect / GraphDialect parse of helper SQL).
    let mut statement = connection.prepare_internal(sql)?;
    bind_parameters(&mut statement, parameters, internal)?;
    statement.run_ignore_rows()
}

fn run_rows(
    connection: &Arc<Connection>,
    sql: &str,
    parameters: &Parameters,
    internal: &HashMap<String, Value>,
) -> Result<Vec<Vec<Value>>, turso_core::LimboError> {
    // Engine-generated mutation SQL: InternalHelper origin so SQLite function
    // resolution applies (no user dialect / GraphDialect parse of helper SQL).
    let mut statement = connection.prepare_internal(sql)?;
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
                // Turso stores bytes, lists, structs, unions, and vectors as
                // BLOB values. The runtime property validator has the semantic
                // type, but a bare parameter does not. Defer BLOB parameters
                // so that validator can check the physical value shape against
                // the target property before the write.
                Value::Blob(_) => (ir::ValueType::Any, ir::Nullability::NonNull),
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
        RelationshipRoleLayout, RelationshipTableLayout, ResolvedProperty,
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
                roles: vec![
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(1).unwrap(),
                        name: "start".to_owned(),
                        column: "src".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                    RelationshipRoleLayout {
                        role: ir::RoleId::new(2).unwrap(),
                        name: "end".to_owned(),
                        column: "dst".to_owned(),
                        cardinality: ir::RoleCardinality::One,
                        spill_table: None,
                    },
                ],
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
    fn blob_mutation_parameters_defer_semantic_type_validation() {
        let parameters = Parameters::from([(
            "value".to_owned(),
            Value::from_slice(&[1, 2, 3]).expect("small blob"),
        )]);

        assert_eq!(
            parameter_types(&parameters).get("value"),
            Some(&(ir::ValueType::Any, ir::Nullability::NonNull)),
            "a BLOB can encode several semantic types and must be checked against its property"
        );
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
    fn single_create_node_uses_closed_create_fast_path() {
        let (connection, catalog, graph) = setup();
        let summary = execute(
            &connection,
            &catalog,
            graph,
            "CREATE (:Person {id: 42, name: 'Grace'})",
        )
        .unwrap();
        assert_eq!(summary.matched_rows, 1);
        assert_eq!(summary.operations_executed, 1);
        assert!(
            take_closed_create_fast_path_hit(),
            "closed single CREATE must take the closed CREATE fast path"
        );
        assert_eq!(
            rows(&connection, "SELECT id, name FROM people WHERE id = 42"),
            vec![vec![Value::from_i64(42), Value::build_text("Grace")]]
        );
    }

    #[test]
    fn multi_stage_mutation_still_uses_savepoint_path() {
        let (connection, catalog, graph) = setup();
        let summary = execute(
            &connection,
            &catalog,
            graph,
            "CREATE (:Person {id: 1, name: 'Ada'}) WITH 1 AS x RETURN x",
        )
        .unwrap();
        assert_eq!(summary.rows, vec![vec![Value::from_i64(1)]]);
        assert!(
            !take_closed_create_fast_path_hit(),
            "WITH stages must stay on the multi-prepare savepoint path"
        );
        assert_eq!(
            rows(&connection, "SELECT id, name FROM people WHERE id = 1"),
            vec![vec![Value::from_i64(1), Value::build_text("Ada")]]
        );
    }

    #[test]
    fn multi_node_create_does_not_use_closed_create_fast_path() {
        let (connection, catalog, graph) = setup();
        execute(
            &connection,
            &catalog,
            graph,
            "CREATE (:Person {id: 1, name: 'Ada'}), (:Person {id: 2, name: 'Grace'})",
        )
        .unwrap();
        assert!(
            !take_closed_create_fast_path_hit(),
            "multi-operation CREATE is outside the closed CREATE fast-path subset"
        );
    }

    #[test]
    fn mutation_savepoint_remains_inside_an_explicit_transaction() {
        let (connection, catalog, graph) = setup();
        // prepare_internal helpers cannot upgrade deferred BEGIN; IMMEDIATE
        // (or a prior write) is required, matching register_graph / FTS.
        connection.execute("BEGIN IMMEDIATE").unwrap();
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
    fn mutation_rejects_deferred_read_transaction() {
        let (connection, catalog, graph) = setup();
        connection.execute("BEGIN").unwrap();
        let error = execute(
            &connection,
            &catalog,
            graph,
            "CREATE (:Person {id: 1, name: 'Ada'})",
        )
        .unwrap_err();
        assert!(matches!(error, MutationError::RequiresWriteTransaction));
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
