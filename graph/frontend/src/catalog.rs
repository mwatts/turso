use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use thiserror::Error;
use turso_core::{
    schema::{
        TURSO_GRAPH_CATALOG_PREFIX, TURSO_GRAPH_GENERATIONS_TABLE_NAME,
        TURSO_GRAPH_GENERATION_TRIGGER_PREFIX,
    },
    Connection, Numeric, Value,
};
use turso_graph_ir::{GraphId, SourceTableId};

const RESERVED_PREFIX: &str = "__turso_";
pub(crate) const GRAPHS_TABLE: &str = "__turso_internal_graph_graphs";
pub(crate) const GENERATIONS_TABLE: &str = TURSO_GRAPH_GENERATIONS_TABLE_NAME;
pub(crate) const SOURCES_TABLE: &str = "__turso_internal_graph_sources";
pub(crate) const NODE_SOURCES_TABLE: &str = "__turso_internal_graph_node_sources";
pub(crate) const RELATIONSHIP_SOURCES_TABLE: &str = "__turso_internal_graph_relationship_sources";

pub const GRAPH_CATALOG_VERSION: u64 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeSourceRegistration {
    pub name: String,
    pub table: String,
    pub identity_column: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipSourceRegistration {
    pub name: String,
    pub table: String,
    pub identity_column: String,
    pub start_column: String,
    pub end_column: String,
    pub start_node_source: String,
    pub end_node_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphRegistration {
    pub name: String,
    pub node_sources: Vec<NodeSourceRegistration>,
    pub relationship_sources: Vec<RelationshipSourceRegistration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredNodeSource {
    pub id: SourceTableId,
    pub name: String,
    pub table: String,
    pub identity_column: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredRelationshipSource {
    pub id: SourceTableId,
    pub name: String,
    pub table: String,
    pub identity_column: String,
    pub start_column: String,
    pub end_column: String,
    pub start_node_source: SourceTableId,
    pub end_node_source: SourceTableId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredGraph {
    pub id: GraphId,
    pub name: String,
    pub generation: u64,
    pub node_sources: Vec<RegisteredNodeSource>,
    pub relationship_sources: Vec<RegisteredRelationshipSource>,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("graph name `{0}` is reserved for internal use")]
    ReservedGraphName(String),
    #[error("{kind} name must not be empty")]
    EmptyName { kind: &'static str },
    #[error("graph registration requires at least one node source")]
    NoNodeSources,
    #[error("graph registration supports exactly one {kind} source; binding, mutation, and detach-delete resolve only the first registered source")]
    MultipleSourcesUnsupported { kind: &'static str },
    #[error("graph registration inside an open transaction requires a write transaction (BEGIN IMMEDIATE or a prior write)")]
    RequiresWriteTransaction,
    #[error("{kind} name `{name}` is duplicated")]
    DuplicateName { kind: &'static str, name: String },
    #[error("graph `{0}` is already registered")]
    GraphAlreadyExists(String),
    #[error("graph `{0}` is not registered")]
    GraphNotFound(String),
    #[error("source table `{0}` does not exist")]
    SourceTableMissing(String),
    #[error("source column `{column}` does not exist on table `{table}`")]
    SourceColumnMissing { table: String, column: String },
    #[error("identity column `{column}` on table `{table}` is not a primary key or single-column unique index")]
    IdentityNotUnique { table: String, column: String },
    #[error("relationship source `{relationship}` references unknown node source `{node_source}`")]
    UnknownEndpoint {
        relationship: String,
        node_source: String,
    },
    #[error("catalog row contains an invalid {kind} identity: {value}")]
    InvalidIdentity { kind: &'static str, value: i64 },
    #[error("catalog row has an invalid value in `{0}`")]
    InvalidCatalogValue(&'static str),
    #[error("source table `{table}` has a struct/union/custom-typed column `{column}` but this connection does not have --experimental-custom-types enabled")]
    CustomTypesDisabled { table: String, column: String },
    #[error("catalog operation failed: {0}")]
    Database(#[from] turso_core::LimboError),
    #[error("registration failed and rollback also failed: {cause}; rollback: {rollback}")]
    RollbackFailed {
        cause: Box<CatalogError>,
        rollback: turso_core::LimboError,
    },
}

const REGISTRATION_SAVEPOINT: &str = "turso_graph_register";

pub fn register_graph(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
) -> Result<RegisteredGraph, CatalogError> {
    validate_registration_names(registration)?;
    // Inside an open user transaction a top-level BEGIN would fail (or
    // interfere with the caller's transaction state), so scope the
    // registration with a savepoint there; it then commits or rolls back
    // with the outer transaction. Registration runs internal (nested)
    // statements, which cannot upgrade a read transaction to a write
    // transaction, so a deferred transaction that has not yet written must
    // be rejected instead of panicking in the engine.
    if !connection.get_auto_commit() && !connection.in_write_transaction() {
        return Err(CatalogError::RequiresWriteTransaction);
    }
    if connection.get_auto_commit() {
        connection.execute("BEGIN IMMEDIATE")?;
        let result = register_graph_in_transaction(connection, registration).and_then(|graph| {
            connection.execute("COMMIT")?;
            Ok(graph)
        });
        match result {
            Ok(graph) => Ok(graph),
            Err(cause) => match connection.execute("ROLLBACK") {
                Ok(()) => Err(cause),
                Err(rollback) => Err(CatalogError::RollbackFailed {
                    cause: Box::new(cause),
                    rollback,
                }),
            },
        }
    } else {
        connection.execute(format!("SAVEPOINT {REGISTRATION_SAVEPOINT}"))?;
        let result = register_graph_in_transaction(connection, registration).and_then(|graph| {
            connection.execute(format!("RELEASE {REGISTRATION_SAVEPOINT}"))?;
            Ok(graph)
        });
        match result {
            Ok(graph) => Ok(graph),
            Err(cause) => {
                let rollback = connection
                    .execute(format!("ROLLBACK TO {REGISTRATION_SAVEPOINT}"))
                    .and_then(|()| connection.execute(format!("RELEASE {REGISTRATION_SAVEPOINT}")));
                match rollback {
                    Ok(()) => Err(cause),
                    Err(rollback) => Err(CatalogError::RollbackFailed {
                        cause: Box::new(cause),
                        rollback,
                    }),
                }
            }
        }
    }
}

pub fn load_registered_graph(
    connection: &Arc<Connection>,
    name: &str,
) -> Result<RegisteredGraph, CatalogError> {
    ensure_catalog_exists(connection)?;
    let graph_rows = query_rows(
        connection,
        &format!(
            "SELECT g.id, g.name, gen.generation FROM {GRAPHS_TABLE} g \
             JOIN {GENERATIONS_TABLE} gen ON gen.graph_id = g.id WHERE g.name = {} COLLATE NOCASE",
            sql_string(name)
        ),
    )?;
    let row = graph_rows
        .first()
        .ok_or_else(|| CatalogError::GraphNotFound(name.to_owned()))?;
    let graph_id = graph_id(integer(row, 0, "graph id")?)?;
    let graph_name = text(row, 1, "graph name")?.to_owned();
    let generation = nonnegative_u64(integer(row, 2, "generation")?, "generation")?;

    let node_rows = query_rows(
        connection,
        &format!(
            "SELECT s.id, s.name, n.table_name, n.identity_column FROM {SOURCES_TABLE} s \
             JOIN {NODE_SOURCES_TABLE} n ON n.source_id = s.id \
             WHERE s.graph_id = {} ORDER BY s.id",
            graph_id.get()
        ),
    )?;
    let mut node_sources = Vec::with_capacity(node_rows.len());
    for row in node_rows {
        let source = RegisteredNodeSource {
            id: source_id(integer(&row, 0, "node source id")?)?,
            name: text(&row, 1, "node source name")?.to_owned(),
            table: text(&row, 2, "node source table")?.to_owned(),
            identity_column: text(&row, 3, "identity column")?.to_owned(),
        };
        require_columns(connection, &source.table, &[&source.identity_column])?;
        node_sources.push(source);
    }

    let relationship_rows = query_rows(
        connection,
        &format!(
            "SELECT s.id, s.name, r.table_name, r.identity_column, r.start_column, r.end_column, \
             r.start_node_source_id, r.end_node_source_id FROM {SOURCES_TABLE} s \
             JOIN {RELATIONSHIP_SOURCES_TABLE} r ON r.source_id = s.id \
             WHERE s.graph_id = {} ORDER BY s.id",
            graph_id.get()
        ),
    )?;
    let mut relationship_sources = Vec::with_capacity(relationship_rows.len());
    for row in relationship_rows {
        let source = RegisteredRelationshipSource {
            id: source_id(integer(&row, 0, "relationship source id")?)?,
            name: text(&row, 1, "relationship source name")?.to_owned(),
            table: text(&row, 2, "relationship source table")?.to_owned(),
            identity_column: text(&row, 3, "relationship identity column")?.to_owned(),
            start_column: text(&row, 4, "start column")?.to_owned(),
            end_column: text(&row, 5, "end column")?.to_owned(),
            start_node_source: source_id(integer(&row, 6, "start node source id")?)?,
            end_node_source: source_id(integer(&row, 7, "end node source id")?)?,
        };
        require_columns(
            connection,
            &source.table,
            &[
                &source.identity_column,
                &source.start_column,
                &source.end_column,
            ],
        )?;
        relationship_sources.push(source);
    }
    Ok(RegisteredGraph {
        id: graph_id,
        name: graph_name,
        generation,
        node_sources,
        relationship_sources,
    })
}

pub fn graph_generation(connection: &Arc<Connection>, name: &str) -> Result<u64, CatalogError> {
    Ok(load_registered_graph(connection, name)?.generation)
}

fn register_graph_in_transaction(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
) -> Result<RegisteredGraph, CatalogError> {
    create_catalog(connection)?;
    if !query_rows(
        connection,
        &format!(
            "SELECT id FROM {GRAPHS_TABLE} WHERE name = {} COLLATE NOCASE",
            sql_string(&registration.name)
        ),
    )?
    .is_empty()
    {
        return Err(CatalogError::GraphAlreadyExists(registration.name.clone()));
    }
    for node in &registration.node_sources {
        let columns = require_columns(connection, &node.table, &[&node.identity_column])?;
        require_unique_identity(connection, &node.table, &node.identity_column, &columns)?;
        require_custom_types_enabled_for_source(connection, &node.table)?;
    }
    for relationship in &registration.relationship_sources {
        let columns = require_columns(
            connection,
            &relationship.table,
            &[
                &relationship.identity_column,
                &relationship.start_column,
                &relationship.end_column,
            ],
        )?;
        require_custom_types_enabled_for_source(connection, &relationship.table)?;
        require_unique_identity(
            connection,
            &relationship.table,
            &relationship.identity_column,
            &columns,
        )?;
    }

    execute_internal(
        connection,
        format!(
            "INSERT INTO {GRAPHS_TABLE}(name) VALUES ({});",
            sql_string(&registration.name)
        ),
    )?;
    let graph_id_value = scalar_integer(
        connection,
        &format!(
            "SELECT id FROM {GRAPHS_TABLE} WHERE name = {} COLLATE NOCASE",
            sql_string(&registration.name)
        ),
        "graph id",
    )?;
    let graph_id = graph_id(graph_id_value)?;
    execute_internal(
        connection,
        format!(
            "INSERT INTO {GENERATIONS_TABLE}(graph_id, generation) VALUES ({}, 0)",
            graph_id.get()
        ),
    )?;

    let mut node_ids = HashMap::new();
    for node in &registration.node_sources {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SOURCES_TABLE}(graph_id, name, kind) VALUES ({}, {}, 'node')",
                graph_id.get(),
                sql_string(&node.name)
            ),
        )?;
        let id = scalar_integer(
            connection,
            &format!(
                "SELECT id FROM {SOURCES_TABLE} WHERE graph_id = {} AND name = {}",
                graph_id.get(),
                sql_string(&node.name)
            ),
            "node source id",
        )?;
        let id = source_id(id)?;
        execute_internal(connection, format!(
            "INSERT INTO {NODE_SOURCES_TABLE}(source_id, table_name, identity_column) VALUES ({}, {}, {})",
            id.get(),
            sql_string(&node.table),
            sql_string(&node.identity_column)
        ))?;
        node_ids.insert(node.name.clone(), id);
    }

    for relationship in &registration.relationship_sources {
        let start = node_ids
            .get(&relationship.start_node_source)
            .ok_or_else(|| CatalogError::UnknownEndpoint {
                relationship: relationship.name.clone(),
                node_source: relationship.start_node_source.clone(),
            })?;
        let end = node_ids.get(&relationship.end_node_source).ok_or_else(|| {
            CatalogError::UnknownEndpoint {
                relationship: relationship.name.clone(),
                node_source: relationship.end_node_source.clone(),
            }
        })?;
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SOURCES_TABLE}(graph_id, name, kind) VALUES ({}, {}, 'relationship')",
                graph_id.get(),
                sql_string(&relationship.name)
            ),
        )?;
        let relationship_id = scalar_integer(
            connection,
            &format!(
                "SELECT id FROM {SOURCES_TABLE} WHERE graph_id = {} AND name = {}",
                graph_id.get(),
                sql_string(&relationship.name)
            ),
            "relationship source id",
        )?;
        execute_internal(connection, format!(
            "INSERT INTO {RELATIONSHIP_SOURCES_TABLE}(source_id, table_name, identity_column, start_column, end_column, start_node_source_id, end_node_source_id) \
             VALUES ({}, {}, {}, {}, {}, {}, {})",
            relationship_id,
            sql_string(&relationship.table),
            sql_string(&relationship.identity_column),
            sql_string(&relationship.start_column),
            sql_string(&relationship.end_column),
            start.get(),
            end.get()
        ))?;
        install_endpoint_index(
            connection,
            graph_id,
            relationship,
            "start",
            &relationship.start_column,
        )?;
        install_endpoint_index(
            connection,
            graph_id,
            relationship,
            "end",
            &relationship.end_column,
        )?;
        // Co-membership patterns bind both endpoints before matching the
        // second relationship; the composite index turns that probe from an
        // in-degree scan into an exact lookup.
        install_endpoint_pair_index(connection, graph_id, relationship)?;
    }

    let mut mapped_tables = HashSet::new();
    mapped_tables.extend(
        registration
            .node_sources
            .iter()
            .map(|source| source.table.as_str()),
    );
    mapped_tables.extend(
        registration
            .relationship_sources
            .iter()
            .map(|source| source.table.as_str()),
    );
    for table in mapped_tables {
        install_generation_triggers(connection, graph_id, table)?;
    }
    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS \"{}\"(node_id INTEGER NOT NULL, label TEXT NOT NULL)",
            labels_table_name(graph_id)
        ),
    )?;
    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS \"{}\"(relationship_id INTEGER NOT NULL, type TEXT NOT NULL)",
            relationship_types_table_name(graph_id)
        ),
    )?;
    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS \"{}\"(id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE)",
            relationship_type_registry_table_name(graph_id)
        ),
    )?;
    // Standard junction indexes: label-filtered scans probe (label,
    // node_id); per-entity lookups (labels(n), type(r), snapshot builds)
    // probe by identity.
    for (table, columns) in [
        (labels_table_name(graph_id), ["node_id, label"]),
        (
            relationship_types_table_name(graph_id),
            ["relationship_id, type"],
        ),
    ] {
        for (index, column_list) in columns.iter().enumerate() {
            execute_internal(
                connection,
                format!(
                    "CREATE INDEX IF NOT EXISTS \"{table}_ix{index}\" ON \"{table}\"({column_list})",
                ),
            )?;
        }
    }
    // Registered relationship sources keep their index-based identities so
    // the registry agrees with the schema catalog's static resolution.
    for (index, relationship) in registration.relationship_sources.iter().enumerate() {
        execute_internal(
            connection,
            format!(
                "INSERT INTO \"{}\"(id, name) VALUES ({}, {})",
                relationship_type_registry_table_name(graph_id),
                index + 1,
                sql_string(&relationship.name)
            ),
        )?;
    }
    load_registered_graph(connection, &registration.name)
}

