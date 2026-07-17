//! Cypher preparation and graph query orchestration for Turso.
//!
//! This crate composes source parsing, binding, graph IR, relational lowering,
//! and traversal services through Turso core APIs. It does not construct VDBE
//! instructions or own database state.

#![forbid(unsafe_code)]

mod binder;
mod catalog;
mod compiler;
mod graph_expand;
mod lowering;
mod snapshot;

pub use binder::{
    bind, BindError, BoundQuery, CatalogEntity, GraphCatalogSnapshot, ParameterTypes,
    ResolvedProperty,
};
pub use catalog::{
    graph_generation, load_registered_graph, register_graph, CatalogError, GraphRegistration,
    NodeSourceRegistration, RegisteredGraph, RegisteredNodeSource, RegisteredRelationshipSource,
    RelationshipSourceRegistration, GRAPH_CATALOG_VERSION,
};
pub use compiler::{graph_frontend_id, GraphCompilationCatalog, GraphCompiler};
pub use graph_expand::{install_graph_catalog, register_graph_catalog, GRAPH_EXPAND_TABLE_NAME};
pub use lowering::{
    lower_relational, LowerError, NodeTableLayout, RelationalCatalogSnapshot,
    RelationshipTableLayout,
};
pub use snapshot::{
    build_traversal_snapshot, NodeCoordinate, PublishOutcome, RelationshipCoordinate,
    SnapshotError, SnapshotStore, SourceIdentity, TraversalSnapshot,
};
