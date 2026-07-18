//! Connection-local statement frontend compilation.
//!
//! A [`PreparedSource`] is the data-only recipe retained by bytecode so every
//! compilation path can recover the same source language. Compiler services
//! live in a [`Connection`](crate::Connection) registry and never enter a
//! prepared program.

use std::fmt;

use thiserror::Error;
use turso_parser::ast::{Cmd, Stmt};

use crate::Result;

/// Stable identity for a statement frontend registered on a connection.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FrontendId(String);

impl FrontendId {
    /// Create a non-empty frontend identifier.
    pub fn new(value: impl Into<String>) -> std::result::Result<Self, FrontendError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(FrontendError::InvalidId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FrontendId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Typed failures at the frontend registry and prepared-source boundary.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum FrontendError {
    #[error("frontend id must not be empty")]
    InvalidId,
    #[error("frontend compiler is not registered: {frontend}")]
    CompilerNotRegistered { frontend: FrontendId },
    #[error("frontend compiler is already registered: {frontend}")]
    CompilerAlreadyRegistered { frontend: FrontendId },
    #[error("frontend compiler returned no statement during reprepare: {frontend}")]
    CompilerReturnedNoStatement { frontend: FrontendId },
    #[error(
        "frontend {frontend} consumed invalid byte offset {consumed} for {source_len}-byte source"
    )]
    InvalidConsumedBytes {
        frontend: FrontendId,
        consumed: usize,
        source_len: usize,
    },
}

/// A compiled frontend statement and the prerequisites its initial prepare
/// must execute first, produced in one pass over the source.
#[derive(Debug)]
pub struct FrontendCompilation {
    /// Statements that must execute before `cmd` is first prepared, e.g. the
    /// implicit `CREATE SEQUENCE` a PostgreSQL `SERIAL` column requires.
    ///
    /// Prerequisites run exactly once, during the initial
    /// [`Connection::prepare_frontend`](crate::Connection::prepare_frontend).
    /// Recompiles (schema-change reprepare, cross-process schema retry)
    /// discard them: a recompile can run mid-step while the statement holds
    /// pager locks, where executing DDL is unsafe. Prerequisites must
    /// therefore be idempotent prepare-time side effects only.
    pub prerequisites: Vec<Stmt>,
    pub cmd: Option<Cmd>,
    /// Bytes of the source consumed; same contract as [`crate::Dialect::parse`].
    pub consumed: usize,
}

/// Converts source-language text into a Turso parser command.
///
/// Implementations may parse and bind frontend syntax, but must not retain
/// connection or prepared-program state. Compilation returns the main
/// command together with any prerequisites so the source is parsed once.
pub trait FrontendCompiler: Send + Sync + 'static {
    fn compile(&self, source: &str) -> Result<FrontendCompilation>;
}

/// Data-only recipe used for initial compilation and every recompile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreparedSource {
    Dialect {
        source: String,
    },
    Frontend {
        frontend: FrontendId,
        source: String,
    },
}

impl PreparedSource {
    pub(crate) fn dialect(source: impl Into<String>) -> Self {
        Self::Dialect {
            source: source.into(),
        }
    }

    pub(crate) fn frontend(frontend: FrontendId, source: impl Into<String>) -> Self {
        Self::Frontend {
            frontend,
            source: source.into(),
        }
    }

    pub fn source(&self) -> &str {
        match self {
            Self::Dialect { source } | Self::Frontend { source, .. } => source,
        }
    }

    pub fn frontend_id(&self) -> Option<&FrontendId> {
        match self {
            Self::Dialect { .. } => None,
            Self::Frontend { frontend, .. } => Some(frontend),
        }
    }

    pub(crate) fn with_source(&self, source: impl Into<String>) -> Self {
        let source = source.into();
        match self {
            Self::Dialect { .. } => Self::Dialect { source },
            Self::Frontend { frontend, .. } => Self::Frontend {
                frontend: frontend.clone(),
                source,
            },
        }
    }
}
