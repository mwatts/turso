//! `CREATE GRAPH` — sugar over [`register_graph`].
//!
//! This module adds no storage model. It parses the DDL, infers the physical
//! names the statement left unsaid, emits the backing `CREATE TABLE`s, and
//! hands the result to the same registration path a Rust caller would use. A
//! graph created this way is indistinguishable from one registered directly.
//!
//! Tables are created `IF NOT EXISTS`, which is what makes one syntax serve
//! both purposes: a fresh graph gets new tables, and a graph declared over an
//! existing schema adopts those tables instead. Adoption is not blind — the
//! registration that follows verifies every named column exists and that the
//! identity column is unique, so a shape mismatch surfaces as
//! [`CatalogError::MissingColumn`] naming the exact column rather than as a
//! graph that half-works.

use std::sync::Arc;

use thiserror::Error;
use turso_core::{Connection, LimboError};
use turso_graph_cypher::{parse_ddl, ColumnDecl, GraphDdl, ParseError, RelationDecl, Spanned};
use turso_graph_ir::RoleCardinality;

use crate::catalog::{
    labels_table_name, register_graph, relationship_types_table_name, sql_string, CatalogError,
    GraphRegistration, NodeSourceRegistration, RegisteredGraph, RelationshipSourceRegistration,
    RoleSourceRegistration,
};
use crate::transaction::{in_write_transaction, WriteTransactionError};

/// Identity column used when a declaration does not say `KEY <column>`.
const DEFAULT_IDENTITY_COLUMN: &str = "id";
/// SQL type given to inferred identity and role columns.
const IDENTITY_TYPE: &str = "INTEGER";
const DDL_SAVEPOINT: &str = "__turso_graph_ddl";

#[derive(Debug, Error)]
pub enum DdlError {
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Catalog(#[from] CatalogError),
    #[error("{0}")]
    Core(#[from] LimboError),
    /// `MANY` players live in a spill table, not in a column on the relation
    /// table, so a `VIA` column for one would be silently ignored by
    /// registration. Refuse rather than accept a binding that does nothing.
    #[error(
        "role `{role}` on relation `{relation}` is MANY, so it has no endpoint column; \
         remove `VIA {column}` (many-valued players are stored in a spill table)"
    )]
    ManyRoleWithVia {
        relation: String,
        role: String,
        column: String,
    },
    #[error("graph DDL inside an open transaction requires a write transaction (BEGIN IMMEDIATE or a prior write)")]
    RequiresWriteTransaction,
    #[error("graph DDL failed and rollback also failed: {cause}; rollback: {rollback}")]
    RollbackFailed {
        cause: Box<DdlError>,
        rollback: LimboError,
    },
}

impl WriteTransactionError for DdlError {
    fn requires_write_transaction() -> Self {
        DdlError::RequiresWriteTransaction
    }

    fn rollback_failed(cause: Self, rollback: LimboError) -> Self {
        DdlError::RollbackFailed {
            cause: Box::new(cause),
            rollback,
        }
    }
}

/// Parse and execute a `CREATE GRAPH` statement.
///
/// Creates the backing tables (unless they already exist) and registers the
/// graph, atomically: either the whole declaration takes effect or none of it
/// does.
pub fn execute_graph_ddl(
    connection: &Arc<Connection>,
    source: &str,
) -> Result<RegisteredGraph, DdlError> {
    let ddl = parse_ddl(source)?;
    let plan = lower(&ddl)?;

    // The tables and the catalog rows must land together: a committed table
    // with no registration is invisible to the graph layer, and a registration
    // over a rolled-back table cannot be opened. `register_graph` does its own
    // transaction management, and takes the savepoint branch once we are
    // already inside a write transaction here.
    in_write_transaction(connection, DDL_SAVEPOINT, || apply(connection, &plan))
}

fn apply(connection: &Arc<Connection>, plan: &DdlPlan) -> Result<RegisteredGraph, DdlError> {
    for statement in &plan.create_tables {
        connection.execute(statement.as_str())?;
    }
    let graph = register_graph(connection, &plan.registration)?;
    backfill_membership(connection, &graph)?;
    Ok(graph)
}

