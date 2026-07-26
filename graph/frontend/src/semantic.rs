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
use crate::semantic_constraints::{
    create_constraint_catalog, insert_additive_rows, load_constraint_snapshot,
    rows_for_registration, SemanticConstraintRegistration, SemanticConstraintSnapshot,
};

pub(crate) const SEMANTIC_TYPES_TABLE: &str = "__turso_internal_graph_semantic_types";
pub(crate) const SEMANTIC_PROPERTIES_TABLE: &str = "__turso_internal_graph_semantic_properties";
pub(crate) const SEMANTIC_OWNERSHIP_TABLE: &str = "__turso_internal_graph_semantic_ownership";
pub(crate) const SEMANTIC_ENDPOINTS_TABLE: &str = "__turso_internal_graph_semantic_endpoints";
pub(crate) const SEMANTIC_FRAGMENTS_TABLE: &str = "__turso_internal_graph_semantic_fragments";
pub(crate) const SEMANTIC_FRAGMENT_MEMBERS_TABLE: &str =
    "__turso_internal_graph_semantic_fragment_members";
pub(crate) const SEMANTIC_FRAGMENT_PROPERTIES_TABLE: &str =
    "__turso_internal_graph_semantic_fragment_properties";
pub(crate) const SEMANTIC_FRAGMENT_OWNERSHIP_TABLE: &str =
    "__turso_internal_graph_semantic_fragment_ownership";

const REGISTRATION_SAVEPOINT: &str = "turso_graph_register_semantic";

/// Immutable semantic catalog loaded once for a graph preparation context.
#[derive(Debug)]
pub struct SemanticSnapshot {
    node_names: HashMap<String, u32>,
    relationship_names: HashMap<String, u32>,
    node_types: HashMap<u32, SemanticTypeInfo>,
    relationship_types: HashMap<u32, SemanticTypeInfo>,
    fragment_names: HashMap<String, u32>,
    fragments: HashMap<u32, SemanticFragmentInfo>,
    endpoints: HashMap<u32, EndpointConstraint>,
    constraints: SemanticConstraintSnapshot,
}

/// Resolved fragment identity and its precomputed concrete member types.
#[derive(Debug)]
pub struct SemanticFragmentInfo {
    /// Preserved conceptual spelling.
    pub name: String,
    /// Persisted `LabelId` value.
    pub fragment_id: u32,
    member_type_ids: Vec<u32>,
}

impl SemanticFragmentInfo {
    /// Concrete node types carrying this fragment, sorted by stable identity.
    pub fn member_type_ids(&self) -> &[u32] {
        &self.member_type_ids
    }
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

    /// Resolve a semantic fragment by conceptual name.
    pub fn fragment(&self, name: &str) -> Option<&SemanticFragmentInfo> {
        self.fragment_names
            .get(&fold(name))
            .and_then(|id| self.fragments.get(id))
    }

    /// Resolve a semantic fragment by persisted label identity.
    pub fn fragment_by_id(&self, id: ir::LabelId) -> Option<&SemanticFragmentInfo> {
        self.fragments.get(&id.get())
    }

