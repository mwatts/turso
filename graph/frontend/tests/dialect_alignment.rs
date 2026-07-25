//! Alignment regressions for GraphDialect + compile seams.

mod fixture;

use turso_graph_frontend::{Parameters, Value};
use turso_graph_ir::ValueType;

#[test]
fn prepare_returns_cypher_result_types_for_boolean_projection() {
    let (_database, graph) = fixture::social_graph_connection();
    let stmt = graph
        .prepare(
            "MATCH (n:Person) RETURN n.name AS name, true AS flag",
            &Parameters::new(),
        )
        .expect("prepare");
    let types = stmt.result_types();
    assert!(
        !types.is_empty(),
        "result_types must come from the single compile path"
    );
    assert_eq!(types.len(), 2);
    assert_eq!(types[0], ValueType::Text);
    assert_eq!(types[1], ValueType::Boolean);
    assert_eq!(stmt.num_columns(), 2);

    let rows = graph
        .query(
            "MATCH (n:Person {name: 'Ada'}) RETURN n.name AS name, true AS flag",
            &Parameters::new(),
        )
        .expect("query");
    assert_eq!(
        rows,
        vec![vec![Value::build_text("Ada"), Value::from_i64(1)]]
    );
}

#[test]
fn prepare_result_types_match_projection_width() {
    let (_database, graph) = fixture::social_graph_connection();
    let stmt = graph
        .prepare(
            "MATCH (n:Person) RETURN n.name AS name, n.age AS age",
            &Parameters::new(),
        )
        .expect("prepare");
    assert_eq!(stmt.result_types().len(), stmt.num_columns());
    assert_eq!(stmt.result_types().len(), 2);
    assert_eq!(stmt.result_types()[0], ValueType::Text);
    assert_eq!(stmt.result_types()[1], ValueType::Integer);
}

/// EXPLAIN must lower Cypher once, then prepare pure SQL `EXPLAIN QUERY PLAN`
/// against Core — never re-parse the Cypher text as a dialect statement.
#[test]
fn explain_match_returns_core_eqp_rows() {
    let (_database, graph) = fixture::social_graph_connection();
    let rows = graph
        .query("EXPLAIN MATCH (n:Person) RETURN n.name", &Parameters::new())
        .expect("explain");
    assert!(!rows.is_empty(), "EXPLAIN must return EQP rows from core");
    let plan_text = rows
        .iter()
        .flatten()
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        plan_text.contains("SCAN")
            || plan_text.contains("SEARCH")
            || plan_text.contains("USING INDEX")
            || plan_text.contains("USING COVERING INDEX"),
        "EXPLAIN QUERY PLAN must describe a core access method, got:\n{plan_text}"
    );

    // EXPLAIN reports core plan columns, not the Cypher projection types.
    let stmt = graph
        .prepare("EXPLAIN MATCH (n:Person) RETURN n.name", &Parameters::new())
        .expect("prepare explain");
    assert!(
        stmt.result_types().is_empty(),
        "EXPLAIN must not surface Cypher result_types"
    );
}
