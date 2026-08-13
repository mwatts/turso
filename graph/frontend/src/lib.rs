//! Cypher preparation and graph query orchestration for Turso.
//!
//! This crate composes source parsing, binding, graph IR, relational lowering,
//! and traversal services through Turso core APIs. It does not construct VDBE
//! instructions or own database state.

#![forbid(unsafe_code)]

#[cfg(all(feature = "fts", target_family = "wasm"))]
compile_error!("the graph `fts` feature is not supported on wasm targets");

mod binder;
mod catalog;
mod catalog_extend;
mod compiler;
mod ddl;
mod dialect;
mod expand_estimate;
#[cfg(feature = "fts")]
mod fts;
mod functions;
mod graph_expand;
mod inspection;
mod lowering;
mod mutation;
mod procedures;
mod property_physical;
mod schema_catalog;
mod semantic;
mod semantic_constraints;
mod session;
mod snapshot;
mod statement;
mod statement_cache;
mod transaction;

pub use binder::{
    BindError, BoundMutation, BoundQuery, CatalogEntity, GraphCatalogSnapshot, ParameterTypes,
    PropertyResolution, ResolvedNodeType, ResolvedProperty, StatementKind, bind, bind_mutation,
    classify_statement,
};
#[allow(deprecated)]
pub use catalog::{
    CatalogError, GRAPH_CATALOG_VERSION, GraphRegisterOptions, GraphRegistration,
    NodeSourceRegistration, PolymorphicRoleRegistration, RegisteredGraph, RegisteredNodeSource,
    RegisteredRelationshipRole, RegisteredRelationshipSource, RelationshipSourceRegistration,
    RoleSourceRegistration, edge_props_table_name, graph_generation, labels_table_name,
    load_registered_graph, node_props_table_name, prop_dict_table_name, register_graph,
    register_graph_with_options, register_graph_with_polymorphic_roles,
    relationship_type_registry_table_name, relationship_types_table_name,
};
pub use catalog_extend::extend_graph_registration;
pub use compiler::{GraphCompilationCatalog, GraphCompiler, graph_frontend_id};
pub use ddl::{DdlError, execute_graph_ddl};
pub use dialect::{GRAPH_DIALECT_NAME, GraphDialect};
#[cfg(feature = "fts")]
pub use fts::{
    GraphFtsEntityKind, GraphFtsError, GraphFtsIndex, GraphFtsIndexSpec, GraphFtsPropertyWeight,
    GraphFtsTokenizer, MAX_GRAPH_FTS_INDEX_NAME_BYTES, MAX_GRAPH_FTS_PROPERTIES,
};
pub use graph_expand::{GRAPH_EXPAND_TABLE_NAME, install_graph_catalog, register_graph_catalog};
pub use inspection::{
    GraphNodeSourceInspection, GraphPropertyInspection, GraphRelationshipSourceInspection,
    GraphRoleInspection, GraphSchemaInspection, GraphSemanticTypeInspection,
};
pub use lowering::{
    ExpandLowerOptions, LowerError, NodeTableLayout, RelationalCatalogSnapshot,
    RelationshipRoleLayout, RelationshipTableLayout, lower_relational,
    lower_relational_with_options,
};
pub use property_physical::{
    NODE_PROPS_CELL_DDL, PROP_DICT_DDL, PropertyDictEntry, PropertyDictError, PropertyDictionary,
    PropertyPhysical, dict_value_type_name, parse_dict_value_type, resolve_property_physical,
};
pub use turso_graph_runtime::{BuildLimits, TraversalLimits};
// `execute_cypher_mutation` now takes the session's statement cache, which is
// an internal type, so it is reached through `GraphConnection::execute`.
pub use mutation::{
    CLOSED_CREATE_FAST_PATH_HITS, MutationError, MutationSummary, Parameters,
    take_closed_create_fast_path_hit,
};
pub use schema_catalog::SchemaCatalog;
pub use semantic::{
    OwnedProperty, SemanticCatalogError, SemanticFragment, SemanticFragmentInfo,
    SemanticFragmentMember, SemanticFragmentRegistration, SemanticNodeType, SemanticProperty,
    SemanticRelationshipType, SemanticReplaceOutcome, SemanticRole, SemanticRoleCardinality,
    SemanticRoleRegistration, SemanticSchemaRegistration, SemanticSnapshot, SemanticTypeInfo,
    load_semantic_snapshot, register_semantic_constraints, register_semantic_schema,
    register_semantic_schema_with_fragments, replace_semantic_overlay,
};
pub use semantic_constraints::{
    SemanticConstraintRegistration, SemanticEndpoint, SemanticKeyConstraint,
    SemanticPropertyValueConstraint, SemanticRangeBound, SemanticRelationshipCardinality,
    SemanticRequiredProperty, SemanticScalar, SemanticUniqueProperty, SemanticValuePredicate,
};
pub use session::{
    Error, GraphConnection, GraphConnection as Connection, GraphHostMode, open_database,
    open_database_with_io, strip_explain_prefix,
};
pub use snapshot::{
    GraphDiagnostics, NodeCoordinate, PublishOutcome, RelationshipCoordinate, SessionSnapshotStore,
    SnapshotError, SnapshotMetadata, SnapshotPersistenceMode, SnapshotStatus, SnapshotStore,
    SourceIdentity, TraversalSnapshot, build_traversal_snapshot, build_visible_traversal_snapshot,
};
pub use statement::Statement;

/// Full access to the underlying engine, mirroring `turso`'s `core` re-export.
pub use turso_core as core;
pub use turso_core::{
    Database, DatabaseOpts, LimboError, Numeric, OpenFlags, Row, StepResult, Value,
};

pub type Result<T> = std::result::Result<T, Error>;
