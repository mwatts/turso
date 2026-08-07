//! Coverage for `Connection::table_change_token`, the per-table change signal
//! the graph frontend uses instead of AFTER-DML triggers to decide whether a
//! traversal snapshot is still good.
//!
//! Every test here pins one of the properties the triggers had. The two that
//! matter most are that a rolled-back write leaves the token alone (otherwise
//! callers rebuild caches they did not need to) and that the token moves when
//! a table is dropped and recreated (otherwise callers keep a cache of rows
//! that no longer exist).

use crate::common::{run_query, TempDatabase};
use std::sync::Arc;
use turso_core::Connection;

fn token(conn: &Arc<Connection>, table: &str) -> u64 {
    conn.table_change_token(table)
        .unwrap_or_else(|| panic!("no change token for `{table}`"))
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn committed_write_moves_the_token(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let before = token(&conn, "t");
    run_query(&tmp_db, &conn, "INSERT INTO t VALUES (1)")?;
    assert_ne!(before, token(&conn, "t"), "an insert must move the token");
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn a_write_leaves_other_tables_tokens_alone(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    run_query(&tmp_db, &conn, "CREATE TABLE other(y)")?;
    let before = token(&conn, "other");
    run_query(&tmp_db, &conn, "INSERT INTO t VALUES (1)")?;
    assert_eq!(
        before,
        token(&conn, "other"),
        "writing one table must not invalidate caches over another; this \
         per-table precision is the whole reason for replacing the triggers"
    );
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn rolled_back_write_leaves_the_token_alone(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let before = token(&conn, "t");
    run_query(&tmp_db, &conn, "BEGIN")?;
    run_query(&tmp_db, &conn, "INSERT INTO t VALUES (1)")?;
    run_query(&tmp_db, &conn, "ROLLBACK")?;
    assert_eq!(
        before,
        token(&conn, "t"),
        "a rolled-back write changed nothing, so the token must not move"
    );
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn committed_transaction_moves_the_token(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let before = token(&conn, "t");
    run_query(&tmp_db, &conn, "BEGIN")?;
    run_query(&tmp_db, &conn, "INSERT INTO t VALUES (1)")?;
    run_query(&tmp_db, &conn, "COMMIT")?;
    assert_ne!(before, token(&conn, "t"));
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn a_statement_that_matches_no_row_leaves_the_token_alone(
    tmp_db: TempDatabase,
) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    run_query(&tmp_db, &conn, "INSERT INTO t VALUES (1)")?;
    let before = token(&conn, "t");
    run_query(&tmp_db, &conn, "UPDATE t SET x = 2 WHERE x = 999")?;
    run_query(&tmp_db, &conn, "DELETE FROM t WHERE x = 999")?;
    assert_eq!(
        before,
        token(&conn, "t"),
        "opening a write cursor is not writing; a statement that changed no \
         row must not force callers to rebuild"
    );
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn a_second_connection_sees_the_write(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let writer = tmp_db.connect_limbo();
    let reader = tmp_db.connect_limbo();
    let before = token(&reader, "t");
    run_query(&tmp_db, &writer, "INSERT INTO t VALUES (1)")?;
    assert_ne!(
        before,
        token(&reader, "t"),
        "the counters live on the database, so any connection's commit is \
         visible to every other one"
    );
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn dropping_and_recreating_a_table_moves_the_token(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    let before = token(&conn, "t");
    // DROP frees the root page without opening a write cursor on it, and
    // CREATE can be handed the same page straight back. A token built only
    // from the per-root counter would read unchanged here while every row is
    // gone -- a stale cache, not just a missed reload.
    run_query(&tmp_db, &conn, "DROP TABLE t")?;
    run_query(&tmp_db, &conn, "CREATE TABLE t(x)")?;
    assert_ne!(before, token(&conn, "t"));
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE src(x)")]
fn a_trigger_write_counts_for_the_table_it_writes(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    run_query(&tmp_db, &conn, "CREATE TABLE dst(y)")?;
    run_query(
        &tmp_db,
        &conn,
        "CREATE TRIGGER mirror AFTER INSERT ON src BEGIN INSERT INTO dst VALUES (new.x); END",
    )?;
    let before = token(&conn, "dst");
    run_query(&tmp_db, &conn, "INSERT INTO src VALUES (1)")?;
    assert_ne!(
        before,
        token(&conn, "dst"),
        "the statement never names `dst`; only the trigger subprogram's write \
         set makes this table's cache invalid"
    );
    Ok(())
}

#[turso_macros::test(init_sql = "CREATE TABLE t(x)")]
fn an_unknown_table_has_no_token(tmp_db: TempDatabase) -> anyhow::Result<()> {
    let conn = tmp_db.connect_limbo();
    assert_eq!(
        conn.table_change_token("no_such_table"),
        None,
        "callers must treat a table we cannot identify as always-changed"
    );
    Ok(())
}
