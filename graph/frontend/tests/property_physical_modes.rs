//! Phase 2+ spike: Cypher through Column / JsonBag / Cell physical modes.
//!
//! Cell mode uses **integer prop_id** + a property dictionary (name → id, type),
//! not repeated prop_key strings on every cell row. Value type from the dict
//! drives text-predicate legality (`supports_text_predicate`).
//!
//! Plan: docs/superpowers/plans/2026-08-10-graph-json-bag-property-store-spike.md

use std::sync::Arc;

use turso_core::{Database, DatabaseOpts, MemoryIO, OpenFlags, SqliteDialect, Value};
use turso_graph_frontend::{
    BuildLimits, CatalogEntity, GraphCatalogSnapshot, GraphConnection, NodeTableLayout,
    ParameterTypes, Parameters, PropertyPhysical, RegisteredGraph, RegisteredNodeSource,
    RelationalCatalogSnapshot, ResolvedProperty, SnapshotStore,
};
use turso_graph_ir::{GraphId, Nullability, PropertyId, SourceTableId, ValueType};

#[derive(Clone, Copy, Debug)]
enum Mode {
    JsonBagText,
    JsonBagJsonb,
    JsonBagJsonbIndexed,
    /// Integer prop_id cells + prop_dict (open property store shape).
    CellDict,
}

struct ModeCatalog {
    mode: Mode,
}

impl ModeCatalog {
    fn physical_for(&self, property: PropertyId) -> Option<PropertyPhysical> {
        let (name, value_type) = match property.get() {
            1 => ("name", ValueType::Text),
            2 => ("age", ValueType::Integer),
            _ => return None,
        };
        Some(match self.mode {
            Mode::JsonBagText | Mode::JsonBagJsonb | Mode::JsonBagJsonbIndexed => {
                PropertyPhysical::JsonBag {
                    bag_column: "props".to_owned(),
                    key: name.to_owned(),
                }
            }
            Mode::CellDict => PropertyPhysical::Cell {
                source_id: 1,
                props_table: "node_props".to_owned(),
                entity_column: "node_id".to_owned(),
                identity_column: "id".to_owned(),
                // Integer id = IR PropertyId; dictionary maps name ↔ id + type.
                prop_id: u64::from(property.get()),
                prop_id_column: "prop_id".to_owned(),
                value_column: "value".to_owned(),
                value_type,
            },
        })
    }
}

impl GraphCatalogSnapshot for ModeCatalog {
    fn node_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        SourceTableId::new(1).ok()
    }

    fn relationship_source(&self, _graph: GraphId) -> Option<SourceTableId> {
        None
    }

    fn label(&self, _graph: GraphId, name: &str) -> Option<turso_graph_ir::LabelId> {
        let _ = name;
        None
    }

    fn relationship_type(
        &self,
        _graph: GraphId,
        _name: &str,
    ) -> Option<turso_graph_ir::RelationshipTypeId> {
        None
    }

    fn property(
        &self,
        _graph: GraphId,
        entity: CatalogEntity,
        name: &str,
    ) -> Option<ResolvedProperty> {
        if entity != CatalogEntity::Node {
            return None;
        }
        let (id, value_type) = match name {
            "name" => (1, ValueType::Text),
            "age" => (2, ValueType::Integer),
            _ => return None,
        };
        Some(ResolvedProperty {
            id: PropertyId::new(id).ok()?,
            value_type,
            nullability: Nullability::Nullable,
        })
    }

    fn relationship_source_roles(
        &self,
        _source: SourceTableId,
    ) -> Option<turso_graph_frontend::RelationshipTableLayout> {
        None
    }
}

impl RelationalCatalogSnapshot for ModeCatalog {
    fn node_layout(&self, source: SourceTableId) -> Option<NodeTableLayout> {
        (source.get() == 1).then(|| NodeTableLayout {
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        })
    }

    fn relationship_layout(
        &self,
        _source: SourceTableId,
    ) -> Option<turso_graph_frontend::RelationshipTableLayout> {
        None
    }

