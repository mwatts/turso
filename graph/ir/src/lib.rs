//! Turso-owned graph identities, catalog contracts, and bound query IR.
//!
//! This crate is the dependency root for graph language and runtime crates. It
//! must not expose parser, storage-engine, PostgreSQL, or donor implementation
//! types.

#![forbid(unsafe_code)]
