//! Opt-in semantic schema definitions.
//!
//! Semantic types and properties are conceptual graph identities. Physical
//! source and column names remain registration inputs and never become graph
//! identities.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use turso_core::Connection;
use turso_graph_ir as ir;

use crate::catalog::{
    execute_internal, integer, load_registered_graph, query_rows, scalar_integer, sql_string, text,
    CatalogError, RegisteredGraph, GENERATIONS_TABLE,
};

pub(crate) const SEMANTIC_TYPES_TABLE: &str = "__turso_internal_graph_semantic_types";
pub(crate) const SEMANTIC_PROPERTIES_TABLE: &str = "__turso_internal_graph_semantic_properties";
pub(crate) const SEMANTIC_OWNERSHIP_TABLE: &str = "__turso_internal_graph_semantic_ownership";
pub(crate) const SEMANTIC_ENDPOINTS_TABLE: &str = "__turso_internal_graph_semantic_endpoints";

const REGISTRATION_SAVEPOINT: &str = "turso_graph_register_semantic";

/// Immutable semantic catalog loaded once for a graph preparation context.
#[derive(Debug)]
pub struct SemanticSnapshot {
    node_names: HashMap<String, u32>,
    relationship_names: HashMap<String, u32>,
    node_types: HashMap<u32, SemanticTypeInfo>,
    relationship_types: HashMap<u32, SemanticTypeInfo>,
    endpoints: HashMap<u32, EndpointConstraint>,
}

/// Resolved semantic type identity, source mapping, and owned properties.
#[derive(Debug)]
pub struct SemanticTypeInfo {
    /// Preserved conceptual spelling.
    pub name: String,
    /// Persisted `LabelId` or `RelationshipTypeId` value.
    pub type_id: u32,
    /// Physical source mapped to this conceptual type.
    pub source: ir::SourceTableId,
    properties: HashMap<String, OwnedProperty>,
}

impl SemanticTypeInfo {
    /// Resolve an owned semantic property case-insensitively.
    pub fn property(&self, name: &str) -> Option<&OwnedProperty> {
        self.properties.get(&fold(name))
    }

    pub(crate) fn property_by_id(&self, id: ir::PropertyId) -> Option<&OwnedProperty> {
        self.properties.values().find(|property| property.id == id)
    }

    pub(crate) fn property_values(&self) -> impl Iterator<Item = &OwnedProperty> {
        self.properties.values()
    }
}

/// Resolved semantic property identity, type, nullability, and physical
/// lowering mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnedProperty {
    /// Preserved conceptual property spelling.
    pub name: String,
    /// Persisted conceptual property identity.
    pub id: ir::PropertyId,
    /// Value type derived from the physical schema.
    pub value_type: ir::ValueType,
    /// Nullability derived from the physical schema.
    pub nullability: ir::Nullability,
    /// Physical column used only by relational lowering.
    pub column: String,
}

/// Allowed semantic node types for a relationship's stored endpoints.
#[derive(Debug, Default)]
pub struct EndpointConstraint {
    /// Permitted start endpoint type IDs; empty means unconstrained.
    pub start: Vec<u32>,
    /// Permitted end endpoint type IDs; empty means unconstrained.
    pub end: Vec<u32>,
}

impl SemanticSnapshot {
    /// Resolve a semantic node type by conceptual name.
    pub fn node_type(&self, name: &str) -> Option<&SemanticTypeInfo> {
        self.node_names
            .get(&fold(name))
            .and_then(|id| self.node_types.get(id))
    }

    /// Resolve a semantic relationship type by conceptual name.
    pub fn relationship_type(&self, name: &str) -> Option<&SemanticTypeInfo> {
        self.relationship_names
            .get(&fold(name))
            .and_then(|id| self.relationship_types.get(id))
    }

    /// Resolve a semantic node type by persisted identity.
    pub fn node_type_by_id(&self, id: ir::LabelId) -> Option<&SemanticTypeInfo> {
        self.node_types.get(&id.get())
    }

    /// Resolve a semantic relationship type by persisted identity.
    pub fn relationship_type_by_id(&self, id: ir::RelationshipTypeId) -> Option<&SemanticTypeInfo> {
        self.relationship_types.get(&id.get())
    }

    /// Return stored endpoint constraints for a semantic relationship type.
    pub fn endpoints(&self, relationship: ir::RelationshipTypeId) -> Option<&EndpointConstraint> {
        self.endpoints.get(&relationship.get())
    }

    pub(crate) fn node_type_values(&self) -> impl Iterator<Item = &SemanticTypeInfo> {
        self.node_types.values()
    }

    pub(crate) fn relationship_type_values(&self) -> impl Iterator<Item = &SemanticTypeInfo> {
        self.relationship_types.values()
    }
}

