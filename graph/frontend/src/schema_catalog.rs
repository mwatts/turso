use std::sync::Arc;

use turso_core::{
    schema::{Column, Schema, Table},
    Connection,
};
use turso_graph_ir as ir;

use crate::binder::{CatalogEntity, GraphCatalogSnapshot, ResolvedProperty};
use crate::catalog::{RegisteredGraph, RegisteredNodeSource, RegisteredRelationshipSource};
use crate::lowering::{NodeTableLayout, RelationalCatalogSnapshot, RelationshipTableLayout};

/// Production catalog snapshot backed directly by `core::Schema` — no PRAGMA
/// string-parsing, no parallel type model. Column classification reuses
/// `Schema::classify_column` (`core/schema.rs`), the same function
/// `Statement::get_column_type_info` uses for SQL result columns.
pub struct SchemaCatalog {
    connection: Arc<Connection>,
    graph: RegisteredGraph,
}

impl SchemaCatalog {
    pub fn new(connection: Arc<Connection>, graph: RegisteredGraph) -> Self {
        Self { connection, graph }
    }

    fn node_source_entry(&self) -> Option<&RegisteredNodeSource> {
        self.graph.node_sources.first()
    }

    fn relationship_source_entry(&self) -> Option<&RegisteredRelationshipSource> {
        self.graph.relationship_sources.first()
    }

    fn table_for(&self, entity: CatalogEntity) -> Option<Arc<Table>> {
        let table_name = match entity {
            CatalogEntity::Node => &self.node_source_entry()?.table,
            CatalogEntity::Relationship => &self.relationship_source_entry()?.table,
        };
        self.connection.current_schema().get_table(table_name)
    }
}

/// Maps a column's declared (or resolved custom-type base) primitive type
/// name to the corresponding graph `ValueType`. `NUMERIC`/`ANY` have no
/// single fixed runtime representation, so they (and any unrecognized name)
/// fall back to `Any` rather than a specific scalar.
///
/// Only exact-matches the four canonical keywords; used exclusively for
/// **array** columns (see `column_value_type`), where `declared_name`/
/// `base_type` are the only fields not overwritten by core's array-affinity
/// override. Non-array columns must go through `sqlite_type_value_type`
/// instead, since their declared spelling can be any legal SQLite type name
/// (`INT`, `VARCHAR(50)`, `DOUBLE`, ...), not just these four keywords.
fn primitive_value_type(primitive: &str) -> ir::ValueType {
    match primitive.to_ascii_uppercase().as_str() {
        "INTEGER" => ir::ValueType::Integer,
        "REAL" => ir::ValueType::Real,
        "TEXT" => ir::ValueType::Text,
        "BLOB" => ir::ValueType::Bytes,
        _ => ir::ValueType::Any,
    }
}

/// Maps a column's resolved SQLite storage class (`Column::affinity().to_type()`)
/// to the corresponding graph `ValueType`. `NUMERIC` has no single fixed
/// runtime representation (it stores whichever of INTEGER/REAL/TEXT best fits
/// the value), so it maps to `Any` rather than a specific scalar.
///
/// Used for all **non-array** columns: `column.affinity()` is accurate there
/// (SQLite's substring-based affinity algorithm for `Builtin`, core's
/// resolved base affinity for `Custom`/`Domain`), unlike for array columns,
/// which core unconditionally forces to `Blob` affinity for record-format
/// packing (see `column_value_type`).
fn sqlite_type_value_type(ty: turso_core::schema::Type) -> ir::ValueType {
    use turso_core::schema::Type;

    match ty {
        Type::Integer => ir::ValueType::Integer,
        Type::Real => ir::ValueType::Real,
        Type::Text => ir::ValueType::Text,
        Type::Numeric | Type::Null => ir::ValueType::Any,
        Type::Blob => ir::ValueType::Bytes,
    }
}

