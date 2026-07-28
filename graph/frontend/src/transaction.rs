//! The write-transaction guard every mutating path in this crate needs.
//!
//! Registration, semantic schema changes, Cypher mutations, FTS admin, and
//! graph DDL all have the same requirement: run a group of statements so that
//! either all of them take effect or none do, whether the caller is in
//! autocommit mode or already inside their own transaction. That produces one
//! shape, repeated — and the repetition had already drifted before this module
//! existed.
//!
//! Two things make the shape non-obvious enough to be worth naming:
//!
//! - Inside an open user transaction a top-level `BEGIN` would fail or
//!   interfere with the caller's transaction state, so the group scopes itself
//!   with a savepoint there and commits or rolls back with the outer
//!   transaction.
//! - These paths run internal (nested) statements, which cannot upgrade a read
//!   transaction to a write transaction. A deferred transaction that has not
//!   yet written must be rejected up front rather than panicking in the engine.

use std::sync::Arc;

use turso_core::{Connection, LimboError};

/// Lets each caller keep its own error type while sharing the guard.
///
/// Both variants are already spelled out per-module because they appear in
/// public error enums; this trait only asks for the two constructors, so no
/// caller's error surface changes.
pub(crate) trait WriteTransactionError: From<LimboError> + Sized {
    /// The caller is in an open transaction that has not written yet.
    fn requires_write_transaction() -> Self;
    /// The operation failed and so did the unwind, losing the original cause
    /// unless it is carried along.
    fn rollback_failed(cause: Self, rollback: LimboError) -> Self;
}

/// Runs `operation` as one atomic group, using `savepoint` when the caller
/// already has a transaction open.
///
/// `savepoint` must be unique per call site: a nested guard reusing an outer
/// guard's name would release the outer scope early.
pub(crate) fn in_write_transaction<T, E>(
    connection: &Arc<Connection>,
    savepoint: &str,
    operation: impl FnOnce() -> Result<T, E>,
) -> Result<T, E>
where
    E: WriteTransactionError,
{
    if !connection.get_auto_commit() && !connection.in_write_transaction() {
        return Err(E::requires_write_transaction());
    }

    // The savepoint unwind releases as well as rolls back: `ROLLBACK TO` alone
    // leaves the savepoint on the stack, which would leak a scope into the
    // caller's transaction.
    let (enter, commit, unwind) = if connection.get_auto_commit() {
        (
            "BEGIN IMMEDIATE".to_owned(),
            "COMMIT".to_owned(),
            "ROLLBACK".to_owned(),
        )
    } else {
        (
            format!("SAVEPOINT {savepoint}"),
            format!("RELEASE {savepoint}"),
            format!("ROLLBACK TO {savepoint}; RELEASE {savepoint}"),
        )
    };

    connection.execute(enter)?;
    let result = operation().and_then(|value| {
        connection.execute(commit)?;
        Ok(value)
    });
    match result {
        Ok(value) => Ok(value),
        Err(cause) => match connection.execute(unwind) {
            Ok(()) => Err(cause),
            Err(rollback) => Err(E::rollback_failed(cause, rollback)),
        },
    }
}