    fn property_column(&self, _source: SourceTableId, property: PropertyId) -> Option<String> {
        match property.get() {
            1 => Some("name".to_owned()),
            2 => Some("age".to_owned()),
            _ => None,
        }
    }

    fn property_physical(
        &self,
        _source: SourceTableId,
        property: PropertyId,
    ) -> Option<PropertyPhysical> {
        self.physical_for(property)
    }

    fn labels_table(&self) -> Option<String> {
        None
    }
}

fn registered_graph() -> RegisteredGraph {
    RegisteredGraph {
        id: GraphId::new(1).expect("graph id"),
        name: "props".to_owned(),
        generation: 1,
        schema_generation: Some(1),
        derived_generation: Some(1),
        node_sources: vec![RegisteredNodeSource {
            id: SourceTableId::new(1).expect("source"),
            name: "Person".to_owned(),
            table: "people".to_owned(),
            identity_column: "id".to_owned(),
        }],
        relationship_sources: Vec::new(),
    }
}

fn session(mode: Mode) -> (GraphConnection, Arc<turso_core::Connection>) {
    let custom_types = matches!(mode, Mode::JsonBagJsonb | Mode::JsonBagJsonbIndexed);
    let database = Database::open_file_with_flags(
        Arc::new(MemoryIO::new()),
        &format!(":memory:property-physical-{mode:?}"),
        OpenFlags::default(),
        DatabaseOpts::new().with_custom_types(custom_types),
        None,
        Arc::new(SqliteDialect),
    )
    .expect("open database");
    let connection = database.connect().expect("connect");

    match mode {
        Mode::JsonBagText => {
            connection
                .execute(
                    "CREATE TABLE people(id INTEGER PRIMARY KEY, props TEXT NOT NULL);\
                     INSERT INTO people VALUES (1, '{\"name\":\"Ada\",\"age\":36}');",
                )
                .expect("text bag schema");
        }
        Mode::JsonBagJsonb => {
            connection
                .execute(
                    "CREATE TABLE people(id INTEGER PRIMARY KEY, props JSONB NOT NULL);\
                     INSERT INTO people VALUES (1, jsonb('{\"name\":\"Ada\",\"age\":36}'));",
                )
                .expect("jsonb bag schema");
        }
        Mode::JsonBagJsonbIndexed => {
            connection
                .execute(
                    "CREATE TABLE people(id INTEGER PRIMARY KEY, props JSONB NOT NULL);\
                     CREATE INDEX people_name_expr ON people(json_extract(props, '$.name'));\
                     INSERT INTO people VALUES (1, jsonb('{\"name\":\"Ada\",\"age\":36}'));",
                )
                .expect("jsonb bag + expr index schema");
        }
        Mode::CellDict => {
            // Dictionary: name→prop_id + declared type. Cells store prop_id only.
            connection
                .execute(
                    "CREATE TABLE prop_dict(\
                        prop_id INTEGER PRIMARY KEY, \
                        name TEXT NOT NULL COLLATE NOCASE UNIQUE, \
                        value_type TEXT NOT NULL\
                     );\
                     INSERT INTO prop_dict VALUES (1, 'name', 'text'), (2, 'age', 'integer');\
                     CREATE TABLE people(id INTEGER PRIMARY KEY);\
                     CREATE TABLE node_props(\
                        source_id INTEGER NOT NULL, \
                        node_id INTEGER NOT NULL, \
                        prop_id INTEGER NOT NULL, \
                        value, \
                        PRIMARY KEY(source_id, node_id, prop_id)\
                     );\
                     CREATE INDEX node_props_by_kv ON node_props(prop_id, value);\
                     INSERT INTO people VALUES (1);\
                     INSERT INTO node_props VALUES (1, 1, 1, 'Ada'), (1, 1, 2, 36);",
                )
                .expect("cell dict schema");
        }
    }

    let graph = registered_graph();
    let catalog = Arc::new(ModeCatalog { mode });
    let sql = connection.clone();
    let session = GraphConnection::install(
        connection,
        &graph,
        catalog,
        ParameterTypes::new(),
        Arc::new(SnapshotStore::default()),
        BuildLimits::default(),
    )
    .expect("install graph session");
    (session, sql)
}

