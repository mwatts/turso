//! Derived adjacency, traversal, and path services for Turso graph queries.
//!
//! Runtime state is rebuildable from canonical Turso rows. This crate does not
//! own storage, transactions, or frontend syntax.

#![forbid(unsafe_code)]
