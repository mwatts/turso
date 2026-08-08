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
use turso_core::{Connection, LimboError};
use turso_graph_ir as ir;

use crate::catalog::{
    execute_internal, integer, load_registered_graph, query_rows, scalar_integer, sql_string, text,
    CatalogError, RegisteredGraph, GENERATIONS_TABLE, SCHEMA_GENERATION_COLUMN,
};
use crate::semantic_constraints::{
    create_constraint_catalog, insert_additive_rows, load_constraint_snapshot,
    rows_for_registration, SemanticConstraintRegistration, SemanticConstraintSnapshot,
    ValidationScope,
};
use crate::transaction::{in_write_transaction, WriteTransactionError};

pub(crate) const SEMANTIC_TYPES_TABLE: &str = "__turso_internal_graph_semantic_types";
pub(crate) const SEMANTIC_PROPERTIES_TABLE: &str = "__turso_internal_graph_semantic_properties";
pub(crate) const SEMANTIC_OWNERSHIP_TABLE: &str = "__turso_internal_graph_semantic_ownership";
pub(crate) const SEMANTIC_ROLE_TABLE: &str = "__turso_internal_graph_semantic_roles";
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
    /// Roles declared for a relationship type, in physical declaration
    /// order. Empty for node types.
    pub roles: Vec<SemanticRole>,
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

    /// Resolve a declared role by name, case-insensitively.
    pub fn role(&self, name: &str) -> Option<&SemanticRole> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }

    /// Roles that must be filled: not marked optional.
    pub fn required_roles(&self) -> impl Iterator<Item = &SemanticRole> {
        self.roles.iter().filter(|role| !role.optional)
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

/// A named role on a relationship type: what its player may be, whether it
/// must be filled, and how many players it may hold.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticRole {
    /// Persisted role identity, shared with the physical registration's
    /// `RegisteredRelationshipRole::role` for the same role.
    pub role: ir::RoleId,
    /// Preserved conceptual spelling.
    pub name: String,
    /// What a player may be. Empty means unconstrained. Node labels and
    /// relationship types are kept in distinct kinds so a role can target a
    /// relationship type (relation-as-player) without colliding with a node
    /// label that happens to share its persisted number.
    pub targets: Vec<ir::RoleTarget>,
    /// Whether the role may be left unfilled.
    pub optional: bool,
    /// How many players the role may hold.
    pub cardinality: ir::RoleCardinality,
}

impl From<crate::lowering::RelationshipRoleLayout> for SemanticRole {
    /// Projects a physical role registration into the schema-free view used
    /// by schemaless catalogs: the `RoleId` is reused directly (never
    /// re-derived), there is no target-type constraint (schemaless imposes
    /// none), and the role is never optional (a physical registration
    /// requires every declared role to be filled).
    fn from(role: crate::lowering::RelationshipRoleLayout) -> Self {
        SemanticRole {
            role: role.role,
            name: role.name,
            targets: Vec::new(),
            optional: false,
            cardinality: role.cardinality,
        }
    }
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
    /// Declared roles, in the order a role's targets are persisted. Must
    /// name every role the physical relationship source declares; a role
    /// with no entry here is recovered from the physical registration with
    /// an unconstrained target list.
    pub roles: Vec<SemanticRoleRegistration>,
    /// Properties owned by this semantic type.
    pub properties: Vec<SemanticProperty>,
}

impl SemanticRelationshipType {
    /// A binary relationship type: two required, single-valued roles named
    /// `start`/`end`, each constrained to `start`/`end`'s node types. This
    /// is a layout of the general role model, not a separate kind.
    pub fn binary(
        name: impl Into<String>,
        source: impl Into<String>,
        start: Vec<String>,
        end: Vec<String>,
        properties: Vec<SemanticProperty>,
    ) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            roles: vec![
                SemanticRoleRegistration {
                    name: "start".to_owned(),
                    targets: start,
                    optional: false,
                    cardinality: SemanticRoleCardinality::One,
                },
                SemanticRoleRegistration {
                    name: "end".to_owned(),
                    targets: end,
                    optional: false,
                    cardinality: SemanticRoleCardinality::One,
                },
            ],
            properties,
        }
    }

    /// Resolve a declared role registration by name, case-insensitively.
    pub fn role(&self, name: &str) -> Option<&SemanticRoleRegistration> {
        self.roles
            .iter()
            .find(|role| role.name.eq_ignore_ascii_case(name))
    }
}

/// A role declared on a `SemanticRelationshipType` registration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticRoleRegistration {
    /// Role name, matched case-insensitively against the physical
    /// registration's role of the same name.
    pub name: String,
    /// Semantic node or relationship type names permitted as a player.
    /// Empty means unconstrained.
    pub targets: Vec<String>,
    /// Whether the role may be left unfilled.
    pub optional: bool,
    /// How many players the role may hold.
    pub cardinality: SemanticRoleCardinality,
}

/// Serializable mirror of `ir::RoleCardinality` for registration payloads
/// (`ir::RoleCardinality` does not derive `Serialize`/`Deserialize`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SemanticRoleCardinality {
    One,
    Many,
}

