//! Compile-time surface check: the crate exposes baseline-aligned names so
//! consumers do not need a direct `turso_core` dependency for common types.

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
