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
mod schema_catalog;
mod semantic;
mod semantic_constraints;
mod session;
mod snapshot;
mod statement;
mod statement_cache;
mod transaction;

pub use binder::{
    bind, bind_mutation, classify_statement, BindError, BoundMutation, BoundQuery, CatalogEntity,
    GraphCatalogSnapshot, ParameterTypes, PropertyResolution, ResolvedNodeType, ResolvedProperty,
    StatementKind,
};
pub use catalog::{
    graph_generation, labels_table_name, load_registered_graph, register_graph,
    register_graph_with_polymorphic_roles, relationship_type_registry_table_name,
    relationship_types_table_name, CatalogError, GraphRegistration, NodeSourceRegistration,
    PolymorphicRoleRegistration, RegisteredGraph, RegisteredNodeSource, RegisteredRelationshipRole,
    RegisteredRelationshipSource, RelationshipSourceRegistration, RoleSourceRegistration,
    GRAPH_CATALOG_VERSION,
};
pub use compiler::{graph_frontend_id, GraphCompilationCatalog, GraphCompiler};
pub use ddl::{execute_graph_ddl, DdlError};
pub use dialect::{GraphDialect, GRAPH_DIALECT_NAME};
#[cfg(feature = "fts")]
pub use fts::{
    GraphFtsEntityKind, GraphFtsError, GraphFtsIndex, GraphFtsIndexSpec, GraphFtsPropertyWeight,
    GraphFtsTokenizer, MAX_GRAPH_FTS_INDEX_NAME_BYTES, MAX_GRAPH_FTS_PROPERTIES,
};
pub use graph_expand::{install_graph_catalog, register_graph_catalog, GRAPH_EXPAND_TABLE_NAME};
pub use inspection::{
    GraphNodeSourceInspection, GraphPropertyInspection, GraphRelationshipSourceInspection,
    GraphRoleInspection, GraphSchemaInspection, GraphSemanticTypeInspection,
};
pub use lowering::{
    lower_relational, lower_relational_with_options, ExpandLowerOptions, LowerError,
    NodeTableLayout, RelationalCatalogSnapshot, RelationshipRoleLayout, RelationshipTableLayout,
};
pub use turso_graph_runtime::{BuildLimits, TraversalLimits};
// `execute_cypher_mutation` now takes the session's statement cache, which is
// an internal type, so it is reached through `GraphConnection::execute`.
pub use mutation::{
    take_closed_create_fast_path_hit, MutationError, MutationSummary, Parameters,
    CLOSED_CREATE_FAST_PATH_HITS,
};
pub use schema_catalog::SchemaCatalog;
pub use semantic::{
    load_semantic_snapshot, register_semantic_constraints, register_semantic_schema,
    register_semantic_schema_with_fragments, OwnedProperty, SemanticCatalogError, SemanticFragment,
    SemanticFragmentInfo, SemanticFragmentMember, SemanticFragmentRegistration, SemanticNodeType,
    SemanticProperty, SemanticRelationshipType, SemanticRole, SemanticRoleCardinality,
    SemanticRoleRegistration, SemanticSchemaRegistration, SemanticSnapshot, SemanticTypeInfo,
};
pub use semantic_constraints::{
    SemanticConstraintRegistration, SemanticEndpoint, SemanticKeyConstraint,
    SemanticPropertyValueConstraint, SemanticRangeBound, SemanticRelationshipCardinality,
    SemanticRequiredProperty, SemanticScalar, SemanticUniqueProperty, SemanticValuePredicate,
};
pub use session::{
    open_database, open_database_with_io, strip_explain_prefix, Error, GraphConnection,
    GraphConnection as Connection, GraphHostMode,
};
pub use snapshot::{
    build_traversal_snapshot, build_visible_traversal_snapshot, GraphDiagnostics, NodeCoordinate,
    PublishOutcome, RelationshipCoordinate, SessionSnapshotStore, SnapshotError, SnapshotMetadata,
    SnapshotPersistenceMode, SnapshotStatus, SnapshotStore, SourceIdentity, TraversalSnapshot,
};
pub use statement::Statement;

/// Full access to the underlying engine, mirroring `turso`'s `core` re-export.
pub use turso_core as core;
pub use turso_core::{
    Database, DatabaseOpts, LimboError, Numeric, OpenFlags, Row, StepResult, Value,
};

pub type Result<T> = std::result::Result<T, Error>;
