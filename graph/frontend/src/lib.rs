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
mod dialect;
#[cfg(feature = "fts")]
mod fts;
mod functions;
mod graph_expand;
mod lowering;
mod mutation;
mod procedures;
mod schema_catalog;
mod semantic;
mod semantic_constraints;
mod session;
mod snapshot;
mod statement;

pub use binder::{
    bind, bind_mutation, BindError, BoundMutation, BoundQuery, CatalogEntity, GraphCatalogSnapshot,
    ParameterTypes, PropertyResolution, ResolvedNodeType, ResolvedProperty,
};
pub use catalog::{
    graph_generation, labels_table_name, load_registered_graph, register_graph,
    relationship_type_registry_table_name, relationship_types_table_name, CatalogError,
    GraphRegistration, NodeSourceRegistration, RegisteredGraph, RegisteredNodeSource,
    RegisteredRelationshipSource, RelationshipSourceRegistration, GRAPH_CATALOG_VERSION,
};
pub use compiler::{graph_frontend_id, GraphCompilationCatalog, GraphCompiler};
pub use dialect::{GraphDialect, GRAPH_DIALECT_NAME};
#[cfg(feature = "fts")]
pub use fts::{
    GraphFtsEntityKind, GraphFtsError, GraphFtsIndex, GraphFtsIndexSpec, GraphFtsPropertyWeight,
    GraphFtsTokenizer, MAX_GRAPH_FTS_INDEX_NAME_BYTES, MAX_GRAPH_FTS_PROPERTIES,
};
pub use graph_expand::{install_graph_catalog, register_graph_catalog, GRAPH_EXPAND_TABLE_NAME};
pub use lowering::{
    lower_relational, LowerError, NodeTableLayout, RelationalCatalogSnapshot,
    RelationshipTableLayout,
};
pub use mutation::{execute_cypher_mutation, MutationError, MutationSummary, Parameters};
pub use schema_catalog::SchemaCatalog;
pub use semantic::{
    load_semantic_snapshot, register_semantic_constraints, register_semantic_schema,
    register_semantic_schema_with_fragments, EndpointConstraint, OwnedProperty,
    SemanticCatalogError, SemanticFragment, SemanticFragmentInfo, SemanticFragmentMember,
    SemanticFragmentRegistration, SemanticNodeType, SemanticProperty, SemanticRelationshipType,
    SemanticSchemaRegistration, SemanticSnapshot, SemanticTypeInfo,
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
