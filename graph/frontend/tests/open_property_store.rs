//! End-to-end: `register_graph` always installs prop_dict + node_props and
//! routes Cypher node properties through integer Cell storage.

use std::sync::Arc;

use turso_core::{Database, MemoryIO, SqliteDialect, Value};
use turso_graph_frontend::{
    execute_graph_ddl, load_registered_graph, node_props_table_name, prop_dict_table_name,
    register_graph, GraphConnection, GraphRegistration, NodeSourceRegistration, Parameters,
};

fn open_cell_session() -> (GraphConnection, Arc<turso_core::Connection>, String) {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:open-property-store",
        Arc::new(SqliteDialect),
    )
    .expect("open");
    let connection = database.connect().expect("connect");
    // Identity-only source table — no property columns.
    connection
        .execute("CREATE TABLE entities(id INTEGER PRIMARY KEY);")
        .expect("create entities");
    let registered = register_graph(
        &connection,
        &GraphRegistration {
            name: "ontology".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Entity".to_owned(),
                table: "entities".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        },
    )
    .expect("register graph (always Cell)");
    let reloaded = load_registered_graph(&connection, "ontology").expect("reload");
    assert_eq!(reloaded.id, registered.id);

    let dict = prop_dict_table_name(registered.id);
    let props = node_props_table_name(registered.id);
    let edges = turso_graph_frontend::edge_props_table_name(registered.id);
    // Tables must exist after registration.
    connection
        .prepare(format!("SELECT 1 FROM \"{dict}\" LIMIT 0"))
        .expect("prop_dict exists")
        .run_collect_rows()
        .expect("scan dict");
    connection
        .prepare(format!("SELECT 1 FROM \"{props}\" LIMIT 0"))
        .expect("node_props exists")
        .run_collect_rows()
        .expect("scan props");
    connection
        .prepare(format!("SELECT 1 FROM \"{edges}\" LIMIT 0"))
        .expect("edge_props exists")
        .run_collect_rows()
        .expect("scan edge props");

    let session =
        GraphConnection::open(connection.clone(), "ontology").expect("open GraphConnection");
    (session, connection, props)
}

#[test]
fn open_cell_register_create_filter_set_and_merge() {
    let (session, sql, props_table) = open_cell_session();

    session
        .execute("CREATE (n {name: 'Ada', age: 36})", &Parameters::new())
        .expect("CREATE with open properties");

    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age",
            &Parameters::new(),
        )
        .expect("filter on open property");
    assert_eq!(rows, vec![vec![Value::from_i64(36)]]);

    session
        .execute(
            "MATCH (n) WHERE n.name = 'Ada' SET n.age = 37",
            &Parameters::new(),
        )
        .expect("SET open property");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age",
            &Parameters::new(),
        )
        .expect("read after SET");
    assert_eq!(rows, vec![vec![Value::from_i64(37)]]);

    session
        .execute(
            "MERGE (n {name: 'Ada'}) ON MATCH SET n.age = 38 ON CREATE SET n.age = 0",
            &Parameters::new(),
        )
        .expect("MERGE open property key");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age",
            &Parameters::new(),
        )
        .expect("read after MERGE");
    assert_eq!(rows, vec![vec![Value::from_i64(38)]]);

    // Physical: cells keyed by integer prop_id (auto-allocated in prop_dict).
    let cell_count = sql
        .prepare(format!("SELECT COUNT(*) FROM \"{props_table}\""))
        .expect("prepare")
        .run_collect_rows()
        .expect("count");
    assert!(
        matches!(
            &cell_count[0][0],
            Value::Numeric(turso_core::Numeric::Integer(n)) if *n >= 2
        ),
        "expected at least name+age cells, got {:?}",
        cell_count[0][0]
    );

    let entity_count = sql
        .prepare("SELECT COUNT(*) FROM entities")
        .expect("prepare")
        .run_collect_rows()
        .expect("entities");
    assert_eq!(
        entity_count[0][0],
        Value::from_i64(1),
        "MERGE must not duplicate Ada"
    );
}

#[test]
fn open_cell_open_property_and_filter() {
    let (session, _, _) = open_cell_session();
    session
        .execute(
            "CREATE (a {code: 'X1', kind: 'gene'}), (b {code: 'X2', kind: 'protein'})",
            &Parameters::new(),
        )
        .expect("create two entities");
    let rows = session
        .query(
            "MATCH (n) WHERE n.kind = 'gene' AND n.code = 'X1' RETURN n.code",
            &Parameters::new(),
        )
        .expect("multi open property AND");
    assert_eq!(rows, vec![vec![Value::build_text("X1")]]);
}

