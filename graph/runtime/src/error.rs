use thiserror::Error;
use turso_graph_ir::{NodeId, RelationshipId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitKind {
    Nodes,
    Edges,
    Paths,
    Hops,
    Work,
    Memory,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RuntimeError {
    #[error("duplicate graph node {0}")]
    DuplicateNode(NodeId),
    #[error("duplicate graph relationship {0}")]
    DuplicateRelationship(RelationshipId),
    #[error("relationship {relationship} references unknown {endpoint} node {node}")]
    UnknownEndpoint {
        relationship: RelationshipId,
        endpoint: &'static str,
        node: NodeId,
    },
    #[error("graph node {0} is not present in this snapshot")]
    UnknownNode(NodeId),
    #[error("invalid hop range {min}..{max}")]
    InvalidHopRange { min: u32, max: u32 },
    #[error("{kind:?} resource limit exceeded: limit {limit}")]
    LimitExceeded { kind: LimitKind, limit: u64 },
    #[error("graph operation was cancelled")]
    Cancelled,
    #[error("path cost overflowed u64")]
    CostOverflow,
    #[error("unsupported path combination: {reason}")]
    UnsupportedPathCombination { reason: &'static str },
    #[error("path algorithm {algorithm} is sound but not implemented")]
    PathAlgorithmNotImplemented { algorithm: &'static str },
    #[error("variable-length traversal over a relation with {arity} roles must name a role pair")]
    RolePairRequired { arity: usize },
}

impl From<crate::PathPolicyError> for RuntimeError {
    fn from(error: crate::PathPolicyError) -> Self {
        match error {
            crate::PathPolicyError::Unsupported { reason, .. } => {
                Self::UnsupportedPathCombination { reason }
            }
            crate::PathPolicyError::RolePairRequired { arity } => Self::RolePairRequired { arity },
        }
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;