fn explain_uses_index(conn: &Arc<turso_core::Connection>, sql: &str) -> bool {
    let rows = conn
        .prepare(format!("EXPLAIN QUERY PLAN {sql}"))
        .expect("explain prepare")
        .run_collect_rows()
        .expect("explain");
    let text = rows
        .iter()
        .flat_map(|row| row.iter())
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.as_str().to_ascii_lowercase()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    text.contains("using index") || text.contains("using covering index")
}

fn assert_bag_read_set(mode: Mode, expected_set_age: i64) {
    let (session, sql) = session(mode);
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.name, n.age",
            &Parameters::new(),
        )
        .unwrap_or_else(|e| panic!("{mode:?} bag read: {e}"));
    assert_eq!(
        rows,
        vec![vec![Value::build_text("Ada"), Value::from_i64(36)]],
        "{mode:?} bag read must project json_extract properties"
    );

    session
        .execute(
            &format!("MATCH (n) WHERE n.name = 'Ada' SET n.age = {expected_set_age}"),
            &Parameters::new(),
        )
        .unwrap_or_else(|e| panic!("{mode:?} bag set: {e}"));

    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age",
            &Parameters::new(),
        )
        .unwrap_or_else(|e| panic!("{mode:?} bag read after set: {e}"));
    assert_eq!(rows, vec![vec![Value::from_i64(expected_set_age)]]);

    let physical = sql
        .prepare("SELECT json(props) FROM people WHERE id = 1")
        .expect("prepare")
        .run_collect_rows()
        .expect("props");
    let props = match &physical[0][0] {
        Value::Text(text) => text.as_str().to_owned(),
        other => panic!("{mode:?}: expected json(props) as text, got {other:?}"),
    };
    assert!(
        props.contains(&expected_set_age.to_string()),
        "{mode:?} bag should contain updated age: {props}"
    );
}

#[test]
fn text_json_bag_mode_reads_and_sets_property_through_cypher() {
    assert_bag_read_set(Mode::JsonBagText, 37);
}

#[test]
fn jsonb_bag_mode_reads_and_sets_property_through_cypher() {
    assert_bag_read_set(Mode::JsonBagJsonb, 38);
}

#[test]
fn jsonb_bag_with_expression_index_reads_sets_and_uses_index_on_hot_key() {
    assert_bag_read_set(Mode::JsonBagJsonbIndexed, 39);

    let (_, sql) = session(Mode::JsonBagJsonbIndexed);
    let filter_sql = "SELECT id FROM people WHERE json_extract(props, '$.name') = 'Ada'";
    assert!(
        explain_uses_index(&sql, filter_sql),
        "JSONB bag + expression index on $.name must use an index"
    );
}

#[test]
fn jsonb_bag_without_expression_index_does_not_use_property_index() {
    let (_, sql) = session(Mode::JsonBagJsonb);
    let filter_sql = "SELECT id FROM people WHERE json_extract(props, '$.name') = 'Ada'";
    assert!(
        !explain_uses_index(&sql, filter_sql),
        "JSONB bag without expression index must not claim a property-value index"
    );
}

#[test]
fn cell_dict_mode_reads_and_sets_through_integer_prop_id() {
    let (session, sql) = session(Mode::CellDict);
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.name, n.age",
            &Parameters::new(),
        )
        .expect("cell dict read");
    assert_eq!(
        rows,
        vec![vec![Value::build_text("Ada"), Value::from_i64(36)]]
    );

    session
        .execute(
            "MATCH (n) WHERE n.name = 'Ada' SET n.age = 40",
            &Parameters::new(),
        )
        .expect("cell dict set");

    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age",
            &Parameters::new(),
        )
        .expect("cell dict read after set");
    assert_eq!(rows, vec![vec![Value::from_i64(40)]]);

    // Physical: rows keyed by integer prop_id (2 = age), not 'age' text.
    let physical = sql
        .prepare("SELECT prop_id, value FROM node_props WHERE node_id = 1 ORDER BY prop_id")
        .expect("prepare")
        .run_collect_rows()
        .expect("cells");
    assert_eq!(physical.len(), 2);
    assert_eq!(physical[0][0], Value::from_i64(1)); // name
    assert_eq!(physical[1][0], Value::from_i64(2)); // age
    assert_eq!(physical[1][1], Value::from_i64(40));

    // Open filter on any prop_id uses (prop_id, value) index.
    let filter = "SELECT node_id FROM node_props WHERE prop_id = 1 AND value = 'Ada'";
    assert!(
        explain_uses_index(&sql, filter),
        "open filter must use node_props_by_kv on (prop_id, value)"
    );
}