    /// Resolve the concrete node types selected by a label conjunction.
    pub fn node_types_for_labels(&self, labels: &[ir::LabelId]) -> Vec<&SemanticTypeInfo> {
        let mut selected = self.node_types.keys().copied().collect::<HashSet<_>>();
        for label in labels {
            if self.node_types.contains_key(&label.get()) {
                selected.retain(|type_id| *type_id == label.get());
            } else if let Some(fragment) = self.fragments.get(&label.get()) {
                selected.retain(|type_id| fragment.member_type_ids.binary_search(type_id).is_ok());
            } else {
                selected.clear();
            }
        }
        let mut types = selected
            .into_iter()
            .filter_map(|type_id| self.node_types.get(&type_id))
            .collect::<Vec<_>>();
        types.sort_by_key(|type_info| type_info.type_id);
        types
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

    pub(crate) fn fragment_values(&self) -> impl Iterator<Item = &SemanticFragmentInfo> {
        self.fragments.values()
    }

    pub(crate) fn constraints(&self) -> &SemanticConstraintSnapshot {
        &self.constraints
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

/// Complete fragment-interface definition registered with a semantic schema.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticFragmentRegistration {
    /// Graph-scoped node fragments.
    pub fragments: Vec<SemanticFragment>,
}

/// An uninstantiable node interface with contributed property ownership.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticFragment {
    /// Conceptual name addressed through a Cypher label.
    pub name: String,
    /// Conceptual properties every member must map.
    pub properties: Vec<String>,
    /// Concrete node types carrying this fragment.
    pub members: Vec<SemanticFragmentMember>,
}

/// One concrete node type's membership and physical property mappings.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticFragmentMember {
    /// Name of a concrete semantic node type.
    pub node_type: String,
    /// Physical mappings for every property declared by the fragment.
    pub properties: Vec<SemanticProperty>,
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
    /// A fragment name collides with another fragment or concrete type.
    #[error("semantic fragment name `{name}` collides with an existing semantic name")]
    DuplicateFragmentName {
        /// Colliding fragment spelling.
        name: String,
    },
    /// A fragment names the same member type more than once.
    #[error("semantic fragment `{fragment}` contains duplicate member `{node_type}`")]
    DuplicateFragmentMember {
        /// Fragment containing the duplicate.
        fragment: String,
        /// Duplicated concrete node type.
        node_type: String,
    },
    /// A fragment without concrete members cannot produce an executable scan.
    #[error("semantic fragment `{fragment}` must contain at least one member")]
    EmptyFragment {
        /// Empty fragment.
        fragment: String,
    },
    /// A fragment references a node type absent from this registration.
    #[error("semantic fragment `{fragment}` references unknown node type `{node_type}`")]
    UnknownFragmentMember {
        /// Fragment containing the invalid membership.
        fragment: String,
        /// Unknown concrete node type.
        node_type: String,
    },
    /// A member omits a property declared by its fragment.
    #[error(
        "semantic fragment `{fragment}` member `{node_type}` is missing property mapping `{property}`"
    )]
    MissingFragmentProperty {
        /// Owning fragment.
        fragment: Box<str>,
        /// Concrete member.
        node_type: Box<str>,
        /// Missing declared property.
        property: Box<str>,
    },
    /// A member maps a property not declared by its fragment.
    #[error(
        "semantic fragment `{fragment}` member `{node_type}` maps undeclared property `{property}`"
    )]
    UndeclaredFragmentProperty {
        /// Owning fragment.
        fragment: Box<str>,
        /// Concrete member.
        node_type: Box<str>,
        /// Extra mapped property.
        property: Box<str>,
    },
    /// Direct or contributed ownership maps one property inconsistently.
    #[error(
        "semantic property `{property}` on type `{node_type}` has conflicting physical mappings `{first_column}` and `{second_column}`"
    )]
    ConflictingPropertyMapping {
        /// Concrete semantic node type.
        node_type: Box<str>,
        /// Conceptual property.
        property: Box<str>,
        /// First physical mapping.
        first_column: Box<str>,
        /// Conflicting physical mapping.
        second_column: Box<str>,
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
    /// A semantic endpoint type maps to a different physical node source than
    /// the relationship source stores at that endpoint.
    #[error(
        "relationship type `{relationship_type}` {endpoint} endpoint type `{node_type}` maps to source `{actual_source}`, but relationship source `{relationship_source}` requires `{required_source}`"
    )]
    EndpointSourceMismatch {
        /// Semantic relationship type.
        relationship_type: Box<str>,
        /// Stored endpoint kind.
        endpoint: &'static str,
        /// Semantic node type used by the constraint.
        node_type: Box<str>,
        /// Node source mapped by that semantic type.
        actual_source: Box<str>,
        /// Relationship source carrying the endpoints.
        relationship_source: Box<str>,
        /// Node source required by the physical relationship layout.
        required_source: Box<str>,
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
    /// A constraint references an unknown concrete semantic owner.
    #[error("semantic constraint references unknown owner `{owner}`")]
    UnknownConstraintOwner {
        /// Unknown node or relationship type.
        owner: String,
    },
    /// A constraint references a property absent from its owner.
    #[error("semantic constraint references unknown property `{owner}.{property}`")]
    UnknownConstraintProperty {
        /// Concrete semantic owner.
        owner: String,
        /// Unknown property.
        property: String,
    },
    /// The same constraint identity appears more than once in one registration.
    #[error("semantic constraint `{constraint}` is duplicated")]
    DuplicateConstraint {
        /// Duplicated constraint description.
        constraint: String,
    },
    /// A constraint definition is structurally invalid.
    #[error("invalid semantic constraint `{constraint}`: {detail}")]
    InvalidConstraint {
        /// Constraint being validated.
        constraint: String,
        /// Invalid invariant.
        detail: String,
    },
    /// Existing data or a graph mutation violates an active constraint.
    #[error("semantic constraint `{constraint}` violated: {detail}")]
    ConstraintViolation {
        /// Constraint that failed.
        constraint: String,
        /// Failure detail.
        detail: String,
    },
    /// Additive registration cannot change an already-active constraint.
    #[error(
        "semantic constraint `{constraint}` already exists with a different definition; constraint evolution is not supported"
    )]
    ConstraintEvolutionUnsupported {
        /// Constraint whose definition would change.
        constraint: String,
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

#[cfg(test)]
pub(crate) fn validate_registration_shape(
    registration: &SemanticSchemaRegistration,
) -> Result<(), SemanticCatalogError> {
    validate_registration_shape_with_fragments(
        registration,
        &SemanticFragmentRegistration::default(),
    )
}

fn validate_registration_shape_with_fragments(
    registration: &SemanticSchemaRegistration,
    fragments: &SemanticFragmentRegistration,
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
    let mut fragment_names = HashSet::new();
    for fragment in &fragments.fragments {
        require_name("semantic fragment", &fragment.name)?;
        let folded = fold(&fragment.name);
        if type_names.contains(&folded) || !fragment_names.insert(folded) {
            return Err(SemanticCatalogError::DuplicateFragmentName {
                name: fragment.name.clone(),
            });
        }
        validate_fragment_shape(fragment, &node_type_names)?;
    }
    for relationship in &registration.relationship_types {
        require_name("semantic type", &relationship.name)?;
        require_name("source", &relationship.source)?;
        if fragment_names.contains(&fold(&relationship.name)) {
            return Err(SemanticCatalogError::DuplicateFragmentName {
                name: relationship.name.clone(),
            });
        }
        if !type_names.insert(fold(&relationship.name)) {
            return Err(SemanticCatalogError::DuplicateTypeName {
                name: relationship.name.clone(),
            });
        }
        validate_properties(&relationship.name, &relationship.properties)?;
        for (endpoint, allowed) in [("start", &relationship.start), ("end", &relationship.end)] {
            for node_type in allowed {
                let folded = fold(node_type);
                if !node_type_names.contains(&folded) && !fragment_names.contains(&folded) {
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

fn validate_fragment_shape(
    fragment: &SemanticFragment,
    node_type_names: &HashSet<String>,
) -> Result<(), SemanticCatalogError> {
    if fragment.members.is_empty() {
        return Err(SemanticCatalogError::EmptyFragment {
            fragment: fragment.name.clone(),
        });
    }
    let mut declared = HashSet::new();
    for property in &fragment.properties {
        require_name("property", property)?;
        if !declared.insert(fold(property)) {
            return Err(SemanticCatalogError::DuplicatePropertyName {
                owner: fragment.name.clone(),
                property: property.clone(),
            });
        }
    }
    let mut members = HashSet::new();
    for member in &fragment.members {
        require_name("semantic node type", &member.node_type)?;
        let folded_member = fold(&member.node_type);
        if !node_type_names.contains(&folded_member) {
            return Err(SemanticCatalogError::UnknownFragmentMember {
                fragment: fragment.name.clone(),
                node_type: member.node_type.clone(),
            });
        }
        if !members.insert(folded_member) {
            return Err(SemanticCatalogError::DuplicateFragmentMember {
                fragment: fragment.name.clone(),
                node_type: member.node_type.clone(),
            });
        }
        validate_properties(&fragment.name, &member.properties)?;
        let mapped = member
            .properties
            .iter()
            .map(|property| fold(&property.name))
            .collect::<HashSet<_>>();
        if let Some(property) = fragment
            .properties
            .iter()
            .find(|property| !mapped.contains(&fold(property)))
        {
            return Err(SemanticCatalogError::MissingFragmentProperty {
                fragment: fragment.name.clone().into_boxed_str(),
                node_type: member.node_type.clone().into_boxed_str(),
                property: property.clone().into_boxed_str(),
            });
        }
        if let Some(property) = member
            .properties
            .iter()
            .find(|property| !declared.contains(&fold(&property.name)))
        {
            return Err(SemanticCatalogError::UndeclaredFragmentProperty {
                fragment: fragment.name.clone().into_boxed_str(),
                node_type: member.node_type.clone().into_boxed_str(),
                property: property.name.clone().into_boxed_str(),
            });
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
    register_semantic_schema_with_fragments(
        connection,
        graph_name,
        registration,
        &SemanticFragmentRegistration::default(),
    )
}

/// Register a semantic schema and its fragment interfaces atomically.
///
/// Identical replay is idempotent; a changed schema, membership, or physical
/// mapping is rejected.
pub fn register_semantic_schema_with_fragments(
    connection: &Arc<Connection>,
    graph_name: &str,
    registration: &SemanticSchemaRegistration,
    fragments: &SemanticFragmentRegistration,
) -> Result<(), SemanticCatalogError> {
    validate_registration_shape_with_fragments(registration, fragments)?;
    let graph = match load_registered_graph(connection, graph_name) {
        Ok(graph) => graph,
        Err(CatalogError::GraphNotFound(_)) => {
            return Err(SemanticCatalogError::GraphNotFound(graph_name.to_owned()));
        }
        Err(error) => return Err(error.into()),
    };
    validate_against_graph(connection, &graph, registration, fragments)?;
    run_in_registration_transaction(connection, |connection| {
        register_semantic_in_transaction(connection, &graph, registration, fragments)
    })
}

/// Add semantic constraints to an already-registered semantic schema.
///
/// Registration is append-only, atomic, and idempotent. Every newly requested
/// constraint validates all visible graph data before it becomes active.
pub fn register_semantic_constraints(
    connection: &Arc<Connection>,
    graph_name: &str,
    registration: &SemanticConstraintRegistration,
) -> Result<(), SemanticCatalogError> {
    let graph = match load_registered_graph(connection, graph_name) {
        Ok(graph) => graph,
        Err(CatalogError::GraphNotFound(_)) => {
            return Err(SemanticCatalogError::GraphNotFound(graph_name.to_owned()));
        }
        Err(error) => return Err(error.into()),
    };
    let semantic = load_semantic_snapshot(connection, &graph)?.ok_or_else(|| {
        SemanticCatalogError::InvalidConstraint {
            constraint: "constraint registration".to_owned(),
            detail: "requires a registered semantic schema".to_owned(),
        }
    })?;
    let requested = rows_for_registration(registration, &semantic)?;
    if requested.is_empty() {
        return Ok(());
    }
    run_in_registration_transaction(connection, |connection| {
        create_constraint_catalog(connection)?;
        if !insert_additive_rows(connection, graph.id.get(), &requested)? {
            return Ok(());
        }
        let updated = load_semantic_snapshot(connection, &graph)?.ok_or(
            SemanticCatalogError::InvalidCatalogValue("semantic schema disappeared"),
        )?;
        updated.constraints().validate_state(connection)?;
        bump_semantic_generation(connection, graph.id.get())
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
    fragments: &SemanticFragmentRegistration,
) -> Result<(), SemanticCatalogError> {
    let mut property_types = HashMap::<String, (String, ir::ValueType)>::new();
    let node_sources = registration
        .node_types
        .iter()
        .map(|node_type| (fold(&node_type.name), node_type.source.as_str()))
        .collect::<HashMap<_, _>>();

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

    let node_types = registration
        .node_types
        .iter()
        .map(|node_type| (fold(&node_type.name), node_type))
        .collect::<HashMap<_, _>>();
    let mut owner_mappings = registration
        .node_types
        .iter()
        .flat_map(|node_type| {
            node_type.properties.iter().map(move |property| {
                (
                    (fold(&node_type.name), fold(&property.name)),
                    property.column.clone(),
                )
            })
        })
        .collect::<HashMap<_, _>>();
    for fragment in &fragments.fragments {
        for member in &fragment.members {
            let node_type = node_types
                .get(&fold(&member.node_type))
                .expect("fragment shape validated member type");
            let source = graph
                .node_sources
                .iter()
                .find(|source| source.name.eq_ignore_ascii_case(&node_type.source))
                .expect("semantic node source validated above");
            check_owned_columns(
                connection,
                &fragment.name,
                &member.properties,
                &source.table,
                &[source.identity_column.as_str()],
                &mut property_types,
            )?;
            for property in &member.properties {
                let key = (fold(&member.node_type), fold(&property.name));
                if let Some(first_column) = owner_mappings.get(&key) {
                    if !first_column.eq_ignore_ascii_case(&property.column) {
                        return Err(SemanticCatalogError::ConflictingPropertyMapping {
                            node_type: member.node_type.clone().into_boxed_str(),
                            property: property.name.clone().into_boxed_str(),
                            first_column: first_column.clone().into_boxed_str(),
                            second_column: property.column.clone().into_boxed_str(),
                        });
                    }
                } else {
                    owner_mappings.insert(key, property.column.clone());
                }
            }
        }
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
        let start_role = source
            .role_by_name("start")
            .expect("binary relationship source has a start role");
        let end_role = source
            .role_by_name("end")
            .expect("binary relationship source has an end role");
        check_owned_columns(
            connection,
            &relationship.name,
            &relationship.properties,
            &source.table,
            &[
                source.identity_column.as_str(),
                start_role.column.as_str(),
                end_role.column.as_str(),
            ],
            &mut property_types,
        )?;
        for (endpoint, allowed, required_source) in [
            ("start", &relationship.start, start_role.node_source),
            ("end", &relationship.end, end_role.node_source),
        ] {
            let required = graph
                .node_sources
                .iter()
                .find(|node_source| node_source.id == required_source)
                .expect("registered relationship endpoint source exists");
            for node_type in expand_endpoint_names(allowed, fragments) {
                let actual_name = node_sources
                    .get(&fold(node_type))
                    .expect("registration shape validated endpoint type");
                let actual = graph
                    .node_sources
                    .iter()
                    .find(|node_source| node_source.name.eq_ignore_ascii_case(actual_name))
                    .expect("semantic node source validated above");
                if actual.id != required.id {
                    return Err(SemanticCatalogError::EndpointSourceMismatch {
                        relationship_type: relationship.name.clone().into_boxed_str(),
                        endpoint,
                        node_type: node_type.to_owned().into_boxed_str(),
                        actual_source: actual.name.clone().into_boxed_str(),
                        relationship_source: source.name.clone().into_boxed_str(),
                        required_source: required.name.clone().into_boxed_str(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn expand_endpoint_names<'a>(
    names: &'a [String],
    fragments: &'a SemanticFragmentRegistration,
) -> Vec<&'a str> {
    names
        .iter()
        .flat_map(|name| {
            fragments
                .fragments
                .iter()
                .find(|fragment| fragment.name.eq_ignore_ascii_case(name))
                .map(|fragment| {
                    fragment
                        .members
                        .iter()
                        .map(|member| member.node_type.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| vec![name.as_str()])
        })
        .collect()
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
    fragments: Vec<(u64, String)>,
    fragment_members: Vec<(u64, u64)>,
    fragment_properties: Vec<(u64, u64)>,
    fragment_ownership: Vec<(u64, u64, u64, u64, String)>,
}

fn register_semantic_in_transaction(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    registration: &SemanticSchemaRegistration,
    fragments: &SemanticFragmentRegistration,
) -> Result<(), SemanticCatalogError> {
    create_semantic_catalog(connection, !fragments.fragments.is_empty())?;
    let expected = catalog_rows_for_registration(graph, registration, fragments);
    let existing = load_catalog_rows(connection, graph.id.get())?;
    let has_existing = !existing.types.is_empty()
        || !existing.properties.is_empty()
        || !existing.ownership.is_empty()
        || !existing.endpoints.is_empty()
        || !existing.fragments.is_empty()
        || !existing.fragment_members.is_empty()
        || !existing.fragment_properties.is_empty()
        || !existing.fragment_ownership.is_empty();
    if has_existing {
        if existing.clone().canonicalized() == expected.clone().canonicalized() {
            return Ok(());
        }
        if fragments.fragments.is_empty()
            && !existing.fragments.is_empty()
            && existing.clone().without_fragments().canonicalized()
                == expected.clone().canonicalized()
        {
            return Ok(());
        }
        if !fragments.fragments.is_empty()
            && existing.fragments.is_empty()
            && !endpoints_reference_fragments(registration, fragments)
        {
            let base = catalog_rows_for_registration(
                graph,
                registration,
                &SemanticFragmentRegistration::default(),
            );
            let base_matches = existing.canonicalized() == base.clone().canonicalized();
            if base_matches {
                let base_property_ids = base
                    .properties
                    .iter()
                    .map(|(property_id, _)| *property_id)
                    .collect::<HashSet<_>>();
                let delta = CatalogRows {
                    types: Vec::new(),
                    properties: expected
                        .properties
                        .into_iter()
                        .filter(|(property_id, _)| !base_property_ids.contains(property_id))
                        .collect(),
                    ownership: Vec::new(),
                    endpoints: Vec::new(),
                    fragments: expected.fragments,
                    fragment_members: expected.fragment_members,
                    fragment_properties: expected.fragment_properties,
                    fragment_ownership: expected.fragment_ownership,
                };
                insert_catalog_rows(connection, graph.id.get(), &delta)?;
                bump_semantic_generation(connection, graph.id.get())?;
                return Ok(());
            }
        }
        return Err(SemanticCatalogError::ConflictingSchema(graph.name.clone()));
    }

    insert_catalog_rows(connection, graph.id.get(), &expected)?;
    bump_semantic_generation(connection, graph.id.get())?;
    Ok(())
}

fn endpoints_reference_fragments(
    registration: &SemanticSchemaRegistration,
    fragments: &SemanticFragmentRegistration,
) -> bool {
    registration.relationship_types.iter().any(|relationship| {
        relationship
            .start
            .iter()
            .chain(&relationship.end)
            .any(|endpoint| {
                fragments
                    .fragments
                    .iter()
                    .any(|fragment| fragment.name.eq_ignore_ascii_case(endpoint))
            })
    })
}

fn bump_semantic_generation(
    connection: &Arc<Connection>,
    graph_id: u64,
) -> Result<(), SemanticCatalogError> {
    execute_internal(
        connection,
        format!(
            "UPDATE {GENERATIONS_TABLE} SET generation = generation + 1 \
             WHERE graph_id = {graph_id}"
        ),
    )?;
    Ok(())
}

fn create_semantic_catalog(
    connection: &Arc<Connection>,
    include_fragments: bool,
) -> Result<(), SemanticCatalogError> {
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
    if !include_fragments {
        return Ok(());
    }
    for ddl in [
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_FRAGMENTS_TABLE}(\
                graph_id INTEGER NOT NULL, \
                fragment_id INTEGER NOT NULL CHECK(fragment_id > 0), \
                name TEXT NOT NULL COLLATE NOCASE, \
                PRIMARY KEY(graph_id, fragment_id), \
                UNIQUE(graph_id, name)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_FRAGMENT_MEMBERS_TABLE}(\
                graph_id INTEGER NOT NULL, \
                fragment_id INTEGER NOT NULL, \
                node_type_id INTEGER NOT NULL, \
                PRIMARY KEY(graph_id, fragment_id, node_type_id)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_FRAGMENT_PROPERTIES_TABLE}(\
                graph_id INTEGER NOT NULL, \
                fragment_id INTEGER NOT NULL, \
                property_id INTEGER NOT NULL, \
                PRIMARY KEY(graph_id, fragment_id, property_id)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_FRAGMENT_OWNERSHIP_TABLE}(\
                graph_id INTEGER NOT NULL, \
                fragment_id INTEGER NOT NULL, \
                node_type_id INTEGER NOT NULL, \
                property_id INTEGER NOT NULL, \
                source_id INTEGER NOT NULL, \
                column_name TEXT NOT NULL, \
                PRIMARY KEY(graph_id, fragment_id, node_type_id, property_id)\
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
    fragment_registration: &SemanticFragmentRegistration,
) -> CatalogRows {
    let mut types = Vec::new();
    let mut properties = Vec::new();
    let mut ownership = Vec::new();
    let mut endpoints = Vec::new();
    let mut fragments = Vec::new();
    let mut fragment_members = Vec::new();
    let mut fragment_properties = Vec::new();
    let mut fragment_ownership = Vec::new();
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
            for node_type in expand_endpoint_names(allowed, fragment_registration) {
                endpoints.push((type_id, endpoint.to_owned(), node_ids[&fold(node_type)]));
            }
        }
    }
    for (index, fragment) in fragment_registration.fragments.iter().enumerate() {
        let fragment_id = registration.node_types.len() as u64 + index as u64 + 1;
        fragments.push((fragment_id, fragment.name.clone()));
        for property in &fragment.properties {
            let folded_name = fold(property);
            let property_id = *property_ids.entry(folded_name).or_insert_with(|| {
                let id = properties.len() as u64 + 1;
                properties.push((id, property.clone()));
                id
            });
            fragment_properties.push((fragment_id, property_id));
        }
        for member in &fragment.members {
            let node_type_id = node_ids[&fold(&member.node_type)];
            let node_type = registration
                .node_types
                .iter()
                .find(|node_type| node_type.name.eq_ignore_ascii_case(&member.node_type))
                .expect("fragment shape validated member type");
            let source = graph
                .node_sources
                .iter()
                .find(|source| source.name.eq_ignore_ascii_case(&node_type.source))
                .expect("registration was physically validated");
            fragment_members.push((fragment_id, node_type_id));
            for property in &member.properties {
                let property_id = property_ids[&fold(&property.name)];
                fragment_ownership.push((
                    fragment_id,
                    node_type_id,
                    property_id,
                    source.id.get(),
                    property.column.clone(),
                ));
            }
        }
    }
    endpoints.sort();
    endpoints.dedup();
    let mut rows = CatalogRows {
        types,
        properties,
        ownership,
        endpoints,
        fragments,
        fragment_members,
        fragment_properties,
        fragment_ownership,
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
    let has_fragment_catalog = connection
        .current_schema()
        .get_table(SEMANTIC_FRAGMENTS_TABLE)
        .is_some();
    let (mut fragments, mut fragment_members, mut fragment_properties, mut fragment_ownership) =
        if has_fragment_catalog {
            load_fragment_catalog_rows(connection, graph_id)?
        } else {
            (Vec::new(), Vec::new(), Vec::new(), Vec::new())
        };
    let mut rows = CatalogRows {
        types: std::mem::take(&mut types),
        properties: std::mem::take(&mut properties),
        ownership: std::mem::take(&mut ownership),
        endpoints: std::mem::take(&mut endpoints),
        fragments: std::mem::take(&mut fragments),
        fragment_members: std::mem::take(&mut fragment_members),
        fragment_properties: std::mem::take(&mut fragment_properties),
        fragment_ownership: std::mem::take(&mut fragment_ownership),
    };
    sort_catalog_rows(&mut rows);
    Ok(rows)
}

type FragmentCatalogRows = (
    Vec<(u64, String)>,
    Vec<(u64, u64)>,
    Vec<(u64, u64)>,
    Vec<(u64, u64, u64, u64, String)>,
);

fn load_fragment_catalog_rows(
    connection: &Arc<Connection>,
    graph_id: u64,
) -> Result<FragmentCatalogRows, SemanticCatalogError> {
    let fragments = query_rows(
        connection,
        &format!(
            "SELECT fragment_id, name FROM {SEMANTIC_FRAGMENTS_TABLE} \
             WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            positive_u64(
                integer(row, 0, "semantic fragment id")?,
                "semantic fragment id",
            )?,
            text(row, 1, "semantic fragment name")?.to_owned(),
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let members = query_rows(
        connection,
        &format!(
            "SELECT fragment_id, node_type_id FROM {SEMANTIC_FRAGMENT_MEMBERS_TABLE} \
             WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            positive_u64(
                integer(row, 0, "semantic fragment member fragment")?,
                "semantic fragment member fragment",
            )?,
            positive_u64(
                integer(row, 1, "semantic fragment member type")?,
                "semantic fragment member type",
            )?,
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let properties = query_rows(
        connection,
        &format!(
            "SELECT fragment_id, property_id FROM {SEMANTIC_FRAGMENT_PROPERTIES_TABLE} \
             WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            positive_u64(
                integer(row, 0, "semantic fragment property fragment")?,
                "semantic fragment property fragment",
            )?,
            positive_u64(
                integer(row, 1, "semantic fragment property")?,
                "semantic fragment property",
            )?,
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let ownership = query_rows(
        connection,
        &format!(
            "SELECT fragment_id, node_type_id, property_id, source_id, column_name \
             FROM {SEMANTIC_FRAGMENT_OWNERSHIP_TABLE} WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            positive_u64(
                integer(row, 0, "semantic fragment owner fragment")?,
                "semantic fragment owner fragment",
            )?,
            positive_u64(
                integer(row, 1, "semantic fragment owner type")?,
                "semantic fragment owner type",
            )?,
            positive_u64(
                integer(row, 2, "semantic fragment owner property")?,
                "semantic fragment owner property",
            )?,
            positive_u64(
                integer(row, 3, "semantic fragment owner source")?,
                "semantic fragment owner source",
            )?,
            text(row, 4, "semantic fragment owner column")?.to_owned(),
        ))
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    Ok((fragments, members, properties, ownership))
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
    rows.fragments.sort();
    rows.fragment_members.sort_unstable();
    rows.fragment_properties.sort_unstable();
    rows.fragment_ownership.sort();
}

impl CatalogRows {
    fn without_fragments(mut self) -> Self {
        let direct_property_ids = self
            .ownership
            .iter()
            .map(|(_, _, property_id, _, _)| *property_id)
            .collect::<HashSet<_>>();
        self.properties
            .retain(|(property_id, _)| direct_property_ids.contains(property_id));
        self.fragments.clear();
        self.fragment_members.clear();
        self.fragment_properties.clear();
        self.fragment_ownership.clear();
        self
    }

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
        for (_, name) in &mut self.fragments {
            *name = fold(name);
        }
        for (_, _, _, _, column) in &mut self.fragment_ownership {
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
    for (fragment_id, name) in &rows.fragments {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_FRAGMENTS_TABLE}(graph_id, fragment_id, name) \
                 VALUES ({graph_id}, {fragment_id}, {})",
                sql_string(name)
            ),
        )?;
    }
    for (fragment_id, node_type_id) in &rows.fragment_members {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_FRAGMENT_MEMBERS_TABLE}(\
                    graph_id, fragment_id, node_type_id\
                 ) VALUES ({graph_id}, {fragment_id}, {node_type_id})"
            ),
        )?;
    }
    for (fragment_id, property_id) in &rows.fragment_properties {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_FRAGMENT_PROPERTIES_TABLE}(\
                    graph_id, fragment_id, property_id\
                 ) VALUES ({graph_id}, {fragment_id}, {property_id})"
            ),
        )?;
    }
    for (fragment_id, node_type_id, property_id, source_id, column) in &rows.fragment_ownership {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_FRAGMENT_OWNERSHIP_TABLE}(\
                    graph_id, fragment_id, node_type_id, property_id, source_id, column_name\
                 ) VALUES (\
                    {graph_id}, {fragment_id}, {node_type_id}, {property_id}, {source_id}, {}\
                 )",
                sql_string(column)
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
        fragment_names: HashMap::new(),
        fragments: HashMap::new(),
        endpoints: HashMap::new(),
        constraints: SemanticConstraintSnapshot::default(),
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

    let has_fragment_catalog = connection
        .current_schema()
        .get_table(SEMANTIC_FRAGMENTS_TABLE)
        .is_some();
    let mut has_fragments = false;
    if has_fragment_catalog {
        let fragment_rows = query_rows(
            connection,
            &format!(
                "SELECT fragment_id, name FROM {SEMANTIC_FRAGMENTS_TABLE} \
                 WHERE graph_id = {}",
                graph.id.get()
            ),
        )?;
        has_fragments = !fragment_rows.is_empty();
        if has_fragments {
            for row in &fragment_rows {
                let fragment_id = positive_u32(
                    integer(row, 0, "semantic fragment id")?,
                    "semantic fragment id",
                )?;
                let name = text(row, 1, "semantic fragment name")?.to_owned();
                if snapshot.node_types.contains_key(&fragment_id)
                    || snapshot.node_names.contains_key(&fold(&name))
                    || snapshot.relationship_names.contains_key(&fold(&name))
                {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "semantic fragment identity collision",
                    ));
                }
                let info = SemanticFragmentInfo {
                    name: name.clone(),
                    fragment_id,
                    member_type_ids: Vec::new(),
                };
                if snapshot
                    .fragment_names
                    .insert(fold(&name), fragment_id)
                    .is_some()
                    || snapshot.fragments.insert(fragment_id, info).is_some()
                {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "duplicate semantic fragment",
                    ));
                }
            }
            let member_rows = query_rows(
                connection,
                &format!(
                    "SELECT fragment_id, node_type_id FROM {SEMANTIC_FRAGMENT_MEMBERS_TABLE} \
                 WHERE graph_id = {}",
                    graph.id.get()
                ),
            )?;
            for row in &member_rows {
                let fragment_id = positive_u32(
                    integer(row, 0, "semantic fragment member fragment")?,
                    "semantic fragment member fragment",
                )?;
                let node_type_id = positive_u32(
                    integer(row, 1, "semantic fragment member type")?,
                    "semantic fragment member type",
                )?;
                if !snapshot.node_types.contains_key(&node_type_id) {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "semantic fragment member type",
                    ));
                }
                let fragment = snapshot.fragments.get_mut(&fragment_id).ok_or(
                    SemanticCatalogError::InvalidCatalogValue("semantic fragment member fragment"),
                )?;
                fragment.member_type_ids.push(node_type_id);
            }
            for fragment in snapshot.fragments.values_mut() {
                fragment.member_type_ids.sort_unstable();
                if fragment
                    .member_type_ids
                    .windows(2)
                    .any(|ids| ids[0] == ids[1])
                {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "duplicate semantic fragment member",
                    ));
                }
            }
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
    let mut fragment_properties = snapshot
        .fragments
        .keys()
        .map(|fragment_id| (*fragment_id, HashSet::new()))
        .collect::<HashMap<_, _>>();
    if has_fragments {
        let fragment_property_rows = query_rows(
            connection,
            &format!(
                "SELECT fragment_id, property_id FROM {SEMANTIC_FRAGMENT_PROPERTIES_TABLE} \
                 WHERE graph_id = {}",
                graph.id.get()
            ),
        )?;
        for row in &fragment_property_rows {
            let fragment_id = positive_u32(
                integer(row, 0, "semantic fragment property fragment")?,
                "semantic fragment property fragment",
            )?;
            let property_id = positive_u32(
                integer(row, 1, "semantic fragment property")?,
                "semantic fragment property",
            )?;
            if !property_names.contains_key(&property_id) {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic fragment property",
                ));
            }
            let properties = fragment_properties.get_mut(&fragment_id).ok_or(
                SemanticCatalogError::InvalidCatalogValue("semantic fragment property fragment"),
            )?;
            if !properties.insert(property_id) {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "duplicate semantic fragment property",
                ));
            }
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

    if has_fragments {
        let mut loaded_fragment_ownership = HashSet::new();
        let fragment_ownership_rows = query_rows(
            connection,
            &format!(
                "SELECT fragment_id, node_type_id, property_id, source_id, column_name \
                 FROM {SEMANTIC_FRAGMENT_OWNERSHIP_TABLE} WHERE graph_id = {}",
                graph.id.get()
            ),
        )?;
        for row in &fragment_ownership_rows {
            let fragment_id = positive_u32(
                integer(row, 0, "semantic fragment owner fragment")?,
                "semantic fragment owner fragment",
            )?;
            let node_type_id = positive_u32(
                integer(row, 1, "semantic fragment owner type")?,
                "semantic fragment owner type",
            )?;
            let property_id = positive_u32(
                integer(row, 2, "semantic fragment owner property")?,
                "semantic fragment owner property",
            )?;
            let source_id = ir::SourceTableId::new(positive_u64(
                integer(row, 3, "semantic fragment owner source")?,
                "semantic fragment owner source",
            )?)
            .map_err(|_| {
                SemanticCatalogError::InvalidCatalogValue("semantic fragment owner source")
            })?;
            let column_name = text(row, 4, "semantic fragment owner column")?.to_owned();
            let fragment = snapshot.fragments.get(&fragment_id).ok_or(
                SemanticCatalogError::InvalidCatalogValue("semantic fragment owner fragment"),
            )?;
            if !fragment.member_type_ids.contains(&node_type_id) {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic fragment owner membership",
                ));
            }
            if !fragment_properties
                .get(&fragment_id)
                .is_some_and(|properties| properties.contains(&property_id))
            {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "undeclared semantic fragment ownership",
                ));
            }
            if !loaded_fragment_ownership.insert((fragment_id, node_type_id, property_id)) {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "duplicate semantic fragment ownership",
                ));
            }
            let property_name = property_names.get(&property_id).ok_or(
                SemanticCatalogError::InvalidCatalogValue("semantic fragment owner property"),
            )?;
            let (type_info, table_name) =
                semantic_owner_and_table(&mut snapshot, graph, "node", node_type_id, source_id)?;
            let schema = connection.current_schema();
            let table = schema.get_table(table_name).ok_or_else(|| {
                SemanticCatalogError::Catalog(CatalogError::SourceTableMissing(
                    table_name.to_owned(),
                ))
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
            let folded_property = fold(property_name);
            if let Some(existing) = type_info.properties.get(&folded_property) {
                if existing.id != owned.id
                    || existing.value_type != owned.value_type
                    || existing.nullability != owned.nullability
                    || !existing.name.eq_ignore_ascii_case(&owned.name)
                    || !existing.column.eq_ignore_ascii_case(&owned.column)
                {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "conflicting semantic fragment ownership",
                    ));
                }
            } else {
                type_info.properties.insert(folded_property, owned);
            }
        }
        for (fragment_id, fragment) in &snapshot.fragments {
            for node_type_id in &fragment.member_type_ids {
                for property_id in &fragment_properties[fragment_id] {
                    if !loaded_fragment_ownership.contains(&(
                        *fragment_id,
                        *node_type_id,
                        *property_id,
                    )) {
                        return Err(SemanticCatalogError::InvalidCatalogValue(
                            "missing semantic fragment ownership",
                        ));
                    }
                }
            }
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
    snapshot.constraints = load_constraint_snapshot(connection, graph, &snapshot)?;
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

    #[test]
    fn fragment_registration_round_trips_through_serde_json() {
        let registration = SemanticFragmentRegistration {
            fragments: vec![SemanticFragment {
                name: "Named".to_owned(),
                properties: vec!["displayName".to_owned()],
                members: vec![SemanticFragmentMember {
                    node_type: "Customer".to_owned(),
                    properties: vec![SemanticProperty {
                        name: "displayName".to_owned(),
                        column: "name".to_owned(),
                    }],
                }],
            }],
        };
        let json = serde_json::to_string(&registration).expect("serialize fragments");
        let decoded = serde_json::from_str::<SemanticFragmentRegistration>(&json)
            .expect("deserialize fragments");

        assert_eq!(registration, decoded);
    }
}
