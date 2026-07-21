//! Compile-time surface check: the crate exposes baseline-aligned names so
//! consumers do not need a direct `turso_core` dependency for common types.

mod fixture;

#[test]
fn prepare_exposes_result_types_on_the_statement() {
    let (connection, session) = fixture::social_graph_connection();
    let stmt = session
        .prepare("MATCH (n:Person) RETURN n.name, n.age", &Default::default())
        .expect("prepare");
    // Metadata rides on the statement — no second parse call.
    assert_eq!(stmt.result_types().len(), 2);
    // Deref gives the full core statement surface.
    assert_eq!(stmt.num_columns(), 2);
    drop(stmt);
    drop(session);
    drop(connection);
}

#[test]
fn baseline_aligned_reexports_are_usable() {
    // Value/Row/StepResult come from the crate root, mirroring turso_pg.
    let v: turso_graph_frontend::Value = turso_graph_frontend::Value::Null;
    assert!(matches!(v, turso_graph_frontend::Value::Null));

    // Full core access via the `core` module, mirroring `turso::core`.
    fn _takes_core_stmt(_s: &turso_graph_frontend::core::Statement) {}
    fn _takes_flags(_f: turso_graph_frontend::OpenFlags) {}

    // Error/Result aliases exist.
    fn _returns_result() -> turso_graph_frontend::Result<()> {
        Ok(())
    }
}

#[test]
fn session_type_names_match_baseline_convention() {
    // GraphConnection is aliased to Connection at the root, mirroring
    // `pub use session::PgConnection as Connection` in turso_pg.
    fn _takes_conn(_c: &turso_graph_frontend::Connection) {}
    fn _takes_graph_conn(_c: &turso_graph_frontend::GraphConnection) {}

    let params: turso_graph_frontend::Parameters = Default::default();
    assert!(params.is_empty());
}
