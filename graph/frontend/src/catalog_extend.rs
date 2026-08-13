//! Additive graph registration: add sources without renumbering existing ids.

use std::{collections::BTreeMap, sync::Arc};

use turso_core::{Connection, Numeric, Value};
use turso_graph_ir::{GraphId, RoleCardinality, SourceTableId};

use crate::catalog::{
    CatalogError, GraphRegistration, NODE_SOURCES_TABLE, NodeSourceRegistration,
    PolymorphicRoleRegistration, RELATIONSHIP_ROLES_TABLE, RELATIONSHIP_SOURCES_TABLE,
    RegisteredGraph, RegisteredRelationshipRole, RegisteredRelationshipSource,
    RelationshipSourceRegistration, RoleSourceRegistration, SOURCES_TABLE, bump_catalog_generation,
    execute_internal, install_role_index, install_role_pair_indexes, install_spill_table,
    load_registered_graph, polymorphic_role, query_rows, relationship_type_registry_table_name,
    require_columns, require_custom_types_enabled_for_source, require_unique_identity,
    scalar_integer, source_id, sql_string, validate_registration_names,
};
use crate::transaction::in_write_transaction;

const EXTEND_SAVEPOINT: &str = "turso_graph_extend";

/// Add sources to an already-registered graph, keeping existing source ids.
///
/// The host passes the desired complete registration. Existing sources whose
/// table, identity, and roles match are left alone. New sources are inserted.
/// A missing graph is [`CatalogError::GraphNotFound`]. A shape change is
/// [`CatalogError::SourceConflict`]. Omitting a stored source is
/// [`CatalogError::SourceRemoved`].
pub fn extend_graph_registration(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<RegisteredGraph, CatalogError> {
    validate_registration_names(registration, polymorphic_roles)?;
    in_write_transaction(connection, EXTEND_SAVEPOINT, || {
        extend_in_transaction(connection, registration, polymorphic_roles)
    })
}

fn extend_in_transaction(
    connection: &Arc<Connection>,
    registration: &GraphRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<RegisteredGraph, CatalogError> {
    let current = match load_registered_graph(connection, &registration.name) {
        Ok(graph) => graph,
        Err(CatalogError::GraphNotFound(_)) | Err(CatalogError::IncompatibleGraphLayout { .. }) => {
            return Err(CatalogError::GraphNotFound(registration.name.clone()));
        }
        Err(error) => return Err(error),
    };

    reject_removed_sources(&current, registration)?;
    detect_node_conflicts(&current, registration)?;
    detect_relationship_conflicts(&current, registration, polymorphic_roles)?;

    let existing_node_names = current
        .node_sources
        .iter()
        .map(|source| source.name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    let existing_relationship_names = current
        .relationship_sources
        .iter()
        .map(|source| source.name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();

    let mut inserted = false;
    for source in &registration.node_sources {
        if existing_node_names.contains(&source.name.to_ascii_lowercase()) {
            continue;
        }
        validate_new_node_source(connection, source)?;
        insert_node_source(connection, current.id, source)?;
        inserted = true;
    }

    let after_nodes = load_registered_graph(connection, &registration.name)?;
    let node_ids = after_nodes
        .node_sources
        .iter()
        .map(|source| (source.name.to_ascii_lowercase(), source.id))
        .collect::<BTreeMap<_, _>>();

    for source in &registration.relationship_sources {
        if existing_relationship_names.contains(&source.name.to_ascii_lowercase()) {
            continue;
        }
        validate_new_relationship_source(connection, source, polymorphic_roles)?;
        insert_relationship_source(connection, current.id, source, &node_ids, polymorphic_roles)?;
        inserted = true;
    }

    if inserted {
        bump_catalog_generation(connection, current.id.get())?;
    }
    load_registered_graph(connection, &registration.name)
}

fn reject_removed_sources(
    current: &RegisteredGraph,
    registration: &GraphRegistration,
) -> Result<(), CatalogError> {
    let planned_nodes = registration
        .node_sources
        .iter()
        .map(|source| source.name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    let planned_relationships = registration
        .relationship_sources
        .iter()
        .map(|source| source.name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    for source in &current.node_sources {
        if !planned_nodes.contains(&source.name.to_ascii_lowercase()) {
            return Err(CatalogError::SourceRemoved {
                graph: current.name.clone(),
                source_name: source.name.clone(),
            });
        }
    }
    for source in &current.relationship_sources {
        if !planned_relationships.contains(&source.name.to_ascii_lowercase()) {
            return Err(CatalogError::SourceRemoved {
                graph: current.name.clone(),
                source_name: source.name.clone(),
            });
        }
    }
    Ok(())
}

fn detect_node_conflicts(
    current: &RegisteredGraph,
    registration: &GraphRegistration,
) -> Result<(), CatalogError> {
    let stored = current
        .node_sources
        .iter()
        .map(|source| (source.name.to_ascii_lowercase(), source))
        .collect::<BTreeMap<_, _>>();
    for source in &registration.node_sources {
        let Some(existing) = stored.get(&source.name.to_ascii_lowercase()) else {
            continue;
        };
        if !existing.table.eq_ignore_ascii_case(&source.table) {
            return Err(CatalogError::SourceConflict {
                graph: current.name.clone(),
                source_name: source.name.clone(),
                reason: "table".to_owned(),
            });
        }
        if !existing
            .identity_column
            .eq_ignore_ascii_case(&source.identity_column)
        {
            return Err(CatalogError::SourceConflict {
                graph: current.name.clone(),
                source_name: source.name.clone(),
                reason: "identity_column".to_owned(),
            });
        }
    }
    Ok(())
}

fn detect_relationship_conflicts(
    current: &RegisteredGraph,
    registration: &GraphRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<(), CatalogError> {
    let stored = current
        .relationship_sources
        .iter()
        .map(|source| (source.name.to_ascii_lowercase(), source))
        .collect::<BTreeMap<_, _>>();
    let node_names = current
        .node_sources
        .iter()
        .map(|source| (source.id, source.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    for source in &registration.relationship_sources {
        let Some(existing) = stored.get(&source.name.to_ascii_lowercase()) else {
            continue;
        };
        if !existing.table.eq_ignore_ascii_case(&source.table) {
            return Err(CatalogError::SourceConflict {
                graph: current.name.clone(),
                source_name: source.name.clone(),
                reason: "table".to_owned(),
            });
        }
        if !existing
            .identity_column
            .eq_ignore_ascii_case(&source.identity_column)
        {
            return Err(CatalogError::SourceConflict {
                graph: current.name.clone(),
                source_name: source.name.clone(),
                reason: "identity_column".to_owned(),
            });
        }
        if !roles_match(existing, source, polymorphic_roles, &node_names) {
            return Err(CatalogError::SourceConflict {
                graph: current.name.clone(),
                source_name: source.name.clone(),
                reason: "roles".to_owned(),
            });
        }
    }
    Ok(())
}

fn roles_match(
    existing: &RegisteredRelationshipSource,
    planned: &RelationshipSourceRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
    node_names: &BTreeMap<SourceTableId, &str>,
) -> bool {
    if existing.roles.len() != planned.roles.len() {
        return false;
    }
    existing
        .roles
        .iter()
        .zip(planned.roles.iter())
        .all(|(stored, role)| {
            stored.name.eq_ignore_ascii_case(&role.name)
                && stored.column.eq_ignore_ascii_case(&role.column)
                && stored.cardinality == role.cardinality
                && stored_role_targets(stored, node_names)
                    == planned_role_targets(planned, role, polymorphic_roles)
                && stored.discriminator_column.as_deref().unwrap_or("")
                    == polymorphic_role(polymorphic_roles, planned, role)
                        .map(|registration| registration.discriminator_column.as_str())
                        .unwrap_or("")
        })
}

fn stored_role_targets(
    role: &RegisteredRelationshipRole,
    node_names: &BTreeMap<SourceTableId, &str>,
) -> Vec<String> {
    let mut names = role
        .node_sources
        .iter()
        .filter_map(|id| node_names.get(id).map(|name| name.to_ascii_lowercase()))
        .collect::<Vec<_>>();
    names.sort();
    names
}

fn planned_role_targets(
    source: &RelationshipSourceRegistration,
    role: &RoleSourceRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Vec<String> {
    let mut names = polymorphic_role(polymorphic_roles, source, role)
        .map(|registration| {
            registration
                .node_sources
                .iter()
                .map(|name| name.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![role.node_source.to_ascii_lowercase()]);
    names.sort();
    names
}

fn validate_new_node_source(
    connection: &Arc<Connection>,
    source: &NodeSourceRegistration,
) -> Result<(), CatalogError> {
    let columns = require_columns(connection, &source.table, &[&source.identity_column])?;
    require_unique_identity(connection, &source.table, &source.identity_column, &columns)?;
    require_custom_types_enabled_for_source(connection, &source.table)
}

fn validate_new_relationship_source(
    connection: &Arc<Connection>,
    source: &RelationshipSourceRegistration,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<(), CatalogError> {
    let mut required_columns = vec![source.identity_column.as_str()];
    required_columns.extend(
        source
            .roles
            .iter()
            .filter(|role| role.cardinality == RoleCardinality::One)
            .map(|role| role.column.as_str()),
    );
    required_columns.extend(source.roles.iter().filter_map(|role| {
        polymorphic_role(polymorphic_roles, source, role)
            .map(|registration| registration.discriminator_column.as_str())
    }));
    let columns = require_columns(connection, &source.table, &required_columns)?;
    require_custom_types_enabled_for_source(connection, &source.table)?;
    require_unique_identity(connection, &source.table, &source.identity_column, &columns)
}

fn insert_node_source(
    connection: &Arc<Connection>,
    graph_id: GraphId,
    source: &NodeSourceRegistration,
) -> Result<SourceTableId, CatalogError> {
    execute_internal(
        connection,
        format!(
            "INSERT INTO {SOURCES_TABLE}(graph_id, name, kind) VALUES ({}, {}, 'node')",
            graph_id.get(),
            sql_string(&source.name)
        ),
    )?;
    let id = scalar_integer(
        connection,
        &format!(
            "SELECT id FROM {SOURCES_TABLE} WHERE graph_id = {} AND name = {}",
            graph_id.get(),
            sql_string(&source.name)
        ),
        "node source id",
    )?;
    let id = source_id(id)?;
    execute_internal(
        connection,
        format!(
            "INSERT INTO {NODE_SOURCES_TABLE}(source_id, table_name, identity_column) \
             VALUES ({}, {}, {})",
            id.get(),
            sql_string(&source.table),
            sql_string(&source.identity_column)
        ),
    )?;
    Ok(id)
}

fn insert_relationship_source(
    connection: &Arc<Connection>,
    graph_id: GraphId,
    source: &RelationshipSourceRegistration,
    node_ids: &BTreeMap<String, SourceTableId>,
    polymorphic_roles: &[PolymorphicRoleRegistration],
) -> Result<(), CatalogError> {
    execute_internal(
        connection,
        format!(
            "INSERT INTO {SOURCES_TABLE}(graph_id, name, kind) VALUES ({}, {}, 'relationship')",
            graph_id.get(),
            sql_string(&source.name)
        ),
    )?;
    let relationship_id = scalar_integer(
        connection,
        &format!(
            "SELECT id FROM {SOURCES_TABLE} WHERE graph_id = {} AND name = {}",
            graph_id.get(),
            sql_string(&source.name)
        ),
        "relationship source id",
    )?;
    execute_internal(
        connection,
        format!(
            "INSERT INTO {RELATIONSHIP_SOURCES_TABLE}(source_id, table_name, identity_column) \
             VALUES ({}, {}, {})",
            relationship_id,
            sql_string(&source.table),
            sql_string(&source.identity_column)
        ),
    )?;
    for (ordinal, role) in source.roles.iter().enumerate() {
        let polymorphic = polymorphic_role(polymorphic_roles, source, role);
        let names = polymorphic
            .map(|registration| registration.node_sources.as_slice())
            .unwrap_or_else(|| std::slice::from_ref(&role.node_source));
        let node_source_ids = names
            .iter()
            .map(|name| {
                node_ids
                    .get(&name.to_ascii_lowercase())
                    .copied()
                    .ok_or_else(|| CatalogError::UnknownEndpoint {
                        relationship: source.name.clone(),
                        node_source: name.clone(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .map(|source| source.get().to_string())
            .collect::<Vec<_>>()
            .join(",");
        execute_internal(
            connection,
            format!(
                "INSERT INTO {RELATIONSHIP_ROLES_TABLE}(\
                    source_id, ordinal, name, column_name, node_source_ids, \
                    node_source_column, cardinality\
                 ) VALUES ({}, {}, {}, {}, {}, {}, {})",
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
            ),
        )?;
        match role.cardinality {
            RoleCardinality::One => {
                install_role_index(connection, graph_id, source, role, polymorphic)?;
            }
            RoleCardinality::Many => {
                install_spill_table(connection, graph_id, source, role)?;
            }
        }
    }
    install_role_pair_indexes(connection, graph_id, source, polymorphic_roles)?;
    insert_type_registry_row(connection, graph_id, &source.name)?;
    Ok(())
}

fn insert_type_registry_row(
    connection: &Arc<Connection>,
    graph_id: GraphId,
    name: &str,
) -> Result<(), CatalogError> {
    let table = relationship_type_registry_table_name(graph_id);
    let next_id = match query_rows(connection, &format!("SELECT max(id) FROM \"{table}\""))?
        .first()
        .and_then(|row| row.first())
    {
        Some(Value::Numeric(Numeric::Integer(value))) => value + 1,
        _ => 1,
    };
    execute_internal(
        connection,
        format!(
            "INSERT INTO \"{table}\"(id, name) VALUES ({next_id}, {})",
            sql_string(name)
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{
        GraphRegistration, NodeSourceRegistration, RelationshipSourceRegistration, register_graph,
    };
    use turso_core::{Database, MemoryIO, SqliteDialect, schema::TURSO_GRAPH_CATALOG_PREFIX};

    fn connection() -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file(io, ":memory:graph-extend", Arc::new(SqliteDialect))
            .expect("open database")
            .connect()
            .expect("connect")
    }

    fn person_graph() -> GraphRegistration {
        GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        }
    }

    fn extended_graph() -> GraphRegistration {
        GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![
                NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                },
                NodeSourceRegistration {
                    name: "Team".to_owned(),
                    table: "teams".to_owned(),
                    identity_column: "id".to_owned(),
                },
            ],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "Member",
                "memberships",
                "id",
                "person_id",
                "team_id",
                "Person",
                "Team",
            )],
        }
    }

    fn create_tables(connection: &Arc<Connection>) {
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
                 CREATE TABLE teams(id INTEGER PRIMARY KEY, name TEXT); \
                 CREATE TABLE memberships(id INTEGER PRIMARY KEY, person_id INTEGER, team_id INTEGER);",
            )
            .expect("create tables");
    }

    #[test]
    fn extend_on_a_missing_graph_is_not_found() {
        let connection = connection();
        create_tables(&connection);
        let error = extend_graph_registration(&connection, &extended_graph(), &[])
            .expect_err("missing graph");
        assert!(
            matches!(error, CatalogError::GraphNotFound(ref name) if name == "social"),
            "expected GraphNotFound, got {error:?}"
        );
    }

    #[test]
    fn extend_adds_sources_and_keeps_existing_ids() {
        let connection = connection();
        create_tables(&connection);
        let first = register_graph(&connection, &person_graph()).expect("register");
        let person_id = first.node_sources[0].id;
        let generation = first.schema_generation;

        let extended =
            extend_graph_registration(&connection, &extended_graph(), &[]).expect("extend");
        let person = extended
            .node_sources
            .iter()
            .find(|source| source.name == "Person")
            .expect("Person");
        let team = extended
            .node_sources
            .iter()
            .find(|source| source.name == "Team")
            .expect("Team");
        assert_eq!(person.id, person_id);
        assert_ne!(team.id, person_id);
        assert_eq!(extended.relationship_sources.len(), 1);
        assert_eq!(extended.relationship_sources[0].name, "Member");
        assert_eq!(
            extended
                .schema_generation
                .map(|value| value.saturating_sub(generation.unwrap_or(0))),
            Some(1)
        );
    }

    #[test]
    fn a_second_identical_extend_is_a_noop() {
        let connection = connection();
        create_tables(&connection);
        register_graph(&connection, &person_graph()).expect("register");
        let first = extend_graph_registration(&connection, &extended_graph(), &[]).expect("extend");
        let second =
            extend_graph_registration(&connection, &extended_graph(), &[]).expect("extend again");
        assert_eq!(first.node_sources, second.node_sources);
        assert_eq!(first.relationship_sources, second.relationship_sources);
        assert_eq!(first.schema_generation, second.schema_generation);
    }

    #[test]
    fn changing_an_existing_source_table_is_a_conflict() {
        let connection = connection();
        create_tables(&connection);
        let first = register_graph(&connection, &person_graph()).expect("register");
        let mut drifted = person_graph();
        drifted.node_sources[0].table = "teams".to_owned();
        let error =
            extend_graph_registration(&connection, &drifted, &[]).expect_err("shape conflict");
        assert!(matches!(
            error,
            CatalogError::SourceConflict { ref reason, .. } if reason == "table"
        ));
        let after = load_registered_graph(&connection, "social").expect("reload");
        assert_eq!(after.node_sources, first.node_sources);
        assert_eq!(after.schema_generation, first.schema_generation);
    }

    #[test]
    fn omitting_a_stored_source_is_refused() {
        let connection = connection();
        create_tables(&connection);
        register_graph(&connection, &person_graph()).expect("register");
        let only_team = GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Team".to_owned(),
                table: "teams".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        };
        let error = extend_graph_registration(&connection, &only_team, &[]).expect_err("removed");
        assert!(matches!(
            error,
            CatalogError::SourceRemoved { ref source_name, .. } if source_name == "Person"
        ));
    }

    #[test]
    fn a_new_one_role_gets_an_endpoint_index_once() {
        let connection = connection();
        create_tables(&connection);
        register_graph(&connection, &person_graph()).expect("register");
        extend_graph_registration(&connection, &extended_graph(), &[]).expect("extend");
        let count = endpoint_index_count(&connection);
        assert!(count >= 1, "extend must create at least one endpoint index");
        extend_graph_registration(&connection, &extended_graph(), &[]).expect("extend again");
        assert_eq!(endpoint_index_count(&connection), count);
    }

    fn endpoint_index_count(connection: &Arc<Connection>) -> usize {
        query_rows(
            connection,
            &format!(
                "SELECT name FROM sqlite_schema WHERE type = 'index' \
                 AND name LIKE '{TURSO_GRAPH_CATALOG_PREFIX}ep_%'"
            ),
        )
        .expect("list indexes")
        .len()
    }
}