#[test]
fn open_cell_whole_map_set_clears_omitted_keys() {
    let (session, sql, props_table) = open_cell_session();
    session
        .execute(
            "CREATE (n {name: 'Ada', age: 36, city: 'London'})",
            &Parameters::new(),
        )
        .expect("create");
    // SET n = map replaces the whole property bag.
    session
        .execute(
            "MATCH (n) WHERE n.name = 'Ada' SET n = {name: 'Ada', age: 37}",
            &Parameters::new(),
        )
        .expect("whole-map SET");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age, n.city",
            &Parameters::new(),
        )
        .expect("read after replace");
    assert_eq!(
        rows,
        vec![vec![Value::from_i64(37), Value::Null]],
        "omitted city must be cleared"
    );
    let cell_count = sql
        .prepare(format!("SELECT COUNT(*) FROM \"{props_table}\""))
        .expect("prepare")
        .run_collect_rows()
        .expect("count");
    assert_eq!(
        cell_count[0][0],
        Value::from_i64(2),
        "exactly name+age cells remain"
    );
}

#[test]
fn open_cell_delete_purges_property_cells() {
    let (session, sql, props_table) = open_cell_session();
    session
        .execute("CREATE (n {name: 'Ada', age: 36})", &Parameters::new())
        .expect("create");
    let before = sql
        .prepare(format!("SELECT COUNT(*) FROM \"{props_table}\""))
        .expect("prepare")
        .run_collect_rows()
        .expect("count");
    assert!(
        matches!(
            &before[0][0],
            Value::Numeric(turso_core::Numeric::Integer(n)) if *n >= 2
        ),
        "cells exist before delete"
    );
    session
        .execute(
            "MATCH (n) WHERE n.name = 'Ada' DELETE n",
            &Parameters::new(),
        )
        .expect("delete node");
    let after = sql
        .prepare(format!("SELECT COUNT(*) FROM \"{props_table}\""))
        .expect("prepare")
        .run_collect_rows()
        .expect("count");
    assert_eq!(
        after[0][0],
        Value::from_i64(0),
        "delete must purge node_props rows"
    );
    let entities = sql
        .prepare("SELECT COUNT(*) FROM entities")
        .expect("prepare")
        .run_collect_rows()
        .expect("entities");
    assert_eq!(entities[0][0], Value::from_i64(0));
}

/// CREATE GRAPH is topology-only: declared properties seed prop_dict but do
/// not create SQL columns. CREATE/MATCH/SET must use Cell, not fail with
/// "no such column" / "missing relational column".
#[test]
fn create_graph_declared_properties_use_cell_not_missing_columns() {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:create-graph-cell",
        Arc::new(SqliteDialect),
    )
    .expect("open");
    let connection = database.connect().expect("connect");
    execute_graph_ddl(
        &connection,
        "CREATE GRAPH social \
         NODE Person (name TEXT, age INTEGER, email TEXT) \
         RELATION KNOWS (since INTEGER) \
           ROLE start -> Person \
           ROLE end -> Person",
    )
    .expect("CREATE GRAPH topology-only");

    let session = GraphConnection::open(connection.clone(), "social").expect("open");
    session
        .execute(
            "CREATE (a:Person {id: 1, name: 'Ada', age: 36, email: 'ada@example.com'}), \
                    (b:Person {id: 2, name: 'Grace', age: 45})",
            &Parameters::new(),
        )
        .expect("CREATE must not require SQL property columns");
    session
        .execute(
            "MATCH (a:Person {id: 1}), (b:Person {id: 2}) \
             CREATE (a)-[:KNOWS {id: 1, since: 1952}]->(b)",
            &Parameters::new(),
        )
        .expect("edge property must use Cell");

    let rows = session
        .query(
            "MATCH (a:Person)-[k:KNOWS]->(b:Person) \
             RETURN a.name, a.email, b.age, k.since",
            &Parameters::new(),
        )
        .expect("read Cell properties");
    assert_eq!(
        rows,
        vec![vec![
            Value::build_text("Ada"),
            Value::build_text("ada@example.com"),
            Value::from_i64(45),
            Value::from_i64(1952),
        ]]
    );

    // Topology tables must stay identity/endpoints only.
    let person_cols = connection
        .prepare("SELECT name FROM pragma_table_info('Person') ORDER BY cid")
        .expect("pragma")
        .run_collect_rows()
        .expect("cols");
    let names: Vec<String> = person_cols
        .iter()
        .filter_map(|row| match row.first() {
            Some(Value::Text(t)) => Some(t.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(names, vec!["id".to_owned()], "no property columns on node table");
}