/// Name of the per-graph node-label junction table.
pub fn labels_table_name(graph: GraphId) -> String {
    format!("__turso_graph_node_labels_{}", graph.get())
}

/// Name of the per-graph relationship-type junction table.
pub fn relationship_types_table_name(graph: GraphId) -> String {
    format!("__turso_graph_relationship_types_{}", graph.get())
}

/// Name of the per-graph relationship-type identity registry.
pub fn relationship_type_registry_table_name(graph: GraphId) -> String {
    format!("__turso_graph_relationship_type_registry_{}", graph.get())
}

fn create_catalog(connection: &Arc<Connection>) -> Result<(), CatalogError> {
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {GRAPHS_TABLE}(id INTEGER PRIMARY KEY, name TEXT NOT NULL COLLATE NOCASE UNIQUE)"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {GENERATIONS_TABLE}(graph_id INTEGER PRIMARY KEY, generation INTEGER NOT NULL CHECK(generation >= 0))"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {SOURCES_TABLE}(id INTEGER PRIMARY KEY, graph_id INTEGER NOT NULL, name TEXT NOT NULL COLLATE NOCASE, kind TEXT NOT NULL CHECK(kind IN ('node', 'relationship')), UNIQUE(graph_id, name))"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {NODE_SOURCES_TABLE}(source_id INTEGER PRIMARY KEY, table_name TEXT NOT NULL, identity_column TEXT NOT NULL)"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {RELATIONSHIP_SOURCES_TABLE}(source_id INTEGER PRIMARY KEY, table_name TEXT NOT NULL, identity_column TEXT NOT NULL, start_column TEXT NOT NULL, end_column TEXT NOT NULL, start_node_source_id INTEGER NOT NULL, end_node_source_id INTEGER NOT NULL)"
    ))?;
    Ok(())
}

