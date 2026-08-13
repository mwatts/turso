use std::sync::Arc;

use turso_core::Connection;
use turso_graph_ir::GraphId;

use crate::{
    Error, SemanticSnapshot, SemanticTypeInfo,
    catalog::{quote_identifier, scalar_integer},
    load_registered_graph, load_semantic_snapshot,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphSchemaInspection {
    pub graph_id: GraphId,
    pub graph_name: String,
    pub generation: u64,
    pub node_sources: Vec<GraphNodeSourceInspection>,
    pub relationship_sources: Vec<GraphRelationshipSourceInspection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphNodeSourceInspection {
    pub source_name: String,
    pub table: String,
    pub identity_column: String,
    pub row_count: u64,
    pub semantic_types: Vec<GraphSemanticTypeInspection>,
    pub fts_indexes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphRelationshipSourceInspection {
    pub source_name: String,
    pub table: String,
    pub identity_column: String,
    pub row_count: u64,
    pub semantic_types: Vec<GraphSemanticTypeInspection>,
    pub roles: Vec<GraphRoleInspection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphSemanticTypeInspection {
    pub name: String,
    pub properties: Vec<GraphPropertyInspection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphPropertyInspection {
    pub name: String,
    pub column: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphRoleInspection {
    pub name: String,
    pub column: String,
    pub node_sources: Vec<String>,
    pub discriminator_column: Option<String>,
    pub cardinality: String,
}

pub(crate) fn inspect(
    connection: &Arc<Connection>,
    graph_name: &str,
) -> Result<GraphSchemaInspection, Error> {
    let graph = load_registered_graph(connection, graph_name)?;
    let semantic = load_semantic_snapshot(connection, &graph)?;
    #[cfg(feature = "fts")]
    let fts = crate::fts::list(connection, graph_name)?;

    let mut node_sources = Vec::with_capacity(graph.node_sources.len());
    for source in &graph.node_sources {
        #[cfg(feature = "fts")]
        let mut fts_indexes = fts
            .iter()
            .filter(|index| index.spec.source.eq_ignore_ascii_case(&source.name))
            .map(|index| index.spec.name.clone())
            .collect::<Vec<_>>();
        #[cfg(not(feature = "fts"))]
        let mut fts_indexes = Vec::new();
        fts_indexes.sort();
        node_sources.push(GraphNodeSourceInspection {
            source_name: source.name.clone(),
            table: source.table.clone(),
            identity_column: source.identity_column.clone(),
            row_count: row_count(connection, &source.table)?,
            semantic_types: semantic_types_for_source(semantic.as_ref(), source.id, true),
            fts_indexes,
        });
    }

    let mut relationship_sources = Vec::with_capacity(graph.relationship_sources.len());
    for source in &graph.relationship_sources {
        let roles = source
            .roles
            .iter()
            .map(|role| GraphRoleInspection {
                name: role.name.clone(),
                column: role.column.clone(),
                node_sources: role
                    .node_sources
                    .iter()
                    .filter_map(|id| graph.node_sources.iter().find(|source| source.id == *id))
                    .map(|source| source.name.clone())
                    .collect(),
                discriminator_column: role.discriminator_column.clone(),
                cardinality: match role.cardinality {
                    turso_graph_ir::RoleCardinality::One => "one".to_owned(),
                    turso_graph_ir::RoleCardinality::Many => "many".to_owned(),
                },
            })
            .collect();
        relationship_sources.push(GraphRelationshipSourceInspection {
            source_name: source.name.clone(),
            table: source.table.clone(),
            identity_column: source.identity_column.clone(),
            row_count: row_count(connection, &source.table)?,
            semantic_types: semantic_types_for_source(semantic.as_ref(), source.id, false),
            roles,
        });
    }

    Ok(GraphSchemaInspection {
        graph_id: graph.id,
        graph_name: graph.name,
        generation: graph.generation,
        node_sources,
        relationship_sources,
    })
}

fn semantic_types_for_source(
    semantic: Option<&SemanticSnapshot>,
    source: turso_graph_ir::SourceTableId,
    node: bool,
) -> Vec<GraphSemanticTypeInspection> {
    let Some(semantic) = semantic else {
        return Vec::new();
    };
    let values: Box<dyn Iterator<Item = &SemanticTypeInfo> + '_> = if node {
        Box::new(semantic.node_type_values())
    } else {
        Box::new(semantic.relationship_type_values())
    };
    let mut types = values
        .filter(|type_info| type_info.source == source)
        .map(|type_info| {
            let mut properties = type_info
                .property_values()
                .map(|property| GraphPropertyInspection {
                    name: property.name.clone(),
                    column: property.column.clone(),
                })
                .collect::<Vec<_>>();
            properties.sort_by(|left, right| left.name.cmp(&right.name));
            GraphSemanticTypeInspection {
                name: type_info.name.clone(),
                properties,
            }
        })
        .collect::<Vec<_>>();
    types.sort_by(|left, right| left.name.cmp(&right.name));
    types
}

fn row_count(connection: &Arc<Connection>, table: &str) -> Result<u64, Error> {
    let count = scalar_integer(
        connection,
        &format!("SELECT COUNT(*) FROM {}", quote_identifier(table)),
        "graph source row count",
    )?;
    u64::try_from(count).map_err(|_| {
        Error::Database(turso_core::LimboError::InternalError(format!(
            "negative row count for graph source {table}"
        )))
    })
}
