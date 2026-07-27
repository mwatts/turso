//! Schemaless catalog for donor fixtures.
//!
//! Donor corpora create arbitrary labels, relationship types, and
//! properties, while a fixture registers one node and one relationship
//! table. This catalog delegates to the schema-backed catalog first and
//! provisions anything unknown on demand: labels and relationship types
//! get fresh identities (the engine does not filter scans by label), and
//! properties get a real column added to the backing table.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use turso_core::Connection;
use turso_graph_frontend::{
    CatalogEntity, GraphCatalogSnapshot, NodeTableLayout, RegisteredGraph,
    RelationalCatalogSnapshot, RelationshipTableLayout, ResolvedProperty, SchemaCatalog,
};
use turso_graph_ir as ir;

/// Identity offset that keeps dynamically provisioned ids clear of the
/// schema catalog's own allocations.
const DYNAMIC_ID_BASE: u32 = 100_000;

struct DynamicState {
    labels: HashMap<String, ir::LabelId>,
    relationship_types: HashMap<String, ir::RelationshipTypeId>,
    properties: HashMap<(bool, String), ResolvedProperty>,
    property_columns: HashMap<ir::PropertyId, String>,
    next_id: u32,
}

pub struct DynamicCatalog {
    inner: SchemaCatalog,
    connection: Arc<Connection>,
    node_table: String,
    relationship_table: String,
    state: Mutex<DynamicState>,
}

impl DynamicCatalog {
    pub fn new(
        connection: Arc<Connection>,
        graph: RegisteredGraph,
        node_table: String,
        relationship_table: String,
    ) -> Self {
        Self {
            inner: SchemaCatalog::new(connection.clone(), graph),
            connection,
            node_table,
            relationship_table,
            state: Mutex::new(DynamicState {
                labels: HashMap::new(),
                relationship_types: HashMap::new(),
                properties: HashMap::new(),
                property_columns: HashMap::new(),
                next_id: DYNAMIC_ID_BASE,
            }),
        }
    }

    fn next_id(state: &mut DynamicState) -> u32 {
        state.next_id += 1;
        state.next_id
    }
}

impl GraphCatalogSnapshot for DynamicCatalog {
    fn node_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId> {
        self.inner.node_source(graph)
    }

    fn node_sources(&self, graph: ir::GraphId) -> Vec<ir::SourceTableId> {
        self.inner.node_sources(graph)
    }

    fn relationship_source(&self, graph: ir::GraphId) -> Option<ir::SourceTableId> {
        self.inner.relationship_source(graph)
    }

    fn relationship_sources(&self, graph: ir::GraphId) -> Vec<ir::SourceTableId> {
        self.inner.relationship_sources(graph)
    }

    fn relationship_endpoint_sources(
        &self,
        graph: ir::GraphId,
        relationship_source: ir::SourceTableId,
    ) -> Option<(ir::SourceTableId, ir::SourceTableId)> {
        self.inner
            .relationship_endpoint_sources(graph, relationship_source)
    }

    fn relationship_role_node_source(
        &self,
        graph: ir::GraphId,
        relationship_source: ir::SourceTableId,
        role: ir::RoleId,
    ) -> Option<ir::SourceTableId> {
        self.inner
            .relationship_role_node_source(graph, relationship_source, role)
    }

    fn relationship_source_roles(
        &self,
        source: ir::SourceTableId,
    ) -> Option<RelationshipTableLayout> {
        self.inner.relationship_source_roles(source)
    }

    fn label(&self, graph: ir::GraphId, name: &str) -> Option<ir::LabelId> {
        if let Some(label) = self.inner.label(graph, name) {
            return Some(label);
        }
        let mut state = self.state.lock().expect("catalog state lock");
        if let Some(label) = state.labels.get(name) {
            return Some(*label);
        }
        let label = ir::LabelId::new(Self::next_id(&mut state)).ok()?;
        state.labels.insert(name.to_owned(), label);
        Some(label)
    }

    fn relationship_type(&self, graph: ir::GraphId, name: &str) -> Option<ir::RelationshipTypeId> {
        if let Some(relationship_type) = self.inner.relationship_type(graph, name) {
            return Some(relationship_type);
        }
        {
            let state = self.state.lock().expect("catalog state lock");
            if let Some(relationship_type) = state.relationship_types.get(name) {
                return Some(*relationship_type);
            }
        }
        // Allocate through the persistent registry so the traversal
        // snapshot resolves the same identity from storage.
        let registry = turso_graph_frontend::relationship_type_registry_table_name(graph);
        let escaped = name.replace('\'', "''");
        self.connection
            .execute(format!(
                "INSERT INTO \"{registry}\"(name) SELECT '{escaped}' \
                 WHERE NOT EXISTS (SELECT 1 FROM \"{registry}\" WHERE name = '{escaped}')"
            ))
            .ok()?;
        let rows = self
            .connection
            .prepare(format!(
                "SELECT id FROM \"{registry}\" WHERE name = '{escaped}'"
            ))
            .and_then(|mut statement| statement.run_collect_rows())
            .ok()?;
        let id = match rows.first().and_then(|row| row.first()) {
            Some(turso_core::Value::Numeric(turso_core::Numeric::Integer(id))) => *id,
            _ => return None,
        };
        let relationship_type = ir::RelationshipTypeId::new(u32::try_from(id).ok()?).ok()?;
        self.state
            .lock()
            .expect("catalog state lock")
            .relationship_types
            .insert(name.to_owned(), relationship_type);
        Some(relationship_type)
    }

