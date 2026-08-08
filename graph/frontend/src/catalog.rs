use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

use thiserror::Error;
use turso_core::{
    schema::{TURSO_GRAPH_CATALOG_PREFIX, TURSO_GRAPH_GENERATIONS_TABLE_NAME},
    Connection, Numeric, Value,
};
use turso_graph_ir::{self as ir, GraphId, RoleCardinality, SourceTableId};

use crate::transaction::{in_write_transaction, WriteTransactionError};

const RESERVED_PREFIX: &str = "__turso_";
pub(crate) const GRAPHS_TABLE: &str = "__turso_internal_graph_graphs";
pub(crate) const GENERATIONS_TABLE: &str = TURSO_GRAPH_GENERATIONS_TABLE_NAME;
/// Catalog-only generation, kept beside the data generation in
/// [`GENERATIONS_TABLE`]. See [`RegisteredGraph::schema_generation`].
pub(crate) const SCHEMA_GENERATION_COLUMN: &str = "schema_generation";
/// Name prefix of the AFTER-DML triggers older builds installed on every mapped
/// source table to bump [`GENERATIONS_TABLE`]. Nothing installs them any more;
/// the prefix survives only so [`drop_stale_generation_triggers`] can recognize
/// what one of those builds left behind.
const GENERATION_TRIGGER_PREFIX: &str = "__turso_internal_graph_gen_";
pub(crate) const SOURCES_TABLE: &str = "__turso_internal_graph_sources";
pub(crate) const NODE_SOURCES_TABLE: &str = "__turso_internal_graph_node_sources";
pub(crate) const RELATIONSHIP_SOURCES_TABLE: &str = "__turso_internal_graph_relationship_sources";
pub(crate) const RELATIONSHIP_ROLES_TABLE: &str = "__turso_internal_graph_relationship_roles";

pub const GRAPH_CATALOG_VERSION: u64 = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeSourceRegistration {
    pub name: String,
    pub table: String,
    pub identity_column: String,
}

