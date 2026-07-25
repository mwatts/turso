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
