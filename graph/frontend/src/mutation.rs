use std::{collections::HashMap, sync::Arc};

use thiserror::Error;
use turso_core::{Connection, Numeric, Value};
use turso_graph_ir as ir;

use crate::{
    bind_mutation,
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
    let result = execute_bound(
        connection,
        catalog.as_ref(),
        &bound.request,
        &bound.returns,
        &input,
        parameters,
    )
    .map(|mut summary| {
        if let Some(skip) = bound.returns_skip {
            summary.rows.drain(..skip.min(summary.rows.len()));
        }
        if let Some(limit) = bound.returns_limit {
            summary.rows.truncate(limit);
        }
        summary
    });
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
    request: &ir::MutationRequest,
    returns: &[ir::TypedExpression],
    input: &LoweredMutationInput,
    parameters: &MutationParameters,
) -> Result<MutationSummary, MutationError> {
    let input_bindings = request
        .input
        .as_ref()
        .map(|plan| plan.scope().iter().map(ir::Binding::id).collect::<Vec<_>>())
        .unwrap_or_default();
    let rows = if request.input.is_some() {
        let sql = mutation_rows_sql(input, &input_bindings);
        run_rows(connection, &sql, parameters, &HashMap::new())?
    } else {
        vec![Vec::new()]
    };
    let matched_rows = rows.len() as u64;
    let mut operations_executed = 0_u64;
    let mut returned_rows = Vec::new();
    for row in rows {
        let mut values = input_bindings
            .iter()
            .copied()
            .zip(row)
            .collect::<HashMap<_, _>>();
        let mut entity_layouts = HashMap::new();
        for operation in &request.operations {
            execute_operation(
                connection,
                catalog,
                request.graph,
                input,
                operation,
                parameters,
                &mut values,
                &mut entity_layouts,
            )?;
            operations_executed += 1;
        }
        if !returns.is_empty() {
            let references = reference_parameters(&values);
            let columns = returns
                .iter()
                .map(|expression| {
                    lower_mutation_expression(
                        expression,
                        input,
                        catalog,
                        &references.sql,
                        &entity_layouts,
                    )
                    .map(|sql| format!("({sql})"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            let mut produced = run_rows(
                connection,
                &format!("SELECT {columns}"),
                parameters,
                &references.values,
            )?;
            returned_rows.append(&mut produced);
        }
    }
    Ok(MutationSummary {
        matched_rows,
        operations_executed,
        rows: returned_rows,
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