/// One named role of a relationship source and the physical column that
/// stores its player.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleSourceRegistration {
    pub name: String,
    /// Endpoint column on the relationship table. Ignored for `Many` roles,
    /// which store players in `<table>__<role>` instead; pass an empty string.
    pub column: String,
    /// Name of the registered node source that plays this role.
    pub node_source: String,
    pub cardinality: RoleCardinality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolymorphicRoleRegistration {
    pub relationship: String,
    pub role: String,
    /// Column containing the registered node source id for each endpoint.
    pub discriminator_column: String,
    pub node_sources: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipSourceRegistration {
    pub name: String,
    pub table: String,
    pub identity_column: String,
    /// Declaration order is stable and becomes role ordinal order.
    pub roles: Vec<RoleSourceRegistration>,
}

impl RelationshipSourceRegistration {
    /// A two-endpoint table registered as a two-role relation named
    /// `start`/`end`. This is a layout of the role model, not a separate kind:
    /// every donor corpus source registers this way and keeps working.
    #[allow(clippy::too_many_arguments)]
    pub fn binary(
        name: impl Into<String>,
        table: impl Into<String>,
        identity_column: impl Into<String>,
        start_column: impl Into<String>,
        end_column: impl Into<String>,
        start_node_source: impl Into<String>,
        end_node_source: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            table: table.into(),
            identity_column: identity_column.into(),
            roles: vec![
                RoleSourceRegistration {
                    name: "start".to_owned(),
                    column: start_column.into(),
                    node_source: start_node_source.into(),
                    cardinality: RoleCardinality::One,
                },
                RoleSourceRegistration {
                    name: "end".to_owned(),
                    column: end_column.into(),
                    node_source: end_node_source.into(),
                    cardinality: RoleCardinality::One,
                },
            ],
        }
    }
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
pub struct RegisteredRelationshipRole {
    pub role: ir::RoleId,
    pub name: String,
    pub column: String,
    pub node_sources: Vec<SourceTableId>,
    pub discriminator_column: Option<String>,
    pub cardinality: RoleCardinality,
}

impl RegisteredRelationshipRole {
    pub fn fixed_node_source(&self) -> Option<SourceTableId> {
        (self.discriminator_column.is_none() && self.node_sources.len() == 1)
            .then(|| self.node_sources[0])
    }

    pub fn accepts(&self, source: SourceTableId) -> bool {
        self.node_sources.contains(&source)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredRelationshipSource {
    pub id: SourceTableId,
    pub name: String,
    pub table: String,
    pub identity_column: String,
    pub roles: Vec<RegisteredRelationshipRole>,
}

impl RegisteredRelationshipSource {
    pub fn role_by_name(&self, name: &str) -> Option<&RegisteredRelationshipRole> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }

    pub fn role_by_id(&self, role: ir::RoleId) -> Option<&RegisteredRelationshipRole> {
        self.roles.iter().find(|entry| entry.role == role)
    }

    /// Roles stored in an endpoint column on the relation table.
    pub fn single_valued_roles(&self) -> impl Iterator<Item = &RegisteredRelationshipRole> {
        self.roles
            .iter()
            .filter(|role| role.cardinality == RoleCardinality::One)
    }

    /// Spill table holding the players of a `Many` role.
    pub fn spill_table(&self, role: &RegisteredRelationshipRole) -> String {
        format!("{}__{}", self.table, role.name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredGraph {
    pub id: GraphId,
    pub name: String,
    /// Advances on every catalog change. It used to advance on every write to a
    /// mapped source table as well, via AFTER-DML triggers that are now gone --
    /// [`RegisteredGraph::derived_generation`] carries that signal instead, for
    /// free. Kept because sessions on a database predating
    /// [`RegisteredGraph::schema_generation`] still watch it to decide when to
    /// reload their catalog.
    pub generation: u64,
    /// Advances only when the catalog itself changes: graph registration,
    /// semantic schema, semantic constraints, fragments. Sessions reload
    /// their `SchemaCatalog` when it moves, so ordinary row writes no longer
    /// force a catalog reload.
    ///
    /// `None` when the database predates this column; callers treat that as
    /// "assume stale" and fall back to reloading on every statement.
    pub schema_generation: Option<u64>,
    /// Moves whenever a mapped source table changes, derived from the engine's
    /// per-table change tokens. This is what traversal snapshots compare.
    /// `None` when any input is unavailable -- a table the engine cannot
    /// tokenize, multiprocess WAL, or a database predating `schema_generation`
    /// -- which callers treat as "assume stale".
    ///
    /// Covers exactly the tables the retired AFTER-DML triggers fired on, and
    /// catches cases they missed entirely, such as DDL on a mapped table. It is
    /// also per-table where their counter was per-graph, so a write to one
    /// graph's tables no longer invalidates another's snapshots.
    pub derived_generation: Option<u64>,
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
    #[error("polymorphic role registration references unknown role `{role}` on relationship source `{relationship}`")]
    UnknownPolymorphicRole { relationship: String, role: String },
    #[error("catalog row contains an invalid {kind} identity: {value}")]
    InvalidIdentity { kind: &'static str, value: i64 },
    #[error("catalog row has an invalid value in `{0}`")]
    InvalidCatalogValue(&'static str),
    #[error("graph catalog predates native relationship roles ({detail}); this build reads only role-shaped catalogs and there is no migration, so the graph must be created fresh")]
    IncompatibleGraphLayout { detail: String },
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

impl WriteTransactionError for CatalogError {
    fn requires_write_transaction() -> Self {
        CatalogError::RequiresWriteTransaction
    }

    fn rollback_failed(cause: Self, rollback: turso_core::LimboError) -> Self {
        CatalogError::RollbackFailed {
            cause: Box::new(cause),
            rollback,
        }
    }
}

const REGISTRATION_SAVEPOINT: &str = "turso_graph_register";
const STALE_TRIGGER_SAVEPOINT: &str = "turso_graph_drop_stale_triggers";

pub fn register_graph(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
) -> Result<RegisteredGraph, CatalogError> {
    register_graph_with_polymorphic_roles(connection, registration, &[])
}

pub fn register_graph_with_polymorphic_roles(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<RegisteredGraph, CatalogError> {
    validate_registration_names(registration, polymorphic_roles)?;
    in_write_transaction(connection, REGISTRATION_SAVEPOINT, || {
        register_graph_in_transaction(connection, registration, polymorphic_roles)
    })
}

pub fn load_registered_graph(
    connection: &Arc<Connection>,
    name: &str,
) -> Result<RegisteredGraph, CatalogError> {
    ensure_catalog_exists(connection)?;
    if query_rows(
        connection,
        &format!(
            "SELECT name FROM sqlite_schema WHERE type = 'table' AND name = {}",
            sql_string(RELATIONSHIP_ROLES_TABLE)
        ),
    )?
    .is_empty()
    {
        return Err(CatalogError::IncompatibleGraphLayout {
            detail: format!("{RELATIONSHIP_ROLES_TABLE} is absent"),
        });
    }
    let role_catalog_columns = query_rows(
        connection,
        &format!(
            "PRAGMA table_info({})",
            sql_string(RELATIONSHIP_ROLES_TABLE)
        ),
    )?
    .into_iter()
    .map(|row| text(&row, 1, "relationship role catalog column").map(str::to_owned))
    .collect::<Result<Vec<_>, _>>()?;
    for required in ["node_source_ids", "node_source_column"] {
        if !role_catalog_columns.iter().any(|column| column == required) {
            return Err(CatalogError::IncompatibleGraphLayout {
                detail: format!("{RELATIONSHIP_ROLES_TABLE}.{required} is absent"),
            });
        }
    }
    // `schema_generation` was added after the first databases were written.
    // Selecting it from a database that predates it fails to compile, so probe
    // the column and fall back to the pre-column shape.
    let has_schema_generation = generations_table_has_schema_generation(connection)?;
    let schema_generation_column = if has_schema_generation {
        ", gen.schema_generation"
    } else {
        ""
    };
    let graph_rows = query_rows(
        connection,
        &format!(
            "SELECT g.id, g.name, gen.generation{schema_generation_column} FROM {GRAPHS_TABLE} g \
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
    let schema_generation = if has_schema_generation {
        Some(nonnegative_u64(
            integer(row, 3, "schema generation")?,
            "schema generation",
        )?)
    } else {
        None
    };

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
            "SELECT s.id, s.name, r.table_name, r.identity_column FROM {SOURCES_TABLE} s \
             JOIN {RELATIONSHIP_SOURCES_TABLE} r ON r.source_id = s.id \
             WHERE s.graph_id = {} ORDER BY s.id",
            graph_id.get()
        ),
    )?;
    let mut relationship_sources = Vec::with_capacity(relationship_rows.len());
    for row in relationship_rows {
        let id = source_id(integer(&row, 0, "relationship source id")?)?;
        let table = text(&row, 2, "relationship source table")?.to_owned();
        let identity_column = text(&row, 3, "relationship identity column")?.to_owned();
        let role_rows = query_rows(
            connection,
            &format!(
                "SELECT ordinal, name, column_name, node_source_ids, node_source_column, cardinality \
                 FROM {RELATIONSHIP_ROLES_TABLE} WHERE source_id = {} ORDER BY ordinal",
                id.get()
            ),
        )?;
        let mut roles = Vec::with_capacity(role_rows.len());
        let mut required_columns = vec![identity_column.clone()];
        for role_row in role_rows {
            let ordinal = integer(&role_row, 0, "role ordinal")?;
            let role = u32::try_from(ordinal)
                .ok()
                .and_then(|value| ir::RoleId::new(value).ok())
                .ok_or(CatalogError::InvalidIdentity {
                    kind: "role",
                    value: ordinal,
                })?;
            let cardinality = match text(&role_row, 5, "role cardinality")? {
                "one" => RoleCardinality::One,
                "many" => RoleCardinality::Many,
                _ => return Err(CatalogError::InvalidCatalogValue("role cardinality")),
            };
            let column = text(&role_row, 2, "role column")?.to_owned();
            if cardinality == RoleCardinality::One {
                required_columns.push(column.clone());
            }
            let discriminator_column = text(&role_row, 4, "role node source column")?;
            let discriminator_column =
                (!discriminator_column.is_empty()).then(|| discriminator_column.to_owned());
            if let Some(column) = &discriminator_column {
                required_columns.push(column.clone());
            }
            let node_sources = text(&role_row, 3, "role node source ids")?
                .split(',')
                .map(|value| {
                    value
                        .parse::<i64>()
                        .map_err(|_| CatalogError::InvalidCatalogValue("role node source ids"))
                        .and_then(source_id)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if node_sources.is_empty() {
                return Err(CatalogError::InvalidCatalogValue("role node source ids"));
            }
            roles.push(RegisteredRelationshipRole {
                role,
                name: text(&role_row, 1, "role name")?.to_owned(),
                column,
                node_sources,
                discriminator_column,
                cardinality,
            });
        }
        let borrowed = required_columns
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        require_columns(connection, &table, &borrowed)?;
        relationship_sources.push(RegisteredRelationshipSource {
            id,
            name: text(&row, 1, "relationship source name")?.to_owned(),
            table,
            identity_column,
            roles,
        });
    }
    // Before the tokens are read, not after: a leftover trigger would fail the
    // next write to a mapped table, and every graph operation loads the
    // registration first, so this is the last point that is still ahead of one.
    drop_stale_generation_triggers(connection, &node_sources, &relationship_sources)?;
    let derived_generation = derive_generation(
        connection,
        schema_generation,
        &node_sources,
        &relationship_sources,
    );
    Ok(RegisteredGraph {
        id: graph_id,
        name: graph_name,
        generation,
        schema_generation,
        derived_generation,
        node_sources,
        relationship_sources,
    })
}

/// Folds the change tokens of the graph's mapped source tables together with
/// the catalog's own generation.
///
/// The table set is the one the retired generation triggers fired on, so this
/// cannot miss a write they would have caught. Order is fixed by
/// the `ORDER BY s.id` the caller loads with, and the same table mapped twice
/// contributes twice -- neither affects whether the value *moves*, which is
/// all callers compare.
fn derive_generation(
    connection: &Arc<Connection>,
    schema_generation: Option<u64>,
    node_sources: &[RegisteredNodeSource],
    relationship_sources: &[RegisteredRelationshipSource],
) -> Option<u64> {
    let mut hasher = DefaultHasher::new();
    // A database predating the catalog split cannot say whether the catalog
    // changed, so the whole derived value has to be unavailable too.
    schema_generation?.hash(&mut hasher);
    let tables = node_sources
        .iter()
        .map(|source| source.table.as_str())
        .chain(
            relationship_sources
                .iter()
                .map(|source| source.table.as_str()),
        );
    for table in tables {
        connection.table_change_token(table)?.hash(&mut hasher);
    }
    Some(hasher.finish())
}

pub fn graph_generation(connection: &Arc<Connection>, name: &str) -> Result<u64, CatalogError> {
    Ok(load_registered_graph(connection, name)?.generation)
}

/// Reads just the catalog's schema generation for one graph.
///
/// This is the cheap staleness probe a session runs before every statement: a
/// single primary-key lookup instead of the dozen-plus statements
/// [`load_registered_graph`] compiles. Only call it once
/// [`load_registered_graph`] has reported a `schema_generation` — on a database
/// that predates the column this fails to compile.
pub(crate) fn load_schema_generation(
    connection: &Arc<Connection>,
    graph: GraphId,
) -> Result<Option<u64>, CatalogError> {
    let rows = query_rows(
        connection,
        &format!(
            "SELECT {SCHEMA_GENERATION_COLUMN} FROM {GENERATIONS_TABLE} WHERE graph_id = {}",
            graph.get()
        ),
    )?;
    let Some(row) = rows.first() else {
        return Ok(None);
    };
    nonnegative_u64(integer(row, 0, "schema generation")?, "schema generation").map(Some)
}

fn generations_table_has_schema_generation(
    connection: &Arc<Connection>,
) -> Result<bool, CatalogError> {
    let columns = query_rows(
        connection,
        &format!("PRAGMA table_info({})", sql_string(GENERATIONS_TABLE)),
    )?;
    for row in &columns {
        if text(row, 1, "generations catalog column")? == SCHEMA_GENERATION_COLUMN {
            return Ok(true);
        }
    }
    Ok(false)
}

fn register_graph_in_transaction(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
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
        let mut required_columns = vec![relationship.identity_column.as_str()];
        required_columns.extend(
            relationship
                .roles
                .iter()
                .filter(|role| role.cardinality == RoleCardinality::One)
                .map(|role| role.column.as_str()),
        );
        required_columns.extend(relationship.roles.iter().filter_map(|role| {
            polymorphic_role(polymorphic_roles, relationship, role)
                .map(|registration| registration.discriminator_column.as_str())
        }));
        let columns = require_columns(connection, &relationship.table, &required_columns)?;
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
            "INSERT INTO {GENERATIONS_TABLE}(graph_id, generation, {SCHEMA_GENERATION_COLUMN}) \
             VALUES ({}, 0, 0)",
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
            "INSERT INTO {RELATIONSHIP_SOURCES_TABLE}(source_id, table_name, identity_column) VALUES ({}, {}, {})",
            relationship_id,
            sql_string(&relationship.table),
            sql_string(&relationship.identity_column)
        ))?;
        for (ordinal, role) in relationship.roles.iter().enumerate() {
            let polymorphic = polymorphic_role(polymorphic_roles, relationship, role);
            let names = polymorphic
                .map(|registration| registration.node_sources.as_slice())
                .unwrap_or_else(|| std::slice::from_ref(&role.node_source));
            let node_sources = names
                .iter()
                .map(|name| {
                    node_ids
                        .get(name)
                        .copied()
                        .ok_or_else(|| CatalogError::UnknownEndpoint {
                            relationship: relationship.name.clone(),
                            node_source: name.clone(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let node_source_ids = node_sources
                .iter()
                .map(|source| source.get().to_string())
                .collect::<Vec<_>>()
                .join(",");
            execute_internal(connection, format!(
                "INSERT INTO {RELATIONSHIP_ROLES_TABLE}(source_id, ordinal, name, column_name, node_source_ids, node_source_column, cardinality) \
                 VALUES ({}, {}, {}, {}, {}, {}, {})",
                relationship_id,
                ordinal + 1,
                sql_string(&role.name),
                sql_string(&role.column),
                sql_string(&node_source_ids),
                sql_string(
                    polymorphic
                        .map(|registration| registration.discriminator_column.as_str())
                        .unwrap_or(""),
                ),
                sql_string(match role.cardinality {
                    RoleCardinality::One => "one",
                    RoleCardinality::Many => "many",
                })
            ))?;
            match role.cardinality {
                RoleCardinality::One => {
                    install_role_index(connection, graph_id, relationship, role, polymorphic)?;
                }
                RoleCardinality::Many => {
                    install_spill_table(connection, graph_id, relationship, role)?;
                }
            }
        }
        // Co-membership patterns bind two role players before matching the
        // second relationship; the composite index turns that probe from an
        // in-degree scan into an exact lookup. A two-role relation gets
        // exactly one such index, which is today's (start, end) index.
        install_role_pair_indexes(connection, graph_id, relationship, polymorphic_roles)?;
    }

    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS \"{}\"(source_id INTEGER NOT NULL, node_id INTEGER NOT NULL, label TEXT NOT NULL)",
            labels_table_name(graph_id)
        ),
    )?;
    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS \"{}\"(source_id INTEGER NOT NULL, relationship_id INTEGER NOT NULL, type TEXT NOT NULL)",
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
        (
            labels_table_name(graph_id),
            ["source_id, node_id, label", "source_id, label, node_id"],
        ),
        (
            relationship_types_table_name(graph_id),
            [
                "source_id, relationship_id, type",
                "source_id, type, relationship_id",
            ],
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
        "CREATE TABLE IF NOT EXISTS {GENERATIONS_TABLE}(graph_id INTEGER PRIMARY KEY, generation INTEGER NOT NULL CHECK(generation >= 0), {SCHEMA_GENERATION_COLUMN} INTEGER NOT NULL DEFAULT 0 CHECK({SCHEMA_GENERATION_COLUMN} >= 0))"
    ))?;
    // Databases registered before the split carry only `generation`. Adding
    // the column here (registration always holds a write transaction) lets
    // them take the fast staleness path from the next open onwards.
    if !generations_table_has_schema_generation(connection)? {
        execute_internal(
            connection,
            format!(
                "ALTER TABLE {GENERATIONS_TABLE} \
                 ADD COLUMN {SCHEMA_GENERATION_COLUMN} INTEGER NOT NULL DEFAULT 0"
            ),
        )?;
    }
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {SOURCES_TABLE}(id INTEGER PRIMARY KEY, graph_id INTEGER NOT NULL, name TEXT NOT NULL COLLATE NOCASE, kind TEXT NOT NULL CHECK(kind IN ('node', 'relationship')), UNIQUE(graph_id, name))"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {NODE_SOURCES_TABLE}(source_id INTEGER PRIMARY KEY, table_name TEXT NOT NULL, identity_column TEXT NOT NULL)"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {RELATIONSHIP_SOURCES_TABLE}(source_id INTEGER PRIMARY KEY, table_name TEXT NOT NULL, identity_column TEXT NOT NULL)"
    ))?;
    execute_internal(connection, format!(
        "CREATE TABLE IF NOT EXISTS {RELATIONSHIP_ROLES_TABLE}(source_id INTEGER NOT NULL, ordinal INTEGER NOT NULL, name TEXT NOT NULL COLLATE NOCASE, column_name TEXT NOT NULL, node_source_ids TEXT NOT NULL, node_source_column TEXT NOT NULL, cardinality TEXT NOT NULL CHECK(cardinality IN ('one', 'many')), PRIMARY KEY(source_id, ordinal))"
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

fn validate_registration_names(
    registration: &GraphRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<(), CatalogError> {
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
    let mut polymorphic_names = HashSet::new();
    for polymorphic in polymorphic_roles {
        let Some(relationship) = registration
            .relationship_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case(&polymorphic.relationship))
        else {
            return Err(CatalogError::UnknownPolymorphicRole {
                relationship: polymorphic.relationship.clone(),
                role: polymorphic.role.clone(),
            });
        };
        if !relationship
            .roles
            .iter()
            .any(|role| role.name.eq_ignore_ascii_case(&polymorphic.role))
        {
            return Err(CatalogError::UnknownPolymorphicRole {
                relationship: polymorphic.relationship.clone(),
                role: polymorphic.role.clone(),
            });
        }
        let key = format!(
            "{}.{}",
            polymorphic.relationship.to_ascii_lowercase(),
            polymorphic.role.to_ascii_lowercase()
        );
        if !polymorphic_names.insert(key.clone()) {
            return Err(CatalogError::DuplicateName {
                kind: "polymorphic role registration",
                name: key,
            });
        }
        validate_name("source column", &polymorphic.discriminator_column)?;
        let mut allowed = HashSet::new();
        for source in &polymorphic.node_sources {
            if !allowed.insert(source.to_ascii_lowercase()) {
                return Err(CatalogError::DuplicateName {
                    kind: "polymorphic role node source",
                    name: source.clone(),
                });
            }
        }
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
        let mut columns = vec![source.identity_column.as_str()];
        let mut role_names = HashSet::new();
        for role in &source.roles {
            validate_name("role", &role.name)?;
            if !role_names.insert(role.name.to_ascii_lowercase()) {
                return Err(CatalogError::DuplicateName {
                    kind: "role",
                    name: role.name.clone(),
                });
            }
            if role.cardinality == RoleCardinality::One {
                columns.push(role.column.as_str());
            }
            if let Some(registration) = polymorphic_role(polymorphic_roles, source, role) {
                columns.push(registration.discriminator_column.as_str());
                if role.cardinality == RoleCardinality::Many {
                    return Err(CatalogError::InvalidCatalogValue(
                        "polymorphic Many role is not supported",
                    ));
                }
            }
            if polymorphic_role(polymorphic_roles, source, role)
                .is_some_and(|registration| registration.node_sources.is_empty())
            {
                return Err(CatalogError::InvalidCatalogValue(
                    "role node sources must not be empty",
                ));
            }
        }
        validate_source_identifiers(&source.table, &columns)?;
        if !source_names.insert(source.name.to_ascii_lowercase()) {
            return Err(CatalogError::DuplicateName {
                kind: "relationship source",
                name: source.name.clone(),
            });
        }
    }
    Ok(())
}

fn polymorphic_role<'a>(
    registrations: &'a [PolymorphicRoleRegistration],
    relationship: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
) -> Option<&'a PolymorphicRoleRegistration> {
    registrations.iter().find(|registration| {
        registration
            .relationship
            .eq_ignore_ascii_case(&relationship.name)
            && registration.role.eq_ignore_ascii_case(&role.name)
    })
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

fn install_role_index(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
    polymorphic: Option<&PolymorphicRoleRegistration>,
) -> Result<(), CatalogError> {
    let name = format!(
        "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_{}_{:016x}",
        graph.get(),
        role.name.to_ascii_lowercase(),
        stable_hash(&format!("{}:{}", source.table, role.column))
    );
    let columns = polymorphic
        .map(|registration| {
            format!(
                "{}, {}",
                quote_identifier(&registration.discriminator_column),
                quote_identifier(&role.column)
            )
        })
        .unwrap_or_else(|| quote_identifier(&role.column));
    let required = polymorphic
        .map(|registration| {
            vec![
                registration.discriminator_column.as_str(),
                role.column.as_str(),
            ]
        })
        .unwrap_or_else(|| vec![role.column.as_str()]);
    if has_covering_btree_index(connection, &source.table, &required)? {
        return Ok(());
    }
    execute_internal(
        connection,
        format!(
            "CREATE INDEX IF NOT EXISTS {} ON {}({columns})",
            quote_identifier(&name),
            quote_identifier(&source.table)
        ),
    )?;
    Ok(())
}

/// One composite index per unordered pair of single-valued roles. A two-role
/// relation therefore gets exactly the (start, end) index it has today.
fn install_role_pair_indexes(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<(), CatalogError> {
    let single = source
        .roles
        .iter()
        .filter(|role| role.cardinality == RoleCardinality::One)
        .collect::<Vec<_>>();
    for (index, left) in single.iter().enumerate() {
        for right in single.iter().skip(index + 1) {
            let name = format!(
                "{TURSO_GRAPH_CATALOG_PREFIX}ep_{}_pair_{:016x}",
                graph.get(),
                stable_hash(&format!(
                    "{}:{}:{}",
                    source.table, left.column, right.column
                ))
            );
            let required = [
                polymorphic_role(polymorphic_roles, source, left)
                    .map(|registration| registration.discriminator_column.as_str()),
                Some(left.column.as_str()),
                polymorphic_role(polymorphic_roles, source, right)
                    .map(|registration| registration.discriminator_column.as_str()),
                Some(right.column.as_str()),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
            if has_covering_btree_index(connection, &source.table, &required)? {
                continue;
            }
            execute_internal(
                connection,
                format!(
                    "CREATE INDEX IF NOT EXISTS {} ON {}({})",
                    quote_identifier(&name),
                    quote_identifier(&source.table),
                    required
                        .into_iter()
                        .map(quote_identifier)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )?;
        }
    }
    Ok(())
}

fn has_covering_btree_index(
    connection: &Arc<Connection>,
    table: &str,
    required: &[&str],
) -> Result<bool, CatalogError> {
    for row in query_rows(
        connection,
        &format!("PRAGMA index_list({})", sql_string(table)),
    )? {
        let partial = row
            .get(4)
            .map(|value| value_integer(value, "index partial"))
            .transpose()?
            .unwrap_or(0)
            != 0;
        if partial {
            continue;
        }
        let name = text(&row, 1, "index name")?;
        let schema = query_rows(
            connection,
            &format!(
                "SELECT sql FROM sqlite_schema WHERE type = 'index' AND name = {}",
                sql_string(name)
            ),
        )?;
        let custom_method = schema
            .first()
            .and_then(|row| row.first())
            .is_some_and(|value| match value {
                Value::Text(sql) => sql
                    .as_str()
                    .split_ascii_whitespace()
                    .any(|token| token.eq_ignore_ascii_case("USING")),
                _ => false,
            });
        if custom_method {
            continue;
        }
        let columns = query_rows(
            connection,
            &format!("PRAGMA index_info({})", sql_string(name)),
        )?;
        if columns.len() < required.len() {
            continue;
        }
        if columns
            .iter()
            .take(required.len())
            .zip(required)
            .all(|(row, required)| {
                text(row, 2, "indexed column")
                    .is_ok_and(|column| column.eq_ignore_ascii_case(required))
            })
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// A `Many` role stores its players in `<table>__<role>(relation_id, node_id)`,
/// indexed in both directions so a hop is an index probe from either side.
fn install_spill_table(
    connection: &Arc<Connection>,
    graph: GraphId,
    source: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
) -> Result<(), CatalogError> {
    let table = format!("{}__{}", source.table, role.name);
    execute_internal(
        connection,
        format!(
            "CREATE TABLE IF NOT EXISTS {}(relation_id INTEGER NOT NULL, node_id INTEGER NOT NULL)",
            quote_identifier(&table)
        ),
    )?;
    for (suffix, columns) in [
        ("fwd", "relation_id, node_id"),
        ("rev", "node_id, relation_id"),
    ] {
        let name = format!(
            "{TURSO_GRAPH_CATALOG_PREFIX}spill_{}_{suffix}_{:016x}",
            graph.get(),
            stable_hash(&table)
        );
        execute_internal(
            connection,
            format!(
                "CREATE INDEX IF NOT EXISTS {} ON {}({columns})",
                quote_identifier(&name),
                quote_identifier(&table)
            ),
        )?;
    }
    Ok(())
}

/// Removes the generation triggers an older build installed on this graph's
/// mapped source tables.
///
/// Nothing installs them any more: the change tokens carry the same signal
/// without a row write per written row. Leaving them behind is not merely
/// wasteful but broken, because their bodies update a protected internal table
/// and core no longer makes an exception for them, so the next write to a
/// mapped table would fail.
///
/// The common case -- a database this build registered -- costs one hash lookup
/// per mapped table against the in-memory schema and issues no SQL at all.
fn drop_stale_generation_triggers(
    connection: &Arc<Connection>,
    node_sources: &[RegisteredNodeSource],
    relationship_sources: &[RegisteredRelationshipSource],
) -> Result<(), CatalogError> {
    let schema = connection.current_schema();
    let mut stale = Vec::new();
    for table in node_sources
        .iter()
        .map(|source| source.table.as_str())
        .chain(
            relationship_sources
                .iter()
                .map(|source| source.table.as_str()),
        )
    {
        let Some(triggers) = schema.triggers.get(&table.to_lowercase()) else {
            continue;
        };
        stale.extend(
            triggers
                .iter()
                .filter(|trigger| trigger.name.starts_with(GENERATION_TRIGGER_PREFIX))
                .map(|trigger| trigger.name.clone()),
        );
    }
    if stale.is_empty() {
        return Ok(());
    }
    // Dropping is DDL, so it needs a write transaction. A caller sitting in a
    // read transaction cannot open one -- but it cannot fire the trigger
    // either, so leaving the work to the next load in a writable context is
    // safe rather than merely convenient.
    if !connection.get_auto_commit() && !connection.in_write_transaction() {
        return Ok(());
    }
    in_write_transaction(connection, STALE_TRIGGER_SAVEPOINT, || {
        for name in &stale {
            execute_internal(
                connection,
                format!("DROP TRIGGER IF EXISTS {}", quote_identifier(name)),
            )?;
        }
        Ok(())
    })
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

pub(crate) fn source_id(value: i64) -> Result<SourceTableId, CatalogError> {
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

pub(crate) fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub(crate) fn stable_hash(value: &str) -> u64 {
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
    use turso_graph_ir::RoleCardinality;

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
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "friendships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
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
    fn register_graph_indexes_junctions_by_identity_and_semantic_type() {
        let connection = connection();
        create_sources(&connection);
        let graph = register_graph(&connection, &registration("social")).expect("register graph");

        for (table, expected) in [
            (
                labels_table_name(graph.id),
                [
                    vec!["source_id", "node_id", "label"],
                    vec!["source_id", "label", "node_id"],
                ],
            ),
            (
                relationship_types_table_name(graph.id),
                [
                    vec!["source_id", "relationship_id", "type"],
                    vec!["source_id", "type", "relationship_id"],
                ],
            ),
        ] {
            for (index, expected_columns) in expected.iter().enumerate() {
                let rows = query_rows(
                    &connection,
                    &format!(
                        "PRAGMA index_info({})",
                        sql_string(&format!("{table}_ix{index}"))
                    ),
                )
                .expect("inspect junction index");
                let columns = rows
                    .iter()
                    .map(|row| text(row, 2, "index column").expect("text column"))
                    .collect::<Vec<_>>();
                assert_eq!(columns, *expected_columns);
            }
        }
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
    fn register_graph_preserves_multiple_sources_per_kind() {
        let connection = connection();
        create_sources(&connection);

        let mut multi_node = registration("multi_node");
        multi_node.node_sources.push(NodeSourceRegistration {
            name: "Company".to_owned(),
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        });
        let registered = register_graph(&connection, &multi_node).expect("multi-node graph");
        assert_eq!(registered.node_sources.len(), 2);
        assert_eq!(registered.node_sources[1].name, "Company");

        let mut multi_relationship = registration("multi_relationship");
        let mut second = multi_relationship.relationship_sources[0].clone();
        second.name = "LIKES".to_owned();
        multi_relationship.relationship_sources.push(second);
        let registered =
            register_graph(&connection, &multi_relationship).expect("multi-relationship graph");
        assert_eq!(registered.relationship_sources.len(), 2);
        assert_eq!(registered.relationship_sources[1].name, "LIKES");
        let reloaded =
            load_registered_graph(&connection, "multi_relationship").expect("reload graph");
        assert_eq!(reloaded.node_sources, registered.node_sources);
        assert_eq!(
            reloaded.relationship_sources,
            registered.relationship_sources
        );
    }

    #[test]
    fn registered_multi_source_graph_reopens_with_stable_source_ids() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let path = directory.path().join("multi-source.db");
        let path = path.to_str().expect("database path is UTF-8");
        let registered = {
            let (_io, database) =
                crate::open_database(path, None, OpenFlags::default(), DatabaseOpts::new())
                    .expect("open database");
            let connection = database.connect().expect("connect");
            connection
                .execute(
                    "CREATE TABLE people(id INTEGER PRIMARY KEY); \
                     CREATE TABLE companies(id INTEGER PRIMARY KEY); \
                     CREATE TABLE employment(\
                         id INTEGER PRIMARY KEY, person_id INTEGER, company_id INTEGER\
                     ); \
                     CREATE TABLE ownership(\
                         id INTEGER PRIMARY KEY, company_id INTEGER, person_id INTEGER\
                     );",
                )
                .expect("create multi-source tables");
            let registered = register_graph(
                &connection,
                &GraphRegistration {
                    name: "multi".to_owned(),
                    node_sources: vec![
                        NodeSourceRegistration {
                            name: "people".to_owned(),
                            table: "people".to_owned(),
                            identity_column: "id".to_owned(),
                        },
                        NodeSourceRegistration {
                            name: "companies".to_owned(),
                            table: "companies".to_owned(),
                            identity_column: "id".to_owned(),
                        },
                    ],
                    relationship_sources: vec![
                        RelationshipSourceRegistration::binary(
                            "employment",
                            "employment",
                            "id",
                            "person_id",
                            "company_id",
                            "people",
                            "companies",
                        ),
                        RelationshipSourceRegistration::binary(
                            "ownership",
                            "ownership",
                            "id",
                            "company_id",
                            "person_id",
                            "companies",
                            "people",
                        ),
                    ],
                },
            )
            .expect("register multi-source graph");
            connection.close().expect("close connection");
            registered
        };

        let (_io, database) =
            crate::open_database(path, None, OpenFlags::default(), DatabaseOpts::new())
                .expect("reopen database");
        let connection = database.connect().expect("reconnect");
        let reopened = load_registered_graph(&connection, "multi").expect("reload graph");
        // The derived generation is a change signal for this process, not
        // catalog content: a reopened database counts changes from scratch, so
        // it is expected to differ and callers rebuild once. Everything the
        // catalog actually persists has to survive the round trip.
        assert!(
            reopened.derived_generation.is_some(),
            "a reopened graph must still produce a change signal"
        );
        assert_eq!(
            RegisteredGraph {
                derived_generation: registered.derived_generation,
                ..reopened
            },
            registered
        );
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

    /// Reads the change signal traversal snapshots actually compare.
    fn derived(connection: &Arc<Connection>) -> u64 {
        load_registered_graph(connection, "social")
            .expect("load graph")
            .derived_generation
            .expect("a registered graph must produce a change signal")
    }

    #[test]
    fn registration_detects_source_writes_without_installing_triggers() {
        let connection = connection();
        create_sources(&connection);
        let graph = register_graph(&connection, &registration("social")).expect("register graph");

        assert_eq!(graph.node_sources.len(), 1);
        assert_eq!(graph.relationship_sources.len(), 1);
        assert_ne!(graph.node_sources[0].id, graph.relationship_sources[0].id);

        // Registration must not put a trigger on a mapped table. Each one cost a
        // row write on the single hottest row in the database for every row
        // written; the change tokens below give the same answer for free.
        let triggers = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'trigger' \
             AND name LIKE '__turso_internal_graph_%'",
        )
        .expect("query internal triggers");
        assert!(
            triggers.is_empty(),
            "registration installed graph triggers: {triggers:?}"
        );

        // Every shape of write the retired triggers fired on still has to move
        // the signal, or a snapshot outlives the rows it was built from.
        let mut previous = derived(&connection);
        for (label, sql) in [
            ("insert node", "INSERT INTO people VALUES (1, 'Ada')"),
            (
                "update node",
                "UPDATE people SET name = 'Grace' WHERE id = 1",
            ),
            ("insert edge", "INSERT INTO friendships VALUES (1, 1, 1)"),
            ("delete edge", "DELETE FROM friendships WHERE id = 1"),
        ] {
            connection
                .execute(sql)
                .unwrap_or_else(|_| panic!("{label}"));
            let current = derived(&connection);
            assert_ne!(previous, current, "{label} did not move the change signal");
            previous = current;
        }
    }

    #[test]
    fn a_two_role_registration_lands_on_todays_physical_shape() {
        // Binary is a layout of the role model, not a separate kind. The
        // registration that used to name start_column/end_column must produce
        // the same two indexed columns plus the composite pair index, or every
        // donor corpus source silently changes its access path.
        let connection = connection();
        create_sources(&connection);
        let graph = register_graph(&connection, &registration("social")).expect("register graph");

        let source = &graph.relationship_sources[0];
        assert_eq!(source.roles.len(), 2);
        assert_eq!(source.roles[0].name, "start");
        assert_eq!(source.roles[0].column, "src");
        assert_eq!(source.roles[0].cardinality, RoleCardinality::One);
        assert_eq!(source.roles[1].name, "end");
        assert_eq!(source.roles[1].column, "dst");
        assert!(
            source.role_by_name("START").is_some(),
            "role lookup is case-insensitive"
        );

        let indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // One per role plus the composite pair index: exactly today's three.
        assert_eq!(indexes.len(), 3);
    }

    #[test]
    fn registration_reuses_existing_endpoint_btree_indexes() {
        // Schema producers often index relationship endpoints before the
        // graph is registered. A second index with the same leading columns
        // adds write and storage cost but cannot provide a new access path.
        let connection = connection();
        create_sources(&connection);
        connection
            .execute(
                "CREATE INDEX friendships_src ON friendships(src); \
                 CREATE INDEX friendships_dst ON friendships(dst);",
            )
            .expect("create producer endpoint indexes");

        register_graph(&connection, &registration("social")).expect("register graph");

        let graph_indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query graph endpoint indexes");
        assert_eq!(
            graph_indexes.len(),
            1,
            "only the missing composite pair index should be installed"
        );
    }

    #[test]
    fn a_three_role_registration_indexes_every_role_and_every_ordered_pair() {
        let connection = connection();
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY); \
                 CREATE TABLE texts(id INTEGER PRIMARY KEY); \
                 CREATE TABLE folios(id INTEGER PRIMARY KEY); \
                 CREATE TABLE transcriptions(\
                     id INTEGER PRIMARY KEY, scribe INTEGER, txt INTEGER, folio INTEGER);",
            )
            .expect("create ternary sources");
        let graph = register_graph(
            &connection,
            &GraphRegistration {
                name: "scriptorium".to_owned(),
                node_sources: vec![
                    NodeSourceRegistration {
                        name: "Person".to_owned(),
                        table: "people".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "Text".to_owned(),
                        table: "texts".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "Folio".to_owned(),
                        table: "folios".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                ],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "Transcription".to_owned(),
                    table: "transcriptions".to_owned(),
                    identity_column: "id".to_owned(),
                    roles: vec![
                        RoleSourceRegistration {
                            name: "scribe".to_owned(),
                            column: "scribe".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "text".to_owned(),
                            column: "txt".to_owned(),
                            node_source: "Text".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "folio".to_owned(),
                            column: "folio".to_owned(),
                            node_source: "Folio".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                    ],
                }],
            },
        )
        .expect("register ternary graph");

        assert_eq!(graph.relationship_sources[0].roles.len(), 3);
        let indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // Three single-role indexes plus one composite per unordered role pair
        // (scribe,text), (scribe,folio), (text,folio).
        assert_eq!(indexes.len(), 6);
    }

    #[test]
    fn a_many_role_spills_to_a_side_table_indexed_both_directions_and_is_excluded_from_pair_indexes(
    ) {
        // A Many role has no endpoint column on the relation table itself:
        // its players live in <table>__<role>(relation_id, node_id), indexed
        // from both directions, and it must not appear in any composite pair
        // index (those cover single-valued roles only).
        let connection = connection();
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY); \
                 CREATE TABLE texts(id INTEGER PRIMARY KEY); \
                 CREATE TABLE citations(\
                     id INTEGER PRIMARY KEY, author_id INTEGER, work_id INTEGER);",
            )
            .expect("create sources with a many-valued role");
        let graph = register_graph(
            &connection,
            &GraphRegistration {
                name: "library".to_owned(),
                node_sources: vec![
                    NodeSourceRegistration {
                        name: "Person".to_owned(),
                        table: "people".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "Text".to_owned(),
                        table: "texts".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                ],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "Citation".to_owned(),
                    table: "citations".to_owned(),
                    identity_column: "id".to_owned(),
                    roles: vec![
                        RoleSourceRegistration {
                            name: "author".to_owned(),
                            column: "author_id".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "work".to_owned(),
                            column: "work_id".to_owned(),
                            node_source: "Text".to_owned(),
                            cardinality: RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "endorsers".to_owned(),
                            column: "endorsers".to_owned(),
                            node_source: "Person".to_owned(),
                            cardinality: RoleCardinality::Many,
                        },
                    ],
                }],
            },
        )
        .expect("register graph with a many-valued role");

        let source = &graph.relationship_sources[0];
        assert_eq!(source.roles.len(), 3);
        let endorsers = source.role_by_name("endorsers").expect("endorsers role");
        assert_eq!(endorsers.cardinality, RoleCardinality::Many);

        // No endpoint column for the Many role on the relation table itself.
        let columns = query_rows(&connection, "PRAGMA table_info(citations)")
            .expect("query relation table columns")
            .into_iter()
            .map(|row| {
                text(&row, 1, "column name")
                    .expect("column name")
                    .to_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(columns, vec!["id", "author_id", "work_id"]);

        // The spill table exists, named <table>__<role>, with the
        // (relation_id, node_id) shape.
        let spill_table = source.spill_table(endorsers);
        assert_eq!(spill_table, "citations__endorsers");
        let spill_columns = query_rows(&connection, &format!("PRAGMA table_info({spill_table})"))
            .expect("query spill table columns")
            .into_iter()
            .map(|row| {
                text(&row, 1, "column name")
                    .expect("column name")
                    .to_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(spill_columns, vec!["relation_id", "node_id"]);

        // Indexed in both directions.
        let spill_indexes = query_rows(
            &connection,
            &format!(
                "SELECT name FROM sqlite_schema WHERE type = 'index' AND tbl_name = {}",
                sql_string(&spill_table)
            ),
        )
        .expect("query spill table indexes");
        assert_eq!(spill_indexes.len(), 2, "indexed in both directions");
        // Count alone can't distinguish forward-twice from forward-and-reverse;
        // assert the actual indexed column order for each index.
        let spill_index_columns = spill_indexes
            .iter()
            .map(|row| {
                let index_name = text(row, 0, "index name").expect("index name");
                query_rows(
                    &connection,
                    &format!("PRAGMA index_info({})", sql_string(index_name)),
                )
                .expect("query index_info")
                .into_iter()
                .map(|info_row| {
                    text(&info_row, 2, "indexed column name")
                        .expect("indexed column name")
                        .to_owned()
                })
                .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert!(
            spill_index_columns.contains(&vec!["relation_id".to_owned(), "node_id".to_owned()]),
            "expected a forward (relation_id, node_id) index, got {spill_index_columns:?}"
        );
        assert!(
            spill_index_columns.contains(&vec!["node_id".to_owned(), "relation_id".to_owned()]),
            "expected a reverse (node_id, relation_id) index, got {spill_index_columns:?}"
        );

        // The composite pair index covers the (author, work) single-valued
        // pair only; the Many role is excluded from pairing entirely.
        let endpoint_indexes = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'index' \
             AND name LIKE '__turso_internal_graph_ep_%'",
        )
        .expect("query endpoint indexes");
        // One index per single-valued role (author, work) plus one composite
        // pair index for (author, work). The Many role gets no endpoint
        // index of any kind.
        assert_eq!(endpoint_indexes.len(), 3);

        // The registration round-trips through load_registered_graph with
        // role name, ordinal, and cardinality intact.
        let reloaded = load_registered_graph(&connection, "library").expect("reload graph");
        let reloaded_source = &reloaded.relationship_sources[0];
        assert_eq!(reloaded_source.roles.len(), 3);
        assert_eq!(reloaded_source.roles[0].name, "author");
        assert_eq!(reloaded_source.roles[0].role, ir::RoleId::new(1).unwrap());
        assert_eq!(reloaded_source.roles[0].cardinality, RoleCardinality::One);
        assert_eq!(reloaded_source.roles[1].name, "work");
        assert_eq!(reloaded_source.roles[1].role, ir::RoleId::new(2).unwrap());
        assert_eq!(reloaded_source.roles[1].cardinality, RoleCardinality::One);
        assert_eq!(reloaded_source.roles[2].name, "endorsers");
        assert_eq!(reloaded_source.roles[2].role, ir::RoleId::new(3).unwrap());
        assert_eq!(reloaded_source.roles[2].cardinality, RoleCardinality::Many);
    }

    #[test]
    fn a_role_must_name_a_registered_node_source() {
        let connection = connection();
        create_sources(&connection);
        let mut graph = registration("bad_endpoint");
        graph.relationship_sources[0].roles[1].node_source = "Missing".to_owned();
        assert!(matches!(
            register_graph(&connection, &graph),
            Err(CatalogError::UnknownEndpoint { relationship, node_source })
                if relationship == "KNOWS" && node_source == "Missing"
        ));
    }

    #[test]
    fn loading_a_graph_drops_generation_triggers_an_older_build_left_behind() {
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");

        // Exactly what an older build installed, recreated by hand because
        // nothing installs it any more. Its body updates a protected internal
        // table, which core no longer excuses, so leaving it in place would
        // fail the next write to `people`.
        connection.execute("BEGIN IMMEDIATE").expect("begin");
        connection
            .prepare_internal(format!(
                "CREATE TRIGGER {GENERATION_TRIGGER_PREFIX}1_insert_stale AFTER INSERT ON people \
                 BEGIN UPDATE {GENERATIONS_TABLE} SET generation = generation + 1 WHERE graph_id = 1; END"
            ))
            .expect("prepare legacy trigger")
            .run_ignore_rows()
            .expect("install legacy trigger");
        connection.execute("COMMIT").expect("commit");

        load_registered_graph(&connection, "social").expect("load graph");

        let triggers = query_rows(
            &connection,
            "SELECT name FROM sqlite_schema WHERE type = 'trigger' \
             AND name LIKE '__turso_internal_graph_%'",
        )
        .expect("query internal triggers");
        assert!(
            triggers.is_empty(),
            "loading the graph must drop what an older build installed: {triggers:?}"
        );
        connection
            .execute("INSERT INTO people VALUES (1, 'Ada')")
            .expect("writes to a mapped table must work once the trigger is gone");
    }

    #[test]
    fn source_write_rollback_restores_the_generation() {
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");

        let before = derived(&connection);
        connection.execute("BEGIN").expect("begin");
        connection
            .execute("INSERT INTO people VALUES (1, 'Ada')")
            .expect("insert node");
        // The writer reads its own uncommitted row, so a snapshot built before
        // the insert is already stale for this connection.
        assert_ne!(before, derived(&connection), "inside the transaction");
        connection.execute("ROLLBACK").expect("rollback");
        assert_eq!(
            before,
            derived(&connection),
            "the rollback undid every row, so a snapshot built before the \
             transaction is good again and must not be thrown away"
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
    fn a_catalog_predating_roles_fails_at_open_and_names_the_fresh_start_policy() {
        // Fresh start: there is no legacy reader and no migration. Opening a
        // pre-role catalog must say so rather than reporting a confusing
        // "invalid catalog value" from a missing column.
        let connection = connection();
        create_sources(&connection);
        register_graph(&connection, &registration("social")).expect("register graph");
        // Simulate the pre-role layout: the roles table did not exist.
        // DROP TABLE on a reserved-prefixed table is only permitted for a
        // nested (internal) statement, and a nested statement cannot open
        // its own write transaction, so drive it inside an explicit one.
        connection.execute("BEGIN IMMEDIATE").expect("begin");
        execute_internal(
            &connection,
            format!("DROP TABLE {RELATIONSHIP_ROLES_TABLE}"),
        )
        .expect("drop roles table");
        connection.execute("COMMIT").expect("commit");

        let error = load_registered_graph(&connection, "social").expect_err("pre-role catalog");
        let message = error.to_string();
        assert!(
            matches!(error, CatalogError::IncompatibleGraphLayout { .. }),
            "expected IncompatibleGraphLayout, got {message}"
        );
        assert!(
            message.contains("no migration"),
            "the error must name the fresh-start policy, got {message}"
        );
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
    fn users_cannot_mutate_catalog_or_touch_reserved_graph_objects() {
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

        // The reserved name space is checked before the trigger is looked up, so
        // this is rejected for its name rather than for not existing. Nothing
        // installs a graph trigger any more, but the frontend still drops ones
        // an older build left behind, and that has to stay off limits to users.
        assert!(connection
            .execute("DROP TRIGGER __turso_internal_graph_gen_forged")
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