/// Records the label of every row that already existed in an adopted table.
///
/// Membership lives in the per-graph junction tables, which Cypher `CREATE`
/// writes as it inserts. Rows written by plain SQL before the declaration have
/// no junction row, so without this a graph declared over populated tables
/// would match nothing — adoption in name only. Newly created tables are
/// empty, which makes this a no-op for them.
///
/// The junction tables are created by the registration immediately above, for
/// a graph id that is new (graph names are unique), so they hold no rows yet
/// and these inserts need no duplicate guard.
fn backfill_membership(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
) -> Result<(), DdlError> {
    let labels = labels_table_name(graph.id);
    for source in &graph.node_sources {
        connection.execute(format!(
            "INSERT INTO {}(source_id, node_id, label) SELECT {}, {}, {} FROM {}",
            quote(&labels),
            source.id.get(),
            quote(&source.identity_column),
            sql_string(&source.name),
            quote(&source.table),
        ))?;
    }

    let types = relationship_types_table_name(graph.id);
    for source in &graph.relationship_sources {
        connection.execute(format!(
            "INSERT INTO {}(source_id, relationship_id, type) SELECT {}, {}, {} FROM {}",
            quote(&types),
            source.id.get(),
            quote(&source.identity_column),
            sql_string(&source.name),
            quote(&source.table),
        ))?;
    }
    Ok(())
}

/// A lowered statement: the tables to ensure, then the registration to apply.
#[derive(Debug)]
struct DdlPlan {
    create_tables: Vec<String>,
    registration: GraphRegistration,
}

fn lower(ddl: &GraphDdl) -> Result<DdlPlan, DdlError> {
    let mut create_tables = Vec::new();
    let mut node_sources = Vec::new();

    for node in &ddl.nodes {
        let table = declared_or(node.table.as_ref(), &node.name.value);
        let identity = declared_or(node.key.as_ref(), DEFAULT_IDENTITY_COLUMN);

        let mut columns = vec![identity_column(&identity)];
        columns.extend(column_definitions(&node.columns));
        create_tables.push(create_table(&table, &columns));

        node_sources.push(NodeSourceRegistration {
            name: node.name.value.clone(),
            table,
            identity_column: identity,
        });
    }

    let mut relationship_sources = Vec::new();
    for relation in &ddl.relations {
        let (statement, source) = lower_relation(relation)?;
        create_tables.push(statement);
        relationship_sources.push(source);
    }

    Ok(DdlPlan {
        create_tables,
        registration: GraphRegistration {
            name: ddl.name.value.clone(),
            node_sources,
            relationship_sources,
        },
    })
}

fn lower_relation(
    relation: &RelationDecl,
) -> Result<(String, RelationshipSourceRegistration), DdlError> {
    let table = declared_or(relation.table.as_ref(), &relation.name.value);
    let identity = declared_or(relation.key.as_ref(), DEFAULT_IDENTITY_COLUMN);

    let mut columns = vec![identity_column(&identity)];
    let mut roles = Vec::new();

    for role in &relation.roles {
        if role.many {
            if let Some(via) = &role.via {
                return Err(DdlError::ManyRoleWithVia {
                    relation: relation.name.value.clone(),
                    role: role.name.value.clone(),
                    column: via.value.clone(),
                });
            }
            // Registration builds `<table>__<role>` for these; the relation
            // table gets no column.
            roles.push(RoleSourceRegistration {
                name: role.name.value.clone(),
                column: String::new(),
                node_source: role.target.value.clone(),
                cardinality: RoleCardinality::Many,
            });
            continue;
        }

        let column = declared_or(role.via.as_ref(), &role.name.value);
        columns.push(format!("{} {IDENTITY_TYPE}", quote(&column)));
        roles.push(RoleSourceRegistration {
            name: role.name.value.clone(),
            column,
            node_source: role.target.value.clone(),
            cardinality: RoleCardinality::One,
        });
    }

    columns.extend(column_definitions(&relation.columns));

    Ok((
        create_table(&table, &columns),
        RelationshipSourceRegistration {
            name: relation.name.value.clone(),
            table,
            identity_column: identity,
            roles,
        },
    ))
}

/// The physical name a declaration chose, or the inferred one. Every override
/// in this DDL — `AS TABLE`, `KEY`, `VIA` — resolves this way.
fn declared_or(declared: Option<&Spanned<String>>, inferred: &str) -> String {
    declared.map_or_else(|| inferred.to_owned(), |name| name.value.clone())
}

fn identity_column(name: &str) -> String {
    format!("{} {IDENTITY_TYPE} PRIMARY KEY", quote(name))
}

/// Column types are emitted verbatim: the declaration writes SQL types, and
/// this DDL does not introduce a type system of its own.
fn column_definitions(columns: &[ColumnDecl]) -> impl Iterator<Item = String> + '_ {
    columns
        .iter()
        .map(|column| format!("{} {}", quote(&column.name.value), column.column_type.value))
}