/// Complete semantic schema registration for one already-registered graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticSchemaRegistration {
    /// Conceptual node types addressed through Cypher labels.
    pub node_types: Vec<SemanticNodeType>,
    /// Conceptual relationship types addressed through Cypher relationship
    /// types.
    pub relationship_types: Vec<SemanticRelationshipType>,
}

/// A conceptual node type and its physical source mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticNodeType {
    /// Conceptual name addressed through a Cypher label.
    pub name: String,
    /// Name of a registered node source.
    pub source: String,
    /// Properties owned by this semantic type.
    pub properties: Vec<SemanticProperty>,
}

/// A conceptual relationship type and its physical source mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticRelationshipType {
    /// Conceptual relationship type name.
    pub name: String,
    /// Name of a registered relationship source.
    pub source: String,
    /// Semantic node types permitted at the stored start endpoint.
    pub start: Vec<String>,
    /// Semantic node types permitted at the stored end endpoint.
    pub end: Vec<String>,
    /// Properties owned by this semantic type.
    pub properties: Vec<SemanticProperty>,
}

/// A conceptual property and its physical column mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticProperty {
    /// Conceptual property name used in Cypher.
    pub name: String,
    /// Physical payload column on the owning type's source.
    pub column: String,
}

/// Errors produced while validating or registering a semantic schema.
#[derive(Debug, Error)]
pub enum SemanticCatalogError {
    /// A required semantic identifier is empty or contains a NUL byte.
    #[error("{kind} name must not be empty")]
    EmptyName {
        /// Kind of identifier being validated.
        kind: &'static str,
    },
    /// A semantic type name is duplicated case-insensitively.
    #[error("semantic type name `{name}` is duplicated")]
    DuplicateTypeName {
        /// Duplicated spelling from the registration.
        name: String,
    },
    /// A property is duplicated within one semantic owner.
    #[error("semantic property `{property}` is duplicated on type `{owner}`")]
    DuplicatePropertyName {
        /// Owning semantic type.
        owner: String,
        /// Duplicated property name.
        property: String,
    },
    /// An endpoint references a node type absent from this registration.
    #[error(
        "relationship type `{relationship_type}` {endpoint} endpoint references unknown semantic node type `{node_type}`"
    )]
    UnknownEndpointType {
        /// Relationship whose endpoint is invalid.
        relationship_type: String,
        /// Stored endpoint kind.
        endpoint: &'static str,
        /// Unknown semantic node type.
        node_type: String,
    },
    /// A semantic type references a source of the wrong kind or an unknown
    /// source.
    #[error(
        "semantic type `{semantic_type}` references unknown {kind} source `{referenced_source}`"
    )]
    UnknownSource {
        /// Semantic type containing the mapping.
        semantic_type: String,
        /// Expected source kind.
        kind: &'static str,
        /// Referenced source name.
        referenced_source: String,
    },
    /// A semantic property maps to an identity or endpoint column.
    #[error(
        "semantic property `{property}` on type `{owner}` maps to structural column `{column}`"
    )]
    StructuralColumn {
        /// Owning semantic type.
        owner: String,
        /// Semantic property name.
        property: String,
        /// Structural physical column.
        column: String,
    },
    /// A semantic property maps to a column absent from its source table.
    #[error(
        "semantic property `{property}` on type `{owner}` maps to missing column `{column}` on table `{table}`"
    )]
    ColumnMissing {
        /// Owning semantic type.
        owner: String,
        /// Semantic property name.
        property: String,
        /// Missing physical column.
        column: String,
        /// Physical source table.
        table: String,
    },
    /// Owners of the same semantic property map it to incompatible types.
    #[error(
        "semantic property `{property}` has incompatible value types across owners: {first_owner} maps it to {first_type:?}, {second_owner} to {second_type:?}"
    )]
    IncompatiblePropertyType {
        /// Shared semantic property.
        property: String,
        /// First owner encountered during validation.
        first_owner: String,
        /// First owner's graph value type.
        first_type: Box<ir::ValueType>,
        /// Conflicting owner.
        second_owner: String,
        /// Conflicting graph value type.
        second_type: Box<ir::ValueType>,
    },
    /// A different schema is already registered for the graph.
    #[error("graph `{0}` already has a different semantic schema registered")]
    ConflictingSchema(String),
    /// The target physical graph is not registered.
    #[error("graph `{0}` is not registered")]
    GraphNotFound(String),
    /// A semantic catalog row could not be decoded.
    #[error("semantic catalog row has an invalid value in `{0}`")]
    InvalidCatalogValue(&'static str),
    /// The physical graph catalog rejected an operation.
    #[error("semantic catalog operation failed: {0}")]
    Catalog(#[from] CatalogError),
    /// The database rejected a semantic catalog operation.
    #[error("semantic catalog database operation failed: {0}")]
    Database(#[from] turso_core::LimboError),
    /// Both registration and its required rollback failed.
    #[error(
        "semantic registration failed and rollback also failed: {cause}; rollback: {rollback}"
    )]
    RollbackFailed {
        /// Original registration error.
        cause: Box<SemanticCatalogError>,
        /// Rollback failure.
        rollback: turso_core::LimboError,
    },
}