    fn property(
        &self,
        graph: ir::GraphId,
        entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        let is_node = matches!(entity, CatalogEntity::Node);
        {
            // Dynamically provisioned properties stay Any-typed even after
            // their ALTERed column becomes visible to the schema catalog,
            // which would otherwise re-resolve them with a text affinity.
            let state = self.state.lock().expect("catalog state lock");
            if let Some(property) = state.properties.get(&(is_node, name.to_owned())) {
                return Some(property.clone());
            }
        }
        // Identity and endpoint columns are structural: a Cypher property
        // that shares their name (donor data often uses `id`) must not
        // resolve onto them, or writes hit datatype mismatches against the
        // INTEGER PRIMARY KEY. Those names get a prefixed payload column.
        let reserved: &[&str] = if is_node {
            &["id"]
        } else {
            &["id", "src", "dst"]
        };
        let is_reserved = reserved.contains(&name);
        if !is_reserved {
            if let Some(property) = self.inner.property(graph, entity, name) {
                return Some(property);
            }
        }
        let mut state = self.state.lock().expect("catalog state lock");
        let key = (is_node, name.to_owned());
        let table = if is_node {
            &self.node_table
        } else {
            &self.relationship_table
        };
        let physical = if is_reserved {
            format!("cyprop_{name}")
        } else {
            name.to_owned()
        };
        let column = physical.replace('"', "\"\"");
        self.connection
            .execute(format!("ALTER TABLE \"{table}\" ADD COLUMN \"{column}\""))
            .ok()?;
        let property = ResolvedProperty {
            id: ir::PropertyId::new(Self::next_id(&mut state)).ok()?,
            value_type: ir::ValueType::Any,
            nullability: ir::Nullability::Nullable,
        };
        state.property_columns.insert(property.id, physical);
        state.properties.insert(key, property.clone());
        Some(property)
    }
}

impl RelationalCatalogSnapshot for DynamicCatalog {
    fn registered_node_sources(&self) -> Vec<ir::SourceTableId> {
        self.inner.registered_node_sources()
    }

    fn registered_relationship_sources(&self) -> Vec<ir::SourceTableId> {
        self.inner.registered_relationship_sources()
    }

    fn source_qualified_membership(&self) -> bool {
        self.inner.source_qualified_membership()
    }

    fn labels_table(&self) -> Option<String> {
        self.inner.labels_table()
    }

    fn relationship_types_table(&self) -> Option<String> {
        self.inner.relationship_types_table()
    }

    fn relationship_type_name(&self, relationship_type: ir::RelationshipTypeId) -> Option<String> {
        if let Some(name) = self.inner.relationship_type_name(relationship_type) {
            return Some(name);
        }
        self.state
            .lock()
            .expect("catalog state lock")
            .relationship_types
            .iter()
            .find(|(_, id)| **id == relationship_type)
            .map(|(name, _)| name.clone())
    }

    fn procedure_labels(&self) -> Option<Vec<String>> {
        self.inner.procedure_labels()
    }

    fn procedure_relationship_types(&self) -> Option<Vec<String>> {
        self.inner.procedure_relationship_types()
    }

    fn label_name(&self, label: ir::LabelId) -> Option<String> {
        if let Some(name) = self.inner.label_name(label) {
            return Some(name);
        }
        self.state
            .lock()
            .expect("catalog state lock")
            .labels
            .iter()
            .find(|(_, id)| **id == label)
            .map(|(name, _)| name.clone())
    }

    fn node_layout(&self, source: ir::SourceTableId) -> Option<NodeTableLayout> {
        self.inner.node_layout(source)
    }

    fn relationship_layout(&self, source: ir::SourceTableId) -> Option<RelationshipTableLayout> {
        self.inner.relationship_layout(source)
    }

    fn property_column(
        &self,
        source: ir::SourceTableId,
        property: ir::PropertyId,
    ) -> Option<String> {
        if let Some(column) = self.inner.property_column(source, property) {
            return Some(column);
        }
        self.state
            .lock()
            .expect("catalog state lock")
            .property_columns
            .get(&property)
            .cloned()
    }

    fn property_column_is_jsonb(
        &self,
        source: ir::SourceTableId,
        property: ir::PropertyId,
    ) -> bool {
        self.inner.property_column_is_jsonb(source, property)
    }

    fn payload_columns(&self, source: ir::SourceTableId) -> Option<Vec<(String, String)>> {
        self.inner.payload_columns(source)
    }

    fn procedure_property_keys(&self, source: ir::SourceTableId) -> Option<Vec<String>> {
        self.inner.procedure_property_keys(source)
    }
}
