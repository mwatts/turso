//! Prepared statements kept for reuse, keyed on the exact SQL text.
//!
//! Every `Connection::prepare` is a full parse, plan and codegen. A graph
//! mutation runs a small set of statements whose text does not change between
//! mutations — the catalog freshness probe and the constraint validation
//! queries are built from the schema, not from the row being written — so a
//! bootstrap loop recompiles the same SQL once per statement, forever.
//!
//! ## Why a cached statement cannot go stale
//!
//! Core re-prepares on its own. `Statement::step` compares the connection's
//! prepare-context generation before running and re-prepares when a setting,
//! attach or extension registration has moved it, and a schema change surfaces
//! from the VDBE as `SchemaUpdated`, which also re-prepares and retries. So a
//! statement held across a `CREATE TABLE` or a `PRAGMA` is not wrong, only
//! occasionally recompiled — which is exactly the behaviour a caller that
//! prepared fresh would have got anyway.
//!
//! ## Why the cache does not live on `Connection`
//!
//! A `Statement` owns an `Arc<Connection>`. A connection holding its own
//! statements is a reference cycle, and the connection would never drop. The
//! cache belongs to something the caller owns and drops — here, the session.

use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use turso_core::{Connection, Statement, Value};

use crate::catalog::CatalogError;

/// How many distinct SQL strings one cache keeps statements for.
///
/// The working set is small and fixed: one freshness probe plus a couple of
/// queries per constraint in scope. The bound is here so a session that runs
/// unusual one-off SQL cannot grow the map without limit.
const CAPACITY: usize = 64;

/// Statements a session reuses instead of recompiling.
#[derive(Default)]
pub(crate) struct StatementCache {
    statements: Mutex<HashMap<String, Statement>>,
}

impl StatementCache {
    /// Run `sql` and collect its rows, reusing a prepared statement when this
    /// cache already holds one for that exact text.
    pub(crate) fn query_rows(
        &self,
        connection: &Arc<Connection>,
        sql: &str,
    ) -> Result<Vec<Vec<Value>>, CatalogError> {
        // The statement leaves the map for the duration of the call. A
        // re-entrant query on the same SQL then prepares its own rather than
        // stepping a statement that is already mid-execution.
        let cached = self.statements.lock().remove(sql);
        let mut statement = match cached {
            Some(statement) => statement,
            None => connection.prepare(sql)?,
        };
        let rows = statement.run_collect_rows();
        // A statement that failed part-way can hold execution state that reset
        // is not guaranteed to unwind, so only a clean run earns a place back
        // in the cache.
        if rows.is_ok() && statement.reset().is_ok() {
            let mut statements = self.statements.lock();
            if statements.len() >= CAPACITY {
                statements.clear();
            }
            statements.insert(sql.to_owned(), statement);
        }
        Ok(rows?)
    }
}
