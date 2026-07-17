use thiserror::Error;

use crate::BindingId;

/// An opaque identity was constructed from the reserved zero value.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("{kind} identity must be non-zero, got {value}")]
pub struct InvalidId {
    pub kind: &'static str,
    pub value: u64,
}

/// A bound plan violates a graph IR invariant.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PlanError {
    #[error("binding name must not be empty")]
    EmptyBindingName,
    #[error("result column name must not be empty")]
    EmptyResultColumnName,
    #[error("duplicate binding id: {0}")]
    DuplicateBindingId(BindingId),
    #[error("duplicate binding name: {0}")]
    DuplicateBindingName(String),
    #[error("result column references unknown binding: {0}")]
    UnknownResultBinding(BindingId),
    #[error("UNION requires at least two inputs")]
    UnionNeedsMultipleInputs,
    #[error("UNION input {input} has {actual} columns; expected {expected}")]
    UnionShapeMismatch {
        input: usize,
        expected: usize,
        actual: usize,
    },
}