fn fold(name: &str) -> String {
    name.to_lowercase()
}

fn require_name(kind: &'static str, name: &str) -> Result<(), SemanticCatalogError> {
    if name.trim().is_empty() || name.contains('\0') {
        return Err(SemanticCatalogError::EmptyName { kind });
    }
    Ok(())
}

pub(crate) fn validate_registration_shape(
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    let mut type_names = HashSet::new();
    let mut node_type_names = HashSet::new();
    for node_type in &registration.node_types {
        require_name("semantic type", &node_type.name)?;
        require_name("source", &node_type.source)?;
        if !type_names.insert(fold(&node_type.name)) {
            return Err(SemanticCatalogError::DuplicateTypeName {
                name: node_type.name.clone(),
            });
        }
        node_type_names.insert(fold(&node_type.name));
        validate_properties(&node_type.name, &node_type.properties)?;
    }
    for relationship in &registration.relationship_types {
        require_name("semantic type", &relationship.name)?;
        require_name("source", &relationship.source)?;
        if !type_names.insert(fold(&relationship.name)) {
            return Err(SemanticCatalogError::DuplicateTypeName {
                name: relationship.name.clone(),
            });
        }
        validate_properties(&relationship.name, &relationship.properties)?;
        for (endpoint, allowed) in [("start", &relationship.start), ("end", &relationship.end)] {
            for node_type in allowed {
                if !node_type_names.contains(&fold(node_type)) {
                    return Err(SemanticCatalogError::UnknownEndpointType {
                        relationship_type: relationship.name.clone(),
                        endpoint,
                        node_type: node_type.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_properties(
    owner: &str,
    properties: &[SemanticProperty],
) -> Result<(), SemanticCatalogError> {
    let mut names = HashSet::new();
    for property in properties {
        require_name("property", &property.name)?;
        require_name("column", &property.column)?;
        if !names.insert(fold(&property.name)) {
            return Err(SemanticCatalogError::DuplicatePropertyName {
                owner: owner.to_owned(),
                property: property.name.clone(),
            });
        }
    }
    Ok(())
}

/// Register an opt-in semantic schema for an existing physical graph.
///
/// The complete definition is validated before catalog writes begin.
/// Identical replay is idempotent; a conflicting replay is rejected.
pub fn register_semantic_schema(
    connection: &Arc<Connection>,
    graph_name: &str,
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    validate_registration_shape(registration)?;
    let graph = match load_registered_graph(connection, graph_name) {
        Ok(graph) => graph,
        Err(CatalogError::GraphNotFound(_)) => {
            return Err(SemanticCatalogError::GraphNotFound(graph_name.to_owned()));
        }
        Err(error) => return Err(error.into()),
    };
    validate_against_graph(connection, &graph, registration)?;
    run_in_registration_transaction(connection, |connection| {
        register_semantic_in_transaction(connection, &graph, registration)
    })
}

fn run_in_registration_transaction(
    connection: &Arc<Connection>,
    operation: impl FnOnce(&Arc<Connection>) -> Result<(), SemanticCatalogError>,
) -> Result<(), SemanticCatalogError> {
    if !connection.get_auto_commit() && !connection.in_write_transaction() {
        return Err(CatalogError::RequiresWriteTransaction.into());
    }
    if connection.get_auto_commit() {
        connection.execute("BEGIN IMMEDIATE")?;
        let result = operation(connection).and_then(|()| {
            connection.execute("COMMIT")?;
            Ok(())
        });
        match result {
            Ok(()) => Ok(()),
            Err(cause) => match connection.execute("ROLLBACK") {
                Ok(()) => Err(cause),
                Err(rollback) => Err(SemanticCatalogError::RollbackFailed {
                    cause: Box::new(cause),
                    rollback,
                }),
            },
        }
    } else {
        connection.execute(format!("SAVEPOINT {REGISTRATION_SAVEPOINT}"))?;
        let result = operation(connection).and_then(|()| {
            connection.execute(format!("RELEASE {REGISTRATION_SAVEPOINT}"))?;
            Ok(())
        });
        match result {
            Ok(()) => Ok(()),
            Err(cause) => {
                let rollback = connection
                    .execute(format!("ROLLBACK TO {REGISTRATION_SAVEPOINT}"))
                    .and_then(|()| connection.execute(format!("RELEASE {REGISTRATION_SAVEPOINT}")));
                match rollback {
                    Ok(()) => Err(cause),
                    Err(rollback) => Err(SemanticCatalogError::RollbackFailed {
                        cause: Box::new(cause),
                        rollback,
                    }),
                }
            }
        }
    }
}

fn validate_against_graph(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    let mut property_types = HashMap::<String, (String, ir::ValueType)>::new();

    for node_type in &registration.node_types {
        let source = graph
            .node_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(&node_type.source))
            .ok_or_else(|| SemanticCatalogError::UnknownSource {
                semantic_type: node_type.name.clone(),
                kind: "node",
                referenced_source: node_type.source.clone(),
            })?;
        check_owned_columns(
            connection,
            &node_type.name,
            &node_type.properties,
            &source.table,
            &[source.identity_column.as_str()],
            &mut property_types,
        )?;
    }

    for relationship in &registration.relationship_types {
        let source = graph
            .relationship_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(&relationship.source))
            .ok_or_else(|| SemanticCatalogError::UnknownSource {
                semantic_type: relationship.name.clone(),
                kind: "relationship",
                referenced_source: relationship.source.clone(),
            })?;
        check_owned_columns(
            connection,
            &relationship.name,
            &relationship.properties,
            &source.table,
            &[
                source.identity_column.as_str(),
                source.start_column.as_str(),
                source.end_column.as_str(),
            ],
            &mut property_types,
        )?;
    }
    Ok(())
}

fn check_owned_columns(
    connection: &Arc<Connection>,
    owner: &str,
    properties: &[SemanticProperty],
    table_name: &str,
    structural: &[&str],
    property_types: &mut HashMap<String, (String, ir::ValueType)>,
) -> Result<(), SemanticCatalogError> {
    let schema = connection.current_schema();
    let table = schema.get_table(table_name).ok_or_else(|| {
        SemanticCatalogError::Catalog(CatalogError::SourceTableMissing(table_name.to_owned()))
    })?;
    for property in properties {
        if structural
            .iter()
            .any(|column| column.eq_ignore_ascii_case(&property.column))
        {
            return Err(SemanticCatalogError::StructuralColumn {
                owner: owner.to_owned(),
                property: property.name.clone(),
                column: property.column.clone(),
            });
        }
        let Some((_, column)) = table.get_column_by_name(&property.column) else {
            return Err(SemanticCatalogError::ColumnMissing {
                owner: owner.to_owned(),
                property: property.name.clone(),
                column: property.column.clone(),
                table: table_name.to_owned(),
            });
        };
        let value_type =
            crate::schema_catalog::column_value_type(&schema, column, table.is_strict());
        match property_types.entry(fold(&property.name)) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert((owner.to_owned(), value_type));
            }
            std::collections::hash_map::Entry::Occupied(entry) => {
                let (first_owner, first_type) = entry.get();
                let compatible = *first_type == value_type
                    || *first_type == ir::ValueType::Any
                    || value_type == ir::ValueType::Any;
                if !compatible {
                    return Err(SemanticCatalogError::IncompatiblePropertyType {
                        property: property.name.clone(),
                        first_owner: first_owner.clone(),
                        first_type: Box::new(first_type.clone()),
                        second_owner: owner.to_owned(),
                        second_type: Box::new(value_type),
                    });
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CatalogRows {
    types: Vec<(String, u64, String, u64)>,
    properties: Vec<(u64, String)>,
    ownership: Vec<(String, u64, u64, u64, String)>,
    endpoints: Vec<(u64, String, u64)>,
}

fn register_semantic_in_transaction(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    create_semantic_catalog(connection)?;
    let expected = catalog_rows_for_registration(graph, registration);
    let existing = load_catalog_rows(connection, graph.id.get())?;
    if !existing.types.is_empty()
        || !existing.properties.is_empty()
        || !existing.ownership.is_empty()
        || !existing.endpoints.is_empty()
    {
        return if existing.canonicalized() == expected.canonicalized() {
            Ok(())
        } else {
            Err(SemanticCatalogError::ConflictingSchema(graph.name.clone()))
        };
    }

    insert_catalog_rows(connection, graph.id.get(), &expected)?;
    execute_internal(
        connection,
        format!(
            "UPDATE {GENERATIONS_TABLE} SET generation = generation + 1 WHERE graph_id = {}",
            graph.id.get()
        ),
    )?;
    Ok(())
}

fn create_semantic_catalog(connection: &Arc<Connection>) -> Result<(), SemanticCatalogError> {
    for ddl in [
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_TYPES_TABLE}(\
                graph_id INTEGER NOT NULL, \
                kind TEXT NOT NULL CHECK(kind IN ('node', 'relationship')), \
                type_id INTEGER NOT NULL CHECK(type_id > 0), \
                name TEXT NOT NULL COLLATE NOCASE, \
                source_id INTEGER NOT NULL, \
                PRIMARY KEY(graph_id, kind, type_id), \
                UNIQUE(graph_id, name)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_PROPERTIES_TABLE}(\
                graph_id INTEGER NOT NULL, \
                property_id INTEGER NOT NULL CHECK(property_id > 0), \
                name TEXT NOT NULL COLLATE NOCASE, \
                PRIMARY KEY(graph_id, property_id), \
                UNIQUE(graph_id, name)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_OWNERSHIP_TABLE}(\
                graph_id INTEGER NOT NULL, \
                kind TEXT NOT NULL CHECK(kind IN ('node', 'relationship')), \
                type_id INTEGER NOT NULL, \
                property_id INTEGER NOT NULL, \
                source_id INTEGER NOT NULL, \
                column_name TEXT NOT NULL, \
                PRIMARY KEY(graph_id, kind, type_id, property_id)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_ENDPOINTS_TABLE}(\
                graph_id INTEGER NOT NULL, \
                relationship_type_id INTEGER NOT NULL, \
                endpoint TEXT NOT NULL CHECK(endpoint IN ('start', 'end')), \
                node_type_id INTEGER NOT NULL, \
                PRIMARY KEY(graph_id, relationship_type_id, endpoint, node_type_id)\
            )"
        ),
    ] {
        execute_internal(connection, ddl)?;
    }
    Ok(())
}

fn catalog_rows_for_registration(
    graph: &RegisteredGraph,
    registration: &SemanticSchemaRegistration,
) -> CatalogRows {
    let mut types = Vec::new();
    let mut properties = Vec::new();
    let mut ownership = Vec::new();
    let mut endpoints = Vec::new();
    let node_ids = registration
        .node_types
        .iter()
        .enumerate()
        .map(|(index, node_type)| (fold(&node_type.name), (index + 1) as u64))
        .collect::<HashMap<_, _>>();
    let mut property_ids = HashMap::<String, u64>::new();

    for (index, node_type) in registration.node_types.iter().enumerate() {
        let type_id = (index + 1) as u64;
        let source = graph
            .node_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(&node_type.source))
            .expect("registration was physically validated");
        types.push((
            "node".to_owned(),
            type_id,
            node_type.name.clone(),
            source.id.get(),
        ));
        add_property_rows(
            "node",
            type_id,
            source.id.get(),
            &node_type.properties,
            &mut property_ids,
            &mut properties,
            &mut ownership,
        );
    }
    for (index, relationship) in registration.relationship_types.iter().enumerate() {
        let type_id = (index + 1) as u64;
        let source = graph
            .relationship_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(&relationship.source))
            .expect("registration was physically validated");
        types.push((
            "relationship".to_owned(),
            type_id,
            relationship.name.clone(),
            source.id.get(),
        ));
        add_property_rows(
            "relationship",
            type_id,
            source.id.get(),
            &relationship.properties,
            &mut property_ids,
            &mut properties,
            &mut ownership,
        );
        for (endpoint, allowed) in [("start", &relationship.start), ("end", &relationship.end)] {
            for node_type in allowed {
                endpoints.push((type_id, endpoint.to_owned(), node_ids[&fold(node_type)]));
            }
        }
    }
    let mut rows = CatalogRows {
        types,
        properties,
        ownership,
        endpoints,
    };
    sort_catalog_rows(&mut rows);
    rows
}

fn add_property_rows(
    kind: &str,
    type_id: u64,
    source_id: u64,
    owner_properties: &[SemanticProperty],
    property_ids: &mut HashMap<String, u64>,
    properties: &mut Vec<(u64, String)>,
    ownership: &mut Vec<(String, u64, u64, u64, String)>,
) {
    for property in owner_properties {
        let folded_name = fold(&property.name);
        let property_id = *property_ids.entry(folded_name).or_insert_with(|| {
            let id = properties.len() as u64 + 1;
            properties.push((id, property.name.clone()));
            id
        });
        ownership.push((
            kind.to_owned(),
            type_id,
            property_id,
            source_id,
            property.column.clone(),
        ));
    }
}

fn load_catalog_rows(
    connection: &Arc<Connection>,
    graph_id: u64,
) -> Result<CatalogRows, SemanticCatalogError> {
    let mut types = query_rows(
        connection,
        &format!(
            "SELECT kind, type_id, name, source_id FROM {SEMANTIC_TYPES_TABLE} \
             WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            text(row, 0, "semantic type kind")?.to_owned(),
            positive_u64(integer(row, 1, "semantic type id")?, "semantic type id")?,
            text(row, 2, "semantic type name")?.to_owned(),
            positive_u64(integer(row, 3, "semantic source id")?, "semantic source id")?,
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let mut properties = query_rows(
        connection,
        &format!(
            "SELECT property_id, name FROM {SEMANTIC_PROPERTIES_TABLE} \
             WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            positive_u64(
                integer(row, 0, "semantic property id")?,
                "semantic property id",
            )?,
            text(row, 1, "semantic property name")?.to_owned(),
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let mut ownership = query_rows(
        connection,
        &format!(
            "SELECT kind, type_id, property_id, source_id, column_name \
             FROM {SEMANTIC_OWNERSHIP_TABLE} WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            text(row, 0, "semantic owner kind")?.to_owned(),
            positive_u64(
                integer(row, 1, "semantic owner type")?,
                "semantic owner type",
            )?,
            positive_u64(
                integer(row, 2, "semantic owner property")?,
                "semantic owner property",
            )?,
            positive_u64(
                integer(row, 3, "semantic owner source")?,
                "semantic owner source",
            )?,
            text(row, 4, "semantic owner column")?.to_owned(),
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let mut endpoints = query_rows(
        connection,
        &format!(
            "SELECT relationship_type_id, endpoint, node_type_id \
             FROM {SEMANTIC_ENDPOINTS_TABLE} WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            positive_u64(
                integer(row, 0, "semantic relationship type")?,
                "semantic relationship type",
            )?,
            text(row, 1, "semantic endpoint")?.to_owned(),
            positive_u64(
                integer(row, 2, "semantic endpoint node type")?,
                "semantic endpoint node type",
            )?,
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let mut rows = CatalogRows {
        types: std::mem::take(&mut types),
        properties: std::mem::take(&mut properties),
        ownership: std::mem::take(&mut ownership),
        endpoints: std::mem::take(&mut endpoints),
    };
    sort_catalog_rows(&mut rows);
    Ok(rows)
}

fn positive_u64(value: i64, kind: &'static str) -> Result<u64, SemanticCatalogError> {
    u64::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(SemanticCatalogError::InvalidCatalogValue(kind))
}

fn sort_catalog_rows(rows: &mut CatalogRows) {
    rows.types.sort();
    rows.properties.sort();
    rows.ownership.sort();
    rows.endpoints.sort();
}

impl CatalogRows {
    fn canonicalized(mut self) -> Self {
        for (_, _, name, _) in &mut self.types {
            *name = fold(name);
        }
        for (_, name) in &mut self.properties {
            *name = fold(name);
        }
        for (_, _, _, _, column) in &mut self.ownership {
            *column = fold(column);
        }
        sort_catalog_rows(&mut self);
        self
    }
}

fn insert_catalog_rows(
    connection: &Arc<Connection>,
    graph_id: u64,
    rows: &CatalogRows,
) -> Result<(), SemanticCatalogError> {
    for (kind, type_id, name, source_id) in &rows.types {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_TYPES_TABLE}(graph_id, kind, type_id, name, source_id) \
                 VALUES ({graph_id}, {}, {type_id}, {}, {source_id})",
                sql_string(kind),
                sql_string(name)
            ),
        )?;
    }
    for (property_id, name) in &rows.properties {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_PROPERTIES_TABLE}(graph_id, property_id, name) \
                 VALUES ({graph_id}, {property_id}, {})",
                sql_string(name)
            ),
        )?;
    }
    for (kind, type_id, property_id, source_id, column) in &rows.ownership {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_OWNERSHIP_TABLE}(\
                    graph_id, kind, type_id, property_id, source_id, column_name\
                 ) VALUES ({graph_id}, {}, {type_id}, {property_id}, {source_id}, {})",
                sql_string(kind),
                sql_string(column)
            ),
        )?;
    }
    for (relationship_type, endpoint, node_type) in &rows.endpoints {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_ENDPOINTS_TABLE}(\
                    graph_id, relationship_type_id, endpoint, node_type_id\
                 ) VALUES ({graph_id}, {relationship_type}, {}, {node_type})",
                sql_string(endpoint)
            ),
        )?;
    }
    Ok(())
}

