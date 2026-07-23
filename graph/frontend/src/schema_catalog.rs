use std::sync::Arc;

use turso_core::{
    schema::{Column, Schema, Table},
    Connection,
};
use turso_graph_ir as ir;

use crate::binder::{CatalogEntity, GraphCatalogSnapshot, PropertyResolution, ResolvedProperty};
use crate::catalog::{RegisteredGraph, RegisteredNodeSource, RegisteredRelationshipSource};
use crate::lowering::{NodeTableLayout, RelationalCatalogSnapshot, RelationshipTableLayout};
use crate::semantic::{OwnedProperty, SemanticSnapshot, SemanticTypeInfo};

/// Production catalog snapshot backed directly by `core::Schema` — no PRAGMA
/// string-parsing, no parallel type model. Column classification reuses
/// `Schema::classify_column` (`core/schema.rs`), the same function
/// `Statement::get_column_type_info` uses for SQL result columns.
pub struct SchemaCatalog {
    connection: Arc<Connection>,
    graph: RegisteredGraph,
    semantic: Option<Arc<SemanticSnapshot>>,
}

impl SchemaCatalog {
    pub fn new(connection: Arc<Connection>, graph: RegisteredGraph) -> Self {
        Self {
            connection,
            graph,
            semantic: None,
        }
    }

    pub fn with_semantic(
        connection: Arc<Connection>,
        graph: RegisteredGraph,
        semantic: Option<Arc<SemanticSnapshot>>,
    ) -> Self {
        Self {
            connection,
            graph,
            semantic,
        }
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

    fn semantic_types_for<'a>(
        &'a self,
        entity: CatalogEntity,
        type_names: &[String],
    ) -> Option<Vec<(String, Option<&'a OwnedProperty>)>> {
        let semantic = self.semantic.as_ref()?;
        let type_by_name = |name: &str| -> Option<&SemanticTypeInfo> {
            match entity {
                CatalogEntity::Node => semantic.node_type(name),
                CatalogEntity::Relationship => semantic.relationship_type(name),
            }
        };
        if type_names.is_empty() {
            let types: Box<dyn Iterator<Item = &SemanticTypeInfo>> = match entity {
                CatalogEntity::Node => Box::new(semantic.node_type_values()),
                CatalogEntity::Relationship => Box::new(semantic.relationship_type_values()),
            };
            return Some(
                types
                    .map(|type_info| (type_info.name.clone(), None))
                    .collect(),
            );
        }
        Some(
            type_names
                .iter()
                .map(|name| {
                    (
                        type_by_name(name)
                            .map(|type_info| type_info.name.clone())
                            .unwrap_or_else(|| name.clone()),
                        None,
                    )
                })
                .collect(),
        )
    }

