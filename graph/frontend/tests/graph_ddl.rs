//! End-to-end coverage for `CREATE GRAPH`.
//!
//! The claim under test is that the DDL is *sugar*: a graph declared this way
//! must be usable exactly like one built with `register_graph` and raw
//! `CREATE TABLE`s. So these tests do not stop at inspecting the catalog —
//! each one opens a session and runs Cypher against the result.

use std::sync::Arc;

use turso_core::{
    Connection, Database, DatabaseOpts, MemoryIO, Numeric, OpenOptions, SqliteDialect, Value,
};
use turso_graph_frontend::{
    execute_graph_ddl, load_registered_graph, DdlError, GraphConnection, Parameters,
};
use turso_graph_ir::RoleCardinality;

fn connect(tag: &str) -> (Arc<Database>, Arc<Connection>) {
    let io = Arc::new(MemoryIO::new());
    let database = Database::open(
        io,
        &format!(":memory:ddl-{tag}"),
        OpenOptions::new(Arc::new(SqliteDialect)).db_opts(DatabaseOpts::default()),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");
    (database, connection)
}

/// Count of tables in the schema matching `name`. Used to assert on physical
/// storage the graph catalog does not itself expose. The comparison is
/// case-insensitive because the schema stores table names folded, while the
/// catalog keeps the case the declaration used.
fn table_count(connection: &Arc<Connection>, name: &str) -> i64 {
    let name = name.to_lowercase();
    let rows = connection
        .prepare(format!(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND lower(name) = '{name}'"
        ))
        .and_then(|mut statement| statement.run_collect_rows())
        .expect("query schema");
    match rows[0][0] {
        Value::Numeric(Numeric::Integer(count)) => count,
        ref other => panic!("expected an integer count, got {other:?}"),
    }
}

/// The headline case: no physical names anywhere in the statement, and the
/// result is a working graph. If inference were wrong at any step — table
/// name, identity column, or role column — the CREATE below would fail to
/// bind or the MATCH would return nothing.
#[test]
fn inferred_declaration_produces_a_usable_graph() {
    let (_database, connection) = connect("inferred");

    execute_graph_ddl(
        &connection,
        "CREATE GRAPH social \
         NODE Person (name TEXT, age INTEGER) \
         RELATION KNOWS (since INTEGER) \
           ROLE start -> Person \
           ROLE end -> Person",
    )
    .expect("DDL should succeed");

    let session = GraphConnection::open(connection, "social").expect("open session");
    session
        .execute(
            "CREATE (a:Person {id: 1, name: 'Ada', age: 36}), \
                    (b:Person {id: 2, name: 'Grace', age: 45})",
            &Parameters::new(),
        )
        .expect("create nodes");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) \
             CREATE (a)-[:KNOWS {id: 1, since: 1952}]->(b)",
            &Parameters::new(),
        )
        .expect("create relationship");

    let rows = session
        .query(
            "MATCH (a:Person)-[k:KNOWS]->(b:Person) RETURN a.name, b.name, k.since",
            &Parameters::new(),
        )
        .expect("traverse");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], Value::build_text("Ada"));
    assert_eq!(rows[0][1], Value::build_text("Grace"));
    assert_eq!(rows[0][2], Value::from_i64(1952));
}

/// `IF NOT EXISTS` is what lets one syntax both create and adopt. Declaring a
/// graph over tables that already hold rows must leave those rows intact —
/// adoption that silently started from an empty table would be worse than a
/// refusal.
#[test]
fn declaration_adopts_existing_tables_without_disturbing_rows() {
    let (_database, connection) = connect("adopt");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT); \
             CREATE TABLE knows(id INTEGER PRIMARY KEY, src INTEGER, dst INTEGER); \
             INSERT INTO people VALUES (1, 'Ada'), (2, 'Grace'); \
             INSERT INTO knows VALUES (1, 1, 2);",
        )
        .expect("seed existing schema");

    execute_graph_ddl(
        &connection,
        "CREATE GRAPH social \
         NODE Person AS TABLE people KEY id (name TEXT) \
         RELATION KNOWS AS TABLE knows KEY id \
           ROLE start -> Person VIA src \
           ROLE end -> Person VIA dst",
    )
    .expect("DDL should adopt the existing tables");

    let session = GraphConnection::open(connection, "social").expect("open session");
    let rows = session
        .query(
            "MATCH (a:Person)-[:KNOWS]->(b:Person) RETURN a.name, b.name",
            &Parameters::new(),
        )
        .expect("traverse pre-existing rows");
    assert_eq!(
        rows.len(),
        1,
        "adoption must see rows written before the DDL"
    );
    assert_eq!(rows[0][0], Value::build_text("Ada"));
    assert_eq!(rows[0][1], Value::build_text("Grace"));
}

