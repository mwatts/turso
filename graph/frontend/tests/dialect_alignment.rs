//! Alignment regressions for GraphDialect + compile seams.

mod fixture;

use std::sync::{atomic::Ordering, Arc};

use turso_core::{DatabaseOpts, MemoryIO, OpenFlags};
use turso_graph_frontend::{
    install_graph_catalog, open_database_with_io, register_graph, take_closed_create_fast_path_hit,
    GraphConnection, GraphHostMode, GraphRegistration, NodeSourceRegistration, Parameters,
    RelationshipSourceRegistration, SnapshotStore, Value, GRAPH_EXPAND_TABLE_NAME,
};
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

/// Dialect-pinned open still installs the temporal extension for InternalHelper
/// mutation SQL (SQLite symbol table), while Root reads resolve via GraphDialect.
#[test]
fn dialect_pinned_open_installs_temporal_extension_and_runs_duration() {
    let before = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-pinned-duration",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    assert_eq!(session.host_mode(), GraphHostMode::DialectPinned);
    let after_open = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    assert!(
        after_open - before >= 1,
        "dialect-pinned GraphConnection::open must still install_temporal_extension for InternalHelper symbols (delta={})",
        after_open - before
    );

    let rows = session
        .query("RETURN duration('P1DT25H') AS d", &Parameters::new())
        .expect("duration must resolve (dialect Root path; extension also present)");
    assert_eq!(rows, vec![vec![Value::build_text("P1DT25H")]]);
}

/// Mutation helpers use `prepare_internal` → SQLite function resolve only.
/// Dialect-owned GraphDialect resolve does not cover that path, so
/// `install_temporal_extension` must run even under DialectPinned — otherwise
/// lowered `cypher_equals` (list IN membership) fails at prepare.
#[test]
fn dialect_pinned_mutation_in_predicate_needs_temporal_extension() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-pinned-mutation-cypher-equals",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    assert_eq!(session.host_mode(), GraphHostMode::DialectPinned);

    session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada', age: 36})",
            &Parameters::new(),
        )
        .expect("seed create");

    // IN lowers through cypher_equals inside mutation helper SQL.
    let summary = session
        .execute(
            "MATCH (n:Person) WHERE n.name IN ['Ada'] SET n.age = 40",
            &Parameters::new(),
        )
        .expect("dialect-pinned mutation with cypher_equals must work when install always runs");
    assert_eq!(summary.matched_rows, 1);
    assert!(summary.operations_executed >= 1);

    let rows = session
        .query(
            "MATCH (n:Person {id: 1}) RETURN n.age AS age",
            &Parameters::new(),
        )
        .expect("match after set");
    assert_eq!(rows, vec![vec![Value::from_i64(40)]]);
}

/// Attach mode (foreign dialect) still installs the temporal extension.
#[test]
fn attach_mode_install_increments_temporal_install_count() {
    let before = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    let (_database, session) = fixture::social_graph_connection();
    assert_eq!(session.host_mode(), GraphHostMode::Attach);
    let after = turso_graph_temporal::INSTALL_COUNT.load(Ordering::SeqCst);
    assert!(
        after > before,
        "attach-mode install must call install_temporal_extension"
    );
}

/// Expand stays session-activated: double `install_graph_catalog` must not error.
#[test]
fn install_graph_catalog_is_idempotent() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:expand-idempotent",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    let store = Arc::new(SnapshotStore::default());
    let first = install_graph_catalog(connection.as_ref(), store.clone())
        .expect("first expand catalog install");
    let second = install_graph_catalog(connection.as_ref(), store)
        .expect("second expand catalog install must be idempotent");
    assert_eq!(first, GRAPH_EXPAND_TABLE_NAME);
    assert_eq!(second, GRAPH_EXPAND_TABLE_NAME);
}

/// Mutation helper SQL is prepared via `prepare_internal` (InternalHelper).
/// Under GraphDialect, CREATE must still commit and be visible to MATCH.
#[test]
fn simple_create_mutation_commits_under_graph_dialect() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-pinned-create",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    assert_eq!(session.host_mode(), GraphHostMode::DialectPinned);

    let summary = session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada', age: 36})",
            &Parameters::new(),
        )
        .expect("create under GraphDialect");
    assert_eq!(summary.matched_rows, 1);
    assert!(summary.operations_executed >= 1);

    let rows = session
        .query(
            "MATCH (n:Person {id: 1}) RETURN n.name AS name",
            &Parameters::new(),
        )
        .expect("match after create");
    assert_eq!(rows, vec![vec![Value::build_text("Ada")]]);
}

/// Closed single-node CREATE must take the closed CREATE fast path under
/// GraphDialect and remain visible to MATCH (including label membership).
///
/// A hit means the fast-path branch ran (one prepare for the node INSERT).
/// Labeled creates may still use extra prepares for label-junction rows —
/// this is not a claim of one VDBE program for the whole mutation.
#[test]
fn single_create_node_uses_closed_create_fast_path() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-closed-create-fast-path",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    session
        .execute(
            "CREATE (:Person {id: 42, name: 'Grace'})",
            &Parameters::new(),
        )
        .expect("single create");
    assert!(
        take_closed_create_fast_path_hit(),
        "closed CREATE node must take the closed CREATE fast path"
    );

    let rows = session
        .query(
            "MATCH (n:Person {id: 42}) RETURN n.name AS name",
            &Parameters::new(),
        )
        .expect("match after closed-create fast path");
    assert_eq!(rows, vec![vec![Value::build_text("Grace")]]);
}

/// Multi-stage mutations must not take the closed CREATE fast path.
#[test]
fn multi_stage_mutation_still_uses_savepoint_path() {
    let io = Arc::new(MemoryIO::new());
    let database = open_database_with_io(
        io,
        ":memory:dialect-multi-stage-mutation",
        OpenFlags::default(),
        DatabaseOpts::new(),
    )
    .expect("open graph dialect database");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER); \
             CREATE TABLE relationships(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER);",
        )
        .expect("create sources");
    register_graph(
        &connection,
        &GraphRegistration {
            name: "social".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: vec![RelationshipSourceRegistration::binary(
                "KNOWS",
                "relationships",
                "id",
                "src",
                "dst",
                "Person",
                "Person",
            )],
        },
    )
    .expect("register graph");

    let session = GraphConnection::open(connection, "social").expect("open session");
    let summary = session
        .execute(
            "CREATE (:Person {id: 1, name: 'Ada'}) WITH 1 AS x RETURN x",
            &Parameters::new(),
        )
        .expect("multi-stage mutation");
    assert_eq!(summary.rows, vec![vec![Value::from_i64(1)]]);
    assert!(
        !take_closed_create_fast_path_hit(),
        "WITH stages must stay on the multi-prepare savepoint path"
    );
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