    fn semantic_property_resolution(
        &self,
        entity: CatalogEntity,
        type_names: &[String],
        name: &str,
    ) -> Option<PropertyResolution> {
        let semantic = self.semantic.as_ref()?;
        let mut candidates = self.semantic_types_for(entity, type_names)?;
        for (type_name, property) in &mut candidates {
            let type_info = match entity {
                CatalogEntity::Node => semantic.node_type(type_name),
                CatalogEntity::Relationship => semantic.relationship_type(type_name),
            };
            *property = type_info.and_then(|type_info| type_info.property(name));
        }
        let mut owners = candidates
            .iter()
            .filter_map(|(type_name, property)| property.map(|_| type_name.clone()))
            .collect::<Vec<_>>();
        let mut non_owners = candidates
            .iter()
            .filter_map(|(type_name, property)| property.is_none().then_some(type_name.clone()))
            .collect::<Vec<_>>();
        owners.sort_by_key(|name| name.to_lowercase());
        non_owners.sort_by_key(|name| name.to_lowercase());
        if owners.is_empty() {
            return Some(PropertyResolution::NotOwned { types: non_owners });
        }
        if !non_owners.is_empty() {
            return Some(PropertyResolution::Ambiguous { owners, non_owners });
        }
        let mut properties = candidates
            .iter()
            .filter_map(|(_, property)| property.as_ref().copied());
        let first = properties.next()?;
        let mut resolved = ResolvedProperty {
            id: first.id,
            value_type: first.value_type.clone(),
            nullability: first.nullability,
        };
        for property in properties {
            if property.id != resolved.id {
                return None;
            }
            if property.value_type != resolved.value_type {
                resolved.value_type = ir::ValueType::Any;
            }
            if property.nullability == ir::Nullability::Nullable {
                resolved.nullability = ir::Nullability::Nullable;
            }
        }
        Some(PropertyResolution::Resolved(resolved))
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
/// recursing into STRUCT fields and UNION variants. Falls back to
/// `fallback_affinity` (the field/variant's precomputed SQLite affinity,
/// when available — mirrors `column_value_type`'s use of `column.affinity()`
/// for ordinary columns) when the type registry has no entry for
/// `type_name`, e.g. a STRUCT field declared with a bare primitive keyword
/// like `INTEGER` rather than a registered custom type/domain name. Falls
/// back further to `Any` only when no affinity fallback is available either
/// (the top-level call site, resolving a column's own STRUCT/UNION type
/// name, has none — that name is always registered by construction, since
/// `classify_column` already determined the column's kind).
fn resolve_named_type(
    schema: &Schema,
    type_name: &str,
    fallback_affinity: Option<turso_core::schema::Type>,
    is_strict: bool,
) -> ir::ValueType {
    let Some(resolved) = schema.resolve_type(type_name, is_strict).ok().flatten() else {
        return fallback_affinity
            .map(sqlite_type_value_type)
            .unwrap_or(ir::ValueType::Any);
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
                    resolve_named_type(
                        schema,
                        &field.type_name,
                        Some(field.base_affinity.to_type()),
                        is_strict,
                    ),
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
                    resolve_named_type(
                        schema,
                        &variant.type_name,
                        Some(variant.base_affinity.to_type()),
                        is_strict,
                    ),
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
        column_value_type(schema, column, is_strict)
    }
}

pub(crate) fn column_value_type(
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
        //
        // A `Builtin` column with an *empty* declared type (e.g. `ALTER
        // TABLE t ADD COLUMN prop` with no type name, as dynamic
        // property loaders emit) is SQLite's "no affinity" case: unlike
        // an explicitly declared `BLOB` column, it applies no storage
        // coercion at all, so each row keeps whatever type was inserted.
        // Typing it `Bytes` (via `Blob` affinity) would reject
        // arithmetic and IN-membership on columns that are, in practice,
        // numeric or list-valued in every row. `Any` reflects that the
        // declared schema simply makes no promise here. A genuinely
        // declared `BLOB` column keeps its non-empty `declared_name` and
        // still resolves to `Bytes` below.
        ColumnTypeKind::Builtin | ColumnTypeKind::Domain => {
            if is_array {
                primitive_value_type(info.base_type.as_deref().unwrap_or(&info.declared_name))
            } else if info.declared_name.is_empty() {
                ir::ValueType::Any
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
            resolve_named_type(schema, &info.declared_name, None, is_strict)
        }
        // `ColumnTypeKind` is `#[non_exhaustive]`; fall back to `Any` for any
        // future variant rather than failing to compile on a core upgrade.
        _ => ir::ValueType::Any,
    };
    wrap_array(scalar, column.array_dimensions())
}

pub(crate) fn column_nullability(column: &Column) -> ir::Nullability {
    if column.explicit_notnull() || column.is_rowid_alias() {
        ir::Nullability::NonNull
    } else {
        ir::Nullability::Nullable
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
        if let Some(semantic) = &self.semantic {
            return semantic
                .node_type(name)
                .and_then(|type_info| ir::LabelId::new(type_info.type_id).ok());
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
        if let Some(semantic) = &self.semantic {
            return semantic
                .relationship_type(name)
                .and_then(|type_info| ir::RelationshipTypeId::new(type_info.type_id).ok());
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
        if self.semantic.is_some() {
            return match self.semantic_property_resolution(entity, &[], name)? {
                PropertyResolution::Resolved(property) => Some(property),
                PropertyResolution::NotOwned { .. } | PropertyResolution::Ambiguous { .. } => None,
            };
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
        let nullability = column_nullability(column);
        Some(ResolvedProperty {
            id: ir::PropertyId::new((index as u32) + 1).ok()?,
            value_type,
            nullability,
        })
    }

    fn semantic_mode(&self, graph: ir::GraphId) -> bool {
        graph == self.graph.id && self.semantic.is_some()
    }

    fn node_source_for_label(
        &self,
        graph: ir::GraphId,
        label: ir::LabelId,
    ) -> Option<ir::SourceTableId> {
        if graph != self.graph.id {
            return None;
        }
        self.semantic
            .as_ref()
            .and_then(|semantic| semantic.node_type_by_id(label))
            .map(|type_info| type_info.source)
            .or_else(|| self.node_source(graph))
    }

    fn relationship_source_for_type(
        &self,
        graph: ir::GraphId,
        relationship_type: ir::RelationshipTypeId,
    ) -> Option<ir::SourceTableId> {
        if graph != self.graph.id {
            return None;
        }
        self.semantic
            .as_ref()
            .and_then(|semantic| semantic.relationship_type_by_id(relationship_type))
            .map(|type_info| type_info.source)
            .or_else(|| self.relationship_source(graph))
    }

    fn resolve_owned_property(
        &self,
        graph: ir::GraphId,
        entity: CatalogEntity,
        type_names: &[String],
        name: &str,
    ) -> Option<PropertyResolution> {
        if graph != self.graph.id {
            return None;
        }
        self.semantic_property_resolution(entity, type_names, name)
            .or_else(|| {
                self.property(graph, entity, name)
                    .map(PropertyResolution::Resolved)
            })
    }

    fn relationship_endpoints(
        &self,
        graph: ir::GraphId,
        relationship_type: ir::RelationshipTypeId,
    ) -> Option<(Vec<ir::LabelId>, Vec<ir::LabelId>)> {
        if graph != self.graph.id {
            return None;
        }
        let constraints = self.semantic.as_ref()?.endpoints(relationship_type)?;
        let start = constraints
            .start
            .iter()
            .map(|id| ir::LabelId::new(*id).ok())
            .collect::<Option<Vec<_>>>()?;
        let end = constraints
            .end
            .iter()
            .map(|id| ir::LabelId::new(*id).ok())
            .collect::<Option<Vec<_>>>()?;
        Some((start, end))
    }
}

impl RelationalCatalogSnapshot for SchemaCatalog {
    fn labels_table(&self) -> Option<String> {
        Some(crate::catalog::labels_table_name(self.graph.id))
    }

    fn label_name(&self, label: ir::LabelId) -> Option<String> {
        if let Some(semantic) = &self.semantic {
            return semantic
                .node_type_by_id(label)
                .map(|type_info| type_info.name.clone());
        }
        self.graph
            .node_sources
            .get((label.get() as usize).checked_sub(1)?)
            .map(|source| source.name.clone())
    }

    fn relationship_types_table(&self) -> Option<String> {
        Some(crate::catalog::relationship_types_table_name(self.graph.id))
    }

    fn relationship_type_name(&self, relationship_type: ir::RelationshipTypeId) -> Option<String> {
        if let Some(semantic) = &self.semantic {
            return semantic
                .relationship_type_by_id(relationship_type)
                .map(|type_info| type_info.name.clone());
        }
        if let Some(source) = self
            .graph
            .relationship_sources
            .get((relationship_type.get() as usize).checked_sub(1)?)
        {
            return Some(source.name.clone());
        }
        let rows = self
            .connection
            .prepare(format!(
                "SELECT name FROM \"{}\" WHERE id = {}",
                crate::catalog::relationship_type_registry_table_name(self.graph.id),
                relationship_type.get()
            ))
            .and_then(|mut statement| statement.run_collect_rows())
            .ok()?;
        match rows.first().and_then(|row| row.first()) {
            Some(turso_core::Value::Text(name)) => Some(name.to_string()),
            _ => None,
        }
    }

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
        if let Some(semantic) = &self.semantic {
            let mut columns = semantic
                .node_type_values()
                .chain(semantic.relationship_type_values())
                .filter(|type_info| type_info.source == source)
                .filter_map(|type_info| type_info.property_by_id(property))
                .map(|property| property.column.as_str());
            let first = columns.next()?.to_owned();
            if columns.all(|column| column.eq_ignore_ascii_case(&first)) {
                return Some(first);
            }
            return None;
        }
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

    fn property_column_is_jsonb(
        &self,
        source: ir::SourceTableId,
        property: ir::PropertyId,
    ) -> bool {
        let table_name = if self
            .node_source_entry()
            .is_some_and(|entry| entry.id == source)
        {
            self.node_source_entry().map(|entry| entry.table.clone())
        } else if self
            .relationship_source_entry()
            .is_some_and(|entry| entry.id == source)
        {
            self.relationship_source_entry()
                .map(|entry| entry.table.clone())
        } else {
            None
        };
        let Some(table_name) = table_name else {
            return false;
        };
        let Some(table) = self.connection.current_schema().get_table(&table_name) else {
            return false;
        };
        let Some(index) = (property.get() as usize).checked_sub(1) else {
            return false;
        };
        table
            .get_column_at(index)
            .is_some_and(|column| column.ty_str.eq_ignore_ascii_case("JSONB"))
    }

    fn payload_columns(&self, source: ir::SourceTableId) -> Option<Vec<(String, String)>> {
        let (table_name, structural) =
            if let Some(entry) = self.node_source_entry().filter(|entry| entry.id == source) {
                (entry.table.clone(), vec![entry.identity_column.clone()])
            } else if let Some(entry) = self
                .relationship_source_entry()
                .filter(|entry| entry.id == source)
            {
                (
                    entry.table.clone(),
                    vec![
                        entry.identity_column.clone(),
                        entry.start_column.clone(),
                        entry.end_column.clone(),
                    ],
                )
            } else {
                return None;
            };
        let table = self.connection.current_schema().get_table(&table_name)?;
        let mut columns = Vec::new();
        for index in 0.. {
            let Some(column) = table.get_column_at(index) else {
                break;
            };
            let Some(name) = column.name.clone() else {
                continue;
            };
            if structural.contains(&name) {
                continue;
            }
            // Reserved-name properties live in prefixed payload columns.
            let logical = name
                .strip_prefix("cyprop_")
                .map(str::to_owned)
                .unwrap_or_else(|| name.clone());
            columns.push((logical, name));
        }
        Some(columns)
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

    /// Regression test: a column added via `ALTER TABLE ... ADD COLUMN name`
    /// with no type name at all (as dynamic property loaders — e.g. the
    /// CypherBench fixture loader — emit for benchmark properties) must
    /// resolve to `Any`, not `Bytes`. SQLite gives such a column `Blob`
    /// ("no affinity") storage class, meaning every row keeps whichever type
    /// was inserted; typing it `Bytes` made the binder reject arithmetic and
    /// `IN` on properties that are numeric/list-valued in every row.
    #[test]
    fn untyped_alter_table_column_resolves_any_not_bytes() {
        let connection = connect(false);
        connection
            .execute(
                "CREATE TABLE things(id INTEGER PRIMARY KEY); \
                 ALTER TABLE things ADD COLUMN weight;",
            )
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

        let weight = catalog
            .property(graph_id, CatalogEntity::Node, "weight")
            .expect("weight resolves");
        assert_eq!(
            weight.value_type,
            ir::ValueType::Any,
            "a column with no declared type at all must resolve to Any, not Bytes"
        );
    }

    /// A column explicitly declared `BLOB` is a real fixed-affinity
    /// declaration (not "no type at all") and must keep resolving to
    /// `Bytes`, distinguishing it from the untyped-column case above.
    #[test]
    fn explicitly_declared_blob_column_still_resolves_bytes() {
        let connection = connect(false);
        connection
            .execute("CREATE TABLE things(id INTEGER PRIMARY KEY, payload BLOB);")
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

        let payload = catalog
            .property(graph_id, CatalogEntity::Node, "payload")
            .expect("payload resolves");
        assert_eq!(
            payload.value_type,
            ir::ValueType::Bytes,
            "an explicitly declared BLOB column must still resolve to Bytes"
        );
    }

    /// Regression test: a STRUCT field declared with a bare SQLite primitive
    /// keyword (`x INTEGER`, not a registered `CREATE TYPE`/`CREATE DOMAIN`
    /// name) must resolve to that primitive's `ValueType`, not `Any`.
    /// `resolve_named_type` previously looked `field.type_name` up only in
    /// the custom-type registry via `Schema::resolve_type`, which has no
    /// entry for a bare primitive keyword, and fell straight through to
    /// `ir::ValueType::Any` on that miss — silently breaking every
    /// bare-primitive STRUCT field.
    #[test]
    fn struct_field_with_bare_primitive_type_resolves_scalar_not_any() {
        let connection = connect(true);
        connection
            .execute(
                "CREATE TYPE point AS STRUCT(x INTEGER, y INTEGER); \
                 CREATE TABLE shapes(id INTEGER PRIMARY KEY, origin point) STRICT;",
            )
            .expect("create struct-typed source");
        let graph = crate::catalog::register_graph(
            &connection,
            &GraphRegistration {
                name: "typesys".to_owned(),
                node_sources: vec![NodeSourceRegistration {
                    name: "Shape".to_owned(),
                    table: "shapes".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![],
            },
        )
        .expect("register graph");
        let graph_id = graph.id;
        let catalog = SchemaCatalog::new(connection, graph);

        let origin = catalog
            .property(graph_id, CatalogEntity::Node, "origin")
            .expect("origin resolves");
        assert_eq!(
            origin.value_type,
            ir::ValueType::Struct(vec![
                ("x".to_owned(), ir::ValueType::Integer),
                ("y".to_owned(), ir::ValueType::Integer),
            ]),
            "bare-primitive STRUCT fields must resolve to their scalar type, not Any"
        );
    }

    /// UNION analog of `struct_field_with_bare_primitive_type_resolves_scalar_not_any`:
    /// a UNION variant declared with a bare SQLite primitive keyword must
    /// resolve to that primitive's `ValueType`, not `Any`.
    #[test]
    fn union_variant_with_bare_primitive_type_resolves_scalar_not_any() {
        let connection = connect(true);
        connection
            .execute(
                "CREATE TYPE contact AS UNION(email TEXT, phone TEXT); \
                 CREATE TABLE people(id INTEGER PRIMARY KEY, reach contact) STRICT;",
            )
            .expect("create union-typed source");
        let graph = crate::catalog::register_graph(
            &connection,
            &GraphRegistration {
                name: "typesys".to_owned(),
                node_sources: vec![NodeSourceRegistration {
                    name: "Person".to_owned(),
                    table: "people".to_owned(),
                    identity_column: "id".to_owned(),
                }],
                relationship_sources: vec![],
            },
        )
        .expect("register graph");
        let graph_id = graph.id;
        let catalog = SchemaCatalog::new(connection, graph);

        let reach = catalog
            .property(graph_id, CatalogEntity::Node, "reach")
            .expect("reach resolves");
        assert_eq!(
            reach.value_type,
            ir::ValueType::Union(vec![
                ("email".to_owned(), ir::ValueType::Text),
                ("phone".to_owned(), ir::ValueType::Text),
            ]),
            "bare-primitive UNION variants must resolve to their scalar type, not Any"
        );
    }
}