fn ensure_catalog_exists(connection: &Arc<Connection>) -> Result<(), CatalogError> {
    let rows = query_rows(
        connection,
        &format!(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name = {}",
            sql_string(GRAPHS_TABLE)
        ),
    )?;
    if rows.is_empty() {
        return Err(CatalogError::GraphNotFound(
            "catalog is not initialized".to_owned(),
        ));
    }
    Ok(())
}

fn validate_registration_names(registration: &GraphRegistration) -> Result<(), CatalogError> {
    validate_name("graph", &registration.name)?;
    if registration
        .name
        .to_ascii_lowercase()
        .starts_with(RESERVED_PREFIX)
    {
        return Err(CatalogError::ReservedGraphName(registration.name.clone()));
    }
    if registration.node_sources.is_empty() {
        return Err(CatalogError::NoNodeSources);
    }
    let mut source_names = HashSet::new();
    for source in &registration.node_sources {
        validate_name("node source", &source.name)?;
        validate_source_identifiers(&source.table, &[&source.identity_column])?;
        if !source_names.insert(source.name.to_ascii_lowercase()) {
            return Err(CatalogError::DuplicateName {
                kind: "node source",
                name: source.name.clone(),
            });
        }
    }
    for source in &registration.relationship_sources {
        validate_name("relationship source", &source.name)?;
        validate_source_identifiers(
            &source.table,
            &[
                &source.identity_column,
                &source.start_column,
                &source.end_column,
            ],
        )?;
        if !source_names.insert(source.name.to_ascii_lowercase()) {
            return Err(CatalogError::DuplicateName {
                kind: "relationship source",
                name: source.name.clone(),
            });
        }
    }
    // The rest of the stack (SchemaCatalog layout/property resolution, binder
    // CREATE/MATCH targets, DETACH DELETE) resolves only the first source of
    // each kind. Accepting more would silently misroute reads and writes, so
    // fail closed until multi-source resolution is implemented end to end.
    if registration.node_sources.len() > 1 {
        return Err(CatalogError::MultipleSourcesUnsupported { kind: "node" });
    }
    if registration.relationship_sources.len() > 1 {
        return Err(CatalogError::MultipleSourcesUnsupported {
            kind: "relationship",
        });
    }
    Ok(())
}