impl SemanticRoleCardinality {
    fn as_str(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Many => "many",
        }
    }
}

impl From<SemanticRoleCardinality> for ir::RoleCardinality {
    fn from(value: SemanticRoleCardinality) -> Self {
        match value {
            SemanticRoleCardinality::One => ir::RoleCardinality::One,
            SemanticRoleCardinality::Many => ir::RoleCardinality::Many,
        }
    }
}

impl From<ir::RoleCardinality> for SemanticRoleCardinality {
    fn from(value: ir::RoleCardinality) -> Self {
        match value {
            ir::RoleCardinality::One => Self::One,
            ir::RoleCardinality::Many => Self::Many,
        }
    }
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
    /// A role references a type absent from this registration.
    #[error(
        "role `{role}` of relationship type `{relationship_type}` references unknown semantic type `{node_type}`"
    )]
    UnknownRoleTargetType {
        /// Relationship whose role is invalid.
        relationship_type: String,
        /// Declared role name.
        role: String,
        /// Unknown semantic node or relationship type.
        node_type: String,
    },
    /// A semantic role names no role declared on its relationship's
    /// physical source.
    #[error(
        "role `{role}` of relationship type `{relationship_type}` has no matching physical role on source `{source_name}`"
    )]
    UnknownPhysicalRole {
        /// Semantic relationship type.
        relationship_type: String,
        /// Declared role name absent from the physical source.
        role: String,
        /// Physical relationship source consulted.
        source_name: String,
    },
    /// A semantic role target maps to a different physical node source than
    /// the relationship source stores for that role.
    #[error(
        "role `{role}` of relationship type `{relationship_type}` target `{node_type}` maps to source `{actual_source}`, but relationship source `{relationship_source}` requires `{required_source}`"
    )]
    RoleSourceMismatch {
        /// Semantic relationship type.
        relationship_type: Box<str>,
        /// Stored role name.
        role: Box<str>,
        /// Semantic type used by the target.
        node_type: Box<str>,
        /// Node source mapped by that semantic type.
        actual_source: Box<str>,
        /// Relationship source carrying the role.
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
    // Relationship types may target each other regardless of declaration
    // order, so every relationship type name is known before any role's
    // targets are checked.
    let relationship_type_names = registration
        .relationship_types
        .iter()
        .map(|relationship| fold(&relationship.name))
        .collect::<HashSet<_>>();
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
        for role in &relationship.roles {
            require_name("semantic role", &role.name)?;
            for target in &role.targets {
                let folded = fold(target);
                if !node_type_names.contains(&folded)
                    && !fragment_names.contains(&folded)
                    && !relationship_type_names.contains(&folded)
                {
                    return Err(SemanticCatalogError::UnknownRoleTargetType {
                        relationship_type: relationship.name.clone(),
                        role: role.name.clone(),
                        node_type: target.clone(),
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
        // A new constraint has to hold over data that predates it, so this is
        // the pass that establishes the invariant mutation-time validation then
        // narrows itself against.
        updated
            .constraints()
            .validate_state(connection, &ValidationScope::All)?;
        bump_semantic_generation(connection, graph.id.get())
    })
}

fn run_in_registration_transaction(
    connection: &Arc<Connection>,
    operation: impl FnOnce(&Arc<Connection>) -> Result<(), SemanticCatalogError>,
) -> Result<(), SemanticCatalogError> {
    in_write_transaction(connection, REGISTRATION_SAVEPOINT, || operation(connection))
}

impl WriteTransactionError for SemanticCatalogError {
    fn requires_write_transaction() -> Self {
        CatalogError::RequiresWriteTransaction.into()
    }

    fn rollback_failed(cause: Self, rollback: LimboError) -> Self {
        SemanticCatalogError::RollbackFailed {
            cause: Box::new(cause),
            rollback,
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

    let relationship_type_names = registration
        .relationship_types
        .iter()
        .map(|relationship| fold(&relationship.name))
        .collect::<HashSet<_>>();
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
        let mut structural = vec![source.identity_column.as_str()];
        structural.extend(
            source
                .single_valued_roles()
                .map(|role| role.column.as_str()),
        );
        check_owned_columns(
            connection,
            &relationship.name,
            &relationship.properties,
            &source.table,
            &structural,
            &mut property_types,
        )?;
        for role in &relationship.roles {
            let physical_role = source.role_by_name(&role.name).ok_or_else(|| {
                SemanticCatalogError::UnknownPhysicalRole {
                    relationship_type: relationship.name.clone(),
                    role: role.name.clone(),
                    source_name: source.name.clone(),
                }
            })?;
            for target_name in expand_endpoint_names(&role.targets, fragments) {
                if relationship_type_names.contains(&fold(target_name)) {
                    // Relation-as-player: the physical role holds a
                    // placeholder node source (Task 2 does not yet resolve
                    // a role's physical player against a relationship
                    // source), so there is no physical mapping to compare.
                    continue;
                }
                let actual_name = node_sources
                    .get(&fold(target_name))
                    .expect("registration shape validated role target type");
                let actual = graph
                    .node_sources
                    .iter()
                    .find(|node_source| node_source.name.eq_ignore_ascii_case(actual_name))
                    .expect("semantic node source validated above");
                if !physical_role.accepts(actual.id) {
                    return Err(SemanticCatalogError::RoleSourceMismatch {
                        relationship_type: relationship.name.clone().into_boxed_str(),
                        role: role.name.clone().into_boxed_str(),
                        node_type: target_name.to_owned().into_boxed_str(),
                        actual_source: actual.name.clone().into_boxed_str(),
                        relationship_source: source.name.clone().into_boxed_str(),
                        required_source: physical_role
                            .node_sources
                            .iter()
                            .filter_map(|id| {
                                graph
                                    .node_sources
                                    .iter()
                                    .find(|node_source| node_source.id == *id)
                            })
                            .map(|source| source.name.as_str())
                            .collect::<Vec<_>>()
                            .join(" or ")
                            .into_boxed_str(),
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

/// Resolve a role target's persisted `(target_kind, target_id)`. Node and
/// relationship type numbers are independently sequential and only
/// unambiguous together with `target_kind`, so this never merges the two
/// spaces.
fn resolve_target_id(
    node_ids: &HashMap<String, u64>,
    relationship_ids: &HashMap<String, u64>,
    name: &str,
) -> (&'static str, u64) {
    let folded = fold(name);
    if let Some(id) = node_ids.get(&folded) {
        return ("node", *id);
    }
    if let Some(id) = relationship_ids.get(&folded) {
        return ("relation", *id);
    }
    unreachable!("registration shape validated role target type")
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

/// One row per (role, target): `(type_id, role_id, name, optional,
/// cardinality, target_kind, target_id)`. A role with an empty target list
/// gets no rows; it is recovered from the physical registration's role list
/// when loading.
#[derive(Clone, Debug, Eq, PartialEq)]
struct CatalogRows {
    types: Vec<(String, u64, String, u64)>,
    properties: Vec<(u64, String)>,
    ownership: Vec<(String, u64, u64, u64, String)>,
    roles: Vec<(u64, u64, String, bool, String, String, u64)>,
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
        || !existing.roles.is_empty()
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
            && !roles_reference_fragments(registration, fragments)
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
                    roles: Vec::new(),
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

fn roles_reference_fragments(
    registration: &SemanticSchemaRegistration,
    fragments: &SemanticFragmentRegistration,
) -> bool {
    registration.relationship_types.iter().any(|relationship| {
        relationship.roles.iter().any(|role| {
            role.targets.iter().any(|target| {
                fragments
                    .fragments
                    .iter()
                    .any(|fragment| fragment.name.eq_ignore_ascii_case(target))
            })
        })
    })
}

fn bump_semantic_generation(
    connection: &Arc<Connection>,
    graph_id: u64,
) -> Result<(), SemanticCatalogError> {
    // Both counters move: the catalog changed (sessions must reload their
    // `SchemaCatalog`) and so did the view traversal snapshots were built
    // against (labels and types can appear or disappear).
    execute_internal(
        connection,
        format!(
            "UPDATE {GENERATIONS_TABLE} \
             SET generation = generation + 1, \
                 {SCHEMA_GENERATION_COLUMN} = {SCHEMA_GENERATION_COLUMN} + 1 \
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
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_ROLE_TABLE}(\
                graph_id INTEGER NOT NULL, \
                type_id INTEGER NOT NULL, \
                role_id INTEGER NOT NULL, \
                name TEXT NOT NULL COLLATE NOCASE, \
                optional INTEGER NOT NULL CHECK(optional IN (0, 1)), \
                cardinality TEXT NOT NULL CHECK(cardinality IN ('one', 'many')), \
                target_kind TEXT NOT NULL CHECK(target_kind IN ('node', 'relation')), \
                target_id INTEGER NOT NULL, \
                PRIMARY KEY(graph_id, type_id, role_id, target_kind, target_id)\
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
    let mut roles = Vec::new();
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
    // A relationship type's persisted number shares its value range with
    // node types (both start at 1); `target_kind` on each role row keeps
    // the two identity spaces distinct so a role target is never confused
    // between a node label and a relationship type of the same number.
    let relationship_ids = registration
        .relationship_types
        .iter()
        .enumerate()
        .map(|(index, relationship)| (fold(&relationship.name), (index + 1) as u64))
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
        for role in &relationship.roles {
            let physical_role = source
                .role_by_name(&role.name)
                .expect("registration was physically validated");
            let role_id = u64::from(physical_role.role.get());
            let cardinality = role.cardinality.as_str().to_owned();
            for target_name in expand_endpoint_names(&role.targets, fragment_registration) {
                let (target_kind, target_id) =
                    resolve_target_id(&node_ids, &relationship_ids, target_name);
                roles.push((
                    type_id,
                    role_id,
                    role.name.clone(),
                    role.optional,
                    cardinality.clone(),
                    target_kind.to_owned(),
                    target_id,
                ));
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
    roles.sort();
    roles.dedup();
    let mut rows = CatalogRows {
        types,
        properties,
        ownership,
        roles,
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
    let mut roles = query_rows(
        connection,
        &format!(
            "SELECT type_id, role_id, name, optional, cardinality, target_kind, target_id \
             FROM {SEMANTIC_ROLE_TABLE} WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok((
            positive_u64(integer(row, 0, "semantic role type")?, "semantic role type")?,
            positive_u64(integer(row, 1, "semantic role id")?, "semantic role id")?,
            text(row, 2, "semantic role name")?.to_owned(),
            integer(row, 3, "semantic role optional")? != 0,
            text(row, 4, "semantic role cardinality")?.to_owned(),
            text(row, 5, "semantic role target kind")?.to_owned(),
            positive_u64(
                integer(row, 6, "semantic role target id")?,
                "semantic role target id",
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
        roles: std::mem::take(&mut roles),
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
    rows.roles.sort();
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
        for (_, _, name, _, _, _, _) in &mut self.roles {
            *name = fold(name);
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
    for (type_id, role_id, name, optional, cardinality, target_kind, target_id) in &rows.roles {
        execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_ROLE_TABLE}(\
                    graph_id, type_id, role_id, name, optional, cardinality, \
                    target_kind, target_id\
                 ) VALUES (\
                    {graph_id}, {type_id}, {role_id}, {}, {}, {}, {}, {target_id}\
                 )",
                sql_string(name),
                i32::from(*optional),
                sql_string(cardinality),
                sql_string(target_kind)
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
            roles: Vec::new(),
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

    load_roles(connection, graph, &mut snapshot)?;
    snapshot.constraints = load_constraint_snapshot(connection, graph, &snapshot)?;
    Ok(Some(snapshot))
}

/// Grouped, still-unattached role rows for one (relationship type, role).
struct RoleGroup {
    name: String,
    optional: bool,
    cardinality: ir::RoleCardinality,
    targets: Vec<ir::RoleTarget>,
}

/// Load declared roles and attach them to `snapshot`'s relationship types.
///
/// Rows are grouped by `(type_id, role_id)` first because a role with an
/// empty target list has no rows at all; the physical relationship
/// source's role list is joined as the left side so every physical role
/// gets a `SemanticRole` even when the semantic registration left it
/// unconstrained.
fn load_roles(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    snapshot: &mut SemanticSnapshot,
) -> Result<(), SemanticCatalogError> {
    let role_rows = query_rows(
        connection,
        &format!(
            "SELECT type_id, role_id, name, optional, cardinality, target_kind, target_id \
             FROM {SEMANTIC_ROLE_TABLE} WHERE graph_id = {}",
            graph.id.get()
        ),
    )?;
    let mut grouped = HashMap::<(u32, u32), RoleGroup>::new();
    for row in &role_rows {
        let type_id = positive_u32(integer(row, 0, "semantic role type")?, "semantic role type")?;
        if !snapshot.relationship_types.contains_key(&type_id) {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "semantic role type",
            ));
        }
        let role_id = positive_u32(integer(row, 1, "semantic role id")?, "semantic role id")?;
        let name = text(row, 2, "semantic role name")?.to_owned();
        let optional = integer(row, 3, "semantic role optional")? != 0;
        let cardinality = match text(row, 4, "semantic role cardinality")? {
            "one" => ir::RoleCardinality::One,
            "many" => ir::RoleCardinality::Many,
            _ => {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic role cardinality",
                ));
            }
        };
        let target_id = positive_u32(
            integer(row, 6, "semantic role target id")?,
            "semantic role target id",
        )?;
        let target = match text(row, 5, "semantic role target kind")? {
            "node" => {
                if !snapshot.node_types.contains_key(&target_id) {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "semantic role target",
                    ));
                }
                let label = ir::LabelId::new(target_id).map_err(|_| {
                    SemanticCatalogError::InvalidCatalogValue("semantic role target")
                })?;
                ir::RoleTarget::Node(label)
            }
            "relation" => {
                if !snapshot.relationship_types.contains_key(&target_id) {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "semantic role target",
                    ));
                }
                let relationship_type = ir::RelationshipTypeId::new(target_id).map_err(|_| {
                    SemanticCatalogError::InvalidCatalogValue("semantic role target")
                })?;
                ir::RoleTarget::Relation(relationship_type)
            }
            _ => {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "semantic role target kind",
                ));
            }
        };
        match grouped.entry((type_id, role_id)) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(RoleGroup {
                    name,
                    optional,
                    cardinality,
                    targets: vec![target],
                });
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let group = entry.get_mut();
                if group.name != name
                    || group.optional != optional
                    || group.cardinality != cardinality
                {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "semantic role row",
                    ));
                }
                group.targets.push(target);
            }
        }
    }

    for (&type_id, info) in snapshot.relationship_types.iter_mut() {
        let source = graph
            .relationship_sources
            .iter()
            .find(|source| source.id == info.source)
            .ok_or(SemanticCatalogError::InvalidCatalogValue(
                "semantic owner source",
            ))?;
        for physical_role in &source.roles {
            let role_id = physical_role.role.get();
            let role = match grouped.remove(&(type_id, role_id)) {
                Some(group) => SemanticRole {
                    role: physical_role.role,
                    name: group.name,
                    targets: group.targets,
                    optional: group.optional,
                    cardinality: group.cardinality,
                },
                None => SemanticRole {
                    role: physical_role.role,
                    name: physical_role.name.clone(),
                    targets: Vec::new(),
                    optional: false,
                    cardinality: physical_role.cardinality,
                },
            };
            info.roles.push(role);
        }
    }
    if !grouped.is_empty() {
        return Err(SemanticCatalogError::InvalidCatalogValue(
            "semantic role references unknown physical role",
        ));
    }
    Ok(())
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
            .push(SemanticRelationshipType::binary(
                "OWNS",
                "KNOWS",
                vec!["Customer".to_owned()],
                vec!["Ghost".to_owned()],
                vec![],
            ));

        assert!(matches!(
            validate_registration_shape(&registration),
            Err(SemanticCatalogError::UnknownRoleTargetType { node_type, .. })
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

    // --- Role fixtures -----------------------------------------------------
    //
    // `Schema` is built entirely from `&'static` fields so a fixture can be a
    // plain `const`, matched to a fresh in-memory database by
    // `install_semantic_schema`.

    struct NodeSourceSpec {
        name: &'static str,
        table: &'static str,
        identity_column: &'static str,
    }

    struct RoleSourceSpec {
        name: &'static str,
        column: &'static str,
        node_source: &'static str,
        cardinality: ir::RoleCardinality,
    }

    struct RelationshipSourceSpec {
        name: &'static str,
        table: &'static str,
        identity_column: &'static str,
        roles: &'static [RoleSourceSpec],
    }

    struct PropertySpec {
        name: &'static str,
        column: &'static str,
    }

    struct NodeTypeSpec {
        name: &'static str,
        source: &'static str,
        properties: &'static [PropertySpec],
    }

    struct RoleSpec {
        name: &'static str,
        targets: &'static [&'static str],
        optional: bool,
        cardinality: SemanticRoleCardinality,
    }

    struct RelationshipTypeSpec {
        name: &'static str,
        source: &'static str,
        roles: &'static [RoleSpec],
        properties: &'static [PropertySpec],
    }

    struct Schema {
        graph_name: &'static str,
        create_tables_sql: &'static str,
        node_sources: &'static [NodeSourceSpec],
        relationship_sources: &'static [RelationshipSourceSpec],
        node_types: &'static [NodeTypeSpec],
        relationship_types: &'static [RelationshipTypeSpec],
    }

    /// A ternary relationship type: `scribe`/`folio` are required and
    /// single-valued, `witness` is optional and many-valued.
    const TERNARY_SCHEMA: Schema = Schema {
        graph_name: "scriptorium",
        create_tables_sql: "\
            CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
            CREATE TABLE folios(id INTEGER PRIMARY KEY, label TEXT); \
            CREATE TABLE transcriptions(\
                id INTEGER PRIMARY KEY, \
                scribe_id INTEGER, \
                folio_id INTEGER\
            );",
        node_sources: &[
            NodeSourceSpec {
                name: "people_src",
                table: "people",
                identity_column: "id",
            },
            NodeSourceSpec {
                name: "folios_src",
                table: "folios",
                identity_column: "id",
            },
        ],
        relationship_sources: &[RelationshipSourceSpec {
            name: "transcriptions_src",
            table: "transcriptions",
            identity_column: "id",
            roles: &[
                RoleSourceSpec {
                    name: "scribe",
                    column: "scribe_id",
                    node_source: "people_src",
                    cardinality: ir::RoleCardinality::One,
                },
                RoleSourceSpec {
                    name: "folio",
                    column: "folio_id",
                    node_source: "folios_src",
                    cardinality: ir::RoleCardinality::One,
                },
                RoleSourceSpec {
                    name: "witness",
                    column: "",
                    node_source: "people_src",
                    cardinality: ir::RoleCardinality::Many,
                },
            ],
        }],
        node_types: &[
            NodeTypeSpec {
                name: "Person",
                source: "people_src",
                properties: &[],
            },
            NodeTypeSpec {
                name: "Folio",
                source: "folios_src",
                properties: &[],
            },
        ],
        relationship_types: &[RelationshipTypeSpec {
            name: "Transcription",
            source: "transcriptions_src",
            roles: &[
                RoleSpec {
                    name: "scribe",
                    targets: &["Person"],
                    optional: false,
                    cardinality: SemanticRoleCardinality::One,
                },
                RoleSpec {
                    name: "folio",
                    targets: &["Folio"],
                    optional: false,
                    cardinality: SemanticRoleCardinality::One,
                },
                RoleSpec {
                    name: "witness",
                    targets: &["Person"],
                    optional: true,
                    cardinality: SemanticRoleCardinality::Many,
                },
            ],
            properties: &[],
        }],
    };

    /// Same physical shape as `TERNARY_SCHEMA`, but the semantic registration
    /// omits `witness` entirely: no `RoleSpec` names it, so zero rows are
    /// ever persisted for it in `SEMANTIC_ROLE_TABLE`. `load_roles` must
    /// still recover it from the physical registration by left-joining
    /// physical roles against persisted semantic rows; an inner join would
    /// silently drop it.
    const UNCONSTRAINED_ROLE_SCHEMA: Schema = Schema {
        graph_name: "scriptorium",
        create_tables_sql: "\
            CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
            CREATE TABLE folios(id INTEGER PRIMARY KEY, label TEXT); \
            CREATE TABLE transcriptions(\
                id INTEGER PRIMARY KEY, \
                scribe_id INTEGER, \
                folio_id INTEGER\
            );",
        node_sources: &[
            NodeSourceSpec {
                name: "people_src",
                table: "people",
                identity_column: "id",
            },
            NodeSourceSpec {
                name: "folios_src",
                table: "folios",
                identity_column: "id",
            },
        ],
        relationship_sources: &[RelationshipSourceSpec {
            name: "transcriptions_src",
            table: "transcriptions",
            identity_column: "id",
            roles: &[
                RoleSourceSpec {
                    name: "scribe",
                    column: "scribe_id",
                    node_source: "people_src",
                    cardinality: ir::RoleCardinality::One,
                },
                RoleSourceSpec {
                    name: "folio",
                    column: "folio_id",
                    node_source: "folios_src",
                    cardinality: ir::RoleCardinality::One,
                },
                RoleSourceSpec {
                    name: "witness",
                    column: "",
                    node_source: "people_src",
                    cardinality: ir::RoleCardinality::Many,
                },
            ],
        }],
        node_types: &[
            NodeTypeSpec {
                name: "Person",
                source: "people_src",
                properties: &[],
            },
            NodeTypeSpec {
                name: "Folio",
                source: "folios_src",
                properties: &[],
            },
        ],
        relationship_types: &[RelationshipTypeSpec {
            name: "Transcription",
            source: "transcriptions_src",
            // `witness` intentionally has no entry here.
            roles: &[
                RoleSpec {
                    name: "scribe",
                    targets: &["Person"],
                    optional: false,
                    cardinality: SemanticRoleCardinality::One,
                },
                RoleSpec {
                    name: "folio",
                    targets: &["Folio"],
                    optional: false,
                    cardinality: SemanticRoleCardinality::One,
                },
            ],
            properties: &[],
        }],
    };

    /// Relation-as-player: `Citation.cited` targets `Transcription`, itself a
    /// relationship type. The physical `cited` role uses a placeholder node
    /// source since the physical layer cannot yet resolve a role's player
    /// against a relationship source.
    const CITATION_SCHEMA: Schema = Schema {
        graph_name: "scriptorium",
        create_tables_sql: "\
            CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
            CREATE TABLE transcriptions(\
                id INTEGER PRIMARY KEY, \
                start_id INTEGER, \
                end_id INTEGER\
            ); \
            CREATE TABLE citations(id INTEGER PRIMARY KEY, cited_id INTEGER);",
        node_sources: &[NodeSourceSpec {
            name: "people_src",
            table: "people",
            identity_column: "id",
        }],
        relationship_sources: &[
            RelationshipSourceSpec {
                name: "transcriptions_src",
                table: "transcriptions",
                identity_column: "id",
                roles: &[
                    RoleSourceSpec {
                        name: "start",
                        column: "start_id",
                        node_source: "people_src",
                        cardinality: ir::RoleCardinality::One,
                    },
                    RoleSourceSpec {
                        name: "end",
                        column: "end_id",
                        node_source: "people_src",
                        cardinality: ir::RoleCardinality::One,
                    },
                ],
            },
            RelationshipSourceSpec {
                name: "citations_src",
                table: "citations",
                identity_column: "id",
                roles: &[RoleSourceSpec {
                    name: "cited",
                    column: "cited_id",
                    node_source: "people_src",
                    cardinality: ir::RoleCardinality::One,
                }],
            },
        ],
        node_types: &[NodeTypeSpec {
            name: "Person",
            source: "people_src",
            properties: &[],
        }],
        relationship_types: &[
            RelationshipTypeSpec {
                name: "Transcription",
                source: "transcriptions_src",
                roles: &[
                    RoleSpec {
                        name: "start",
                        targets: &["Person"],
                        optional: false,
                        cardinality: SemanticRoleCardinality::One,
                    },
                    RoleSpec {
                        name: "end",
                        targets: &["Person"],
                        optional: false,
                        cardinality: SemanticRoleCardinality::One,
                    },
                ],
                properties: &[],
            },
            RelationshipTypeSpec {
                name: "Citation",
                source: "citations_src",
                roles: &[RoleSpec {
                    name: "cited",
                    targets: &["Transcription"],
                    optional: false,
                    cardinality: SemanticRoleCardinality::One,
                }],
                properties: &[],
            },
        ],
    };

    fn connection() -> Arc<Connection> {
        use turso_core::{Database, MemoryIO, SqliteDialect};
        Database::open_file(
            Arc::new(MemoryIO::new()),
            ":memory:semantic-roles",
            Arc::new(SqliteDialect),
        )
        .expect("open database")
        .connect()
        .expect("connect")
    }

    fn install_semantic_schema(
        connection: &Arc<Connection>,
        schema: Schema,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::catalog::{
            register_graph, GraphRegistration, NodeSourceRegistration,
            RelationshipSourceRegistration, RoleSourceRegistration,
        };

        connection.execute(schema.create_tables_sql)?;
        let graph_registration = GraphRegistration {
            name: schema.graph_name.to_owned(),
            node_sources: schema
                .node_sources
                .iter()
                .map(|source| NodeSourceRegistration {
                    name: source.name.to_owned(),
                    table: source.table.to_owned(),
                    identity_column: source.identity_column.to_owned(),
                })
                .collect(),
            relationship_sources: schema
                .relationship_sources
                .iter()
                .map(|source| RelationshipSourceRegistration {
                    name: source.name.to_owned(),
                    table: source.table.to_owned(),
                    identity_column: source.identity_column.to_owned(),
                    roles: source
                        .roles
                        .iter()
                        .map(|role| RoleSourceRegistration {
                            name: role.name.to_owned(),
                            column: role.column.to_owned(),
                            node_source: role.node_source.to_owned(),
                            cardinality: role.cardinality,
                        })
                        .collect(),
                })
                .collect(),
        };
        register_graph(connection, &graph_registration)?;

        let semantic_registration = SemanticSchemaRegistration {
            node_types: schema
                .node_types
                .iter()
                .map(|node_type| SemanticNodeType {
                    name: node_type.name.to_owned(),
                    source: node_type.source.to_owned(),
                    properties: node_type
                        .properties
                        .iter()
                        .map(|property| SemanticProperty {
                            name: property.name.to_owned(),
                            column: property.column.to_owned(),
                        })
                        .collect(),
                })
                .collect(),
            relationship_types: schema
                .relationship_types
                .iter()
                .map(|relationship_type| SemanticRelationshipType {
                    name: relationship_type.name.to_owned(),
                    source: relationship_type.source.to_owned(),
                    roles: relationship_type
                        .roles
                        .iter()
                        .map(|role| SemanticRoleRegistration {
                            name: role.name.to_owned(),
                            targets: role
                                .targets
                                .iter()
                                .map(|target| target.to_string())
                                .collect(),
                            optional: role.optional,
                            cardinality: role.cardinality,
                        })
                        .collect(),
                    properties: relationship_type
                        .properties
                        .iter()
                        .map(|property| SemanticProperty {
                            name: property.name.to_owned(),
                            column: property.column.to_owned(),
                        })
                        .collect(),
                })
                .collect(),
        };
        register_semantic_schema(connection, schema.graph_name, &semantic_registration)?;
        Ok(())
    }

    fn load_semantic_catalog(
        connection: &Arc<Connection>,
        name: &str,
    ) -> Result<SemanticSnapshot, Box<dyn std::error::Error>> {
        let graph = load_registered_graph(connection, name)?;
        let snapshot = load_semantic_snapshot(connection, &graph)?.expect("semantic schema exists");
        Ok(snapshot)
    }

    #[test]
    fn a_semantic_role_carries_targets_optionality_and_cardinality() {
        let connection = connection();
        install_semantic_schema(&connection, TERNARY_SCHEMA).expect("install schema");
        let catalog = load_semantic_catalog(&connection, "scriptorium").expect("load catalog");

        let transcription = catalog
            .relationship_type("Transcription")
            .expect("Transcription type");
        assert_eq!(transcription.roles.len(), 3);

        let scribe = transcription.role("scribe").expect("scribe role");
        assert!(!scribe.optional);
        assert_eq!(scribe.cardinality, ir::RoleCardinality::One);
        assert_eq!(scribe.targets.len(), 1, "scribe accepts Person only");

        let witnesses = transcription.role("witness").expect("witness role");
        assert!(witnesses.optional);
        assert_eq!(witnesses.cardinality, ir::RoleCardinality::Many);
    }

    #[test]
    fn a_role_may_target_a_relationship_type() {
        // Relation-as-player: a role whose player is itself a relation. A
        // target list that could only hold node labels would make this
        // unrepresentable.
        let connection = connection();
        install_semantic_schema(&connection, CITATION_SCHEMA).expect("install schema");
        let catalog = load_semantic_catalog(&connection, "scriptorium").expect("load catalog");

        let cites = catalog
            .relationship_type("Citation")
            .expect("Citation type");
        let cited = cites.role("cited").expect("cited role");
        assert!(
            cited
                .targets
                .iter()
                .any(|target| matches!(target, ir::RoleTarget::Relation(_))),
            "cited must accept a relation player, got {:?}",
            cited.targets
        );
    }

    #[test]
    fn semantic_role_id_matches_the_physical_role_id() {
        let connection = connection();
        install_semantic_schema(&connection, TERNARY_SCHEMA).expect("install schema");
        let graph = load_registered_graph(&connection, "scriptorium").expect("load graph");
        let snapshot = load_semantic_snapshot(&connection, &graph)
            .expect("load semantic snapshot")
            .expect("semantic schema exists");

        let transcription = snapshot
            .relationship_type("Transcription")
            .expect("Transcription type");
        let source = graph
            .relationship_sources
            .iter()
            .find(|source| source.name.eq_ignore_ascii_case("transcriptions_src"))
            .expect("physical relationship source");

        for role in &transcription.roles {
            let physical_role = source
                .role_by_name(&role.name)
                .expect("every semantic role has a matching physical role");
            assert_eq!(
                role.role, physical_role.role,
                "semantic and physical RoleId must agree for role `{}`",
                role.name
            );
        }
    }

    #[test]
    fn an_unconstrained_role_survives_the_left_join() {
        // `witness` has no `SemanticRoleRegistration` entry at all, so zero
        // rows are ever persisted for it. `load_roles` must recover it from
        // the physical registration rather than silently dropping it.
        let connection = connection();
        install_semantic_schema(&connection, UNCONSTRAINED_ROLE_SCHEMA).expect("install schema");
        let catalog = load_semantic_catalog(&connection, "scriptorium").expect("load catalog");

        let transcription = catalog
            .relationship_type("Transcription")
            .expect("Transcription type");
        assert_eq!(
            transcription.roles.len(),
            3,
            "witness must survive despite having no semantic entry"
        );

        let witness = transcription
            .role("witness")
            .expect("witness role must be present even though it was never declared");
        assert!(
            witness.targets.is_empty(),
            "an omitted role is unconstrained: targets must be empty, got {:?}",
            witness.targets
        );
    }

    #[test]
    fn check_owned_columns_protects_a_third_roles_structural_column() {
        // The old hardcoded start/end-only structural-column derivation
        // would not have protected `folio_id`: TERNARY_SCHEMA's second role
        // is named `folio`, not `start`/`end`. `check_owned_columns` must
        // derive its structural set from every single-valued role.
        use crate::catalog::{
            register_graph, GraphRegistration, NodeSourceRegistration,
            RelationshipSourceRegistration, RoleSourceRegistration,
        };

        let connection = connection();
        connection
            .execute(TERNARY_SCHEMA.create_tables_sql)
            .expect("create tables");
        register_graph(
            &connection,
            &GraphRegistration {
                name: TERNARY_SCHEMA.graph_name.to_owned(),
                node_sources: vec![
                    NodeSourceRegistration {
                        name: "people_src".to_owned(),
                        table: "people".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                    NodeSourceRegistration {
                        name: "folios_src".to_owned(),
                        table: "folios".to_owned(),
                        identity_column: "id".to_owned(),
                    },
                ],
                relationship_sources: vec![RelationshipSourceRegistration {
                    name: "transcriptions_src".to_owned(),
                    table: "transcriptions".to_owned(),
                    identity_column: "id".to_owned(),
                    roles: vec![
                        RoleSourceRegistration {
                            name: "scribe".to_owned(),
                            column: "scribe_id".to_owned(),
                            node_source: "people_src".to_owned(),
                            cardinality: ir::RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "folio".to_owned(),
                            column: "folio_id".to_owned(),
                            node_source: "folios_src".to_owned(),
                            cardinality: ir::RoleCardinality::One,
                        },
                        RoleSourceRegistration {
                            name: "witness".to_owned(),
                            column: String::new(),
                            node_source: "people_src".to_owned(),
                            cardinality: ir::RoleCardinality::Many,
                        },
                    ],
                }],
            },
        )
        .expect("register graph");

        let registration = SemanticSchemaRegistration {
            node_types: vec![
                SemanticNodeType {
                    name: "Person".to_owned(),
                    source: "people_src".to_owned(),
                    properties: vec![],
                },
                SemanticNodeType {
                    name: "Folio".to_owned(),
                    source: "folios_src".to_owned(),
                    properties: vec![],
                },
            ],
            relationship_types: vec![SemanticRelationshipType {
                name: "Transcription".to_owned(),
                source: "transcriptions_src".to_owned(),
                roles: vec![
                    SemanticRoleRegistration {
                        name: "scribe".to_owned(),
                        targets: vec!["Person".to_owned()],
                        optional: false,
                        cardinality: SemanticRoleCardinality::One,
                    },
                    SemanticRoleRegistration {
                        name: "folio".to_owned(),
                        targets: vec!["Folio".to_owned()],
                        optional: false,
                        cardinality: SemanticRoleCardinality::One,
                    },
                    SemanticRoleRegistration {
                        name: "witness".to_owned(),
                        targets: vec!["Person".to_owned()],
                        optional: true,
                        cardinality: SemanticRoleCardinality::Many,
                    },
                ],
                // `folio_id` is the `folio` role's structural column: a
                // third role, distinct from `start`/`end`.
                properties: vec![SemanticProperty {
                    name: "folioId".to_owned(),
                    column: "folio_id".to_owned(),
                }],
            }],
        };

        assert!(
            matches!(
                register_semantic_schema(&connection, "scriptorium", &registration),
                Err(SemanticCatalogError::StructuralColumn { .. })
            ),
            "folio_id is the folio role's structural column and must not be mappable as a property"
        );
    }
}
