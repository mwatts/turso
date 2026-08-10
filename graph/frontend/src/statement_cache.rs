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

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use parking_lot::Mutex;
use turso_core::{Connection, Statement, Value};

use crate::catalog::CatalogError;

/// How many distinct SQL strings one cache keeps statements for.
///
/// Sized for a freshness probe, a few constraint checks, and the stable
/// mutation helpers (label/type junction, parameterized SET/DELETE/spill)
/// that a bootstrap loop repeats. Value-inlined INSERT text still cannot hit
/// the cache; that is intentional.
const CAPACITY: usize = 96;

enum PrepareKind {
    /// Catalog / constraint SQL via root prepare.
    Root,
    /// Mutation helpers via `prepare_internal` (InternalHelper origin).
    Internal,
}

struct Inner {
    /// Statements keyed on exact SQL text.
    map: HashMap<String, Statement>,
    /// Least-recently-used keys first. Updated on successful re-insert after a
    /// clean run so a hot probe stays resident when the map is full.
    order: VecDeque<String>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
        }
    }
}

/// Statements a session reuses instead of recompiling.
#[derive(Default)]
pub(crate) struct StatementCache {
    statements: Mutex<Inner>,
}

impl StatementCache {
    /// Run `sql` and collect its rows, reusing a prepared statement when this
    /// cache already holds one for that exact text.
    ///
    /// Uses root `prepare` (catalog freshness probes and constraint SQL).
    pub(crate) fn query_rows(
        &self,
        connection: &Arc<Connection>,
        sql: &str,
    ) -> Result<Vec<Vec<Value>>, CatalogError> {
        self.run(
            connection,
            sql,
            PrepareKind::Root,
            |statement| statement.run_collect_rows(),
        )
        .map_err(CatalogError::from)
    }

    /// Mutation helper: `prepare_internal` (or reuse), run `bind`, collect rows.
    pub(crate) fn query_rows_internal(
        &self,
        connection: &Arc<Connection>,
        sql: &str,
        bind: impl FnOnce(&mut Statement) -> Result<(), turso_core::LimboError>,
    ) -> Result<Vec<Vec<Value>>, turso_core::LimboError> {
        self.run(connection, sql, PrepareKind::Internal, |statement| {
            bind(statement)?;
            statement.run_collect_rows()
        })
    }

    /// Mutation helper: `prepare_internal` (or reuse), run `bind`, ignore rows.
    pub(crate) fn run_ignore_internal(
        &self,
        connection: &Arc<Connection>,
        sql: &str,
        bind: impl FnOnce(&mut Statement) -> Result<(), turso_core::LimboError>,
    ) -> Result<(), turso_core::LimboError> {
        self.run(connection, sql, PrepareKind::Internal, |statement| {
            bind(statement)?;
            statement.run_ignore_rows().map(|()| Vec::<Value>::new())
        })
        .map(|_| ())
    }

    fn run<T>(
        &self,
        connection: &Arc<Connection>,
        sql: &str,
        kind: PrepareKind,
        body: impl FnOnce(&mut Statement) -> Result<T, turso_core::LimboError>,
    ) -> Result<T, turso_core::LimboError> {
        // The statement leaves the map for the duration of the call. A
        // re-entrant query on the same SQL then prepares its own rather than
        // stepping a statement that is already mid-execution.
        let cached = {
            let mut inner = self.statements.lock();
            if let Some(statement) = inner.map.remove(sql) {
                if let Some(index) = inner.order.iter().position(|key| key == sql) {
                    inner.order.remove(index);
                }
                Some(statement)
            } else {
                None
            }
        };
        let mut statement = match cached {
            Some(mut statement) => {
                // Cached InternalHelper statements were parked so they do not
                // hold a nested-txn guard while idle. Restore before step.
                statement.unpark_nested_helper();
                statement
            }
            None => match kind {
                PrepareKind::Root => connection.prepare(sql)?,
                PrepareKind::Internal => connection.prepare_internal(sql)?,
            },
        };
        let result = body(&mut statement);
        // A statement that failed part-way can hold execution state that reset
        // is not guaranteed to unwind, so only a clean run earns a place back
        // in the cache.
        if result.is_ok() && statement.reset().is_ok() {
            // Drop nested guards while the statement sits in the cache so a
            // bootstrap loop of prepared helpers does not stack start_nested.
            statement.park_nested_helper();
            let mut inner = self.statements.lock();
            // Drop one oldest entry at a time until there is room. A full clear
            // would throw away the hot catalog probe and every other constraint
            // query just as a bootstrap loop needs them.
            while inner.map.len() >= CAPACITY {
                if let Some(oldest) = inner.order.pop_front() {
                    inner.map.remove(&oldest);
                } else {
                    // Order and map drifted; recover by dropping one arbitrary key.
                    if let Some(key) = inner.map.keys().next().cloned() {
                        inner.map.remove(&key);
                    } else {
                        break;
                    }
                }
            }
            inner.map.insert(sql.to_owned(), statement);
            inner.order.push_back(sql.to_owned());
        }
        result
    }

    /// How many distinct SQL strings currently hold a prepared statement.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.statements.lock().map.len()
    }

    /// Whether this exact SQL text currently has a prepared statement.
    #[cfg(test)]
    fn contains(&self, sql: &str) -> bool {
        self.statements.lock().map.contains_key(sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turso_core::{Database, MemoryIO, SqliteDialect};

    fn connection() -> Arc<Connection> {
        let database = Database::open_file(
            Arc::new(MemoryIO::new()),
            ":memory:statement-cache-unit",
            Arc::new(SqliteDialect),
        )
        .expect("open database");
        database.connect().expect("connect")
    }

    #[test]
    fn eviction_at_capacity_drops_one_entry_not_the_whole_map() {
        let connection = connection();
        let cache = StatementCache::default();

        // Fill the cache to capacity with distinct SQL.
        for index in 0..CAPACITY {
            let sql = format!("SELECT {index}");
            cache
                .query_rows(&connection, &sql)
                .expect("prepare and step");
        }
        assert_eq!(cache.len(), CAPACITY);

        // One more insert must free a single slot, not empty the map.
        cache
            .query_rows(&connection, "SELECT 999999")
            .expect("overflow insert");
        assert_eq!(
            cache.len(),
            CAPACITY,
            "eviction must leave the cache full of useful statements, not cleared to one entry"
        );
        assert!(
            cache.contains("SELECT 999999"),
            "the newest statement must be retained"
        );
    }

    #[test]
    fn a_hot_key_survives_when_colder_entries_are_evicted() {
        let connection = connection();
        let cache = StatementCache::default();

        // Seed capacity-1 cold keys, then touch a hot key so it is most recent.
        for index in 0..(CAPACITY - 1) {
            cache
                .query_rows(&connection, &format!("SELECT {index}"))
                .expect("cold prepare");
        }
        const HOT: &str = "SELECT 'hot-probe'";
        cache.query_rows(&connection, HOT).expect("hot prepare");
        assert_eq!(cache.len(), CAPACITY);

        // Touch the hot key again so it moves to the back of the LRU order.
        cache.query_rows(&connection, HOT).expect("hot reuse");

        // Force one eviction with a new key. The hot probe must remain.
        cache
            .query_rows(&connection, "SELECT 'evictor'")
            .expect("evictor");
        assert!(
            cache.contains(HOT),
            "a recently used catalog/constraint query must not be the victim of drop-one eviction"
        );
        assert_eq!(cache.len(), CAPACITY);
    }
}