fn validate_source_identifiers(table: &str, columns: &[&str]) -> Result<(), CatalogError> {
    validate_name("source table", table)?;
    if table.to_ascii_lowercase().starts_with(RESERVED_PREFIX) {
        return Err(CatalogError::ReservedGraphName(table.to_owned()));
    }
    for column in columns {
        validate_name("source column", column)?;
    }
    Ok(())
}

fn validate_name(kind: &'static str, name: &str) -> Result<(), CatalogError> {
    if name.trim().is_empty() || name.contains('\0') {
        Err(CatalogError::EmptyName { kind })
    } else {
        Ok(())
    }
}

fn require_columns(
    connection: &Arc<Connection>,
    table: &str,
    columns: &[&str],
) -> Result<Vec<(String, bool, bool)>, CatalogError> {
    let rows = query_rows(
        connection,
        &format!("PRAGMA table_info({})", sql_string(table)),
    )?;
    if rows.is_empty() {
        return Err(CatalogError::SourceTableMissing(table.to_owned()));
    }
    let available = rows
        .iter()
        .map(|row| {
            Ok((
                text(row, 1, "column name")?.to_owned(),
                integer(row, 3, "not null")? != 0,
                integer(row, 5, "primary key")? > 0,
            ))
        })
        .collect::<Result<Vec<_>, CatalogError>>()?;
    for column in columns {
        if !available
            .iter()
            .any(|(name, _, _)| name.eq_ignore_ascii_case(column))
        {
            return Err(CatalogError::SourceColumnMissing {
                table: table.to_owned(),
                column: (*column).to_owned(),
            });
        }
    }
    Ok(available)
}