fn create_table(table: &str, columns: &[String]) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        quote(table),
        columns.join(", ")
    )
}

/// Identifiers reach here from backtick-quoted source, so they may contain a
/// double quote. Emit them quoted with any embedded quote doubled.
fn quote(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(source: &str) -> DdlPlan {
        lower(&parse_ddl(source).expect("should parse")).expect("should lower")
    }

    /// Inference is the whole ergonomic point: a declaration with no physical
    /// names must still produce a complete, registerable graph.
    #[test]
    fn infers_physical_names() {
        let plan = plan(
            "CREATE GRAPH social \
             NODE Person (name TEXT) \
             RELATION KNOWS (since INTEGER) \
               ROLE start -> Person \
               ROLE end -> Person",
        );

        assert_eq!(
            plan.create_tables[0],
            "CREATE TABLE IF NOT EXISTS \"Person\" (\"id\" INTEGER PRIMARY KEY, \"name\" TEXT)"
        );
        assert_eq!(
            plan.create_tables[1],
            "CREATE TABLE IF NOT EXISTS \"KNOWS\" (\"id\" INTEGER PRIMARY KEY, \
             \"start\" INTEGER, \"end\" INTEGER, \"since\" INTEGER)"
        );

        let node = &plan.registration.node_sources[0];
        assert_eq!(node.table, "Person");
        assert_eq!(node.identity_column, "id");
        let relation = &plan.registration.relationship_sources[0];
        assert_eq!(relation.roles[0].column, "start");
        assert_eq!(relation.roles[0].cardinality, RoleCardinality::One);
    }

    /// Overrides exist so an existing schema can be adopted. If lowering
    /// preferred the declared name over a written `AS TABLE`, adoption would
    /// create a second, empty table instead.
    #[test]
    fn overrides_win_over_inference() {
        let plan = plan(
            "CREATE GRAPH social \
             NODE Person AS TABLE people KEY pid (name TEXT) \
             RELATION KNOWS AS TABLE knows KEY kid \
               ROLE start -> Person VIA src \
               ROLE end -> Person VIA dst",
        );

        let node = &plan.registration.node_sources[0];
        assert_eq!(node.name, "Person");
        assert_eq!(node.table, "people");
        assert_eq!(node.identity_column, "pid");

        let relation = &plan.registration.relationship_sources[0];
        assert_eq!(relation.table, "knows");
        assert_eq!(relation.identity_column, "kid");
        assert_eq!(relation.roles[0].column, "src");
        assert_eq!(relation.roles[1].column, "dst");
        assert!(plan.create_tables[1].contains("\"src\" INTEGER"));
    }

    /// A `MANY` role must register with an empty column and contribute no
    /// column to the relation table; registration reads `column.is_empty()`
    /// nowhere, but a stray column would be dead schema the user never asked
    /// for.
    #[test]
    fn many_roles_take_no_relation_column() {
        let plan = plan(
            "CREATE GRAPH scriptorium \
             NODE Text (title TEXT) \
             RELATION Citation \
               ROLE cited -> Text \
               ROLE witnesses -> Text MANY",
        );

        let relation = &plan.registration.relationship_sources[0];
        assert_eq!(relation.roles[1].cardinality, RoleCardinality::Many);
        assert_eq!(relation.roles[1].column, "");
        assert!(!plan.create_tables[1].contains("witnesses"));
    }

    /// `VIA` on a `MANY` role is ignored by registration. Accepting it would
    /// let a user believe they had chosen a column name that has no effect.
    #[test]
    fn rejects_via_on_many_role() {
        let ddl = parse_ddl(
            "CREATE GRAPH g NODE Text (title TEXT) \
             RELATION Citation ROLE witnesses -> Text VIA w MANY",
        )
        .expect("should parse");
        let error = lower(&ddl).expect_err("VIA on MANY must be refused");
        assert!(
            matches!(&error, DdlError::ManyRoleWithVia { role, column, .. }
                if role == "witnesses" && column == "w"),
            "unexpected error: {error}"
        );
    }

    /// A backtick-quoted identifier may contain a double quote; emitting it
    /// unescaped would produce invalid SQL or, worse, change the statement.
    #[test]
    fn quotes_embedded_double_quotes() {
        assert_eq!(quote("plain"), "\"plain\"");
        assert_eq!(quote("we\"ird"), "\"we\"\"ird\"");
    }
}
