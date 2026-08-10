//! Physical access methods for Cypher properties on relational sources.
//!
//! Spike: bag and cell modes sit behind the same binder property ids as ordinary
//! columns. Cypher still writes `n.name` / `SET n.name = …`. Layout choice is
//! catalog-internal (`RelationalCatalogSnapshot::property_physical`).
//!
//! ## Open property store (Cell / EAV)
//!
//! Under open queries on arbitrary property names, properties are not SQL
//! columns. They live in a side table keyed by an **integer property id** from
//! a dictionary (not by repeating property-name strings on every cell row):
//!
//! ```text
//! prop_dict(prop_id PK, name UNIQUE, value_type)
//! node_props(node_id, prop_id, value, PRIMARY KEY(node_id, prop_id))
//! INDEX node_props_by_kv(prop_id, value)
//! ```
//!
//! - `prop_id` is stable (reuse IR [`ir::PropertyId`] or a persisted dictionary id).
//! - `value` is one SQLite column with dynamic affinity; **declared type** for
//!   each `prop_id` comes from `prop_dict.value_type` so lowering can reject
//!   `CONTAINS` on integers, choose casts, and keep `(prop_id, value)` seeks
//!   type-homogeneous per prop_id.
//!
//! See `docs/superpowers/plans/2026-08-10-graph-json-bag-property-store-spike.md`
//! and `graph/test-results/property_store_cell_dict.md`.

use turso_graph_ir as ir;

/// One row in the property-name dictionary (name → integer id + type).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyDictEntry {
    pub prop_id: u64,
    pub name: String,
    /// Declared Cypher value type for this property (drives predicates).
    pub value_type: ir::ValueType,
}

/// In-memory property dictionary: maps names to integer ids and types.
///
/// Product catalogs persist the same shape in `graph_prop_dict` (see
/// [`PROP_DICT_DDL`]). Bind uses `name` → `prop_id` + `value_type`; cell rows
/// store only `prop_id`.
#[derive(Clone, Debug, Default)]
pub struct PropertyDictionary {
    by_name: std::collections::BTreeMap<String, PropertyDictEntry>,
    by_id: std::collections::BTreeMap<u64, PropertyDictEntry>,
    next_id: u64,
}