fn wrap_array(mut element: ir::ValueType, dimensions: u32) -> ir::ValueType {
    for _ in 0..dimensions {
        element = ir::ValueType::List(Box::new(element));
    }
    element
}

/// Resolves a named `CREATE TYPE`/`CREATE DOMAIN` to a `ValueType`,
/// recursing into STRUCT fields and UNION variants. Falls back to `Any`
/// only when the type registry has no entry for `type_name` under the
/// caller's strictness mode (mirrors `Schema::classify_column`'s own
/// `None => Builtin` fallback rather than inventing a stricter failure).
fn resolve_named_type(schema: &Schema, type_name: &str, is_strict: bool) -> ir::ValueType {
    let Some(resolved) = schema.resolve_type(type_name, is_strict).ok().flatten() else {
        return ir::ValueType::Any;
    };
    let leaf = resolved.leaf();
    if leaf.is_struct() {
        let fields = leaf
            .struct_def()
            .expect("is_struct implies struct_def")
            .fields
            .iter()
            .map(|field| {
                (
                    field.name.clone(),
                    resolve_named_type(schema, &field.type_name, is_strict),
                )
            })
            .collect();
        ir::ValueType::Struct(fields)
    } else if leaf.is_union() {
        let variants = leaf
            .union_def()
            .expect("is_union implies union_def")
            .variants
            .iter()
            .map(|variant| {
                (
                    variant.tag_name.clone(),
                    resolve_named_type(schema, &variant.type_name, is_strict),
                )
            })
            .collect();
        ir::ValueType::Union(variants)
    } else if leaf.is_domain {
        primitive_value_type(&resolved.primitive)
    } else {
        ir::ValueType::Custom {
            name: type_name.to_owned(),
            base: Box::new(primitive_value_type(&resolved.primitive)),
        }
    }
}

impl SchemaCatalog {
    fn column_value_type(
        &self,
        schema: &Schema,
        column: &Column,
        is_strict: bool,
    ) -> ir::ValueType {
        use turso_core::ColumnTypeKind;

        let info = schema.classify_column(column, is_strict);
        let is_array = column.array_dimensions() > 0;
        let scalar = match info.kind {
            // `column.affinity()` is a physical-storage signal that's only
            // wrong for **array** columns: core's
            // `BTreeTable::resolve_custom_type_affinities` (core/schema.rs)
            // unconditionally forces them to `Blob` affinity for
            // record-format packing, regardless of their declared element
            // type. For those, read the logical type from `classify_column`
            // instead: `base_type` already carries the resolved primitive
            // for a `Domain` column's underlying type (`declared_name` there
            // is the domain's own name, e.g. "posint", not a primitive
            // keyword); for `Builtin`, `base_type` is `None` and
            // `declared_name` *is* the primitive keyword directly — but only
            // when written as one of the four canonical keywords
            // (`primitive_value_type` exact-matches those and falls back to
            // `Any` otherwise). For non-array columns, `column.affinity()`
            // is accurate regardless of declared spelling (`INT`,
            // `VARCHAR(50)`, `DOUBLE`, ...), so use it there instead.
            ColumnTypeKind::Builtin | ColumnTypeKind::Domain => {
                if is_array {
                    primitive_value_type(info.base_type.as_deref().unwrap_or(&info.declared_name))
                } else {
                    sqlite_type_value_type(column.affinity().to_type())
                }
            }
            ColumnTypeKind::Custom => {
                // Same physical-vs-logical distinction as above: `base_type`
                // is the custom type's resolved underlying primitive,
                // unaffected by the array-affinity override, so it's only
                // needed for array columns; non-array `Custom` columns get
                // an accurate resolved base affinity from core already.
                let base = Box::new(if is_array {
                    primitive_value_type(info.base_type.as_deref().unwrap_or(&info.declared_name))
                } else {
                    sqlite_type_value_type(column.affinity().to_type())
                });
                ir::ValueType::Custom {
                    name: info.declared_name,
                    base,
                }
            }
            ColumnTypeKind::Struct | ColumnTypeKind::Union => {
                resolve_named_type(schema, &info.declared_name, is_strict)
            }
            // `ColumnTypeKind` is `#[non_exhaustive]`; fall back to `Any` for any
            // future variant rather than failing to compile on a core upgrade.
            _ => ir::ValueType::Any,
        };
        wrap_array(scalar, column.array_dimensions())
    }
}