/// `columns` is the table's already-fetched `require_columns` result, which
/// must include `column`; passing it in avoids a second `PRAGMA table_info`
/// round trip per source.
fn require_unique_identity(
    connection: &Arc<Connection>,
    table: &str,
    column: &str,
    columns: &[(String, bool, bool)],
) -> Result<(), CatalogError> {
    let primary_columns = columns
        .iter()
        .filter(|(_, _, primary)| *primary)
        .collect::<Vec<_>>();
    if primary_columns.len() == 1 && primary_columns[0].0.eq_ignore_ascii_case(column) {
        return Ok(());
    }
    let identity_not_null = columns
        .iter()
        .find(|(name, _, _)| name.eq_ignore_ascii_case(column))
        .is_some_and(|(_, not_null, _)| *not_null);
    for row in query_rows(
        connection,
        &format!("PRAGMA index_list({})", sql_string(table)),
    )? {
        let unique = integer(&row, 2, "index unique")? != 0;
        let partial = row.get(4).map_or(Ok(false), |value| {
            value_integer(value, "index partial").map(|value| value != 0)
        })?;
        if !unique || partial {
            continue;
        }
        let index_name = text(&row, 1, "index name")?;
        let indexed = query_rows(
            connection,
            &format!("PRAGMA index_info({})", sql_string(index_name)),
        )?;
        if identity_not_null
            && indexed.len() == 1
            && text(&indexed[0], 2, "indexed column")?.eq_ignore_ascii_case(column)
        {
            return Ok(());
        }
    }
    Err(CatalogError::IdentityNotUnique {
        table: table.to_owned(),
        column: column.to_owned(),
    })
}

/// Fails closed when a STRICT source table has a CUSTOM/DOMAIN/STRUCT/UNION
/// column but this connection lacks --experimental-custom-types. Without
/// this, SchemaCatalog would silently type such a column as Any/Bytes with
/// no signal that richer typing exists but is disabled for this connection.
///
/// Deliberately does NOT call `Schema::classify_column`: on a connection
/// with custom types disabled, `core`'s type_registry is entirely empty
/// (see `core/schema.rs:927-929`, `core/lib.rs:1608-1633`), so
/// `classify_column` can only ever report `Builtin` here — it cannot
/// observe a column that was made custom-typed earlier on a different,
/// enabled connection. Instead this compares the column's raw declared
/// type name against the exact builtin keyword set CREATE TABLE's own
/// STRICT column-type validator uses (`core/translate/schema.rs:788-791`).
/// That same validator (`core/translate/schema.rs:818-829`) guarantees the
/// soundness of this signal: a STRICT column can only have a non-builtin
/// type name if a type definition was registered for it at CREATE TABLE
/// time, so "STRICT column, non-builtin type name" is proof of a
/// CUSTOM/DOMAIN/STRUCT/UNION column even when the registry isn't loaded
/// right now. Non-STRICT tables never enforce this, so they're skipped.
fn require_custom_types_enabled_for_source(
    connection: &Arc<Connection>,
    table_name: &str,
) -> Result<(), CatalogError> {
    if connection.experimental_custom_types_enabled() {
        return Ok(());
    }
    let schema = connection.current_schema();
    let Some(table) = schema.get_table(table_name) else {
        return Err(CatalogError::SourceTableMissing(table_name.to_owned()));
    };
    if !table.is_strict() {
        return Ok(());
    }
    for column in table.columns() {
        let Some(name) = column.name.as_ref() else {
            continue;
        };
        let is_builtin = matches!(
            column.ty_str.to_ascii_uppercase().as_str(),
            "INT" | "INTEGER" | "REAL" | "TEXT" | "BLOB" | "ANY"
        );
        if !is_builtin {
            return Err(CatalogError::CustomTypesDisabled {
                table: table_name.to_owned(),
                column: name.clone(),
            });
        }
    }
    Ok(())
}

fn install_endpoint_index(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    role: &str,
    column: &str,
) -> Result<(), CatalogError> {
    let name = format!(
        "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_{}_{:016x}",
        graph.get(),
        role,
        stable_hash(&format!("{}:{}", source.table, column))
    );
    execute_internal(
        connection,
        format!(
            "CREATE INDEX IF NOT EXISTS {} ON {}({})",
            quote_identifier(&name),
            quote_identifier(&source.table),
            quote_identifier(column)
        ),
    )?;
    Ok(())
}

/// Composite (start, end) index for expands whose both endpoints are
/// already bound (co-membership patterns).
fn install_endpoint_pair_index(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
) -> Result<(), CatalogError> {
    let name = format!(
        "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_pair_{:016x}",
        graph.get(),
        stable_hash(&format!(
            "{}:{}:{}",
            source.table, source.start_column, source.end_column
        ))
    );
    execute_internal(
        connection,
        format!(
            "CREATE INDEX IF NOT EXISTS {} ON {}({}, {})",
            quote_identifier(&name),
            quote_identifier(&source.table),
            quote_identifier(&source.start_column),
            quote_identifier(&source.end_column)
        ),
    )?;
    Ok(())
}

