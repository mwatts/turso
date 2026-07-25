//! Derived adjacency, traversal, and path services for Turso graph queries.
//!
//! Runtime state is rebuildable from canonical Turso rows. This crate does not
//! own storage, transactions, or frontend syntax.

#![forbid(unsafe_code)]

mod csr;
mod error;
mod limits;
mod path_policy;
mod shortest;
mod traversal;

pub use csr::{EdgeInput, Graph, Neighbor, NeighborCursor};
pub use error::{LimitKind, RuntimeError, RuntimeResult};
pub use limits::{BuildLimits, Cancellation, NeverCancelled, TraversalLimits};
pub use path_policy::{
    resolve_path_algorithm, PathAlgorithm, PathPolicyError, PathSelector, WeightClass,
    PATH_POLICY_VERSION,
};
pub use shortest::{shortest_path, weighted_shortest_path, ShortestPathRequest};
pub use traversal::{
    traverse, Path, TraversalCursor, TraversalOrder, TraversalRequest, TraversalStep, Uniqueness,
};