/// Binary is a layout, not a kind — so the DDL must reach the whole role
/// model, not just two-role relations. A three-role relation with a `MANY`
/// role exercises both the general arity path and spill-table storage.
#[test]
fn declares_an_n_ary_relation_with_a_many_role() {
    let (_database, connection) = connect("nary");

    let registered = execute_graph_ddl(
        &connection,
        "CREATE GRAPH scriptorium \
         NODE Text (title TEXT) \
         RELATION Citation (label TEXT) \
           ROLE cited -> Text \
           ROLE reference -> Text \
           ROLE witnesses -> Text MANY",
    )
    .expect("DDL should succeed");

    let citation = registered
        .relationship_sources
        .iter()
        .find(|source| source.name == "Citation")
        .expect("Citation registered");
    let witnesses = citation
        .roles
        .iter()
        .find(|role| role.name == "witnesses")
        .expect("witnesses role registered");
    assert_eq!(witnesses.cardinality, RoleCardinality::Many);
    assert_eq!(
        witnesses.column, "",
        "a MANY role takes no column on the relation table"
    );
    assert_eq!(
        table_count(&connection, &citation.spill_table(witnesses)),
        1,
        "a MANY role must be backed by a spill table"
    );

    let session = GraphConnection::open(connection, "scriptorium").expect("open session");
    session
        .execute(
            "CREATE (:Text {id: 1, title: 'Alpha'}), \
                    (:Text {id: 2, title: 'Beta'}), \
                    (:Text {id: 3, title: 'Gamma'})",
            &Parameters::new(),
        )
        .expect("create texts");
    session
        .execute(
            "MATCH (a:Text {id: 1}), (b:Text {id: 2}), (c:Text {id: 3}) \
             CREATE [x:Citation {id: 1, label: 'first'}]\
             (cited: a, reference: b, witnesses: c)",
            &Parameters::new(),
        )
        .expect("create ternary relation");

    let rows = session
        .query(
            "MATCH [x:Citation](cited: c, witnesses: w) RETURN c.title, w.title",
            &Parameters::new(),
        )
        .expect("match through roles");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], Value::build_text("Alpha"));
    assert_eq!(rows[0][1], Value::build_text("Gamma"));
}

/// A failed declaration must leave nothing behind. Without the surrounding
/// transaction, the tables created before the failing registration would
/// persist as orphans that the next attempt then "adopts".
#[test]
fn a_failed_declaration_leaves_no_tables_behind() {
    let (_database, connection) = connect("atomic");

    // `Missing` is never declared as a NODE, so registration rejects the role
    // target — after both tables have already been created.
    let error = execute_graph_ddl(
        &connection,
        "CREATE GRAPH social \
         NODE Person (name TEXT) \
         RELATION KNOWS \
           ROLE start -> Person \
           ROLE end -> Missing",
    )
    .expect_err("unknown role target must fail");
    assert!(
        matches!(error, DdlError::Catalog(_)),
        "unexpected error: {error}"
    );

    assert_eq!(
        table_count(&connection, "Person"),
        0,
        "a rolled-back declaration must not leave its tables committed"
    );
    assert_eq!(table_count(&connection, "KNOWS"), 0);
    assert!(
        load_registered_graph(&connection, "social").is_err(),
        "a rolled-back declaration must not leave the graph registered"
    );
}

/// `VIA` on a `MANY` role names a column that registration ignores. Accepting
/// it would let a user believe they had chosen storage they had not.
#[test]
fn refuses_via_on_a_many_role() {
    let (_database, connection) = connect("many-via");

    let error = execute_graph_ddl(
        &connection,
        "CREATE GRAPH g NODE Text (title TEXT) \
         RELATION Citation ROLE witnesses -> Text VIA w MANY",
    )
    .expect_err("VIA on MANY must be refused");

    let message = error.to_string();
    assert!(
        message.contains("witnesses") && message.contains("spill table"),
        "refusal must name the role and explain why: {message}"
    );
}
