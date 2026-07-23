use std::{collections::BTreeSet, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use turso_core::{Connection, Numeric, Value};
use turso_graph_ir::{GraphId, ValueType};

use crate::{
    catalog::{quote_identifier, stable_hash},
    load_registered_graph, CatalogError, GraphCompilationCatalog, RegisteredGraph,
};

const METADATA_TABLE: &str = "__turso_graph_fts_indexes";
const METADATA_VERSION: i64 = 1;

pub const MAX_GRAPH_FTS_INDEX_NAME_BYTES: usize = 128;
pub const MAX_GRAPH_FTS_PROPERTIES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GraphFtsEntityKind {
    Node,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GraphFtsTokenizer {
    Default,
    Raw,
    Simple,
    Ngram,
}

impl GraphFtsTokenizer {
    fn core_name(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Raw => Some("raw"),
            Self::Simple => Some("simple"),
            Self::Ngram => Some("ngram"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphFtsPropertyWeight {
    pub property: String,
    pub weight: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphFtsIndexSpec {
    pub name: String,
    pub entity: GraphFtsEntityKind,
    pub source: String,
    pub properties: Vec<String>,
    pub tokenizer: GraphFtsTokenizer,
    pub weights: Vec<GraphFtsPropertyWeight>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphFtsIndex {
    pub graph_id: GraphId,
    pub physical_name: String,
    pub spec: GraphFtsIndexSpec,
}

#[derive(Debug, Error)]
pub enum GraphFtsError {
    #[error("graph FTS requires a database opened with index methods enabled")]
    IndexMethodsDisabled,
    #[error("graph FTS index name must not be empty")]
    EmptyName,
    #[error("graph FTS index name must not contain NUL")]
    InvalidName,
    #[error("graph FTS index name exceeds {MAX_GRAPH_FTS_INDEX_NAME_BYTES} bytes")]
    NameTooLong,
    #[error("graph FTS requires at least one property")]
    NoProperties,
    #[error("graph FTS accepts at most {MAX_GRAPH_FTS_PROPERTIES} properties")]
    TooManyProperties,
    #[error("graph FTS property `{0}` is duplicated")]
    DuplicateProperty(String),
    #[error("graph FTS node source `{0}` is not registered")]
    UnknownSource(String),
    #[error("graph FTS property `{property}` is not declared on source `{source_name}`")]
    UnknownProperty {
        source_name: String,
        property: String,
    },
    #[error("graph FTS property `{property}` on source `{source_name}` is not statically text")]
    NonTextProperty {
        source_name: String,
        property: String,
    },
    #[error("graph FTS weight references property `{0}` outside the index")]
    UnknownWeightedProperty(String),
    #[error("graph FTS weight for `{property}` must be finite and greater than zero")]
    InvalidWeight { property: String },
    #[error("graph FTS index `{0}` already exists with a different definition")]
    ConflictingDefinition(String),
    #[error("graph FTS metadata is invalid: {0}")]
    InvalidMetadata(&'static str),
    #[error("graph FTS administration inside an open transaction requires a write transaction")]
    RequiresWriteTransaction,
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error("graph FTS database operation failed: {0}")]
    Database(#[from] turso_core::LimboError),
    #[error("graph FTS operation failed and rollback also failed: {cause}; rollback: {rollback}")]
    RollbackFailed {
        cause: Box<GraphFtsError>,
        rollback: turso_core::LimboError,
    },
}

struct ResolvedIndex {
    index: GraphFtsIndex,
    table: String,
    physical_properties: Vec<String>,
    physical_weights: Vec<(String, f64)>,
}

pub(crate) fn create(
    connection: &Arc<Connection>,
    graph_name: &str,
    catalog: &dyn GraphCompilationCatalog,
    spec: &GraphFtsIndexSpec,
) -> Result<GraphFtsIndex, GraphFtsError> {
    require_index_methods(connection)?;
    let graph = load_registered_graph(connection, graph_name)?;
    let resolved = resolve_index(connection, &graph, catalog, spec)?;
    transaction(connection, || {
        ensure_metadata_table(connection)?;
        if let Some(existing) = load_one(connection, graph.id, &spec.name)? {
            return if existing == resolved.index {
                Ok(existing)
            } else {
                Err(GraphFtsError::ConflictingDefinition(spec.name.clone()))
            };
        }
        let columns = resolved
            .physical_properties
            .iter()
            .map(|column| quote_identifier(column))
            .collect::<Vec<_>>()
            .join(", ");
        let mut options = Vec::new();
        if let Some(tokenizer) = spec.tokenizer.core_name() {
            options.push(format!("tokenizer = {}", sql_string(tokenizer)));
        }
        if !resolved.physical_weights.is_empty() {
            let weights = resolved
                .physical_weights
                .iter()
                .map(|(column, weight)| format!("{column}={weight}"))
                .collect::<Vec<_>>()
                .join(",");
            options.push(format!("weights = {}", sql_string(&weights)));
        }
        let options = if options.is_empty() {
            String::new()
        } else {
            format!(" WITH ({})", options.join(", "))
        };
        execute(
            connection,
            format!(
                "CREATE INDEX {} ON {} USING fts ({columns}){options}",
                quote_identifier(&resolved.index.physical_name),
                quote_identifier(&resolved.table),
            ),
        )?;
        insert_metadata(connection, &resolved.index)?;
        Ok(resolved.index.clone())
    })
}

pub(crate) fn list(
    connection: &Arc<Connection>,
    graph_name: &str,
) -> Result<Vec<GraphFtsIndex>, GraphFtsError> {
    let graph = load_registered_graph(connection, graph_name)?;
    if connection
        .current_schema()
        .get_table(METADATA_TABLE)
        .is_none()
    {
        return Ok(Vec::new());
    }
    load_rows(connection, graph.id, None)
}

pub(crate) fn drop(
    connection: &Arc<Connection>,
    graph_name: &str,
    logical_name: &str,
) -> Result<bool, GraphFtsError> {
    require_index_methods(connection)?;
    let graph = load_registered_graph(connection, graph_name)?;
    if connection
        .current_schema()
        .get_table(METADATA_TABLE)
        .is_none()
    {
        return Ok(false);
    }
    transaction(connection, || {
        let Some(index) = load_one(connection, graph.id, logical_name)? else {
            return Ok(false);
        };
        execute(
            connection,
            format!("DROP INDEX {}", quote_identifier(&index.physical_name)),
        )?;
        execute(
            connection,
            format!(
                "DELETE FROM {METADATA_TABLE} WHERE graph_id = {} AND logical_name = {} COLLATE NOCASE",
                graph.id.get(),
                sql_string(logical_name)
            ),
        )?;
        Ok(true)
    })
}

fn resolve_index(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    catalog: &dyn GraphCompilationCatalog,
    spec: &GraphFtsIndexSpec,
) -> Result<ResolvedIndex, GraphFtsError> {
    validate_spec(spec)?;
    let source = graph
        .node_sources
        .iter()
        .find(|source| source.name.eq_ignore_ascii_case(&spec.source))
        .ok_or_else(|| GraphFtsError::UnknownSource(spec.source.clone()))?;
    let table = connection
        .current_schema()
        .get_table(&source.table)
        .ok_or_else(|| CatalogError::SourceTableMissing(source.table.clone()))?;
    let payload = catalog
        .payload_columns(source.id)
        .ok_or_else(|| GraphFtsError::UnknownSource(spec.source.clone()))?;
    let mut physical_properties = Vec::with_capacity(spec.properties.len());
    for property in &spec.properties {
        let semantic_property =
            GraphCompilationCatalog::semantic_property_for_key(catalog, source.id, &[], property);
        let (physical, semantic_type) = match semantic_property {
            Some(Some((_, value_type, column))) => (column, Some(value_type)),
            Some(None) => {
                return Err(GraphFtsError::UnknownProperty {
                    source_name: spec.source.clone(),
                    property: property.clone(),
                });
            }
            None => {
                let (_, physical) = payload
                    .iter()
                    .find(|(logical, _)| logical.eq_ignore_ascii_case(property))
                    .ok_or_else(|| GraphFtsError::UnknownProperty {
                        source_name: spec.source.clone(),
                        property: property.clone(),
                    })?;
                (physical.clone(), None)
            }
        };
        let (_, column) =
            table
                .get_column_by_name(&physical)
                .ok_or_else(|| GraphFtsError::UnknownProperty {
                    source_name: spec.source.clone(),
                    property: property.clone(),
                })?;
        let is_text = match semantic_type {
            Some(value_type) => value_type == ValueType::Text,
            None => declared_type_has_text_affinity(&column.ty_str),
        };
        if !is_text {
            return Err(GraphFtsError::NonTextProperty {
                source_name: spec.source.clone(),
                property: property.clone(),
            });
        }
        physical_properties.push(physical);
    }
    let physical_weights = spec
        .weights
        .iter()
        .map(|weight| {
            let index = spec
                .properties
                .iter()
                .position(|property| property.eq_ignore_ascii_case(&weight.property))
                .ok_or_else(|| GraphFtsError::UnknownWeightedProperty(weight.property.clone()))?;
            Ok((physical_properties[index].clone(), weight.weight))
        })
        .collect::<Result<Vec<_>, GraphFtsError>>()?;
    let physical_name = format!(
        "__turso_graph_fts_{}_{:016x}",
        graph.id.get(),
        stable_hash(&format!(
            "{}:{}:{}",
            graph.id.get(),
            source.id.get(),
            spec.name.to_ascii_lowercase()
        ))
    );
    Ok(ResolvedIndex {
        index: GraphFtsIndex {
            graph_id: graph.id,
            physical_name,
            spec: spec.clone(),
        },
        table: source.table.clone(),
        physical_properties,
        physical_weights,
    })
}

fn declared_type_has_text_affinity(declared_type: &str) -> bool {
    let declared_type = declared_type.to_ascii_uppercase();
    declared_type.contains("CHAR")
        || declared_type.contains("CLOB")
        || declared_type.contains("TEXT")
}

fn validate_spec(spec: &GraphFtsIndexSpec) -> Result<(), GraphFtsError> {
    if spec.name.is_empty() {
        return Err(GraphFtsError::EmptyName);
    }
    if spec.name.contains('\0') {
        return Err(GraphFtsError::InvalidName);
    }
    if spec.name.len() > MAX_GRAPH_FTS_INDEX_NAME_BYTES {
        return Err(GraphFtsError::NameTooLong);
    }
    if spec.properties.is_empty() {
        return Err(GraphFtsError::NoProperties);
    }
    if spec.properties.len() > MAX_GRAPH_FTS_PROPERTIES {
        return Err(GraphFtsError::TooManyProperties);
    }
    let mut properties = BTreeSet::new();
    for property in &spec.properties {
        if !properties.insert(property.to_ascii_lowercase()) {
            return Err(GraphFtsError::DuplicateProperty(property.clone()));
        }
    }
    let mut weighted = BTreeSet::new();
    for weight in &spec.weights {
        if !weight.weight.is_finite() || weight.weight <= 0.0 {
            return Err(GraphFtsError::InvalidWeight {
                property: weight.property.clone(),
            });
        }
        if !weighted.insert(weight.property.to_ascii_lowercase()) {
            return Err(GraphFtsError::DuplicateProperty(weight.property.clone()));
        }
        if !properties.contains(&weight.property.to_ascii_lowercase()) {
            return Err(GraphFtsError::UnknownWeightedProperty(
                weight.property.clone(),
            ));
        }
    }
    Ok(())
}

fn require_index_methods(connection: &Arc<Connection>) -> Result<(), GraphFtsError> {
    if connection.experimental_index_method_enabled() {
        Ok(())
    } else {
        Err(GraphFtsError::IndexMethodsDisabled)
    }
}

fn ensure_metadata_table(connection: &Arc<Connection>) -> Result<(), GraphFtsError> {
    execute(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS {METADATA_TABLE}(\
             metadata_version INTEGER NOT NULL,\
             graph_id INTEGER NOT NULL,\
             logical_name TEXT NOT NULL COLLATE NOCASE,\
             source_name TEXT NOT NULL,\
             entity_kind TEXT NOT NULL,\
             properties_json TEXT NOT NULL,\
             tokenizer TEXT NOT NULL,\
             weights_json TEXT NOT NULL,\
             physical_name TEXT NOT NULL UNIQUE,\
             PRIMARY KEY(graph_id, logical_name))"
        ),
    )
}

fn insert_metadata(
    connection: &Arc<Connection>,
    index: &GraphFtsIndex,
) -> Result<(), GraphFtsError> {
    let properties = serde_json::to_string(&index.spec.properties)
        .map_err(|_| GraphFtsError::InvalidMetadata("properties"))?;
    let weights = serde_json::to_string(&index.spec.weights)
        .map_err(|_| GraphFtsError::InvalidMetadata("weights"))?;
    execute(
        connection,
        format!(
            "INSERT INTO {METADATA_TABLE} VALUES ({METADATA_VERSION}, {}, {}, {}, 'node', {}, {}, {}, {})",
            index.graph_id.get(),
            sql_string(&index.spec.name),
            sql_string(&index.spec.source),
            sql_string(&properties),
            sql_string(tokenizer_name(index.spec.tokenizer)),
            sql_string(&weights),
            sql_string(&index.physical_name),
        ),
    )
}

fn load_one(
    connection: &Arc<Connection>,
    graph: GraphId,
    logical_name: &str,
) -> Result<Option<GraphFtsIndex>, GraphFtsError> {
    Ok(load_rows(connection, graph, Some(logical_name))?
        .into_iter()
        .next())
}

fn load_rows(
    connection: &Arc<Connection>,
    graph: GraphId,
    logical_name: Option<&str>,
) -> Result<Vec<GraphFtsIndex>, GraphFtsError> {
    let predicate = logical_name
        .map(|name| format!(" AND logical_name = {} COLLATE NOCASE", sql_string(name)))
        .unwrap_or_default();
    let sql = format!(
        "SELECT metadata_version, logical_name, source_name, entity_kind, \
         properties_json, tokenizer, weights_json, physical_name \
         FROM {METADATA_TABLE} WHERE graph_id = {}{predicate} \
         ORDER BY logical_name COLLATE NOCASE",
        graph.get()
    );
    connection
        .prepare_internal(sql)?
        .run_collect_rows()?
        .iter()
        .map(|row| decode_row(graph, row))
        .collect()
}

fn decode_row(graph: GraphId, row: &[Value]) -> Result<GraphFtsIndex, GraphFtsError> {
    if integer(row, 0)? != METADATA_VERSION {
        return Err(GraphFtsError::InvalidMetadata("version"));
    }
    if text(row, 3)? != "node" {
        return Err(GraphFtsError::InvalidMetadata("entity kind"));
    }
    let properties = serde_json::from_str(text(row, 4)?)
        .map_err(|_| GraphFtsError::InvalidMetadata("properties"))?;
    let weights = serde_json::from_str(text(row, 6)?)
        .map_err(|_| GraphFtsError::InvalidMetadata("weights"))?;
    Ok(GraphFtsIndex {
        graph_id: graph,
        physical_name: text(row, 7)?.to_owned(),
        spec: GraphFtsIndexSpec {
            name: text(row, 1)?.to_owned(),
            entity: GraphFtsEntityKind::Node,
            source: text(row, 2)?.to_owned(),
            properties,
            tokenizer: parse_tokenizer(text(row, 5)?)?,
            weights,
        },
    })
}

fn tokenizer_name(tokenizer: GraphFtsTokenizer) -> &'static str {
    match tokenizer {
        GraphFtsTokenizer::Default => "default",
        GraphFtsTokenizer::Raw => "raw",
        GraphFtsTokenizer::Simple => "simple",
        GraphFtsTokenizer::Ngram => "ngram",
    }
}

fn parse_tokenizer(value: &str) -> Result<GraphFtsTokenizer, GraphFtsError> {
    match value {
        "default" => Ok(GraphFtsTokenizer::Default),
        "raw" => Ok(GraphFtsTokenizer::Raw),
        "simple" => Ok(GraphFtsTokenizer::Simple),
        "ngram" => Ok(GraphFtsTokenizer::Ngram),
        _ => Err(GraphFtsError::InvalidMetadata("tokenizer")),
    }
}

fn execute(connection: &Arc<Connection>, sql: String) -> Result<(), GraphFtsError> {
    connection.prepare_internal(sql)?.run_ignore_rows()?;
    Ok(())
}

fn transaction<T>(
    connection: &Arc<Connection>,
    operation: impl FnOnce() -> Result<T, GraphFtsError>,
) -> Result<T, GraphFtsError> {
    if !connection.get_auto_commit() && !connection.in_write_transaction() {
        return Err(GraphFtsError::RequiresWriteTransaction);
    }
    let (begin, commit, rollback) = if connection.get_auto_commit() {
        ("BEGIN IMMEDIATE", "COMMIT", "ROLLBACK")
    } else {
        (
            "SAVEPOINT __turso_graph_fts_admin",
            "RELEASE __turso_graph_fts_admin",
            "ROLLBACK TO __turso_graph_fts_admin; RELEASE __turso_graph_fts_admin",
        )
    };
    connection.execute(begin)?;
    match operation().and_then(|value| {
        connection.execute(commit)?;
        Ok(value)
    }) {
        Ok(value) => Ok(value),
        Err(cause) => match connection.execute(rollback) {
            Ok(()) => Err(cause),
            Err(rollback) => Err(GraphFtsError::RollbackFailed {
                cause: Box::new(cause),
                rollback,
            }),
        },
    }
}

fn integer(row: &[Value], index: usize) -> Result<i64, GraphFtsError> {
    match row.get(index) {
        Some(Value::Numeric(Numeric::Integer(value))) => Ok(*value),
        _ => Err(GraphFtsError::InvalidMetadata("integer")),
    }
}

fn text(row: &[Value], index: usize) -> Result<&str, GraphFtsError> {
    match row.get(index) {
        Some(Value::Text(value)) => Ok(value.as_str()),
        _ => Err(GraphFtsError::InvalidMetadata("text")),
    }
}

fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