impl GraphCatalogSnapshot for SchemaCatalog {
    fn node_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId> {
        (graph == self.graph.id)
            .then(|| self.node_source_entry())
            .flatten()
            .map(|source| source.id)
    }

    fn relationship_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId> {
        (graph == self.graph.id)
            .then(|| self.relationship_source_entry())
            .flatten()
            .map(|source| source.id)
    }

    fn label(&self, graph: ir::GraphId, name: &str) -> Option<ir::LabelId> {
        if graph != self.graph.id {
            return None;
        }
        let index = self
            .graph
            .node_sources
            .iter()
            .position(|source| source.name == name)?;
        ir::LabelId::new((index as u32) + 1).ok()
    }

    fn relationship_type(&self, graph: ir::GraphId, name: &str) -> Option<ir::RelationshipTypeId> {
        if graph != self.graph.id {
            return None;
        }
        let index = self
            .graph
            .relationship_sources
            .iter()
            .position(|source| source.name == name)?;
        ir::RelationshipTypeId::new((index as u32) + 1).ok()
    }

    fn property(
        &self,
        graph: ir::GraphId,
        entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        if graph != self.graph.id {
            return None;
        }
        let table = self.table_for(entity)?;
        let (index, column) = table.get_column_by_name(name)?;
        let schema = self.connection.current_schema();
        let value_type = self.column_value_type(&schema, column, table.is_strict());
        // `explicit_notnull()` alone misses the INTEGER PRIMARY KEY rowid
        // alias: SQLite never lets it hold NULL (an inserted NULL there gets
        // replaced by a fresh rowid), even though no NOT NULL constraint was
        // written. `is_rowid_alias()` is exactly `Column::new`'s check for
        // that case (single-column, ascending, INTEGER-typed primary key).
        let nullability = if column.explicit_notnull() || column.is_rowid_alias() {
            ir::Nullability::NonNull
        } else {
            ir::Nullability::Nullable
        };
        Some(ResolvedProperty {
            id: ir::PropertyId::new((index as u32) + 1).ok()?,
            value_type,
            nullability,
        })
    }
}

impl RelationalCatalogSnapshot for SchemaCatalog {
    fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout> {
        let entry = self
            .node_source_entry()
            .filter(|entry| entry.id == source)?;
        Some(NodeTableLayout {
            table: entry.table.clone(),
            identity_column: entry.identity_column.clone(),
        })
    }

    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout> {
        let entry = self
            .relationship_source_entry()
            .filter(|entry| entry.id == source)?;
        Some(RelationshipTableLayout {
            table: entry.table.clone(),
            identity_column: entry.identity_column.clone(),
            start_column: entry.start_column.clone(),
            end_column: entry.end_column.clone(),
        })
    }

    fn property_column(
        &self,
        source: ir::SourceTableId,
        property: ir::PropertyId,
    ) -> Option<String> {
        let table_name = if self
            .node_source_entry()
            .is_some_and(|entry| entry.id == source)
        {
            &self.node_source_entry()?.table
        } else if self
            .relationship_source_entry()
            .is_some_and(|entry| entry.id == source)
        {
            &self.relationship_source_entry()?.table
        } else {
            return None;
        };
        let table = self.connection.current_schema().get_table(table_name)?;
        let index = (property.get() as usize).checked_sub(1)?;
        table.get_column_at(index)?.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{
        GraphRegistration, NodeSourceRegistration, RelationshipSourceRegistration,
    };
    use std::sync::Arc;
    use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect};