#[test]
fn cell_dict_create_and_merge_through_cypher() {
    let (session, sql) = session(Mode::CellDict);
    session
        .execute("CREATE (n {name: 'Hopper', age: 85})", &Parameters::new())
        .expect("CREATE cells");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Hopper' RETURN n.age",
            &Parameters::new(),
        )
        .expect("read after CREATE");
    assert_eq!(rows, vec![vec![Value::from_i64(85)]]);

    session
        .execute(
            "MERGE (n {name: 'Hopper'}) ON MATCH SET n.age = 86 ON CREATE SET n.age = 0",
            &Parameters::new(),
        )
        .expect("MERGE cell key");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Hopper' RETURN n.age",
            &Parameters::new(),
        )
        .expect("read after MERGE");
    assert_eq!(rows, vec![vec![Value::from_i64(86)]]);

    let people = sql
        .prepare("SELECT COUNT(*) FROM people")
        .expect("prepare")
        .run_collect_rows()
        .expect("count");
    assert_eq!(people[0][0], Value::from_i64(2));

    // New property name would require a new prop_dict row + PropertyId (product).
    let hopper_name = sql
        .prepare(
            "SELECT COUNT(*) FROM node_props p \
             JOIN prop_dict d ON d.prop_id = p.prop_id \
             WHERE d.name = 'name' AND p.value = 'Hopper'",
        )
        .expect("prepare")
        .run_collect_rows()
        .expect("join dict");
    assert_eq!(hopper_name[0][0], Value::from_i64(1));
}

#[test]
fn cell_dict_text_predicate_legality_from_value_type() {
    let name = PropertyPhysical::Cell {
        source_id: 1,
        props_table: "node_props".to_owned(),
        entity_column: "node_id".to_owned(),
        identity_column: "id".to_owned(),
        prop_id: 1,
        prop_id_column: "prop_id".to_owned(),
        value_column: "value".to_owned(),
        value_type: ValueType::Text,
    };
    let age = PropertyPhysical::Cell {
        source_id: 1,
        props_table: "node_props".to_owned(),
        entity_column: "node_id".to_owned(),
        identity_column: "id".to_owned(),
        prop_id: 2,
        prop_id_column: "prop_id".to_owned(),
        value_column: "value".to_owned(),
        value_type: ValueType::Integer,
    };
    assert!(name.supports_text_predicate(), "CONTAINS/LIKE ok on text");
    assert!(
        !age.supports_text_predicate(),
        "CONTAINS/LIKE must not apply to integer props"
    );
    assert!(name.supports_is_null() && age.supports_is_null());
}

#[test]
fn binder_rejects_contains_on_integer_cell_property() {
    // Dictionary type Integer on age → text_compatible fails at bind (same gate
    // as Column props). Open Cell stores rely on prop_dict.value_type for this.
    let (session, _) = session(Mode::CellDict);
    let err = session
        .query(
            "MATCH (n) WHERE n.age CONTAINS '3' RETURN n",
            &Parameters::new(),
        )
        .expect_err("CONTAINS on integer age must fail bind/execute");
    let message = err.to_string();
    assert!(
        message.contains("string predicates")
            || message.contains("non-string")
            || message.contains("not supported")
            || message.contains("Bind"),
        "unexpected error for CONTAINS on integer: {message}"
    );
}

#[test]
fn binder_allows_contains_on_text_cell_property() {
    let (session, _) = session(Mode::CellDict);
    let rows = session
        .query(
            "MATCH (n) WHERE n.name CONTAINS 'da' RETURN n.name",
            &Parameters::new(),
        )
        .expect("CONTAINS on text name");
    assert_eq!(rows, vec![vec![Value::build_text("Ada")]]);
}