fn install_generation_triggers(
    connection: &Arc<Connection>,
    graph: GraphId,
    table: &str,
) -> Result<(), CatalogError> {
    for event in ["INSERT", "UPDATE", "DELETE"] {
        let name = format!(
            "{TURSO_GRAPH_GENERATION_TRIGGER_PREFIX}{}_{}_{:016x}",
            graph.get(),
            event.to_ascii_lowercase(),
            stable_hash(table)
        );
        execute_internal(connection, format!(
            "CREATE TRIGGER {} AFTER {event} ON {} BEGIN UPDATE {GENERATIONS_TABLE} SET generation = generation + 1 WHERE graph_id = {}; END",
            quote_identifier(&name), quote_identifier(table), graph.get()
        ))?;
    }
    Ok(())
}

pub(crate) fn query_rows(
    connection: &Arc<Connection>,
    sql: &str,
) -> Result<Vec<Vec<Value>>, CatalogError> {
    Ok(connection.prepare(sql)?.run_collect_rows()?)
}

pub(crate) fn execute_internal(
    connection: &Arc<Connection>,
    sql: impl AsRef<str>,
) -> Result<(), CatalogError> {
    connection.prepare_internal(sql)?.run_ignore_rows()?;
    Ok(())
}

pub(crate) fn scalar_integer(
    connection: &Arc<Connection>,
    sql: &str,
    kind: &'static str,
) -> Result<i64, CatalogError> {
    let rows = query_rows(connection, sql)?;
    let row = rows
        .first()
        .ok_or(CatalogError::InvalidCatalogValue(kind))?;
    integer(row, 0, kind)
}

pub(crate) fn integer(
    row: &[Value],
    index: usize,
    kind: &'static str,
) -> Result<i64, CatalogError> {
    row.get(index)
        .ok_or(CatalogError::InvalidCatalogValue(kind))
        .and_then(|value| value_integer(value, kind))
}

fn value_integer(value: &Value, kind: &'static str) -> Result<i64, CatalogError> {
    match value {
        Value::Numeric(Numeric::Integer(value)) => Ok(*value),
        _ => Err(CatalogError::InvalidCatalogValue(kind)),
    }
}

pub(crate) fn text<'a>(
    row: &'a [Value],
    index: usize,
    kind: &'static str,
) -> Result<&'a str, CatalogError> {
    match row.get(index) {
        Some(Value::Text(value)) => Ok(value.as_str()),
        _ => Err(CatalogError::InvalidCatalogValue(kind)),
    }
}

fn graph_id(value: i64) -> Result<GraphId, CatalogError> {
    u64::try_from(value)
        .ok()
        .and_then(|value| GraphId::new(value).ok())
        .ok_or(CatalogError::InvalidIdentity {
            kind: "graph",
            value,
        })
}

fn source_id(value: i64) -> Result<SourceTableId, CatalogError> {
    u64::try_from(value)
        .ok()
        .and_then(|value| SourceTableId::new(value).ok())
        .ok_or(CatalogError::InvalidIdentity {
            kind: "source",
            value,
        })
}

fn nonnegative_u64(value: i64, kind: &'static str) -> Result<u64, CatalogError> {
    u64::try_from(value).map_err(|_| CatalogError::InvalidCatalogValue(kind))
}