/// Load the immutable semantic catalog for a graph.
///
/// Graphs without semantic rows return `None` and retain legacy resolution.
pub fn load_semantic_snapshot(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
) -> Result<Option<SemanticSnapshot>, SemanticCatalogError> {
    let table_count = scalar_integer(
        connection,
        &format!(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = {}",
            sql_string(SEMANTIC_TYPES_TABLE)
        ),
        "semantic catalog table count",
    )?;
    if table_count == 0 {
        return Ok(None);
    }
    let type_rows = query_rows(
        connection,
        &format!(
            "SELECT kind, type_id, name, source_id FROM {SEMANTIC_TYPES_TABLE} \
             WHERE graph_id = {}",
            graph.id.get()
        ),
    )?;
    if type_rows.is_empty() {
        return Ok(None);
    }

    let mut snapshot = SemanticSnapshot {
        node_names: HashMap::new(),
        relationship_names: HashMap::new(),
        node_types: HashMap::new(),
        relationship_types: HashMap::new(),
        endpoints: HashMap::new(),
    };
    for row in &type_rows {
        let kind = text(row, 0, "semantic type kind")?;
        let type_id = positive_u32(integer(row, 1, "semantic type id")?, "semantic type id")?;
        let name = text(row, 2, "semantic type name")?.to_owned();
        let source = ir::SourceTableId::new(positive_u64(
            integer(row, 3, "semantic source id")?,
            "semantic source id",
        )?)
        .map_err(|_| SemanticCatalogError::InvalidCatalogValue("semantic source id"))?;
        let info = SemanticTypeInfo {
            name: name.clone(),
            type_id,
            source,
            properties: HashMap::new(),
        };
        let (names, types) = match kind {
            "node" => (&mut snapshot.node_names, &mut snapshot.node_types),
            "relationship" => (
                &mut snapshot.relationship_names,
                &mut snapshot.relationship_types,
            ),
            _ => {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic type kind",
                ));
            }
        };
        if names.insert(fold(&name), type_id).is_some() || types.insert(type_id, info).is_some() {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "duplicate semantic type",
            ));
        }
    }

    let property_rows = query_rows(
        connection,
        &format!(
            "SELECT property_id, name FROM {SEMANTIC_PROPERTIES_TABLE} \
             WHERE graph_id = {}",
            graph.id.get()
        ),
    )?;
    let mut property_names = HashMap::new();
    for row in &property_rows {
        let property_id = positive_u32(
            integer(row, 0, "semantic property id")?,
            "semantic property id",
        )?;
        let name = text(row, 1, "semantic property name")?.to_owned();
        if property_names.insert(property_id, name).is_some() {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "duplicate semantic property",
            ));
        }
    }

    let ownership_rows = query_rows(
        connection,
        &format!(
            "SELECT kind, type_id, property_id, source_id, column_name \
             FROM {SEMANTIC_OWNERSHIP_TABLE} WHERE graph_id = {}",
            graph.id.get()
        ),
    )?;
    for row in &ownership_rows {
        let kind = text(row, 0, "semantic owner kind")?;
        let type_id = positive_u32(
            integer(row, 1, "semantic owner type")?,
            "semantic owner type",
        )?;
        let property_id = positive_u32(
            integer(row, 2, "semantic owner property")?,
            "semantic owner property",
        )?;
        let source_id = ir::SourceTableId::new(positive_u64(
            integer(row, 3, "semantic owner source")?,
            "semantic owner source",
        )?)
        .map_err(|_| SemanticCatalogError::InvalidCatalogValue("semantic owner source"))?;
        let column_name = text(row, 4, "semantic owner column")?.to_owned();
        let property_name =
            property_names
                .get(&property_id)
                .ok_or(SemanticCatalogError::InvalidCatalogValue(
                    "semantic owner property",
                ))?;
        let (type_info, table_name) =
            semantic_owner_and_table(&mut snapshot, graph, kind, type_id, source_id)?;
        let schema = connection.current_schema();
        let table = schema.get_table(table_name).ok_or_else(|| {
            SemanticCatalogError::Catalog(CatalogError::SourceTableMissing(table_name.to_owned()))
        })?;
        let Some((_, column)) = table.get_column_by_name(&column_name) else {
            return Err(SemanticCatalogError::ColumnMissing {
                owner: type_info.name.clone(),
                property: property_name.clone(),
                column: column_name,
                table: table_name.to_owned(),
            });
        };
        let property_id = ir::PropertyId::new(property_id)
            .map_err(|_| SemanticCatalogError::InvalidCatalogValue("semantic property id"))?;
        let owned = OwnedProperty {
            name: property_name.clone(),
            id: property_id,
            value_type: crate::schema_catalog::column_value_type(
                &schema,
                column,
                table.is_strict(),
            ),
            nullability: crate::schema_catalog::column_nullability(column),
            column: column_name,
        };
        if type_info
            .properties
            .insert(fold(property_name), owned)
            .is_some()
        {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "duplicate semantic ownership",
            ));
        }
    }

    let endpoint_rows = query_rows(
        connection,
        &format!(
            "SELECT relationship_type_id, endpoint, node_type_id \
             FROM {SEMANTIC_ENDPOINTS_TABLE} WHERE graph_id = {}",
            graph.id.get()
        ),
    )?;
    for row in &endpoint_rows {
        let relationship_type = positive_u32(
            integer(row, 0, "semantic endpoint relationship")?,
            "semantic endpoint relationship",
        )?;
        if !snapshot.relationship_types.contains_key(&relationship_type) {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "semantic endpoint relationship",
            ));
        }
        let endpoint = text(row, 1, "semantic endpoint kind")?;
        let node_type = positive_u32(
            integer(row, 2, "semantic endpoint node")?,
            "semantic endpoint node",
        )?;
        if !snapshot.node_types.contains_key(&node_type) {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "semantic endpoint node",
            ));
        }
        let constraints = snapshot.endpoints.entry(relationship_type).or_default();
        match endpoint {
            "start" => constraints.start.push(node_type),
            "end" => constraints.end.push(node_type),
            _ => {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic endpoint kind",
                ));
            }
        }
    }
    for constraints in snapshot.endpoints.values_mut() {
        constraints.start.sort_unstable();
        constraints.end.sort_unstable();
    }
    Ok(Some(snapshot))
}

