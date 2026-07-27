//! Additive semantic constraints over registered graph types.

use std::{collections::HashSet, sync::Arc};

use regex::Regex;
use serde::{Deserialize, Serialize};
use turso_core::{Connection, Numeric, Value};
use turso_graph_ir as ir;

use crate::{
    catalog::{
        integer, labels_table_name, query_rows, relationship_types_table_name, sql_string, text,
        RegisteredGraph,
    },
    lowering::quoted_identifier,
    semantic::{SemanticCatalogError, SemanticSnapshot, SemanticTypeInfo},
};

pub(crate) const SEMANTIC_PROPERTY_CONSTRAINTS_TABLE: &str =
    "__turso_internal_graph_semantic_property_constraints";
pub(crate) const SEMANTIC_KEY_CONSTRAINTS_TABLE: &str =
    "__turso_internal_graph_semantic_key_constraints";
pub(crate) const SEMANTIC_CARDINALITY_CONSTRAINTS_TABLE: &str =
    "__turso_internal_graph_semantic_cardinality_constraints";

/// Additive semantic constraints for an already-registered semantic schema.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SemanticConstraintRegistration {
    /// Properties that must be non-NULL on every instance of their owner.
    pub required: Vec<SemanticRequiredProperty>,
    /// Composite keys. Every member is required and each tuple is unique.
    pub keys: Vec<SemanticKeyConstraint>,
    /// Properties whose non-NULL values must be unique within their owner.
    pub unique: Vec<SemanticUniqueProperty>,
    /// Per-property range, allowed-value, and regular-expression predicates.
    pub values: Vec<SemanticPropertyValueConstraint>,
    /// Per-node participation counts at stored relationship endpoints.
    pub cardinalities: Vec<SemanticRelationshipCardinality>,
}

/// A required property on one concrete semantic owner type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticRequiredProperty {
    /// Concrete node or relationship type name.
    pub owner: String,
    /// Owned semantic property name.
    pub property: String,
}

/// A composite key on one concrete semantic owner type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticKeyConstraint {
    /// Concrete node or relationship type name.
    pub owner: String,
    /// Non-empty set of owned properties forming the key.
    pub properties: Vec<String>,
}

/// A unique property on one concrete semantic owner type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticUniqueProperty {
    /// Concrete node or relationship type name.
    pub owner: String,
    /// Owned semantic property name.
    pub property: String,
}

/// A value predicate on one concrete semantic owner's property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticPropertyValueConstraint {
    /// Concrete node or relationship type name.
    pub owner: String,
    /// Owned semantic property name.
    pub property: String,
    /// Predicate applied to every non-NULL value.
    pub predicate: SemanticValuePredicate,
}

/// Supported scalar value predicates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SemanticValuePredicate {
    /// Ordered lower and/or upper bounds.
    Range {
        /// Optional lower bound.
        minimum: Option<SemanticRangeBound>,
        /// Optional upper bound.
        maximum: Option<SemanticRangeBound>,
    },
    /// Finite set of accepted values.
    Allowed {
        /// Non-empty accepted value set.
        values: Vec<SemanticScalar>,
    },
    /// Rust-regex expression matched against the text value.
    Regex {
        /// Regular expression pattern.
        pattern: String,
    },
}

/// One inclusive or exclusive range endpoint.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticRangeBound {
    /// Bound value.
    pub value: SemanticScalar,
    /// Whether equality satisfies this endpoint.
    pub inclusive: bool,
}

/// Persistable scalar used by allowed-value and range constraints.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SemanticScalar {
    /// Boolean value, represented by Turso as integer zero or one.
    Boolean(bool),
    /// Signed integer value.
    Integer(i64),
    /// Finite floating-point value.
    Real(f64),
    /// Text value.
    Text(String),
}

/// Stored endpoint selected by a relationship cardinality.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticEndpoint {
    /// Stored start endpoint.
    Start,
    /// Stored end endpoint.
    End,
}

