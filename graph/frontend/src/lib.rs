//! Cypher preparation and graph query orchestration for Turso.
//!
//! This crate composes source parsing, binding, graph IR, relational lowering,
//! and traversal services through Turso core APIs. It does not construct VDBE
//! instructions or own database state.

#![forbid(unsafe_code)]

mod binder;

pub use binder::{
    bind, BindError, BoundQuery, CatalogEntity, GraphCatalogSnapshot, ParameterTypes,
    ResolvedProperty,
};