fn semantic_owner_and_table<'a>(
    snapshot: &'a mut SemanticSnapshot,
    graph: &'a RegisteredGraph,
    kind: &str,
    type_id: u32,
    source_id: ir::SourceTableId,
) -> Result<(&'a mut SemanticTypeInfo, &'a str), SemanticCatalogError> {
    match kind {
        "node" => {
            let info = snapshot.node_types.get_mut(&type_id).ok_or(
                SemanticCatalogError::InvalidCatalogValue("semantic owner type"),
            )?;
            if info.source != source_id {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic owner source",
                ));
            }
            let source = graph
                .node_sources
                .iter()
                .find(|source| source.id == source_id)
                .ok_or(SemanticCatalogError::InvalidCatalogValue(
                    "semantic owner source",
                ))?;
            Ok((info, &source.table))
        }
        "relationship" => {
            let info = snapshot.relationship_types.get_mut(&type_id).ok_or(
                SemanticCatalogError::InvalidCatalogValue("semantic owner type"),
            )?;
            if info.source != source_id {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic owner source",
                ));
            }
            let source = graph
                .relationship_sources
                .iter()
                .find(|source| source.id == source_id)
                .ok_or(SemanticCatalogError::InvalidCatalogValue(
                    "semantic owner source",
                ))?;
            Ok((info, &source.table))
        }
        _ => Err(SemanticCatalogError::InvalidCatalogValue(
            "semantic owner kind",
        )),
    }
}