pub(crate) fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn stable_hash(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect};

    fn connection() -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file(io, ":memory:graph-catalog", Arc::new(SqliteDialect))
            .expect("open database")
            .connect()
            .expect("connect")
    }

    fn create_sources(connection: &Arc<Connection>) {
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
                 CREATE TABLE friendships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
            )
            .expect("create graph sources");
    }

    fn registration(name: &str) -> GraphRegistration {
        GraphRegistration {
            name: name.to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration {
                name: "KNOWS".to_owned(),
                table: "friendships".to_owned(),
                identity_column: "id".to_owned(),
                start_column: "src".to_owned(),
                end_column: "dst".to_owned(),
                start_node_source: "Person".to_owned(),
                end_node_source: "Person".to_owned(),
            }],
        }
    }

    fn connection_with_custom_types() -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file_with_flags(
            io,
            ":memory:graph-catalog-custom-types",
            OpenFlags::default(),
            DatabaseOpts::new().with_custom_types(true),
            None,
            Arc::new(SqliteDialect),
        )
        .expect("open database")
        .connect()
        .expect("connect")
    }

    #[test]
    fn register_graph_inside_a_transaction_scopes_to_the_outer_transaction() {
        // A bare BEGIN IMMEDIATE would fail inside an open user transaction;
        // registration must fall back to a savepoint and commit or roll back
        // with the caller's transaction.
        let connection = connection();
        create_sources(&connection);
        connection.execute("BEGIN IMMEDIATE").expect("begin");
        register_graph(&connection, &registration("social")).expect("register inside txn");
        load_registered_graph(&connection, "social").expect("visible inside txn");
        connection.execute("ROLLBACK").expect("rollback");
        assert!(matches!(
            load_registered_graph(&connection, "social"),
            Err(CatalogError::GraphNotFound(_))
        ));
        register_graph(&connection, &registration("social")).expect("register after rollback");
    }

    #[test]
    fn register_graph_rejects_a_read_only_open_transaction() {
        // Registration runs internal (nested) statements, which cannot
        // upgrade a deferred read transaction to a write transaction; the
        // engine would otherwise panic on the first catalog DDL.
        let connection = connection();
        create_sources(&connection);
        connection.execute("BEGIN").expect("begin deferred");
        assert!(matches!(
            register_graph(&connection, &registration("social")),
            Err(CatalogError::RequiresWriteTransaction)
        ));
        connection.execute("ROLLBACK").expect("rollback");
        register_graph(&connection, &registration("social")).expect("register in autocommit");
    }

    #[test]
    fn register_graph_rejects_multiple_sources_per_kind() {
        // Layout, binder, and detach-delete resolution only honor the first
        // registered source of each kind; accepting more would silently
        // misroute reads and writes, so registration must fail closed.
        let connection = connection();
        create_sources(&connection);

        let mut multi_node = registration("multi_node");
        multi_node.node_sources.push(NodeSourceRegistration {
            name: "Company".to_owned(),
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        });
        assert!(matches!(
            register_graph(&connection, &multi_node),
            Err(CatalogError::MultipleSourcesUnsupported { kind: "node" })
        ));

        let mut multi_relationship = registration("multi_relationship");
        let mut second = multi_relationship.relationship_sources[0].clone();
        second.name = "LIKES".to_owned();
        multi_relationship.relationship_sources.push(second);
        assert!(matches!(
            register_graph(&connection, &multi_relationship),
            Err(CatalogError::MultipleSourcesUnsupported {
                kind: "relationship"
            })
        ));
    }

    #[test]
    fn register_graph_allows_struct_column_with_custom_types_enabled() {
        let connection = connection_with_custom_types();
        connection
            .execute(
                "CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER); \
                 CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, origin point) STRICT; \
                 CREATE TABLE friendships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER) STRICT;",
            )
            .expect("create struct-typed source");

        let result = register_graph(&connection, &registration("people_graph"));
        assert!(result.is_ok(), "expected success: {result:?}");
    }

    #[test]
    fn register_graph_rejects_struct_column_when_custom_types_disabled() {
        // experimental_custom_types_enabled is fixed at Database-open time
        // (DatabaseOpts), not per-connection and not toggled by CREATE TYPE.
        // Two in-memory open_file calls never share state, so the only way
        // to reach "a STRICT table with a struct column exists, but this
        // connection has custom types disabled" is a real file: create it
        // with custom types enabled, fully close it, then reopen the same
        // file without the flag.
        let temp_dir = tempfile::TempDir::new().expect("tempdir");
        let db_path = temp_dir.path().join("graph-catalog-custom-types.db");
        let db_path_str = db_path.to_str().expect("utf8 path");

        {
            let io: Arc<dyn turso_core::IO> =
                Arc::new(turso_core::PlatformIO::new().expect("platform io"));
            let db = Database::open_file_with_flags(
                io,
                db_path_str,
                OpenFlags::default(),
                DatabaseOpts::new().with_custom_types(true),
                None,
                Arc::new(SqliteDialect),
            )
            .expect("open database with custom types enabled");
            let connection = db.connect().expect("connect");
            connection
                .execute(
                    "CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER); \
                     CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, origin point) STRICT; \
                     CREATE TABLE friendships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER) STRICT;",
                )
                .expect("create struct-typed source");
            // Drop the connection and database so the file is fully closed
            // and the registry's cached Arc<Database> for this path/inode
            // has no live strong references before the next open.
            drop(connection);
            drop(db);
        }

        let io: Arc<dyn turso_core::IO> =
            Arc::new(turso_core::PlatformIO::new().expect("platform io"));
        let connection = Database::open_file_with_flags(
            io,
            db_path_str,
            OpenFlags::default(),
            DatabaseOpts::new(), // custom types NOT enabled on reopen
            None,
            Arc::new(SqliteDialect),
        )
        .expect("reopen database with custom types disabled")
        .connect()
        .expect("connect");

        let result = register_graph(&connection, &registration("people_graph"));
        assert!(
            matches!(
                &result,
                Err(CatalogError::CustomTypesDisabled { table, column })
                    if table == "people" && column == "origin"
            ),
            "expected CustomTypesDisabled error: {result:?}"
        );
    }

    #[test]
    fn registration_installs_stable_sources_indexes_and_generation_triggers() {
        let connection = connection();
        create_sources(&connection);
        let graph = register_graph(&connection, &registration("social")).expect("register graph");

        assert_eq!(graph.generation, 0);
        assert_eq!(graph.node_sources.len(), 1);
        assert_eq!(graph.relationship_sources.len(), 1);
        assert_ne!(graph.node_sources[0].id, graph.relationship_sources[0].id);

        connection
            .execute("INSERT INTO people VALUES (1, 'Ada')")
            .expect("insert node");
        assert_eq!(
            graph_generation(&connection, "social").expect("generation"),
            1
        );
        connection
            .execute("UPDATE people SET name = 'Grace' WHERE id = 1")
            .expect("update node");
        assert_eq!(
            graph_generation(&connection, "social").expect("generation"),
            2
        );
        connection
            .execute("INSERT INTO friendships VALUES (1, 1, 1)")
            .expect("insert edge");
        assert_eq!(
            graph_generation(&connection, "social").expect("generation"),
            3
        );
        connection
            .execute("DELETE FROM friendships WHERE id = 1")
            .expect("delete edge");
        assert_eq!(
            graph_generation(&connection, "social").expect("generation"),
            4
        );

        let indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // start, end, and the composite (start, end) pair index.
        assert_eq!(indexes.len(), 3);
    }

    #[test]
    fn source_write_rollback_restores_the_generation() {
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");

        connection.execute("BEGIN").expect("begin");
        connection
            .execute("INSERT INTO people VALUES (1, 'Ada')")
            .expect("insert node");
        assert_eq!(
            graph_generation(&connection, "social").expect("inside generation"),
            1
        );
        connection.execute("ROLLBACK").expect("rollback");
        assert_eq!(
            graph_generation(&connection, "social").expect("rolled back generation"),
            0
        );
    }

    #[test]
    fn invalid_registration_is_atomic_and_returns_typed_errors() {
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("existing")).expect("baseline catalog");
        let mut invalid = registration("invalid");
        invalid.node_sources[0].identity_column = "missing".to_owned();

        let error = register_graph(&connection, &invalid).expect_err("invalid column");
        assert!(matches!(
            error,
            CatalogError::SourceColumnMissing { table, column }
                if table == "people" && column == "missing"
        ));
        assert!(matches!(
            load_registered_graph(&connection, "invalid"),
            Err(CatalogError::GraphNotFound(name)) if name == "invalid"
        ));
    }

    #[test]
    fn identity_requires_a_primary_key_or_single_column_unique_index() {
        let connection = connection();
        connection
            .execute(
                "CREATE TABLE nodes(id INTEGER NOT NULL, tenant INTEGER NOT NULL, PRIMARY KEY(id, tenant)); \
                 CREATE TABLE edges(src INTEGER, dst INTEGER);",
            )
            .expect("create sources");
        let graph = GraphRegistration {
            name: "invalid_identity".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Node".to_owned(),
                table: "nodes".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        };
        assert!(matches!(
            register_graph(&connection, &graph),
            Err(CatalogError::IdentityNotUnique { table, column })
                if table == "nodes" && column == "id"
        ));

        connection
            .execute("CREATE UNIQUE INDEX nodes_id ON nodes(id)")
            .expect("unique identity");
        assert!(register_graph(&connection, &graph).is_ok());

        connection
            .execute("CREATE TABLE nullable_nodes(id INTEGER UNIQUE)")
            .expect("nullable source");
        let nullable = GraphRegistration {
            name: "nullable_identity".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Node".to_owned(),
                table: "nullable_nodes".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        };
        assert!(matches!(
            register_graph(&connection, &nullable),
            Err(CatalogError::IdentityNotUnique { table, column })
                if table == "nullable_nodes" && column == "id"
        ));
    }

    #[test]
    fn relationship_endpoints_must_name_registered_node_sources() {
        let connection = connection();
        create_sources(&connection);
        let mut graph = registration("bad_endpoint");
        graph.relationship_sources[0].end_node_source = "Missing".to_owned();
        assert!(matches!(
            register_graph(&connection, &graph),
            Err(CatalogError::UnknownEndpoint { relationship, node_source })
                if relationship == "KNOWS" && node_source == "Missing"
        ));
    }

    #[test]
    fn loading_a_graph_detects_removed_source_tables() {
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");
        connection
            .execute("DROP TABLE people")
            .expect("drop source");

        assert!(matches!(
            load_registered_graph(&connection, "social"),
            Err(CatalogError::SourceTableMissing(table)) if table == "people"
        ));
    }

    #[test]
    fn reserved_and_duplicate_names_are_rejected_before_catalog_writes() {
        let connection = connection();
        create_sources(&connection);
        let mut reserved = registration("__turso_graph_catalog");
        assert!(matches!(
            register_graph(&connection, &reserved),
            Err(CatalogError::ReservedGraphName(_))
        ));

        reserved.name = "duplicate_sources".to_owned();
        reserved.node_sources.push(reserved.node_sources[0].clone());
        assert!(matches!(
            register_graph(&connection, &reserved),
            Err(CatalogError::DuplicateName {
                kind: "node source",
                ..
            })
        ));
    }

    #[test]
    fn users_cannot_mutate_catalog_or_forge_internal_generation_triggers() {
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");

        let update = connection.execute(format!("UPDATE {GENERATIONS_TABLE} SET generation = 99"));
        assert!(update.is_err(), "catalog generation must reject direct DML");

        let forged = connection.execute(
            "CREATE TRIGGER __turso_internal_graph_gen_forged AFTER INSERT ON people \
             BEGIN SELECT 1; END",
        );
        assert!(
            forged.is_err(),
            "internal trigger prefix must reject user DDL"
        );

        let trigger_rows = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'trigger' AND name LIKE '__turso_internal_graph_gen_%' LIMIT 1",
        )
        .expect("query trigger");
        let trigger_name = text(&trigger_rows[0], 0, "trigger name").expect("trigger name");
        assert!(connection
            .execute(format!("DROP TRIGGER {}", quote_identifier(trigger_name)))
            .is_err());

        let index_rows = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' AND name LIKE '__turso_internal_graph_ep_%' LIMIT 1",
        )
        .expect("query index");
        let index_name = text(&index_rows[0], 0, "index name").expect("index name");
        assert!(connection
            .execute(format!("DROP INDEX {}", quote_identifier(index_name)))
            .is_err());
        assert_eq!(
            graph_generation(&connection, "social").expect("generation"),
            0
        );
    }
}