impl SemanticEndpoint {
    fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// Participation count for every permitted node at one relationship endpoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SemanticRelationshipCardinality {
    /// Concrete semantic relationship type.
    pub relationship_type: String,
    /// Stored endpoint whose participation is counted.
    pub endpoint: SemanticEndpoint,
    /// Minimum number of relationships required for every permitted node.
    pub minimum: u32,
    /// Optional maximum number of relationships permitted for every node.
    pub maximum: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum OwnerKind {
    Node,
    Relationship,
}

impl OwnerKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Relationship => "relationship",
        }
    }

    fn parse(value: &str) -> Result<Self, SemanticCatalogError> {
        match value {
            "node" => Ok(Self::Node),
            "relationship" => Ok(Self::Relationship),
            _ => Err(SemanticCatalogError::InvalidCatalogValue(
                "semantic constraint owner kind",
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ResolvedPropertyPredicate {
    Required,
    Unique,
    Value(SemanticValuePredicate),
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedPropertyConstraint {
    owner_kind: OwnerKind,
    owner_name: String,
    source: ir::SourceTableId,
    table: String,
    identity_column: String,
    property_id: ir::PropertyId,
    property_name: String,
    column: String,
    predicate: ResolvedPropertyPredicate,
    regex: Option<Regex>,
}

#[derive(Clone, Debug)]
struct ResolvedKeyConstraint {
    owner_kind: OwnerKind,
    owner_name: String,
    source: ir::SourceTableId,
    table: String,
    identity_column: String,
    properties: Vec<(ir::PropertyId, String, String)>,
}

#[derive(Clone, Debug)]
struct ResolvedCardinalityConstraint {
    relationship_name: String,
    relationship_source: ir::SourceTableId,
    relationship_table: String,
    relationship_identity: String,
    endpoint: SemanticEndpoint,
    endpoint_column: String,
    minimum: u32,
    maximum: Option<u32>,
    node_owners: Vec<ResolvedEndpointOwner>,
}

#[derive(Clone, Debug)]
struct ResolvedEndpointOwner {
    name: String,
    source: ir::SourceTableId,
    table: String,
    identity_column: String,
}

/// Immutable constraints resolved against one semantic and relational catalog.
#[derive(Clone, Debug, Default)]
pub struct SemanticConstraintSnapshot {
    labels_table: String,
    relationship_types_table: String,
    source_qualified_membership: bool,
    property_constraints: Vec<ResolvedPropertyConstraint>,
    keys: Vec<ResolvedKeyConstraint>,
    cardinalities: Vec<ResolvedCardinalityConstraint>,
}

impl SemanticConstraintSnapshot {
    pub(crate) fn property_constraints<'a>(
        &'a self,
        source: Option<ir::SourceTableId>,
        type_names: &'a [String],
        property: ir::PropertyId,
    ) -> impl Iterator<Item = &'a ResolvedPropertyConstraint> + 'a {
        self.property_constraints.iter().filter(move |constraint| {
            source.is_none_or(|source| constraint.source == source)
                && constraint.property_id == property
                && (type_names.is_empty()
                    || type_names
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case(&constraint.owner_name)))
        })
    }

    pub(crate) fn validate_literal(
        &self,
        type_names: &[String],
        property: ir::PropertyId,
        value: &ir::Literal,
    ) -> Result<(), String> {
        let is_null = matches!(value, ir::Literal::Null);
        let value = literal_scalar(value);
        for constraint in self.property_constraints(None, type_names, property) {
            match &constraint.predicate {
                ResolvedPropertyPredicate::Required if is_null => {
                    return Err(format!(
                        "required property `{}.{}` cannot be NULL",
                        constraint.owner_name, constraint.property_name
                    ));
                }
                ResolvedPropertyPredicate::Value(predicate) => {
                    if !is_null {
                        let value = value.as_ref().ok_or_else(|| {
                            format!(
                                "property `{}.{}` has a non-scalar constrained value",
                                constraint.owner_name, constraint.property_name
                            )
                        })?;
                        validate_resolved_value(constraint, predicate, value).map_err(
                            |detail| {
                                format!(
                                    "property `{}.{}` {detail}",
                                    constraint.owner_name, constraint.property_name
                                )
                            },
                        )?;
                    }
                }
                ResolvedPropertyPredicate::Required | ResolvedPropertyPredicate::Unique => {}
            }
        }
        Ok(())
    }

    pub(crate) fn validate_runtime(
        &self,
        source: ir::SourceTableId,
        type_names: &[String],
        property: ir::PropertyId,
        value: &Value,
    ) -> Result<(), String> {
        let is_null = matches!(value, Value::Null);
        let value = runtime_scalar(value);
        for constraint in self.property_constraints(Some(source), type_names, property) {
            match &constraint.predicate {
                ResolvedPropertyPredicate::Required if is_null => {
                    return Err(format!(
                        "required property `{}.{}` cannot be NULL",
                        constraint.owner_name, constraint.property_name
                    ));
                }
                ResolvedPropertyPredicate::Value(predicate) => {
                    if !is_null {
                        let value = value.as_ref().ok_or_else(|| {
                            format!(
                                "property `{}.{}` has a non-scalar constrained value",
                                constraint.owner_name, constraint.property_name
                            )
                        })?;
                        validate_resolved_value(constraint, predicate, value).map_err(
                            |detail| {
                                format!(
                                    "property `{}.{}` {detail}",
                                    constraint.owner_name, constraint.property_name
                                )
                            },
                        )?;
                    }
                }
                ResolvedPropertyPredicate::Required | ResolvedPropertyPredicate::Unique => {}
            }
        }
        Ok(())
    }

    pub(crate) fn validate_state(
        &self,
        connection: &Arc<Connection>,
    ) -> Result<(), SemanticCatalogError> {
        for constraint in &self.property_constraints {
            self.validate_property_state(connection, constraint)?;
        }
        for key in &self.keys {
            self.validate_key_state(connection, key)?;
        }
        for cardinality in &self.cardinalities {
            self.validate_cardinality_state(connection, cardinality)?;
        }
        Ok(())
    }

    fn membership_predicate(
        &self,
        owner_kind: OwnerKind,
        source: ir::SourceTableId,
        identity: &str,
        type_name: &str,
    ) -> String {
        let (table, entity_column, type_column) = match owner_kind {
            OwnerKind::Node => (&self.labels_table, "node_id", "label"),
            OwnerKind::Relationship => (&self.relationship_types_table, "relationship_id", "type"),
        };
        let source = if self.source_qualified_membership {
            format!("membership.source_id = {} AND ", source.get())
        } else {
            String::new()
        };
        format!(
            "EXISTS (SELECT 1 FROM {} AS membership WHERE {source}\
             membership.{} = {} AND membership.{} = {})",
            quoted_identifier(table),
            quoted_identifier(entity_column),
            identity,
            quoted_identifier(type_column),
            sql_string(type_name),
        )
    }

    fn validate_property_state(
        &self,
        connection: &Arc<Connection>,
        constraint: &ResolvedPropertyConstraint,
    ) -> Result<(), SemanticCatalogError> {
        let table = quoted_identifier(&constraint.table);
        let identity = format!("entity.{}", quoted_identifier(&constraint.identity_column));
        let column = format!("entity.{}", quoted_identifier(&constraint.column));
        let membership = self.membership_predicate(
            constraint.owner_kind,
            constraint.source,
            &identity,
            &constraint.owner_name,
        );
        match &constraint.predicate {
            ResolvedPropertyPredicate::Required => {
                let rows = query_rows(
                    connection,
                    &format!(
                        "SELECT {identity} FROM {table} AS entity \
                         WHERE {membership} AND {column} IS NULL LIMIT 1"
                    ),
                )?;
                if !rows.is_empty() {
                    return Err(constraint_violation(
                        &constraint.owner_name,
                        &constraint.property_name,
                        "is required but contains NULL",
                    ));
                }
            }
            ResolvedPropertyPredicate::Unique => {
                let rows = query_rows(
                    connection,
                    &format!(
                        "SELECT {column} FROM {table} AS entity \
                         WHERE {membership} AND {column} IS NOT NULL \
                         GROUP BY {column} HAVING COUNT(*) > 1 LIMIT 1"
                    ),
                )?;
                if !rows.is_empty() {
                    return Err(constraint_violation(
                        &constraint.owner_name,
                        &constraint.property_name,
                        "contains a duplicate value",
                    ));
                }
            }
            ResolvedPropertyPredicate::Value(predicate) => {
                let rows = query_rows(
                    connection,
                    &format!(
                        "SELECT {identity}, {column} FROM {table} AS entity \
                         WHERE {membership} AND {column} IS NOT NULL"
                    ),
                )?;
                for row in rows {
                    let value = row.get(1).and_then(runtime_scalar).ok_or_else(|| {
                        constraint_violation(
                            &constraint.owner_name,
                            &constraint.property_name,
                            "has a value incompatible with its predicate",
                        )
                    })?;
                    if let Err(detail) = validate_resolved_value(constraint, predicate, &value) {
                        return Err(constraint_violation(
                            &constraint.owner_name,
                            &constraint.property_name,
                            &detail,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_key_state(
        &self,
        connection: &Arc<Connection>,
        key: &ResolvedKeyConstraint,
    ) -> Result<(), SemanticCatalogError> {
        let table = quoted_identifier(&key.table);
        let identity = format!("entity.{}", quoted_identifier(&key.identity_column));
        let membership =
            self.membership_predicate(key.owner_kind, key.source, &identity, &key.owner_name);
        let columns = key
            .properties
            .iter()
            .map(|(_, _, column)| format!("entity.{}", quoted_identifier(column)))
            .collect::<Vec<_>>();
        let any_null = columns
            .iter()
            .map(|column| format!("{column} IS NULL"))
            .collect::<Vec<_>>()
            .join(" OR ");
        if !query_rows(
            connection,
            &format!(
                "SELECT {identity} FROM {table} AS entity \
                 WHERE {membership} AND ({any_null}) LIMIT 1"
            ),
        )?
        .is_empty()
        {
            return Err(SemanticCatalogError::ConstraintViolation {
                constraint: format!("key on `{}`", key.owner_name),
                detail: "contains a NULL member".to_owned(),
            });
        }
        let grouping = columns.join(", ");
        if !query_rows(
            connection,
            &format!(
                "SELECT {grouping} FROM {table} AS entity \
                 WHERE {membership} GROUP BY {grouping} HAVING COUNT(*) > 1 LIMIT 1"
            ),
        )?
        .is_empty()
        {
            return Err(SemanticCatalogError::ConstraintViolation {
                constraint: format!("key on `{}`", key.owner_name),
                detail: "contains a duplicate tuple".to_owned(),
            });
        }
        Ok(())
    }

    fn validate_cardinality_state(
        &self,
        connection: &Arc<Connection>,
        cardinality: &ResolvedCardinalityConstraint,
    ) -> Result<(), SemanticCatalogError> {
        let relationship_table = quoted_identifier(&cardinality.relationship_table);
        let relationship_identity = format!(
            "relationship.{}",
            quoted_identifier(&cardinality.relationship_identity)
        );
        let relationship_membership = self.membership_predicate(
            OwnerKind::Relationship,
            cardinality.relationship_source,
            &relationship_identity,
            &cardinality.relationship_name,
        );
        for owner in &cardinality.node_owners {
            let node_table = quoted_identifier(&owner.table);
            let node_identity = format!("node.{}", quoted_identifier(&owner.identity_column));
            let node_membership = self.membership_predicate(
                OwnerKind::Node,
                owner.source,
                &node_identity,
                &owner.name,
            );
            let endpoint = format!(
                "relationship.{}",
                quoted_identifier(&cardinality.endpoint_column)
            );
            let maximum = cardinality
                .maximum
                .map(|maximum| format!(" OR COUNT({relationship_identity}) > {maximum}"))
                .unwrap_or_default();
            let rows = query_rows(
                connection,
                &format!(
                    "SELECT {node_identity}, COUNT({relationship_identity}) \
                     FROM {node_table} AS node \
                     LEFT JOIN {relationship_table} AS relationship \
                       ON {endpoint} = {node_identity} AND {relationship_membership} \
                     WHERE {node_membership} \
                     GROUP BY {node_identity} \
                     HAVING COUNT({relationship_identity}) < {}{maximum} LIMIT 1",
                    cardinality.minimum,
                ),
            )?;
            if !rows.is_empty() {
                return Err(SemanticCatalogError::ConstraintViolation {
                    constraint: format!(
                        "{} endpoint cardinality on `{}`",
                        cardinality.endpoint.as_str(),
                        cardinality.relationship_name
                    ),
                    detail: format!(
                        "node type `{}` violates {}..{} participation",
                        owner.name,
                        cardinality.minimum,
                        cardinality
                            .maximum
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "*".to_owned())
                    ),
                });
            }
        }
        Ok(())
    }
}

fn constraint_violation(owner: &str, property: &str, detail: &str) -> SemanticCatalogError {
    SemanticCatalogError::ConstraintViolation {
        constraint: format!("property `{owner}.{property}`"),
        detail: detail.to_owned(),
    }
}

fn literal_scalar(value: &ir::Literal) -> Option<SemanticScalar> {
    match value {
        ir::Literal::Null => None,
        ir::Literal::Boolean(value) => Some(SemanticScalar::Boolean(*value)),
        ir::Literal::Integer(value) => Some(SemanticScalar::Integer(*value)),
        ir::Literal::Real(value) => Some(SemanticScalar::Real(*value)),
        ir::Literal::Text(value) => Some(SemanticScalar::Text(value.clone())),
        ir::Literal::Bytes(_) => None,
    }
}

fn runtime_scalar(value: &Value) -> Option<SemanticScalar> {
    match value {
        Value::Null | Value::Blob(_) => None,
        Value::Numeric(Numeric::Integer(value)) => Some(SemanticScalar::Integer(*value)),
        Value::Numeric(Numeric::Float(value)) => Some(SemanticScalar::Real(f64::from(*value))),
        Value::Text(value) => Some(SemanticScalar::Text(value.to_string())),
    }
}

fn validate_scalar(
    predicate: &SemanticValuePredicate,
    value: &SemanticScalar,
) -> Result<(), String> {
    match predicate {
        SemanticValuePredicate::Range { minimum, maximum } => {
            if let Some(minimum) = minimum {
                let ordering = compare_scalar(value, &minimum.value)
                    .ok_or_else(|| "is not comparable with its range".to_owned())?;
                if ordering == std::cmp::Ordering::Less
                    || (!minimum.inclusive && ordering == std::cmp::Ordering::Equal)
                {
                    return Err("is below its configured range".to_owned());
                }
            }
            if let Some(maximum) = maximum {
                let ordering = compare_scalar(value, &maximum.value)
                    .ok_or_else(|| "is not comparable with its range".to_owned())?;
                if ordering == std::cmp::Ordering::Greater
                    || (!maximum.inclusive && ordering == std::cmp::Ordering::Equal)
                {
                    return Err("is above its configured range".to_owned());
                }
            }
            Ok(())
        }
        SemanticValuePredicate::Allowed { values } => values
            .iter()
            .any(|candidate| scalar_equal(value, candidate))
            .then_some(())
            .ok_or_else(|| "is not in its allowed-value set".to_owned()),
        SemanticValuePredicate::Regex { pattern } => {
            let SemanticScalar::Text(value) = value else {
                return Err("is not text required by its regex".to_owned());
            };
            let regex = Regex::new(pattern)
                .map_err(|error| format!("uses an invalid persisted regex: {error}"))?;
            regex
                .is_match(value)
                .then_some(())
                .ok_or_else(|| "does not match its regular expression".to_owned())
        }
    }
}

fn validate_resolved_value(
    constraint: &ResolvedPropertyConstraint,
    predicate: &SemanticValuePredicate,
    value: &SemanticScalar,
) -> Result<(), String> {
    if let SemanticValuePredicate::Regex { .. } = predicate {
        let SemanticScalar::Text(value) = value else {
            return Err("is not text required by its regex".to_owned());
        };
        return constraint
            .regex
            .as_ref()
            .expect("loaded regex constraints are compiled")
            .is_match(value)
            .then_some(())
            .ok_or_else(|| "does not match its regular expression".to_owned());
    }
    validate_scalar(predicate, value)
}

fn compare_scalar(left: &SemanticScalar, right: &SemanticScalar) -> Option<std::cmp::Ordering> {
    match (left, right) {
        (SemanticScalar::Integer(left), SemanticScalar::Integer(right)) => left.partial_cmp(right),
        (SemanticScalar::Integer(left), SemanticScalar::Real(right)) => {
            (*left as f64).partial_cmp(right)
        }
        (SemanticScalar::Real(left), SemanticScalar::Integer(right)) => {
            left.partial_cmp(&(*right as f64))
        }
        (SemanticScalar::Real(left), SemanticScalar::Real(right)) => left.partial_cmp(right),
        (SemanticScalar::Text(left), SemanticScalar::Text(right)) => left.partial_cmp(right),
        _ => None,
    }
}

fn scalar_equal(left: &SemanticScalar, right: &SemanticScalar) -> bool {
    compare_scalar(left, right).is_some_and(|ordering| ordering == std::cmp::Ordering::Equal)
        || left == right
        || matches!(
            (left, right),
            (SemanticScalar::Boolean(false), SemanticScalar::Integer(0))
                | (SemanticScalar::Integer(0), SemanticScalar::Boolean(false))
                | (SemanticScalar::Boolean(true), SemanticScalar::Integer(1))
                | (SemanticScalar::Integer(1), SemanticScalar::Boolean(true))
        )
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct PropertyConstraintRow {
    kind: String,
    owner_kind: String,
    owner_type_id: u32,
    property_id: u32,
    payload: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct KeyConstraintRow {
    owner_kind: String,
    owner_type_id: u32,
    property_ids: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct CardinalityConstraintRow {
    relationship_type_id: u32,
    endpoint: String,
    minimum: u32,
    maximum: Option<u32>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConstraintRows {
    property: Vec<PropertyConstraintRow>,
    keys: Vec<KeyConstraintRow>,
    cardinality: Vec<CardinalityConstraintRow>,
}

impl ConstraintRows {
    pub(crate) fn is_empty(&self) -> bool {
        self.property.is_empty() && self.keys.is_empty() && self.cardinality.is_empty()
    }
}

pub(crate) fn rows_for_registration(
    registration: &SemanticConstraintRegistration,
    semantic: &SemanticSnapshot,
) -> Result<ConstraintRows, SemanticCatalogError> {
    let mut rows = ConstraintRows::default();
    let mut identities = HashSet::new();
    for required in &registration.required {
        let (kind, owner, property) =
            resolve_property(semantic, &required.owner, &required.property)?;
        insert_property_row(
            &mut rows,
            &mut identities,
            "required",
            kind,
            owner,
            property.id,
            "{}".to_owned(),
        )?;
    }
    for unique in &registration.unique {
        let (kind, owner, property) = resolve_property(semantic, &unique.owner, &unique.property)?;
        insert_property_row(
            &mut rows,
            &mut identities,
            "unique",
            kind,
            owner,
            property.id,
            "{}".to_owned(),
        )?;
    }
    for value in &registration.values {
        let predicate = canonicalize_predicate(&value.predicate)?;
        validate_predicate_shape(&predicate)?;
        let (kind, owner, property) = resolve_property(semantic, &value.owner, &value.property)?;
        validate_predicate_type(
            &predicate,
            &property.value_type,
            &value.owner,
            &value.property,
        )?;
        let constraint_kind = match &predicate {
            SemanticValuePredicate::Range { .. } => "range",
            SemanticValuePredicate::Allowed { .. } => "allowed",
            SemanticValuePredicate::Regex { .. } => "regex",
        };
        insert_property_row(
            &mut rows,
            &mut identities,
            constraint_kind,
            kind,
            owner,
            property.id,
            serde_json::to_string(&predicate)
                .map_err(|_| SemanticCatalogError::InvalidCatalogValue("constraint payload"))?,
        )?;
    }
    let mut key_identities = HashSet::new();
    for key in &registration.keys {
        let (kind, owner) = resolve_owner(semantic, &key.owner)?;
        if key.properties.is_empty() {
            return Err(SemanticCatalogError::InvalidConstraint {
                constraint: format!("key on `{}`", key.owner),
                detail: "must contain at least one property".to_owned(),
            });
        }
        let mut property_ids = Vec::with_capacity(key.properties.len());
        let mut seen = HashSet::new();
        for property_name in &key.properties {
            let property = owner.property(property_name).ok_or_else(|| {
                SemanticCatalogError::UnknownConstraintProperty {
                    owner: key.owner.clone(),
                    property: property_name.clone(),
                }
            })?;
            if !seen.insert(property.id) {
                return Err(SemanticCatalogError::InvalidConstraint {
                    constraint: format!("key on `{}`", key.owner),
                    detail: format!("duplicates property `{property_name}`"),
                });
            }
            property_ids.push(property.id.get());
        }
        property_ids.sort_unstable();
        let signature = property_ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        if !key_identities.insert((kind, owner.type_id, signature.clone())) {
            return Err(SemanticCatalogError::DuplicateConstraint {
                constraint: format!("key on `{}`", key.owner),
            });
        }
        rows.keys.push(KeyConstraintRow {
            owner_kind: kind.as_str().to_owned(),
            owner_type_id: owner.type_id,
            property_ids: signature,
        });
    }
    let mut cardinality_identities = HashSet::new();
    for cardinality in &registration.cardinalities {
        let relationship = semantic
            .relationship_type(&cardinality.relationship_type)
            .ok_or_else(|| SemanticCatalogError::UnknownConstraintOwner {
                owner: cardinality.relationship_type.clone(),
            })?;
        if cardinality
            .maximum
            .is_some_and(|maximum| cardinality.minimum > maximum)
        {
            return Err(SemanticCatalogError::InvalidConstraint {
                constraint: format!(
                    "{} cardinality on `{}`",
                    cardinality.endpoint.as_str(),
                    cardinality.relationship_type
                ),
                detail: "minimum exceeds maximum".to_owned(),
            });
        }
        let endpoints: Vec<u32> = relationship
            .role(cardinality.endpoint.as_str())
            .map(|role| {
                role.targets
                    .iter()
                    .filter_map(|target| match target {
                        ir::RoleTarget::Node(label) => Some(label.get()),
                        ir::RoleTarget::Relation(_) => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        if endpoints.is_empty() {
            return Err(SemanticCatalogError::InvalidConstraint {
                constraint: format!(
                    "{} cardinality on `{}`",
                    cardinality.endpoint.as_str(),
                    cardinality.relationship_type
                ),
                detail: "requires at least one permitted semantic endpoint type".to_owned(),
            });
        }
        if !cardinality_identities.insert((relationship.type_id, cardinality.endpoint)) {
            return Err(SemanticCatalogError::DuplicateConstraint {
                constraint: format!(
                    "{} cardinality on `{}`",
                    cardinality.endpoint.as_str(),
                    cardinality.relationship_type
                ),
            });
        }
        rows.cardinality.push(CardinalityConstraintRow {
            relationship_type_id: relationship.type_id,
            endpoint: cardinality.endpoint.as_str().to_owned(),
            minimum: cardinality.minimum,
            maximum: cardinality.maximum,
        });
    }
    rows.property.sort();
    rows.keys.sort();
    rows.cardinality.sort();
    Ok(rows)
}

fn insert_property_row(
    rows: &mut ConstraintRows,
    identities: &mut HashSet<(String, OwnerKind, u32, u32)>,
    kind: &str,
    owner_kind: OwnerKind,
    owner: &SemanticTypeInfo,
    property: ir::PropertyId,
    payload: String,
) -> Result<(), SemanticCatalogError> {
    if !identities.insert((kind.to_owned(), owner_kind, owner.type_id, property.get())) {
        return Err(SemanticCatalogError::DuplicateConstraint {
            constraint: format!("{kind} constraint on `{}`", owner.name),
        });
    }
    rows.property.push(PropertyConstraintRow {
        kind: kind.to_owned(),
        owner_kind: owner_kind.as_str().to_owned(),
        owner_type_id: owner.type_id,
        property_id: property.get(),
        payload,
    });
    Ok(())
}

fn validate_predicate_shape(
    predicate: &SemanticValuePredicate,
) -> Result<(), SemanticCatalogError> {
    let invalid = |detail: &str| SemanticCatalogError::InvalidConstraint {
        constraint: "property value predicate".to_owned(),
        detail: detail.to_owned(),
    };
    match predicate {
        SemanticValuePredicate::Range { minimum, maximum } => {
            if minimum.is_none() && maximum.is_none() {
                return Err(invalid("range must contain a minimum or maximum"));
            }
            for bound in minimum.iter().chain(maximum) {
                if matches!(bound.value, SemanticScalar::Boolean(_)) {
                    return Err(invalid("range bounds must be numeric or text"));
                }
                validate_scalar_shape(&bound.value)?;
            }
            if let (Some(minimum), Some(maximum)) = (minimum, maximum) {
                let ordering = compare_scalar(&minimum.value, &maximum.value)
                    .ok_or_else(|| invalid("range bounds must be comparable"))?;
                if ordering == std::cmp::Ordering::Greater
                    || (ordering == std::cmp::Ordering::Equal
                        && (!minimum.inclusive || !maximum.inclusive))
                {
                    return Err(invalid("range is empty"));
                }
            }
        }
        SemanticValuePredicate::Allowed { values } => {
            if values.is_empty() {
                return Err(invalid("allowed-value set must not be empty"));
            }
            for value in values {
                validate_scalar_shape(value)?;
            }
        }
        SemanticValuePredicate::Regex { pattern } => {
            Regex::new(pattern).map_err(|error| invalid(&format!("invalid regex: {error}")))?;
        }
    }
    Ok(())
}

fn canonicalize_predicate(
    predicate: &SemanticValuePredicate,
) -> Result<SemanticValuePredicate, SemanticCatalogError> {
    let mut predicate = predicate.clone();
    if let SemanticValuePredicate::Allowed { values } = &mut predicate {
        let mut canonical = values
            .drain(..)
            .map(|value| {
                let encoded = serde_json::to_string(&value)
                    .map_err(|_| SemanticCatalogError::InvalidCatalogValue("constraint scalar"))?;
                Ok((encoded, value))
            })
            .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
        canonical.sort_by(|left, right| left.0.cmp(&right.0));
        canonical.dedup_by(|left, right| left.0 == right.0);
        values.extend(canonical.into_iter().map(|(_, value)| value));
    }
    Ok(predicate)
}

fn validate_predicate_type(
    predicate: &SemanticValuePredicate,
    value_type: &ir::ValueType,
    owner: &str,
    property: &str,
) -> Result<(), SemanticCatalogError> {
    let value_type = match value_type {
        ir::ValueType::Custom { base, .. } => base.as_ref(),
        value_type => value_type,
    };
    let scalar_compatible = |value: &SemanticScalar| {
        matches!(value_type, ir::ValueType::Any)
            || matches!(
                (value_type, value),
                (
                    ir::ValueType::Boolean | ir::ValueType::Integer,
                    SemanticScalar::Boolean(_) | SemanticScalar::Integer(_)
                ) | (
                    ir::ValueType::Real,
                    SemanticScalar::Integer(_) | SemanticScalar::Real(_)
                ) | (ir::ValueType::Text, SemanticScalar::Text(_))
            )
    };
    let compatible = match predicate {
        SemanticValuePredicate::Range { minimum, maximum } => minimum
            .iter()
            .chain(maximum)
            .all(|bound| scalar_compatible(&bound.value)),
        SemanticValuePredicate::Allowed { values } => values.iter().all(scalar_compatible),
        SemanticValuePredicate::Regex { .. } => {
            matches!(value_type, ir::ValueType::Any | ir::ValueType::Text)
        }
    };
    if compatible {
        Ok(())
    } else {
        Err(SemanticCatalogError::InvalidConstraint {
            constraint: format!("property `{owner}.{property}`"),
            detail: format!("predicate is incompatible with {value_type:?}"),
        })
    }
}

fn validate_scalar_shape(value: &SemanticScalar) -> Result<(), SemanticCatalogError> {
    if matches!(value, SemanticScalar::Real(value) if !value.is_finite()) {
        return Err(SemanticCatalogError::InvalidConstraint {
            constraint: "scalar value".to_owned(),
            detail: "floating-point values must be finite".to_owned(),
        });
    }
    Ok(())
}

fn resolve_owner<'a>(
    semantic: &'a SemanticSnapshot,
    owner: &str,
) -> Result<(OwnerKind, &'a SemanticTypeInfo), SemanticCatalogError> {
    semantic
        .node_type(owner)
        .map(|owner| (OwnerKind::Node, owner))
        .or_else(|| {
            semantic
                .relationship_type(owner)
                .map(|owner| (OwnerKind::Relationship, owner))
        })
        .ok_or_else(|| SemanticCatalogError::UnknownConstraintOwner {
            owner: owner.to_owned(),
        })
}

fn resolve_property<'a>(
    semantic: &'a SemanticSnapshot,
    owner: &str,
    property: &str,
) -> Result<
    (
        OwnerKind,
        &'a SemanticTypeInfo,
        &'a crate::semantic::OwnedProperty,
    ),
    SemanticCatalogError,
> {
    let (kind, owner_info) = resolve_owner(semantic, owner)?;
    let property_info = owner_info.property(property).ok_or_else(|| {
        SemanticCatalogError::UnknownConstraintProperty {
            owner: owner.to_owned(),
            property: property.to_owned(),
        }
    })?;
    Ok((kind, owner_info, property_info))
}

pub(crate) fn create_constraint_catalog(
    connection: &Arc<Connection>,
) -> Result<(), SemanticCatalogError> {
    for ddl in [
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_PROPERTY_CONSTRAINTS_TABLE}(\
                graph_id INTEGER NOT NULL, \
                kind TEXT NOT NULL CHECK(kind IN ('required', 'unique', 'range', 'allowed', 'regex')), \
                owner_kind TEXT NOT NULL CHECK(owner_kind IN ('node', 'relationship')), \
                owner_type_id INTEGER NOT NULL CHECK(owner_type_id > 0), \
                property_id INTEGER NOT NULL CHECK(property_id > 0), \
                payload TEXT NOT NULL, \
                PRIMARY KEY(graph_id, kind, owner_kind, owner_type_id, property_id)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_KEY_CONSTRAINTS_TABLE}(\
                graph_id INTEGER NOT NULL, \
                owner_kind TEXT NOT NULL CHECK(owner_kind IN ('node', 'relationship')), \
                owner_type_id INTEGER NOT NULL CHECK(owner_type_id > 0), \
                property_ids TEXT NOT NULL, \
                PRIMARY KEY(graph_id, owner_kind, owner_type_id, property_ids)\
            )"
        ),
        format!(
            "CREATE TABLE IF NOT EXISTS {SEMANTIC_CARDINALITY_CONSTRAINTS_TABLE}(\
                graph_id INTEGER NOT NULL, \
                relationship_type_id INTEGER NOT NULL CHECK(relationship_type_id > 0), \
                endpoint TEXT NOT NULL CHECK(endpoint IN ('start', 'end')), \
                minimum INTEGER NOT NULL CHECK(minimum >= 0), \
                maximum INTEGER CHECK(maximum IS NULL OR maximum >= minimum), \
                PRIMARY KEY(graph_id, relationship_type_id, endpoint)\
            )"
        ),
    ] {
        crate::catalog::execute_internal(connection, ddl)?;
    }
    Ok(())
}

pub(crate) fn load_constraint_rows(
    connection: &Arc<Connection>,
    graph_id: u64,
) -> Result<ConstraintRows, SemanticCatalogError> {
    if connection
        .current_schema()
        .get_table(SEMANTIC_PROPERTY_CONSTRAINTS_TABLE)
        .is_none()
    {
        return Ok(ConstraintRows::default());
    }
    let mut property = query_rows(
        connection,
        &format!(
            "SELECT kind, owner_kind, owner_type_id, property_id, payload \
             FROM {SEMANTIC_PROPERTY_CONSTRAINTS_TABLE} WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok(PropertyConstraintRow {
            kind: text(row, 0, "constraint kind")?.to_owned(),
            owner_kind: text(row, 1, "constraint owner kind")?.to_owned(),
            owner_type_id: positive_u32(integer(row, 2, "constraint owner type")?)?,
            property_id: positive_u32(integer(row, 3, "constraint property")?)?,
            payload: text(row, 4, "constraint payload")?.to_owned(),
        })
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let mut keys = query_rows(
        connection,
        &format!(
            "SELECT owner_kind, owner_type_id, property_ids \
             FROM {SEMANTIC_KEY_CONSTRAINTS_TABLE} WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        Ok(KeyConstraintRow {
            owner_kind: text(row, 0, "key owner kind")?.to_owned(),
            owner_type_id: positive_u32(integer(row, 1, "key owner type")?)?,
            property_ids: text(row, 2, "key properties")?.to_owned(),
        })
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    let mut cardinality = query_rows(
        connection,
        &format!(
            "SELECT relationship_type_id, endpoint, minimum, maximum \
             FROM {SEMANTIC_CARDINALITY_CONSTRAINTS_TABLE} WHERE graph_id = {graph_id}"
        ),
    )?
    .iter()
    .map(|row| {
        let minimum = nonnegative_u32(integer(row, 2, "cardinality minimum")?)?;
        let maximum = match row.get(3) {
            Some(Value::Null) => None,
            Some(_) => Some(nonnegative_u32(integer(row, 3, "cardinality maximum")?)?),
            None => {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "cardinality maximum",
                ));
            }
        };
        Ok(CardinalityConstraintRow {
            relationship_type_id: positive_u32(integer(row, 0, "cardinality relationship type")?)?,
            endpoint: text(row, 1, "cardinality endpoint")?.to_owned(),
            minimum,
            maximum,
        })
    })
    .collect::<Result<Vec<_>, SemanticCatalogError>>()?;
    property.sort();
    keys.sort();
    cardinality.sort();
    Ok(ConstraintRows {
        property,
        keys,
        cardinality,
    })
}

pub(crate) fn insert_additive_rows(
    connection: &Arc<Connection>,
    graph_id: u64,
    requested: &ConstraintRows,
) -> Result<bool, SemanticCatalogError> {
    let existing = load_constraint_rows(connection, graph_id)?;
    let mut changed = false;
    for row in &requested.property {
        if let Some(current) = existing.property.iter().find(|current| {
            current.kind == row.kind
                && current.owner_kind == row.owner_kind
                && current.owner_type_id == row.owner_type_id
                && current.property_id == row.property_id
        }) {
            if current != row {
                return Err(SemanticCatalogError::ConstraintEvolutionUnsupported {
                    constraint: format!(
                        "{} constraint on semantic type {} property {}",
                        row.kind, row.owner_type_id, row.property_id
                    ),
                });
            }
            continue;
        }
        crate::catalog::execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_PROPERTY_CONSTRAINTS_TABLE}(\
                    graph_id, kind, owner_kind, owner_type_id, property_id, payload\
                 ) VALUES ({graph_id}, {}, {}, {}, {}, {})",
                sql_string(&row.kind),
                sql_string(&row.owner_kind),
                row.owner_type_id,
                row.property_id,
                sql_string(&row.payload),
            ),
        )?;
        changed = true;
    }
    for row in &requested.keys {
        if existing.keys.contains(row) {
            continue;
        }
        crate::catalog::execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_KEY_CONSTRAINTS_TABLE}(\
                    graph_id, owner_kind, owner_type_id, property_ids\
                 ) VALUES ({graph_id}, {}, {}, {})",
                sql_string(&row.owner_kind),
                row.owner_type_id,
                sql_string(&row.property_ids),
            ),
        )?;
        changed = true;
    }
    for row in &requested.cardinality {
        if let Some(current) = existing.cardinality.iter().find(|current| {
            current.relationship_type_id == row.relationship_type_id
                && current.endpoint == row.endpoint
        }) {
            if current != row {
                return Err(SemanticCatalogError::ConstraintEvolutionUnsupported {
                    constraint: format!(
                        "{} cardinality on semantic relationship type {}",
                        row.endpoint, row.relationship_type_id
                    ),
                });
            }
            continue;
        }
        let maximum = row
            .maximum
            .map(|value| value.to_string())
            .unwrap_or_else(|| "NULL".to_owned());
        crate::catalog::execute_internal(
            connection,
            format!(
                "INSERT INTO {SEMANTIC_CARDINALITY_CONSTRAINTS_TABLE}(\
                    graph_id, relationship_type_id, endpoint, minimum, maximum\
                 ) VALUES ({graph_id}, {}, {}, {}, {maximum})",
                row.relationship_type_id,
                sql_string(&row.endpoint),
                row.minimum,
            ),
        )?;
        changed = true;
    }
    Ok(changed)
}

pub(crate) fn load_constraint_snapshot(
    connection: &Arc<Connection>,
    graph: &RegisteredGraph,
    semantic: &SemanticSnapshot,
) -> Result<SemanticConstraintSnapshot, SemanticCatalogError> {
    let rows = load_constraint_rows(connection, graph.id.get())?;
    let source_qualified_membership = connection
        .current_schema()
        .get_table(&labels_table_name(graph.id))
        .is_some_and(|table| table.get_column_by_name("source_id").is_some());
    let mut snapshot = SemanticConstraintSnapshot {
        labels_table: labels_table_name(graph.id),
        relationship_types_table: relationship_types_table_name(graph.id),
        source_qualified_membership,
        ..SemanticConstraintSnapshot::default()
    };
    for row in rows.property {
        let owner_kind = OwnerKind::parse(&row.owner_kind)?;
        let owner = owner_by_id(semantic, owner_kind, row.owner_type_id)?;
        let property_id = ir::PropertyId::new(row.property_id)
            .map_err(|_| SemanticCatalogError::InvalidCatalogValue("constraint property"))?;
        let property =
            owner
                .property_by_id(property_id)
                .ok_or(SemanticCatalogError::InvalidCatalogValue(
                    "constraint property ownership",
                ))?;
        let (table, identity_column) = source_layout(graph, owner_kind, owner.source)?;
        let predicate = match row.kind.as_str() {
            "required" => ResolvedPropertyPredicate::Required,
            "unique" => ResolvedPropertyPredicate::Unique,
            "range" | "allowed" | "regex" => {
                let predicate: SemanticValuePredicate = serde_json::from_str(&row.payload)
                    .map_err(|_| SemanticCatalogError::InvalidCatalogValue("constraint payload"))?;
                validate_predicate_shape(&predicate)?;
                let expected = match row.kind.as_str() {
                    "range" => matches!(predicate, SemanticValuePredicate::Range { .. }),
                    "allowed" => matches!(predicate, SemanticValuePredicate::Allowed { .. }),
                    "regex" => matches!(predicate, SemanticValuePredicate::Regex { .. }),
                    _ => unreachable!(),
                };
                if !expected {
                    return Err(SemanticCatalogError::InvalidCatalogValue(
                        "constraint predicate kind",
                    ));
                }
                validate_predicate_type(
                    &predicate,
                    &property.value_type,
                    &owner.name,
                    &property.name,
                )?;
                ResolvedPropertyPredicate::Value(predicate)
            }
            _ => {
                return Err(SemanticCatalogError::InvalidCatalogValue("constraint kind"));
            }
        };
        let regex = match &predicate {
            ResolvedPropertyPredicate::Value(SemanticValuePredicate::Regex { pattern }) => Some(
                Regex::new(pattern)
                    .map_err(|_| SemanticCatalogError::InvalidCatalogValue("constraint regex"))?,
            ),
            _ => None,
        };
        snapshot
            .property_constraints
            .push(ResolvedPropertyConstraint {
                owner_kind,
                owner_name: owner.name.clone(),
                source: owner.source,
                table,
                identity_column,
                property_id,
                property_name: property.name.clone(),
                column: property.column.clone(),
                predicate,
                regex,
            });
    }
    for row in rows.keys {
        let owner_kind = OwnerKind::parse(&row.owner_kind)?;
        let owner = owner_by_id(semantic, owner_kind, row.owner_type_id)?;
        let mut properties = Vec::new();
        for id in row.property_ids.split(',') {
            let id = id
                .parse::<u32>()
                .ok()
                .and_then(|id| ir::PropertyId::new(id).ok())
                .ok_or(SemanticCatalogError::InvalidCatalogValue("key property"))?;
            let property =
                owner
                    .property_by_id(id)
                    .ok_or(SemanticCatalogError::InvalidCatalogValue(
                        "key property ownership",
                    ))?;
            properties.push((id, property.name.clone(), property.column.clone()));
        }
        if properties.is_empty() {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "empty key constraint",
            ));
        }
        let (table, identity_column) = source_layout(graph, owner_kind, owner.source)?;
        snapshot.keys.push(ResolvedKeyConstraint {
            owner_kind,
            owner_name: owner.name.clone(),
            source: owner.source,
            table,
            identity_column,
            properties,
        });
    }
    for row in rows.cardinality {
        let relationship = semantic
            .relationship_type_by_id(
                ir::RelationshipTypeId::new(row.relationship_type_id).map_err(|_| {
                    SemanticCatalogError::InvalidCatalogValue("cardinality relationship type")
                })?,
            )
            .ok_or(SemanticCatalogError::InvalidCatalogValue(
                "cardinality relationship type",
            ))?;
        let endpoint = match row.endpoint.as_str() {
            "start" => SemanticEndpoint::Start,
            "end" => SemanticEndpoint::End,
            _ => {
                return Err(SemanticCatalogError::InvalidCatalogValue(
                    "cardinality endpoint",
                ));
            }
        };
        let relationship_source = graph
            .relationship_sources
            .iter()
            .find(|source| source.id == relationship.source)
            .ok_or(SemanticCatalogError::InvalidCatalogValue(
                "cardinality relationship source",
            ))?;
        let endpoint_type_ids: Vec<u32> = relationship
            .role(match endpoint {
                SemanticEndpoint::Start => "start",
                SemanticEndpoint::End => "end",
            })
            .map(|role| {
                role.targets
                    .iter()
                    .filter_map(|target| match target {
                        ir::RoleTarget::Node(label) => Some(label.get()),
                        ir::RoleTarget::Relation(_) => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        if endpoint_type_ids.is_empty() {
            return Err(SemanticCatalogError::InvalidCatalogValue(
                "cardinality endpoint types",
            ));
        }
        let mut node_owners = Vec::new();
        for type_id in endpoint_type_ids {
            let owner = semantic
                .node_type_by_id(ir::LabelId::new(type_id).map_err(|_| {
                    SemanticCatalogError::InvalidCatalogValue("cardinality endpoint type")
                })?)
                .ok_or(SemanticCatalogError::InvalidCatalogValue(
                    "cardinality endpoint type",
                ))?;
            let source = graph
                .node_sources
                .iter()
                .find(|source| source.id == owner.source)
                .ok_or(SemanticCatalogError::InvalidCatalogValue(
                    "cardinality endpoint source",
                ))?;
            node_owners.push(ResolvedEndpointOwner {
                name: owner.name.clone(),
                source: owner.source,
                table: source.table.clone(),
                identity_column: source.identity_column.clone(),
            });
        }
        snapshot.cardinalities.push(ResolvedCardinalityConstraint {
            relationship_name: relationship.name.clone(),
            relationship_source: relationship.source,
            relationship_table: relationship_source.table.clone(),
            relationship_identity: relationship_source.identity_column.clone(),
            endpoint,
            endpoint_column: match endpoint {
                SemanticEndpoint::Start => relationship_source
                    .role_by_name("start")
                    .ok_or(SemanticCatalogError::InvalidCatalogValue(
                        "cardinality endpoint column",
                    ))?
                    .column
                    .clone(),
                SemanticEndpoint::End => relationship_source
                    .role_by_name("end")
                    .ok_or(SemanticCatalogError::InvalidCatalogValue(
                        "cardinality endpoint column",
                    ))?
                    .column
                    .clone(),
            },
            minimum: row.minimum,
            maximum: row.maximum,
            node_owners,
        });
    }
    Ok(snapshot)
}

fn owner_by_id(
    semantic: &SemanticSnapshot,
    kind: OwnerKind,
    id: u32,
) -> Result<&SemanticTypeInfo, SemanticCatalogError> {
    match kind {
        OwnerKind::Node => semantic.node_type_by_id(
            ir::LabelId::new(id)
                .map_err(|_| SemanticCatalogError::InvalidCatalogValue("constraint owner"))?,
        ),
        OwnerKind::Relationship => semantic.relationship_type_by_id(
            ir::RelationshipTypeId::new(id)
                .map_err(|_| SemanticCatalogError::InvalidCatalogValue("constraint owner"))?,
        ),
    }
    .ok_or(SemanticCatalogError::InvalidCatalogValue(
        "constraint owner",
    ))
}

fn source_layout(
    graph: &RegisteredGraph,
    kind: OwnerKind,
    source: ir::SourceTableId,
) -> Result<(String, String), SemanticCatalogError> {
    match kind {
        OwnerKind::Node => graph
            .node_sources
            .iter()
            .find(|entry| entry.id == source)
            .map(|entry| (entry.table.clone(), entry.identity_column.clone())),
        OwnerKind::Relationship => graph
            .relationship_sources
            .iter()
            .find(|entry| entry.id == source)
            .map(|entry| (entry.table.clone(), entry.identity_column.clone())),
    }
    .ok_or(SemanticCatalogError::InvalidCatalogValue(
        "constraint source",
    ))
}

fn positive_u32(value: i64) -> Result<u32, SemanticCatalogError> {
    u32::try_from(value).ok().filter(|value| *value > 0).ok_or(
        SemanticCatalogError::InvalidCatalogValue("positive constraint integer"),
    )
}

fn nonnegative_u32(value: i64) -> Result<u32, SemanticCatalogError> {
    u32::try_from(value)
        .map_err(|_| SemanticCatalogError::InvalidCatalogValue("nonnegative constraint integer"))
}