fn positive_u32(value: i64, kind: &'static str) -> Result<u32, SemanticCatalogError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(SemanticCatalogError::InvalidCatalogValue(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person_registration() -> SemanticSchemaRegistration {
        SemanticSchemaRegistration {
            node_types: vec![SemanticNodeType {
                name: "Customer".to_owned(),
                source: "Person".to_owned(),
                properties: vec![SemanticProperty {
                    name: "fullName".to_owned(),
                    column: "name".to_owned(),
                }],
            }],
            relationship_types: vec![],
        }
    }

    #[test]
    fn duplicate_type_names_are_rejected_case_insensitively() {
        let mut registration = person_registration();
        let mut duplicate = registration.node_types[0].clone();
        duplicate.name = "CUSTOMER".to_owned();
        registration.node_types.push(duplicate);

        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::DuplicateTypeName { name }) if name == "CUSTOMER"
        ));
    }

    #[test]
    fn empty_names_are_rejected() {
        let mut registration = person_registration();
        registration.node_types[0].properties[0].name = " ".to_owned();

        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::EmptyName { kind: "property" })
        ));
    }

    #[test]
    fn duplicate_property_names_within_one_owner_are_rejected() {
        let mut registration = person_registration();
        let duplicate = registration.node_types[0].properties[0].clone();
        registration.node_types[0].properties.push(duplicate);

        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::DuplicatePropertyName { .. })
        ));
    }

    #[test]
    fn relationship_endpoints_must_reference_declared_node_types() {
        let mut registration = person_registration();
        registration
            .relationship_types
            .push(SemanticRelationshipType {
                name: "OWNS".to_owned(),
                source: "KNOWS".to_owned(),
                start: vec!["Customer".to_owned()],
                end: vec!["Ghost".to_owned()],
                properties: vec![],
            });

        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::UnknownEndpointType { node_type, .. })
                if node_type == "Ghost"
        ));
    }

    #[test]
    fn registration_round_trips_through_serde_json() {
        let registration = person_registration();
        let json = serde_json::to_string(&registration).expect("serialize registration");
        let decoded = serde_json::from_str::<SemanticSchemaRegistration>(&json)
            .expect("deserialize registration");

        assert_eq!(registration, decoded);
    }
}