    fn connect(strict_custom_types: bool) -> Arc<Connection> {
        let io = Arc::new(MemoryIO::new());
        Database::open_file_with_flags(
            io,
            ":memory:schema-catalog",
            OpenFlags::default(),
            DatabaseOpts::new().with_custom_types(strict_custom_types),
            None,
            Arc::new(SqliteDialect),
        )
        .expect("open database")
        .connect()
        .expect("connect")
    }

    fn registered_social_graph(connection: &Arc<Connection>) -> RegisteredGraph {
        connection
            .execute(
                "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
                 CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
            )
            .expect("create sources");
        crate::catalog::register_graph(
            connection,
            &GraphRegistration {
                name: "social".to_owned(),
                node_sources: vec![NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "KNOWS".to_owned(),
                    table: "relationships".to_owned(),
                    identity_column: "id".to_owned(),
                    start_column: "src".to_owned(),
                    end_column: "dst".to_owned(),
                    start_node_source: "Person".to_owned(),
                    end_node_source: "Person".to_owned(),
                }],
            },
        )
        .expect("register graph")
    }

    #[test]
    fn resolves_id_name_age_matching_testkit_stub_identities() {
        let connection = connect(false);
        let graph = registered_social_graph(&connection);
        let graph_id = graph.id;
        let catalog = SchemaCatalog::new(connection, graph);

        let id = catalog
            .property(graph_id, CatalogEntity::Node, "id")
            .expect("id resolves");
        assert_eq!(id.id, ir::PropertyId::new(1).unwrap());
        assert_eq!(id.value_type, ir::ValueType::Integer);
        assert_eq!(id.nullability, ir::Nullability::NonNull);

        let name = catalog
            .property(graph_id, CatalogEntity::Node, "name")
            .expect("name resolves");
        assert_eq!(name.id, ir::PropertyId::new(2).unwrap());
        assert_eq!(name.value_type, ir::ValueType::Text);
        assert_eq!(name.nullability, ir::Nullability::Nullable);
    }

    /// Regression test for a bug introduced alongside the array-element-type
    /// fix in `column_value_type`: that fix routed *every* `Builtin` column
    /// (not just arrays) through `primitive_value_type`, which only
    /// exact-matches the four keywords `INTEGER`/`REAL`/`TEXT`/`BLOB` and
    /// falls back to `Any` for anything else. Non-array `Builtin` columns
    /// declared with any other legal SQLite spelling — `INT`, `VARCHAR(n)`,
    /// `CHAR(n)`, `DOUBLE`, ... — silently mistyped as `Any` instead of
    /// their real type. `column.affinity()` (SQLite's substring-based
    /// affinity algorithm) handles these spellings correctly and must be
    /// used for non-array columns.
    #[test]
    fn non_canonical_builtin_spelling_resolves_correct_scalar_type_not_any() {
        let connection = connect(false);
        connection
            .execute("CREATE TABLE things(id INTEGER PRIMARY KEY, age INT, label VARCHAR(20));")
            .expect("create source");
        let graph = crate::catalog::register_graph(
            &connection,
            &GraphRegistration {
                name: "typesys".to_owned(),
                node_sources: vec![NodeSourceRegistration {
                    name: "Thing".to_owned(),
                    table: "things".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![],
            },
        )
        .expect("register graph");
        let graph_id = graph.id;
        let catalog = SchemaCatalog::new(connection, graph);

        let age = catalog
            .property(graph_id, CatalogEntity::Node, "age")
            .expect("age resolves");
        assert_eq!(
            age.value_type,
            ir::ValueType::Integer,
            "INT is a non-canonical spelling of INTEGER and must not fall back to Any"
        );

        let label = catalog
            .property(graph_id, CatalogEntity::Node, "label")
            .expect("label resolves");
        assert_eq!(
            label.value_type,
            ir::ValueType::Text,
            "VARCHAR(20) must resolve via SQLite affinity (Text) and not fall back to Any"
        );
    }
}
