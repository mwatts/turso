//! Cypher conformance, regression, and lifecycle performance test support.

// Denied rather than forbidden: cypherbench::peak_rss_mb carries the one
// scoped allow for the getrusage FFI call.
#![deny(unsafe_code)]

pub mod age;
pub mod cypherbench;
pub mod dynamic_catalog;
pub mod grafeo;
pub mod history;
pub mod identity;
pub mod manifest;
pub mod model;
pub mod performance;
pub mod query_cache;
pub mod report;
pub mod runner;
pub mod rust_donor;
pub mod tck;