impl PropertyDictionary {
    pub fn new() -> Self {
        Self {
            by_name: std::collections::BTreeMap::new(),
            by_id: std::collections::BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Register a named property with a declared type. Names are matched
    /// case-insensitively; the first spelling is kept.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        value_type: ir::ValueType,
    ) -> Result<PropertyDictEntry, PropertyDictError> {
        let name = name.into();
        if name.is_empty() {
            return Err(PropertyDictError::EmptyName);
        }
        let key = name.to_ascii_lowercase();
        if let Some(existing) = self.by_name.get(&key) {
            if existing.value_type != value_type
                && existing.value_type != ir::ValueType::Any
                && value_type != ir::ValueType::Any
            {
                return Err(PropertyDictError::TypeConflict {
                    name: existing.name.clone(),
                    existing: dict_value_type_name(&existing.value_type).to_owned(),
                    requested: dict_value_type_name(&value_type).to_owned(),
                });
            }
            return Ok(existing.clone());
        }
        let prop_id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let entry = PropertyDictEntry {
            prop_id,
            name,
            value_type,
        };
        self.by_name.insert(key, entry.clone());
        self.by_id.insert(prop_id, entry.clone());
        Ok(entry)
    }

    /// Register with an explicit id (load from durable `prop_dict` rows).
    pub fn register_with_id(
        &mut self,
        prop_id: u64,
        name: impl Into<String>,
        value_type: ir::ValueType,
    ) -> Result<PropertyDictEntry, PropertyDictError> {
        let name = name.into();
        if name.is_empty() {
            return Err(PropertyDictError::EmptyName);
        }
        if prop_id == 0 {
            return Err(PropertyDictError::InvalidId(prop_id));
        }
        let key = name.to_ascii_lowercase();
        if self.by_name.contains_key(&key) || self.by_id.contains_key(&prop_id) {
            return Err(PropertyDictError::Duplicate { prop_id, name });
        }
        let entry = PropertyDictEntry {
            prop_id,
            name,
            value_type,
        };
        self.by_name.insert(key, entry.clone());
        self.by_id.insert(prop_id, entry.clone());
        self.next_id = self.next_id.max(prop_id.saturating_add(1));
        Ok(entry)
    }

    pub fn get(&self, name: &str) -> Option<&PropertyDictEntry> {
        self.by_name.get(&name.to_ascii_lowercase())
    }

    pub fn get_by_id(&self, prop_id: u64) -> Option<&PropertyDictEntry> {
        self.by_id.get(&prop_id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn entries(&self) -> impl Iterator<Item = &PropertyDictEntry> {
        self.by_id.values()
    }

    /// Build a Cell physical descriptor for a dictionary entry.
    pub fn cell_physical(
        &self,
        entry: &PropertyDictEntry,
        props_table: &str,
        entity_column: &str,
        identity_column: &str,
    ) -> PropertyPhysical {
        PropertyPhysical::Cell {
            source_id: 1,
            props_table: props_table.to_owned(),
            entity_column: entity_column.to_owned(),
            identity_column: identity_column.to_owned(),
            prop_id: entry.prop_id,
            prop_id_column: "prop_id".to_owned(),
            value_column: "value".to_owned(),
            value_type: entry.value_type.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum PropertyDictError {
    #[error("property name must not be empty")]
    EmptyName,
    #[error("property id {0} is reserved or invalid")]
    InvalidId(u64),
    #[error(
        "property `{name}` already registered with type {existing}, cannot rebind as {requested}"
    )]
    TypeConflict {
        name: String,
        existing: String,
        requested: String,
    },
    #[error("duplicate property id {prop_id} or name `{name}`")]
    Duplicate { prop_id: u64, name: String },
}

/// Suggested dictionary DDL (product registration installs an equivalent).
pub const PROP_DICT_DDL: &str = "\
CREATE TABLE IF NOT EXISTS graph_prop_dict(\
  prop_id INTEGER PRIMARY KEY, \
  name TEXT NOT NULL COLLATE NOCASE UNIQUE, \
  value_type TEXT NOT NULL\
)";

/// Suggested cell table DDL with integer prop_id (not prop_key TEXT).
pub const NODE_PROPS_CELL_DDL: &str = "\
CREATE TABLE IF NOT EXISTS graph_node_props(\
  source_id INTEGER NOT NULL, \
  node_id INTEGER NOT NULL, \
  prop_id INTEGER NOT NULL, \
  value, \
  PRIMARY KEY(source_id, node_id, prop_id)\
);\
CREATE INDEX IF NOT EXISTS graph_node_props_by_kv ON graph_node_props(prop_id, value)";

/// How one conceptual property is stored and accessed on a source table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyPhysical {
    /// Ordinary SQL column on the entity table.
    Column {
        column: String,
        /// Declared JSONB column; reads wrap with `json(...)`.
        jsonb: bool,
    },
    /// Key inside a JSON object column on the entity table.
    JsonBag {
        bag_column: String,
        /// Property key inside the JSON object (no `$` prefix).
        key: String,
    },
    /// Row in a side table: `(source_id, entity_id, prop_id) → value`.
    ///
    /// `prop_id` is an integer from the property dictionary (not a repeated
    /// UTF-8 property name on every cell). `value_type` is the dictionary type
    /// for predicate lowering. `source_id` namespaces node identities across
    /// multi-source graphs so local ids cannot collide.
    Cell {
        props_table: String,
        /// Graph source table id for this entity (written into the cell row).
        source_id: u64,
        /// Column on the props table that holds the entity identity.
        entity_column: String,
        /// Column on the entity table that holds identity (for correlation).
        identity_column: String,
        /// Integer property id (dictionary / IR property identity).
        prop_id: u64,
        /// Column name of the integer property id on the props table.
        prop_id_column: String,
        /// Column name of the value on the props table.
        value_column: String,
        /// Declared type for `value` (from property dictionary).
        value_type: ir::ValueType,
    },
}

impl PropertyPhysical {
    /// Declared value type when known (Cell and typed columns via catalog).
    pub fn value_type(&self) -> Option<&ir::ValueType> {
        match self {
            Self::Cell { value_type, .. } => Some(value_type),
            Self::Column { .. } | Self::JsonBag { .. } => None,
        }
    }

    /// Whether Cypher text predicates (CONTAINS / STARTS WITH / ENDS WITH /
    /// string equality patterns that become LIKE) are legal for this property.
    ///
    /// Integers/floats/bools reject text predicates at bind/lower using the
    /// dictionary type; they are not “cast and hope.”
    pub fn supports_text_predicate(&self) -> bool {
        match self {
            Self::Cell { value_type, .. } => {
                matches!(value_type, ir::ValueType::Text | ir::ValueType::Any)
            }
            // Bag values and untyped columns: allow; runtime/json may still fail.
            Self::JsonBag { .. } | Self::Column { .. } => true,
        }
    }

    /// Whether IS NULL / IS NOT NULL is always meaningful (all modes).
    pub fn supports_is_null(&self) -> bool {
        true
    }

    /// SQL scalar expression that reads this property from a row alias of the
    /// entity table (scan/materialize path).
    pub fn read_from_alias(&self, alias: &str) -> String {
        match self {
            Self::Column { column, jsonb } => {
                let value = format!("{alias}.{}", quote_identifier(column));
                if *jsonb {
                    format!("json({value})")
                } else {
                    value
                }
            }
            Self::JsonBag { bag_column, key } => {
                format!(
                    "json_extract({alias}.{}, '$.{}')",
                    quote_identifier(bag_column),
                    escape_json_key(key)
                )
            }
            Self::Cell {
                props_table,
                source_id,
                entity_column,
                identity_column,
                prop_id,
                prop_id_column,
                value_column,
                ..
            } => {
                format!(
                    "(SELECT {value} FROM {props} WHERE source_id = {source_id} AND {entity} = {alias}.{identity} AND {id_col} = {prop_id})",
                    value = quote_identifier(value_column),
                    props = quote_identifier(props_table),
                    entity = quote_identifier(entity_column),
                    identity = quote_identifier(identity_column),
                    id_col = quote_identifier(prop_id_column),
                    prop_id = prop_id,
                    alias = alias,
                    source_id = source_id,
                )
            }
        }
    }

    /// Correlated subquery form when the entity row is identified by a SQL
    /// expression (mutation references, join correlation).
    pub fn read_by_identity_sql(
        &self,
        table: &str,
        identity_column: &str,
        identity_expr: &str,
    ) -> Result<String, ()> {
        match self {
            Self::Column { column, jsonb } => {
                let selector = format!("p.{}", quote_identifier(column));
                let selector = if *jsonb {
                    format!("json({selector})")
                } else {
                    selector
                };
                Ok(format!(
                    "(SELECT {selector} FROM {} AS p WHERE p.{} = {})",
                    quote_identifier(table),
                    quote_identifier(identity_column),
                    identity_expr
                ))
            }
            Self::JsonBag { bag_column, key } => Ok(format!(
                "(SELECT json_extract(p.{bag}, '$.{key}') FROM {table} AS p WHERE p.{id} = {identity})",
                bag = quote_identifier(bag_column),
                key = escape_json_key(key),
                table = quote_identifier(table),
                id = quote_identifier(identity_column),
                identity = identity_expr,
            )),
            Self::Cell {
                props_table,
                source_id,
                entity_column,
                prop_id,
                prop_id_column,
                value_column,
                ..
            } => Ok(format!(
                "(SELECT {value} FROM {props} WHERE source_id = {source_id} AND {entity} = {identity} AND {id_col} = {prop_id})",
                value = quote_identifier(value_column),
                props = quote_identifier(props_table),
                entity = quote_identifier(entity_column),
                identity = identity_expr,
                id_col = quote_identifier(prop_id_column),
                prop_id = prop_id,
                source_id = source_id,
            )),
        }
    }

    /// Equality / IS match fragment for MERGE keys and open filters on this property.
    ///
    /// `entity_table` / `entity_identity` name the entity row being matched
    /// (for Cell EXISTS correlation).
    pub fn equality_match_sql(
        &self,
        entity_table: &str,
        entity_identity: &str,
        value_sql: &str,
    ) -> String {
        match self {
            Self::Column { column, .. } => {
                format!("{} IS ({value_sql})", quote_identifier(column))
            }
            Self::JsonBag { bag_column, key } => format!(
                "json_extract({}, '$.{}') IS ({value_sql})",
                quote_identifier(bag_column),
                escape_json_key(key)
            ),
            Self::Cell {
                props_table,
                source_id,
                entity_column,
                prop_id,
                prop_id_column,
                value_column,
                ..
            } => format!(
                "EXISTS (SELECT 1 FROM {} WHERE source_id = {} AND {} = {}.{} AND {} = {} AND {} IS ({value_sql}))",
                quote_identifier(props_table),
                source_id,
                quote_identifier(entity_column),
                quote_identifier(entity_table),
                quote_identifier(entity_identity),
                quote_identifier(prop_id_column),
                prop_id,
                quote_identifier(value_column),
            ),
        }
    }

    /// SQL that writes `value_sql` for the entity identified by `identity_param`
    /// (a bound parameter name without `$`).
    pub fn set_sql(
        &self,
        entity_table: &str,
        entity_identity_column: &str,
        identity_param: &str,
        value_sql: &str,
    ) -> String {
        match self {
            Self::Column { column, .. } => format!(
                "UPDATE {} SET {} = {value_sql} WHERE {} = ${identity_param}",
                quote_identifier(entity_table),
                quote_identifier(column),
                quote_identifier(entity_identity_column),
            ),
            Self::JsonBag { bag_column, key } => format!(
                "UPDATE {} SET {} = json_set(COALESCE({}, '{{}}'), '$.{}', {value_sql}) WHERE {} = ${identity_param}",
                quote_identifier(entity_table),
                quote_identifier(bag_column),
                quote_identifier(bag_column),
                escape_json_key(key),
                quote_identifier(entity_identity_column),
            ),
            Self::Cell {
                props_table,
                source_id,
                entity_column,
                prop_id,
                prop_id_column,
                value_column,
                ..
            } => {
                // Delete-then-insert keeps the path portable without depending
                // on UPSERT support for every VDBE path.
                format!(
                    "DELETE FROM {props} WHERE source_id = {source_id} AND {entity} = ${identity_param} AND {id_col} = {prop_id}; \
                     INSERT INTO {props}(source_id, {entity}, {id_col}, {value}) VALUES ({source_id}, ${identity_param}, {prop_id}, {value_sql})",
                    props = quote_identifier(props_table),
                    entity = quote_identifier(entity_column),
                    id_col = quote_identifier(prop_id_column),
                    value = quote_identifier(value_column),
                    prop_id = prop_id,
                    source_id = source_id,
                )
            }
        }
    }

    /// SQL that clears this property for the entity identified by `identity_param`.
    pub fn remove_sql(
        &self,
        entity_table: &str,
        entity_identity_column: &str,
        identity_param: &str,
    ) -> String {
        match self {
            Self::Column { column, .. } => format!(
                "UPDATE {} SET {} = NULL WHERE {} = ${identity_param}",
                quote_identifier(entity_table),
                quote_identifier(column),
                quote_identifier(entity_identity_column),
            ),
            Self::JsonBag { bag_column, key } => format!(
                "UPDATE {} SET {} = json_remove({}, '$.{}') WHERE {} = ${identity_param}",
                quote_identifier(entity_table),
                quote_identifier(bag_column),
                quote_identifier(bag_column),
                escape_json_key(key),
                quote_identifier(entity_identity_column),
            ),
            Self::Cell {
                props_table,
                source_id,
                entity_column,
                prop_id,
                prop_id_column,
                ..
            } => format!(
                "DELETE FROM {} WHERE source_id = {} AND {} = ${identity_param} AND {} = {}",
                quote_identifier(props_table),
                source_id,
                quote_identifier(entity_column),
                quote_identifier(prop_id_column),
                prop_id,
            ),
        }
    }

    /// Whether SET/REMOVE emits more than one SQL statement (Cell path).
    pub fn multi_statement_write(&self) -> bool {
        matches!(self, Self::Cell { .. })
    }
}

/// Map dictionary `value_type` text to IR types (registration / load path).
pub fn parse_dict_value_type(name: &str) -> ir::ValueType {
    match name.trim().to_ascii_lowercase().as_str() {
        "integer" | "int" | "long" => ir::ValueType::Integer,
        "float" | "double" | "real" | "number" => ir::ValueType::Real,
        "boolean" | "bool" => ir::ValueType::Boolean,
        "text" | "string" => ir::ValueType::Text,
        "any" => ir::ValueType::Any,
        _ => ir::ValueType::Any,
    }
}

pub fn dict_value_type_name(ty: &ir::ValueType) -> &'static str {
    match ty {
        ir::ValueType::Integer => "integer",
        ir::ValueType::Real => "float",
        ir::ValueType::Boolean => "boolean",
        ir::ValueType::Text => "text",
        _ => "any",
    }
}

/// Resolve physical storage for a property id.
///
/// Cell is the default rail. Multi-owner semantic maps may still need a
/// type-qualified SQL column when the same PropertyId maps to different
/// real columns per type (e.g. `weight` → `since` / `share`) — but only when
/// that column actually exists on the source table. Under Cell storage,
/// semantic `column` is a stable key spelling, not a physical field; fall
/// through to [`RelationalCatalogSnapshot::property_physical`].
pub fn resolve_property_physical(
    catalog: &dyn crate::lowering::RelationalCatalogSnapshot,
    source: ir::SourceTableId,
    semantic_types: &[String],
    property: ir::PropertyId,
) -> Option<PropertyPhysical> {
    match catalog.semantic_property_for_id(source, semantic_types, property) {
        Some(None) => None,
        Some(Some((_, _, column))) => {
            // Type-qualified Column only when the mapped SQL field is real.
            // Do not trust the semantic column spelling alone (Cell rail).
            if catalog.source_has_column(source, &column) {
                Some(PropertyPhysical::Column {
                    jsonb: catalog.property_column_is_jsonb(source, property),
                    column,
                })
            } else {
                catalog.property_physical(source, property)
            }
        }
        None => catalog.property_physical(source, property),
    }
}

fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn escape_json_key(key: &str) -> String {
    key.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bag_read_and_set_sql_shapes() {
        let physical = PropertyPhysical::JsonBag {
            bag_column: "props".to_owned(),
            key: "name".to_owned(),
        };
        assert_eq!(
            physical.read_from_alias("n"),
            "json_extract(n.\"props\", '$.name')"
        );
        let set = physical.set_sql("people", "id", "id_1", "'Ada'");
        assert!(set.contains("json_set"), "{set}");
        assert!(set.contains("$.name"), "{set}");
    }

    #[test]
    fn cell_uses_integer_prop_id_not_string_key() {
        let physical = PropertyPhysical::Cell {
            source_id: 1,
            props_table: "node_props".to_owned(),
            entity_column: "node_id".to_owned(),
            identity_column: "id".to_owned(),
            prop_id: 7,
            prop_id_column: "prop_id".to_owned(),
            value_column: "value".to_owned(),
            value_type: ir::ValueType::Text,
        };
        assert!(physical.multi_statement_write());
        assert!(physical.supports_text_predicate());
        let read = physical.read_from_alias("n");
        assert!(
            read.contains("prop_id\" = 7") || read.contains("= 7"),
            "{read}"
        );
        assert!(
            !read.contains("'name'"),
            "must not embed property name string"
        );
        let set = physical.set_sql("people", "id", "id_1", "'Ada'");
        assert!(set.contains("= 7"), "{set}");
        assert!(set.contains("DELETE FROM"), "{set}");
    }

    #[test]
    fn integer_cell_rejects_text_predicates() {
        let physical = PropertyPhysical::Cell {
            source_id: 1,
            props_table: "node_props".to_owned(),
            entity_column: "node_id".to_owned(),
            identity_column: "id".to_owned(),
            prop_id: 2,
            prop_id_column: "prop_id".to_owned(),
            value_column: "value".to_owned(),
            value_type: ir::ValueType::Integer,
        };
        assert!(!physical.supports_text_predicate());
        assert!(physical.supports_is_null());
    }

    #[test]
    fn dictionary_maps_name_to_integer_id_and_type() {
        let mut dict = PropertyDictionary::new();
        let name = dict
            .register("name", ir::ValueType::Text)
            .expect("register name");
        let age = dict
            .register("age", ir::ValueType::Integer)
            .expect("register age");
        assert_eq!(name.prop_id, 1);
        assert_eq!(age.prop_id, 2);
        assert_eq!(dict.get("NAME").map(|e| e.prop_id), Some(1));
        let cell = dict.cell_physical(&age, "node_props", "node_id", "id");
        match cell {
            PropertyPhysical::Cell {
                prop_id,
                value_type,
                ..
            } => {
                assert_eq!(prop_id, 2);
                assert_eq!(value_type, ir::ValueType::Integer);
            }
            other => panic!("expected Cell, got {other:?}"),
        }
        assert!(matches!(
            dict.register("age", ir::ValueType::Text),
            Err(PropertyDictError::TypeConflict { .. })
        ));
    }
}
