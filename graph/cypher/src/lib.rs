//! Cypher source text, parser, source AST, spans, and diagnostics.
//!
//! Parser-specific and adapted donor types stay inside this crate. Consumers
//! cross the boundary through Turso-owned graph IR contracts.

#![forbid(unsafe_code)]