#[test]
fn open_multi_property_and_filter_via_cell_dict() {
    let (session, sql) = session(Mode::CellDict);
    session
        .execute("CREATE (n {name: 'Curie', age: 66})", &Parameters::new())
        .expect("create");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Curie' AND n.age = 66 RETURN n.name",
            &Parameters::new(),
        )
        .expect("multi-prop AND");
    assert_eq!(rows, vec![vec![Value::build_text("Curie")]]);

    // Both predicates are integer prop_id seeks under the hood.
    let plan_name = "SELECT node_id FROM node_props WHERE prop_id = 1 AND value = 'Curie'";
    let plan_age = "SELECT node_id FROM node_props WHERE prop_id = 2 AND value = 66";
    assert!(explain_uses_index(&sql, plan_name));
    assert!(explain_uses_index(&sql, plan_age));
}

#[test]
fn property_dictionary_register_is_case_insensitive_and_typed() {
    use turso_graph_frontend::PropertyDictionary;
    let mut dict = PropertyDictionary::new();
    let a = dict.register("Name", ValueType::Text).expect("register");
    let b = dict
        .register("name", ValueType::Text)
        .expect("same property");
    assert_eq!(a.prop_id, b.prop_id);
    assert!(dict
        .register("name", ValueType::Integer)
        .expect_err("type conflict")
        .to_string()
        .contains("already registered"));
}

#[test]
fn jsonb_bag_create_and_merge_through_cypher() {
    let (session, sql) = session(Mode::JsonBagJsonbIndexed);
    session
        .execute("CREATE (n {name: 'Grace', age: 45})", &Parameters::new())
        .expect("CREATE into JSONB bag");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Grace' RETURN n.age",
            &Parameters::new(),
        )
        .expect("read after CREATE");
    assert_eq!(rows, vec![vec![Value::from_i64(45)]]);

    session
        .execute(
            "MERGE (n {name: 'Grace'}) ON MATCH SET n.age = 46 ON CREATE SET n.age = 0",
            &Parameters::new(),
        )
        .expect("MERGE bag key");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Grace' RETURN n.age",
            &Parameters::new(),
        )
        .expect("read after MERGE");
    assert_eq!(rows, vec![vec![Value::from_i64(46)]]);

    let count = sql
        .prepare("SELECT COUNT(*) FROM people")
        .expect("prepare")
        .run_collect_rows()
        .expect("count");
    assert_eq!(count[0][0], Value::from_i64(2));
}

#[test]
fn column_default_still_works_without_override() {
    let database = Database::open_file(
        Arc::new(MemoryIO::new()),
        ":memory:property-physical-column",
        Arc::new(SqliteDialect),
    )
    .expect("open");
    let connection = database.connect().expect("connect");
    connection
        .execute(
            "CREATE TABLE people(id INTEGER PRIMARY KEY, name TEXT, age INTEGER);\
             INSERT INTO people VALUES (1, 'Ada', 36);",
        )
        .expect("columns");

    use turso_graph_frontend::{register_graph, GraphRegistration, NodeSourceRegistration};
    register_graph(
        &connection,
        &GraphRegistration {
            name: "cols".to_owned(),
            node_sources: vec![NodeSourceRegistration {
                name: "Person".to_owned(),
                table: "people".to_owned(),
                identity_column: "id".to_owned(),
            }],
            relationship_sources: Vec::new(),
        },
    )
    .expect("register");

    let session = GraphConnection::open(connection, "cols").expect("open session");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age",
            &Parameters::new(),
        )
        .expect("column read");
    assert_eq!(rows, vec![vec![Value::from_i64(36)]]);
    session
        .execute(
            "MATCH (n) WHERE n.name = 'Ada' SET n.age = 41",
            &Parameters::new(),
        )
        .expect("column set");
    let rows = session
        .query(
            "MATCH (n) WHERE n.name = 'Ada' RETURN n.age",
            &Parameters::new(),
        )
        .expect("column read after set");
    assert_eq!(rows, vec![vec![Value::from_i64(41)]]);
}
